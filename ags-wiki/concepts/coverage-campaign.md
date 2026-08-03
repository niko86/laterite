---
type: concept
title: "coverage campaign: the strategy, the stopping rule and the ledger"
status: drafted
tags: [concept, testing, process, register]
volatile: [coverage, status]
volatile_asof: 2026-08-02
ags_editions: []
repo_refs:
  python_gate: "repo:.github/workflows/ci.yml"
  rust_gate: "repo:.github/workflows/nightly.yml"
  python_scope: "repo:packages/laterite/pyproject.toml"
  node_thresholds: "repo:rust-packages/laterite-node/vitest.config.ts"
  web_thresholds: "repo:web/vitest.config.ts"
related: [testing-strategy, mutation-sweep, perf-campaign, crate-map, reliquary]
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
   wrong, not one that merely executes the line. In the Rust phase this is
   *enforced*, not just intended: each batch runs a [[mutation-sweep]], and a
   surviving mutant is precisely a line that ran without being asserted.
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

> [!warning] **Two different numbers, and the badge is the strict one.**
> lcov's "line %" counts a line whose branch is only half-taken as **hit**;
> Codecov counts it as a **partial**, i.e. not covered. So a lane can read 98% in
> its own gate and ~90% on its badge with neither being wrong. Verified by
> computing `hits/(hits+partials+misses)` against the very lcov the flag uploads
> (node: lcov 95.82 vs Codecov 89.93, computed 90.59). **Codecov's is the measure
> the 95% goal means**, so BRANCH coverage is what moves it — and a lane with only
> a lines floor cannot see the gap at all.

Measured live from the Codecov API, 2026-08-02:

| language | Codecov (strict) | own gate | floor | gate lives in | gap to 95 |
|---|---|---|---|---|---|
| **Python** | **99.15%** ✓ | — | 95 | `ci.yml` `pytest --cov=laterite` | **met** (floor ratcheted 80→95) |
| **Rust** (shipped engine) | **94.88%** ✓ | 94.44–95.91 (4 nightlies) | **93** | `nightly.yml` `cargo llvm-cov` | **met** (floor ratcheted 88→93) |
| **Node** | 89.93 → **95.42%** ✓ | 98.56 lines / 92.21 br | 98 / 91 | `laterite-node/vitest.config.ts` | **met** |
| **Web** | 84.91 → **97.50%** ✓ | 100 lines / 96.34 br | 99 / 95 | `web/vitest.config.ts` | **met** |
| **wasm** | no flag → **own flag + badge** | 69.03 → **90.86%** lines | 89 | `nightly.yml`, own step | +4 pt (ceiling — see below) |

Web had a lines floor and **no branch floor**, which is precisely why its badge
drifted to 85 while its gate sat green above 95. Both lanes now have one.

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

### Rust (89.24 → 94.88%) — **DONE** ✓

Floor 88 → **93**. No test-writing campaign was needed: the number was already
there and the floor had simply never followed it, sitting ~7 points under reality
where it would not have caught a whole crate going dark.

The margin is sized from run-to-run variation rather than picked: the last four
nightlies read 95.91 / 95.90 / 94.44 / 94.88, and the denominator itself moved
15,522 → 16,659 lines in that window when the `laterite` facade crate landed. 93
sits ~1.4 points under the lowest of those. A floor tighter than the denominator's
own drift fails on arithmetic, not on coverage.

> [!note] **This lane has no strict-vs-lenient gap, and that is not luck.**
> llvm-cov's lcov carries no branch records (`Branches` reads 0/0), so Codecov
> sees no partials and its badge ≈ the gate number. The ten-point divergence that
> caught web and node cannot happen here — and equally, adding a branch floor here
> would gate on data that does not exist. Do not port that reasoning across.

### Node (89.93 → 95.42%) — **DONE** ✓

Floor 97/89 → **98/91**. Five suites, all aimed at decisions that fail *quietly*
rather than loudly:

- **`duckdb-typing`** — the AGS TYPE → DuckDB column matrix. The `0DP` → `BIGINT`
  arm had never been exercised: a count arriving as `7.0` is a different fact
  from `7`, and it round-trips back out that way. Also pins the micros-vs-millis
  scaling in `toMicros`, the same class of bug the web side once had (read
  microseconds as milliseconds and every timestamp lands in 1970, silently).
- **`frame-query-builder`** — `_filteredRows`' `WHERE TRUE` fallback and the
  `* EXCLUDE (_id, _parent_id)` vs plain `*` choice. Both produce a query that
  *runs*, with the wrong columns or the wrong rows.
- **`ragged-and-bytes-doors`** — `rowsToTable`'s null-fill on a short row, and
  `fromExcel`'s path-vs-bytes doors.
- **`registry-error-translation`** — `ancestorChain` / `inheritedKeyNames` catch
  a native throw as `e instanceof Error ? e.message : String(e)`, and only the
  first arm was ever taken. Without the second, a non-`Error` from napi reports
  an `Ags4Error` whose message is the string `"undefined"` — the only diagnostic
  the caller had, gone. Took `registry.ts` from 50 → **100%** branch.
- **`cli-exit-codes`** — exit code 3 for a missing file, and the `--tran-*` fold.

Falsified by mutation: censor's default inverted, the `TRUE`/`FALSE` clause
flipped, the null-fill removed, the `String(e)` arm removed — all four caught.

### Web (84.91 → 97.50%) — **DONE** ✓

Floor 99/89 → **99/95**; lines and functions are both at 100%. Four suites:

