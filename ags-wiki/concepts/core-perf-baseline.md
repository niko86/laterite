---
type: concept
title: "core perf baseline: where the time actually goes"
status: drafted
tags: [concept, architecture, performance, benchmark]
volatile: [timings]
volatile_asof: 2026-07-24
ags_editions: []
repo_refs:
  benches: "repo:rust-packages/laterite-ags4-validator/benches/validate.rs"
  fixtures: "repo:tools/gen-bench-fixtures.sh"
  relational: "repo:rust-packages/laterite-ags4-validator/src/rules/relational.rs"
  line_format: "repo:rust-packages/laterite-ags4-validator/src/rules/line_format.rs"
related: [crate-map, testing-strategy, abi3-perf, laterite-ags4-validator, laterite-ags4-emit, laterite-types, laterite-ags4-core]
sources: []
---
# core perf baseline: where the time actually goes

## Definition

The measured cost of the **core** AGS4 data path — parse, validate, type, emit —
attributed stage by stage, and the two hotspots that attribution exposed. This is
the read-side companion to [[abi3-perf]] (which prices the *binding*, not the
engine). Surface/CLI timing is a separate concern and deliberately not here.

Established 2026-07-24 for issue #71. The numbers are a snapshot; the **shape** is
the durable part.

## Why it matters

Before this, optimisation was by intuition. The workspace had one benchmark, and
it read a real 23 MB delivery from gitignored working space — so it self-skipped
everywhere but one machine, and `cargo bench` reported success while measuring
nothing. **A perf gate you cannot run is not a perf gate.**

Fixtures now come from `forge scale` (`repo:tools/gen-bench-fixtures.sh`), which
synthesises a valid AGS4 file calibrated to a target byte size, byte-identical for
a given size + seed. Three rungs — 1 MB / 10 MB / 25 MB, 123 groups — reproducible
on any machine and in CI, carrying no real delivery data.

## The baseline (25 MB rung, release)

| path | time | throughput |
|---|---|---|
| `parse_bytes` | 165 ms | 144 MiB/s |
| `index_ags4_bytes` | 165 ms | 144 MiB/s |
| `read_ags4_bytes` | 276 ms | 86 MiB/s |
| `check_parsed` (rules only) | 343 ms | 69 MiB/s |
| `check_file` (I/O + parse + rules) | 516 ms | 46 MiB/s |

`check_file` ≈ `parse_bytes` + `check_parsed` (508 ms), so I/O plus dictionary
resolution is only ~8 ms and the split is self-consistent. That arithmetic is the
point of benching the layers separately: the old single number could not say
which half had moved.

Two results worth keeping in view:

- **`index_ags4_bytes` costs as much as a full parse.** The index records GROUP
  byte offsets rather than materialising rows, so it ought to be markedly cheaper.
  It isn't — unexplained, and a candidate.
