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

- **`index_ags4_bytes` cost as much as a full parse — because it WAS one.**
  FIXED 2026-07-24. The line above originally read "the index records GROUP byte
  offsets rather than materialising rows, so it ought to be markedly cheaper — it
  isn't, unexplained". That described the intent, not the code: `ParseOptions::lean()`
  only turns off raw-line retention, so every line was still tokenised into owned
  `String`s and every DATA row materialised, after which the function kept ~123
  GROUP records and dropped the rest. **A page can describe what a function is
  for and be wrong about what it does; the fix was reading it.** See
  [[laterite-ags4-core]] and the locate-only profile below.
- **`read_ags4_bytes` adds ~67% on top of `parse_bytes`** for the re-trim and
  UNIT/TYPE pad that keep the codec byte-identical to the historical reader (the
  #168 convergence). Filed as the price of byte-fidelity, paid on every typed
  read — but that is an assumption nobody has tested, and it is now the largest
  unexamined gap on the read path. Treat it as open, not as settled.

## Rule-family attribution

`rules::run_all` is `pub(crate)`, but every family's `check` is `pub`, so each can
be timed directly over one parsed file — no API change, no profiler needed for the
first cut. On the 25 MB rung:

| family | time | share | rules |
|---|---|---|---|
| **relational** | 228.9 ms → **103.0 ms** (fixed) | 67% → 70% | 10a–10c, 11a–11c |
| **line_format** | 96.8 ms → **32.5 ms** (fixed) | 28% → 22% | 1, 3, 5, 6 |
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

### Rule 3 tokenised a whole line to read one field — FIXED 2026-07-24

`rule_3` (`repo:rust-packages/laterite-ags4-validator/src/rules/line_format.rs`)
calls `split_ags_line`, which allocates a `Vec<String>` — one heap allocation plus
one `String` per field, so ~21 allocations for a 20-column DATA row — and then
reads **only field 0** to check the descriptor. Once per line, every line.

> [!warning] `field_span` is NOT the fix, despite looking like it. Its contract is
> "the field at `field_index + 1` — the `+1` skips the leading tag", so
> `field_span(line, 0)` returns the field AFTER the descriptor. It also returns
> **char** offsets, not bytes, so slicing by them misindexes any non-ASCII line —
> precisely the lines Rule 1 exists to flag. Anyone optimising here will reach for
> it; don't.

Resolved instead by the shared scanner (below): `scan::first_field` borrows field
0. Measured over all 418,638 lines of the 25 MB fixture, `split_ags_line` costs
**66.2 ms** against `first_field`'s **1.75 ms** — **38×** — and `line_format` fell
96.8 → **32.5 ms**. (Per-line on a fat 10-field row the ratio reads 85×; the 38×
is over the real fixture's line mix, so that is the one to quote.)

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

## One scanner, two value policies

The grammar was implemented **three times** in `laterite-ags4-parse`:
`split_ags_line` (owned unescaped values), `field_span` (one field's char span)
and `tokenize_spans` (all spans, lossless reassembly). Three hand-written machines
over one grammar — which is how they came to disagree on **five** behaviours:

| | `split_ags_line` | `field_span` | `tokenize_spans` |
|---|---|---|---|
| empty line | 0 fields | `None` | 1 empty field |
| unquoted field | verbatim | verbatim | **trimmed** |
| field indexing | 0-based | **`i+1`** (skips tag) | 0-based |
| `""` escape | **unescaped** | raw | raw |
| unterminated quote | to end-of-line | to end-of-line | **empty value** |

`scan::scan_line` is the shared core. Two decisions make it work:

- **Bytes, not code points.** `"` and `,` are ASCII and UTF-8 never puts an ASCII
  byte inside a multi-byte sequence, so a byte scan is exactly equivalent — and
  the validator's 418k-line walk stops paying for the code-point offsets only the
  browser needs.
- **The value policy is a parameter.** What was duplicated, and where every
  divergence came from, is the *state machine*. What legitimately differs is how a
  token's inner value is resolved — and those differences are needs, not accidents:
  a browser editor wants display-trimmed bounds; a validator must judge the raw
  bytes, because on an unquoted field the whitespace **is** the Rule 5 violation.
  Collapsing them would let a UI concern define what the validator calls a value.
  So `RAW` and `DISPLAY` policies share one scan.

The core is cheaper than every machine it subsumes. Per line (one 10-field DATA
row, benched in `parse/per-line` so the claim stays reproducible):

| | time | vs core |
|---|---:|---:|
| `tokenize_spans` | 511.5 ns | 3.5× |
| `split_ags_line` | 347.6 ns | 2.4× |
| **`scan_line/raw`** | **147.8 ns** | — |
| **`scan_line/display`** | **147.3 ns** | — |
| `field_span` (one field) | 48.7 ns | |
| `first_field` (field 0) | 4.07 ns | |

**The value policy is free** — RAW and DISPLAY differ by less than noise, because
the divergence is a handful of comparisons at token close, not a second walk. The
separation argued for above on design grounds costs nothing to keep.

`tokenize_spans`' 3.5× was its `Vec<char>` per line plus one `String` per field.
**Claimed 2026-07-24:** it was retired onto the core, and its `AgsSpan` — whose
owned `text` existed only because code-point offsets are unusable from Rust —
became `RawField`. Two implementations remain (`split_ags_line`, `field_span`);
`field_span` is kept deliberately, because folding it would cost the
short-circuit that makes it 48.7 ns.

One divergence is irreducible: a borrowed slice **cannot unescape**, since `""`→`"`
yields a value shorter than its source. `RawField::has_escape` flags it. Sound for
Rule 3 — no descriptor contains a quote.

## Cumulative

| | baseline | now | |
|---|---|---|---|
| `check_parsed` | 343.2 ms | **147.6 ms** | **−57%** |
| `check_file` | 516 ms | **328 ms** | **−36%**, 46 → 72 MiB/s |
| `index_ags4_bytes` | 165 ms | **56.9 ms** | **−66%**, 144 → 418 MiB/s |
| `parse_bytes` | 171.3 ms | **142.8 ms** | **−16.7%** |
| `emit_ags4/autofix` | 45.4 ms | **23.1 ms** | **−49%** |
| `emit_ags4/report` | 30.6 ms | **22.4 ms** | **−27%** |

**Not one of these was an algorithm change.** Every one was work that did not
need doing:

- re-deriving loop-invariant column indices, once per row;
- cloning cells purely to hash them;
- tokenising a whole line to read one field of it;
- running a full parse to keep 123 GROUP records;
- allocating every cell twice — once in the tokenizer, once in the copy out of it;
- parsing the same bytes twice in the same function.

That is the durable lesson of this page, and it is why the benches exist. None of
it was visible by reading the code for slowness, because none of it *is* slow
code. It only became visible once the layers were timed separately.

> [!note] Absolute figures drift ~20% run-to-run on a 25 MB fixture (page cache),
> so the paired criterion comparison is the durable number and the millisecond
> values are not. A second run of identical `index_ags4_bytes` code moved 56.9 →
> 47.0 ms.

## The emit ladder

Measured separately because `AutoFix` is the default and its cost grew after the
default was set (see [[ags4-output]] — mode accepted 2026-06-12, metadata
synthesis added 2026-06-25). 20k rows:

| stage | time | adds |
|---|---|---|
| `write_ags4` | 2.9 ms | the bytes |
| `report` | 30.6 → **22.4 ms** | + dictionary fill + `ags4_str` + validate |
| `autofix-no-synth` | 45.4 → **23.1 ms** | + `compute_fixes` / `apply_fixes` |
| `autofix-with-synth` | 45.5 → **23.8 ms** | + metadata synthesis |

The writer is a few percent of export cost — **the bytes are not the problem** —
and metadata synthesis is ~0.13 ms, **0.3%**.

> [!warning] This page previously read: "the 48% `AutoFix` premium is entirely
> validate-and-fix, i.e. the original 2026-06-12 decision, not the later
> addition." **That attribution was wrong.** The premium was mostly a *duplicate
> parse*: `validate()` parsed the emitted bytes, ran the rules, dropped the
> `ParsedFile` — and `AutoFix` then re-parsed the same bytes to rebuild it for
> `compute_fixes`. Returning the parse from `validate()` (2026-07-24) collapsed
> the premium from **48% to 3%** (23.1 vs 22.4 ms). Fixing *is* nearly free; the
> measurement had been attributing a plumbing defect to a design decision.

The lesson generalises past this page: a number can be reproducible, correctly
measured, and still support the wrong conclusion, because attribution is a claim
about mechanism and a stopwatch does not make one.

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
