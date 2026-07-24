---
type: concept
title: "perf campaign: the profiling ledger"
status: drafted
tags: [concept, performance, process, register]
volatile: [timings, status]
volatile_asof: 2026-07-24
ags_editions: []
repo_refs:
  benches: "repo:rust-packages/laterite-ags4-validator/benches/validate.rs"
  fixtures: "repo:tools/gen-bench-fixtures.sh"
  readme_bench: "repo:tools/bench-vs-python-ags4.py"
related: [core-perf-baseline, abi3-perf, testing-strategy, crate-map, reliquary]
sources: []
---
# perf campaign: the profiling ledger

## Definition

The **living work-list** for performance: what has been measured, what is
claimed, what is priced-and-declined, and what is still an unexamined guess.
[[core-perf-baseline]] records the *findings*; this page records the *campaign* —
so the next session can pick up the list rather than re-derive it, and so
"we profiled things" never again means "we profiled whatever was in front of us".

## Why it matters

Three separate times, this repo recorded a mechanism that was **wrong** while its
measurement was right:

- `index_ags4_bytes` "costs as much as a full parse — unexplained". It *was* a
  full parse.
- The `AutoFix` premium "is entirely validate-and-fix". It was mostly a duplicate
  parse of the same bytes.
- `read_ags4_bytes`' +67% is "the price of byte-fidelity". It was a per-row
  `HashMap` and 8.4M cloned heading names.

Each was plausible, written in good faith, and never checked against the code. A
stopwatch tells you *where*; only reading tells you *why*. **An unverified
attribution is a guess with a number attached** — this ledger marks them as such.

## The rules of engagement

1. **One candidate at a time.** Measure, change, re-measure, land. A batch of
   changes with one benchmark run cannot attribute its own result.
2. **Paired comparison is the number.** Absolute figures drift ~20% run-to-run on
   a large fixture (page cache). Quote criterion's before/after, not milliseconds.
3. **`--quick` is for triage, never for a claim.** It has reported a +10%
   regression that proper sampling showed to be a −6% improvement. Inconsistent
   signs across rungs mean noise.
4. **Read the code before believing the profile.** See above.
5. **A claim needs a committed bench.** If nothing in the repo reproduces it, it
   is folklore. This applies to wiki numbers and README numbers equally.
6. **Cover what you touch.** A perf change that moves a branch into a hot path
   without a test is a latent bug with better timing. If the area has a coverage
   gap, close it *in the same change* — see the coverage column below.

## Order of attack

**Rust core first**, until it is as fast as is reasonably practicable — the
surfaces (wheel, node, wasm, CLI) all sit on it, so a percent there is a percent
everywhere, and optimising a surface over a slow core measures the wrong thing.
Only then move outward.

The layering to work down:

```
parse leaf  →  rules engine / read codec / index  →  emit  →  surfaces
```

## The ledger

Status vocabulary: **open** (unmeasured), **priced** (measured, not yet acted
on), **claimed** (landed), **declined** (measured and deliberately not taken —
with the condition for revisiting).

### Claimed

| candidate | mechanism | result | coverage |
|---|---|---|---|
| rules 10a/10c | loop-invariant column indices re-derived per row; cells cloned to hash | relational −55% | existing rule tests |
| rule 3 | whole line tokenised to read field 0 | line_format −66% | `display_spans.rs` proptest added |
| `index_ags4_bytes` | ran a full parse, kept ~123 records | −66% | `locate_only.rs` added (equivalence + rejection parity) |
| DATA cell double-allocation | `fields[1..].to_vec()` cloned every tokenizer `String` | `parse_bytes` −17% | existing walk tests |
| `AutoFix` duplicate parse | `validate()` parsed, dropped the parse, then re-parsed the same bytes | autofix −49% | existing emit tests |
| `tokenize_spans` retirement | third implementation of the line grammar; `AgsSpan.text` shipped derivable data across the wasm boundary | 511.5 → 147.3 ns/line | TS tiling + contiguity tests added — **caught a real conversion bug** |
| `from_shared` row projection | per-row `HashMap` + heading name cloned per cell + values cloned from a parse about to be dropped | `read_ags4_bytes` −27%, projection −74% | existing `from_shared_trim.rs` pins semantics |

### Priced, declined

| candidate | prize | why not | revisit when |
|---|---|---|---|
| positional row model (`Vec<Vec<String>>` + heading index) | ~25 ms, 13% of the typed read | breaks `r["LOCA_ID"]` at every call site — `lat read`, `laterite-excel`, node, `read_groups_raw` | a caller reads these rows in a hot loop, **or** `AgsGroup` is being reshaped anyway. Not on the 13% alone. |

### Open — unmeasured, needs a bench before an opinion

| candidate | why it is suspected | clarity needed |
|---|---|---|
| `read_ags4_bytes` residual | after the projection fix it is still ~194 ms vs `parse_bytes`' ~143 ms | what the remaining ~50 ms *is* — the per-row `HashMap` accounts for ~25 ms; the rest is unattributed |
| `laterite-types::arrow_cols` | the **Python** read path, so it is what the README's read tables actually exercise. Never benched in isolation | needs its own bench before any claim; currently invisible between `parse_bytes` and the wheel |
| typed-value parsing (`parse_value`) | the DT guessing loop tries formats in sequence; UNIT is not threaded in (item 4b) | whether UNIT-driven single-format parsing is a perf win or purely a correctness one — do NOT assume perf |
| `emit` residual | writer is a few percent; `AutoFix` is now ~3% over `Report` | whether anything is left worth taking, or emit is done |
| surfaces (wheel / node / wasm / CLI) | deliberately not started | blocked on "core is done" — do not start early |

> [!warning] The Open rows are **suspicions, not findings**. Each needs a bench
> and a code read before it earns a mechanism. Given this page's own history,
> writing a plausible cause here without measuring would be the exact failure it
> exists to prevent.

## Coverage discipline

Perf work moves branches into and out of hot paths, which is precisely when an
untested edge becomes a silent wrong answer. Two live examples:

- the `locate_only` profile skips the descriptor model but **must not** skip the
  strict-structure guard — pinned only because the equivalence test asks;
- the wasm byte→code-point conversion was queried out of order and collapsed
  every inner value to empty. The `debug_assert` guarding it is compiled out of
  release wasm; the **TS-side test** is what caught it.

So: when a candidate touches an area with a coverage gap, close the gap in the
same change, and record it in the ledger's coverage column. Nightly coverage
excludes the QA crates (see `.github/workflows/nightly.yml`) — a denominator
choice, not a regression, and worth re-checking before reading any coverage
delta as real.

## Where the benches live

Beside the crate they measure. `tools/gen-bench-fixtures.sh` synthesises the
criterion rungs (1/10/25 MB, forge `wide` scaffold, seed 0 — byte-identical for a
given size+seed, and carrying no real delivery data).
`tools/bench-vs-python-ags4.py` reproduces the README's comparison tables, with
the fixtures SHA-pinned so generator drift fails loudly instead of quietly moving
the numbers.

> [!note] An absent fixture SKIPS rather than fails, so a clean checkout still
> works — but a skipped bench measures nothing, which is exactly how the
> pre-2026-07 bench sat silently dead on every machine but one.

## Related

[[core-perf-baseline]] — the findings this campaign produced.
[[abi3-perf]] — the binding's cost, measured separately.

[[core-perf-baseline]] · [[abi3-perf]] · [[testing-strategy]] · [[crate-map]] · [[reliquary]]