- **`sqlgen-joins`** — join-mode `SELECT` plus the join-mode chart refs. The
  failure mode throughout is a query that runs and answers wrongly: an
  unqualified column in a join is ambiguous exactly when both sides share the
  heading, which for `LOCA_ID` is the *normal* case, and DuckDB resolves it by
  picking one. Also pins the half-open depth band (`<` not `<=`, or a boundary
  sample lands in two strata).
- **`relationships-templates`** — `joinKeys` / `depthRangeOf` / `geologyTemplate`
  / `relExamples`: the SQL the Explore tab *offers*. A wrong answer here is not a
  crash, it is a suggestion that runs and returns the wrong rows.
- **`dict-load`** — the union-dictionary fetch doors. A non-OK response must
  throw: parsing a proxy's JSON error page instead yields a union with no groups,
  i.e. an Explore tab where nothing is a known AGS group and nothing says why.
- **`coords-project`** — `project()`'s two non-finite guards, against the official
  OS test points. The *output* guard is the one the input guard cannot stand in
  for: `1e300` is finite, walks past the first check, and comes back unplottable.

Falsified by mutation: the sensitive-policy guard dropped, the union fetch's
`!res.ok` throw removed, chart columns never alias-qualified, the `MC` gloss
deleted, the template's depth-column check forced true — all five caught.

The ~10 branches still short of 100 are `?.` / `??` guards TypeScript's narrowing
needs but no input can reach (a `present.get()` on a key taken from the same map,
a `dict.get()` after a `dict.get()?.parent` test). Converting those would move
the number without testing anything, so the floor keeps a margin rather than
pinning today's value.

### wasm (69.03 → 90.86%) — **DONE** ✓

Floor 67 → **89**, and it now has a Codecov flag and a README badge like every
other language.

The previous entry called the ceiling structural and named the fix: move the
logic **out** of the fat `#[wasm_bindgen]` exports into plain functions the
native tests can call. That is what was done. Fourteen exports gave up their
bodies to a `*_core` twin — `build_ags4_core`, `build_ipc_core`,
`compute_fixes_core`, `apply_fixes_core`, `read_core`, `meta_core`,
`arrow_ipc_core`, `diff_core`, `merge_core`, `dictionary_core`, `censor_core`,
`ags4_to_xlsx_core`, `xlsx_to_ags4_core` (plus `build_parts`, the shared TRAN
fold) — and what is left inside each export is `decode → core → marshal`.

**First: check whether the dark code should exist at all.** It should — every
one of those exports is a registered cross-surface capability in
`modality.json`, and `build_ags4_ipc` / `list_rules` / `engine_fingerprint`
are unused by *our* web app but are published `@laterite/ags4-wasm` API with
consumers elsewhere (`engine_fingerprint` backs the xcheck engine-identity
gate). Nothing here was deletable; the problem was the *shape*, not the
existence.

**What the residue is, exactly.** ~127 of the ~141 uncovered production lines
are `JsValue`/`JsError` marshalling. `cargo test` cannot call them, and — the
part worth knowing before anyone tries — covering them would need
`wasm-bindgen-test` on **wasm32, a target llvm-cov cannot instrument**, so
those tests would not move this number even once written. They are exercised
end-to-end instead: the `wasm-engine` xcheck leg drives `build_ags4` and
`web/src/lib/content-hash.test.ts` drives `read` + `arrow_ipc`, both against
the real `.wasm` binary through Node. **The boundary is the floor of this
measurement, not a backlog item.**

> **Beware the denominator here.** `lib.rs` is ~45% `#[cfg(test)]` code and
> llvm-cov counts it, as hits — so the headline rises whenever tests are
> added, whatever they assert. Measured separately, PRODUCTION lines went
> **61.42% → 90.90%**; the whole-file figure the gate reads is 90.86%. They
> agree here because the extraction moved real logic, but they would not have
> if the same points had been bought with tests alone.

One real defect fell out of writing the tests: `build_ags4`/`build_ags4_ipc`
serialised through serde-wasm-bindgen's **default** serializer (`None` →
`undefined`) while every other door used `json_compatible` (`None` → `null`),
yet `BuildReport`'s published TS declares `line: number | null` — so
`f.line === null` type-checked clean and never matched. Fixed in #212; there
is now one serializer and a source-level test refusing a second.

> [!caution] **A second "finding" was not one, and the lesson generalises.**
> The same pass flagged `laterite-ags4-merge` widening `{2DP, X}` to `X`
> without a `type_widened` warning. It is **documented, deliberate behaviour**
> — stated in `TypeClashMode::Widen`'s own doc comment ("Typed-vs-`X` resolves
> silently; two *different* non-`X` types warn") and pinned by a named
> acceptance test, `typed_vs_x_widen_is_silent`. Merge made no *choice* there:
> once one file declares `X` the column can only be `X`, so there is no
> resolution to report.
>
> **Coverage work walks into untested code and is therefore primed to read
> deliberate design as oversight.** Before reporting a gap as a defect, look
> for the decision: the enum doc, an acceptance test, a wiki page. Absence of
> a *test* is not absence of a *decision* — and here the test existed too, one
> crate away. Present the finding and ask; do not reverse a recorded decision
> because the axis you happened to measure did not see the reason for it.

## The stopping rule

A language is **done** when its gate floor is 95 over a stable denominator **and**
every remaining uncovered line is either genuinely tested or a documented
exclusion with a stated reason. Stop before that only with an owner decision
(mirrors [[perf-campaign]]'s deferred-candidate rule). "Reached 95 by asserting
`it runs`" is **not** done — it is the one outcome this campaign exists to avoid.

## Related

[[testing-strategy]] · [[mutation-sweep]] · [[perf-campaign]] · [[parity-model]] · [[crate-map]] · [[reliquary]]
