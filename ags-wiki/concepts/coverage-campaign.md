---
type: concept
title: "coverage campaign: the strategy, the stopping rule and the ledger"
status: drafted
tags: [concept, testing, process, register]
volatile: [coverage, status]
volatile_asof: 2026-07-26
ags_editions: []
repo_refs:
  python_gate: "repo:.github/workflows/ci.yml"
  rust_gate: "repo:.github/workflows/nightly.yml"
  python_scope: "repo:packages/laterite/pyproject.toml"
  node_thresholds: "repo:rust-packages/laterite-node/vitest.config.ts"
  web_thresholds: "repo:web/vitest.config.ts"
related: [testing-strategy, perf-campaign, crate-map, reliquary]
sources: []
---
# coverage campaign: the strategy, the stopping rule and the ledger

## Definition

The **living work-list** for test coverage: what to test next, when a gap is a
real hole versus a documented exclusion, the measured baseline per language, and
the doctrine that keeps this honest. [[testing-strategy]] records *what kinds of
test the repo runs and why*; this page records the *campaign* to raise every
language's line floor to **95%** — so the next session picks up the ranked list
rather than re-deriving it. It is the coverage sibling of [[perf-campaign]].

Owner goal (2026-07-26): **95% lines across all languages**, with the load-bearing
qualifier — **useful tests, not ones written for the sake of the number.**

## Why it matters

Coverage is the easiest metric to game: a test that imports a module and asserts
nothing lifts the number while proving nothing. The goal here is *useful* coverage
— every line the gate counts as covered is covered by a test that would **fail if
the behaviour broke**. Two failure modes this campaign refuses:

1. **The "it runs" test.** A `no exception raised` / `output is not None`
   assertion is the weakest possible test — it moves the % and catches almost
   nothing. See [[testing-strategy]].
2. **The fake exercise of dead code.** Reaching a genuinely-unreachable defensive
   branch just to colour it green teaches the next reader it is live. The honest
   move is a **documented exclusion**, not a contrived test (below).