- **`read_ags4_bytes` adds ~67% on top of `parse_bytes`** for the re-trim and
  UNIT/TYPE pad that keep the codec byte-identical to the historical reader (the
  #168 convergence). That is the price of byte-fidelity, paid on every typed read.

## Rule-family attribution

`rules::run_all` is `pub(crate)`, but every family's `check` is `pub`, so each can
be timed directly over one parsed file — no API change, no profiler needed for the
first cut. On the 25 MB rung:

| family | time | share | rules |
|---|---|---|---|
| **relational** | 228.9 ms → **103.0 ms** (fixed, below) | 67% → 49% | 10a–10c, 11a–11c |
| **line_format** | 96.8 ms | **28%** (now the largest) | 1, 3, 5, 6 |
| groups | 8.5 ms | 2.5% | 13–18 |
| typed_values | 5.1 ms | 1.5% | 8 |
| references | 0.46 ms | — | 19b_2/3, 20 |
| structure | 0.43 ms | — | 2, 2a, 2b, 4 |
| dictionary | 0.11 ms | — | 7, 9 |
| naming | 0.01 ms | — | 19, 19a, 19b |

Sums to 340 ms against `check_parsed`'s 343 ms, so nothing is unaccounted for.
**Two families are 95% of the rules engine** — actionable, where "the rules engine
is two thirds of validate" was not.

## The two hotspots — mechanism, not slow code

Neither is inefficient code. Both are *avoidable work*, which is why a bench found
them and a profiler was not needed.

### Rule 3 tokenises a whole line to read one field — OPEN

`rule_3` (`repo:rust-packages/laterite-ags4-validator/src/rules/line_format.rs`)
calls `split_ags_line`, which allocates a `Vec<String>` — one heap allocation plus
one `String` per field, so ~21 allocations for a 20-column DATA row — and then
reads **only field 0** to check the descriptor. Once per line, every line.

> [!warning] `field_span` is NOT the fix, despite looking like it. Its contract is
> "the field at `field_index + 1` — the `+1` skips the leading tag", so
> `field_span(line, 0)` returns the field AFTER the descriptor. It also returns
> **char** offsets, not bytes, so slicing by them misindexes any non-ASCII line —
> precisely the lines Rule 1 exists to flag. Closing this needs a small
> non-allocating first-field accessor on the parse leaf, which is an API addition
> to a shared wasm-clean crate and so an owner decision.

The size of the prize is real though: on one representative line `split_ags_line`
is 390 ns against `field_span`'s 45 ns — **8.6×** — and that gap is allocation.

### Rules 10a/10c re-derived loop-invariant columns and cloned cells to hash them — FIXED 2026-07-24

`rule_10c` (`repo:rust-packages/laterite-ags4-validator/src/rules/relational.rs`)
builds `HashSet<Vec<String>>` of the parent's KEY tuples, then probes it per child
row. Two costs, both removable:

- `tuple()` calls `col(g, name)` per name **per row**, and `col` is a linear scan
  over the heading list. The column indices are identical for every row — they are
  loop-invariant and re-derived anyway.
- Every tuple **clones** its cells into owned `String`s purely to hash them; the
  rows already own that text.

`rule_10a` had the same shape and built **two** tuples per row (one to count, one
to report), so fixing the shared helper fixed both rules.

Resolved by `cols()` (indices once per group) + `tuple_at()` (borrowed `&str`
rather than cloned `String`). Semantics are unchanged — same positional contract,
missing column or ragged row still yields `""` — and the **O-39** empty-parent-KEY
skip is untouched, that being a deliberate divergence that must not shift.

Measured effect:

| | before | after | |
|---|---|---|---|
| `rule-family/relational` | 228.9 ms | 103.0 ms | **−55%** |
| `check_parsed` (whole rules engine) | 343.2 ms | 212.4 ms | **−38%** (p = 0.01) |
| `check_file` (caller-facing) | 516 ms | 390 ms | **−25%**, 46 → 61 MiB/s |

One change, a quarter off the end-to-end validate cost — and it was allocation and
a repeated linear scan, not an algorithm.

## The emit ladder

Measured separately because `AutoFix` is the default and its cost grew after the
default was set (see [[ags4-output]] — mode accepted 2026-06-12, metadata
synthesis added 2026-06-25). 20k rows:

| stage | time | adds |
|---|---|---|
| `write_ags4` | 2.9 ms | the bytes |
| `report` | 30.6 ms | + dictionary fill + `ags4_str` + validate |
| `autofix-no-synth` | 45.4 ms | + `compute_fixes` / `apply_fixes` |
| `autofix-with-synth` | 45.5 ms | + metadata synthesis |

So the writer is ~6% of export cost — **the bytes are not the problem** — and
metadata synthesis is ~0.13 ms, **0.3%**. The 48% `AutoFix` premium is entirely
validate-and-fix, i.e. the original 2026-06-12 decision, not the later addition.

> [!note] Perf is not the argument for making synthesis opt-in. The owner's
> reasoning is that there should be no unexpected magic — the caller decides and
> opts in. A 0.3% measurement neither supports nor undermines that; it just means
> the change is free. See [[ags4-output]].

## Where it shows up

Benches live beside the crate they measure —
[[laterite-ags4-validator]], [[laterite-ags4-core]], [[laterite-types]],
[[laterite-ags4-emit]], and the parse leaf. Run them with
`tools/gen-bench-fixtures.sh` then `cargo bench`; an absent fixture skips rather
than fails, so a clean checkout still works — but a skipped bench measures
nothing, which is exactly how the previous one sat silently dead.

## Related

[[crate-map]] · [[testing-strategy]] · [[abi3-perf]] · [[laterite-ags4-validator]] · [[laterite-ags4-emit]] · [[laterite-types]] · [[laterite-ags4-core]] · [[ags4-output]]