A floor is only meaningful over a **fixed denominator**. The 2026-07-24 nightly
failure (#73) is the cautionary tale: dev-only QA tooling landing on public added
~7.3k lines at 43–73% covered and the Rust workspace total fell 89.6% → 79.2%
overnight with *no change to the engine's own coverage*. So the Rust gate measures
the **shipped engine only** (bindings + dev tooling excluded) — ratchet the floor
and the exclude set **together**.

## The rules of engagement

1. **Test behaviour, not lines.** The target is a test that fails when the code is
   wrong, not one that merely executes the line.
2. **Parity where an oracle exists.** `laterite.compat` is a python-ags4 drop-in;
   any function python-ags4 also exposes is tested by **differential assertion
   against the oracle** (`python_ags4.AGS4`, a dev dependency), the strongest
   possible check. See [[parity-model]].
3. **Invariant / property for laterite-only logic.** Where no oracle exists (the
   JSON-sourced dict-table helpers, the Arrow bridge), assert the structural
   invariant (columns, row counts vs the dictionary, round-trip identity), not a
   golden blob.
4. **A documented exclusion is a valid outcome, a fake test is not.** Genuinely
   defensive / unreachable code gets a `# pragma: no cover` (Python) or the
   equivalent, **each with a one-line reason** — cf. [[perf-campaign]] C2, where
   the Arrow `ArrayFormatter` Err arm is unreachable in arrow 59. That closes the
   gap honestly; a contrived test does not.
5. **Dep-gated code needs the dep, not a skip.** A branch only reachable with
   `pyarrow` installed is covered by *running that arm with pyarrow*, not by a
   `skipif` that hides it from the denominator. (84 of the current Python tests
   skip — some gaps sit behind those skips.)
6. **Ratchet only on real coverage.** Raise a floor to 95 only once the coverage
   is genuinely there; move the floor and its denominator together, never the
   floor alone.

## Measured baseline (2026-07-26)

| language | measured | floor (the gate) | gate lives in | gap to 95 |
|---|---|---|---|---|
| **Python** | **~99%** ✓ | **95** | `ci.yml` `pytest --cov=laterite` | **met** (floor ratcheted 80→95) |
| **Rust** (shipped engine) | ~89.24% | 88 | `nightly.yml` `cargo llvm-cov` | +6 pt |
| **Node** | not re-measured | 93 lines / 84 branches | `laterite-node/vitest.config.ts` | +2 pt |
| **Web** | ~98% | 95 | `web/vitest.config.ts` | **met** |
| **wasm** | none | none | — | needs a gate designed |

Rust's denominator is the shipped engine: the `cargo llvm-cov` run excludes the
binding cdylibs (wasm, tokenizer-wasm, py, node — tested by the Python/Node/browser
suites, ~0% under `cargo test`) and the dev-only QA tooling (xcheck, parity, forge,
corpus-qa, perf, compliance — never shipped). Those crates are still *tested* on
every PR; they are out of the *measurement*, not the test authority.

## The ranked queue

Ranked by (uncovered lines × how load-bearing the code is). Python first — the
biggest gap and the cheapest to measure.

### Python (83% → ~99%) — **DONE** ✓

All five queue items landed; floor ratcheted 80→95 in `ci.yml`. Final per-file:
`compat.py` 71→**100** (#123-follow parity + error arms), `_cli.py` 78→**100**
(end-to-end verb error paths), `__init__.py` 93→**99** (4 intricate long-tail
lines left — the synthesised-handle direct-ctor path + the bytes no-HEADER excel
arm), `_frames.py` 77→**100**, `dynamic.py` 83→**100**. Documented exclusions:
the pandas<2.2 DuckDB emit fallback, the vestigial `except RuntimeError` excel
translations (the native binding raises the typed error directly), and a couple
of defensive `TypeError`/parse-invariant guards.

| # | file | cover | missed | shape of the gap | test strategy |
|---|---|---|---|---|---|
| P1 ✓ | `compat.py` | 71→**100** | 180 | the python-ags4 drop-in surface: `get_{DICT,ABBR,TYPE,UNIT}_table_from_json_file`, `format_numeric_column`/`_format_sf`, scattered error paths | **parity** vs the oracle (incl. byte-for-byte `write_error_report`); faithfulness/invariant for the JSON dict-table helpers; behavioural for the `BadDictError`/error arms |
| P2 ✓ | `_cli.py` | 78→**100** | 114 | the Python CLI — subcommand argument handling + error/exit paths | invoke each subcommand through the CLI entry, assert exit code + output; `__main__` entry → documented exclusion |
| P3 ✓ | `__init__.py` | 93→**99** | 49 | scattered edge cases + guards in the main API surface | behavioural tests per uncovered branch; unreachable guards → exclusion |
| P4 ✓ | `_frames.py` | 77→**100** | 26 | pyarrow-vs-duckdb materialization backends | **ran both backends** (rule 5 — pyarrow present + simulated-absent), not a skip |
| P5 ✓ | `dynamic.py` | 83→**100** | 10 | dynamic group registration edges | behavioural |

### Rust, Node, wasm — measure next

- **Rust** (~89.24 → 95): re-run `cargo llvm-cov` for the per-crate/per-file
  breakdown before ranking (heavy, instrumented build — do it once, deliberately).
  The gap is small; expect a handful of under-tested modules in the shipped engine.
- **Node** (floor 93/84): `npm run test:coverage` for the actual number + the
  uncovered TS in the `laterite-node` wrapper.
- **wasm**: no gate exists. Design one — the browser engine wasm is exercised by
  Playwright e2e ([[tech-stack-wasm]]); decide whether coverage is collected there
  or a unit layer is added, then set a floor.

## The stopping rule

A language is **done** when its gate floor is 95 over a stable denominator **and**
every remaining uncovered line is either genuinely tested or a documented
exclusion with a stated reason. Stop before that only with an owner decision
(mirrors [[perf-campaign]]'s deferred-candidate rule). "Reached 95 by asserting
`it runs`" is **not** done — it is the one outcome this campaign exists to avoid.

## Related

[[testing-strategy]] · [[perf-campaign]] · [[parity-model]] · [[crate-map]] · [[reliquary]]
