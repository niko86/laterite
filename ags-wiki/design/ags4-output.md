---
type: decision
title: "AGS4 output — emit valid AGS4 from data (native + wasm)"
status: accepted
tags: [design, decision, api, wasm, output]
decided: 2026-06-12
supersedes: []
from_gap: []
related: [api-surface-1.0, pyo3-boundary, crate-map, tech-stack-wasm, dec-laterite-types-leaf, reliquary]
sources: []
---

# AGS4 output — emit valid AGS4 from data (native + wasm)

> **status: accepted — COMPLETE (P0–P4 BUILT 2026-06-13): native + browser producers, the
> columnar Arrow-IPC path, and a web Export tab.** Driver: an
> owner app (potentially commercial) needs to **produce valid AGS4 from its own data** —
> pandas/polars frames and/or browser JSON — to import into another system. Today only
> `apply_fixes` emits AGS4 bytes (it *patches an existing file's text*); there was **no "build a
> new valid AGS4 from data" path**. This page designed **one shared Rust emitter exposed to both
> hosts**; the native half is built (see *As built* below) and made the compat write all-Rust.

## As built (2026-06-13, branch `feat/0.3-duckdb-engine`)

- **P0 — crate placement: a new wasm-safe `laterite-ags4-emit` leaf** (the recommended option). `write_ags4`
  moved out of `laterite-ags4-core`; `laterite_ags4_core::ags4_writer` is now a re-export shim; `From<EmitError> for
  CliError` keeps `excel` + `db-to-ags4` callers on `?`. Deps `laterite-types` + `laterite-ags4-validator` only —
  both already wasm-safe `laterite-ags4-wasm` deps. See [[crate-map]] (now 13 crates). First step relieving
  the `laterite-ags4-core` naming smell.
- **P1 — the orchestrator `emit_ags4(groups, opts) -> EmitResult`** in the leaf: hybrid UNIT/TYPE
  fill from the **per-edition `laterite-ags4-validator` Dictionary** (chosen over `laterite-ags4-core::registry` — it's
  edition-aware *and* already wasm-safe), cell formatting via `laterite_types::ags4_str` (typed) /
  verbatim (string), then `write_ags4`, then `Strict | Report | AutoFix` (chains the validator's
  `parse_bytes`/`run_all`/`compute_fixes`/`apply_fixes`, filtered to `FixRisk::Safe`). 7 tests.
- **P2a — native door `laterite.emit_ags4(groups, *, edition, mode) -> EmitResult`** (+ the
  `emit_ags4_from_arrow` PyO3 binding). **Boundary = the frame's own Arrow C-stream PyCapsule**,
  handed straight to `pyo3_arrow::PyTable` (zero-copy). polars exposes `__arrow_c_stream__`
  pyarrow-free, so emit stays a **base** feature (no `[compat]`).
  **Correction (2026-06-16, #111 base-surface audit):** the original design routed *every* frame
  `con.register(...)` through the Python-owned DuckDB engine on the belief that made it pyarrow-free
  for both backends — but an import-blocked test proved `con.register(polars_df)` calls polars
  `.to_arrow()` → **pyarrow**, leaking the `[compat]` dep into a base call. The fix bypasses DuckDB:
  pass the capsule directly to Rust. Only an *old* pandas (no `__arrow_c_stream__`, pre-2.2) now
  falls back through DuckDB — and pandas ships solely via `[compat]` (which carries pyarrow + duckdb),
  so a base polars user never pays. Guarded by `test_base_surface_no_extras` (exercises the whole
  base surface under a simulated base-only install). 11 tests; full suite green.
- **P2b — compat write is now all-Rust.** `compat.dataframe_to_AGS4` hands its polars frames'
  pyarrow-free Arrow capsule to `emit_ags4_compat` (a verbatim string-matrix path reproducing
  python-ags4's `"" if v is None else str(v)`); `_matrix_from_df` and the orphaned old
  `_native.emit_ags4` matrix pyfunction were removed. **python-ags4 parity held byte-identical at
  122/131.** (compat could NOT reuse the AutoFix orchestrator — its frames already carry the
  UNIT/TYPE/DATA tag rows and must stay byte-verbatim — so it's a separate, narrow Rust path.)
- **P3 — wasm doors `to_ags4` (JSON) + `to_ags4_ipc` (Arrow IPC)** (`laterite-ags4-wasm`). The browser
  hands either a JSON array of `{code, headings, units?, types?, rows}`, or — for large, already-
  columnar data (e.g. a duckdb-wasm result) — an array of `{code, ipc: Uint8Array}` Arrow IPC
  streams; Rust runs the *same* shared `laterite-ags4-emit::emit_ags4` and returns `{ text, findings,
  fixes_applied }` (`text` = the AGS4 document, UTF-8/CRLF, for a `Blob`). **Proven live in Chrome
  (`to_ags4` ALL_PASS) + a real `wasm32-unknown-unknown` build + 18 host tests.**
- **DRY — the Arrow→`serde_json::Value` transpose is shared, not copied.** `laterite-ags4-emit` gained an
  optional `arrow` feature exposing `group_from_arrow(code, schema, batches)` + `cell_value`; both
  hosts use it — native (`laterite-py`, off the DuckDB capsule) and wasm (`laterite-ags4-wasm`, off the IPC
  `StreamReader`). Symmetric with the read path's shared *builder* `laterite-types::arrow_cols`
  (`Value`→Arrow). `arrow` stays `default-features=false` so the display fallback adds no
  comfy-table to the wasm bundle.
- **P4 — the web Export tab** (`web/src/components/export/ExportPane.tsx`, SolidJS). A reference UI
  in the validator site: edition + mode selectors, a prefilled JSON editor, *Build & download .ags*
  / *Preview*, and a result panel (text + findings + `fixes_applied`). `to_ags4` is wired through
  the worker bridge (`validator.worker.ts` + `validatorClient.ts`, the no-bytes shape like
  `dictionary()`); the Blob download reuses `downloadBlob`. e2e: `export.spec.ts`. **Wasm-size
  consequence:** the emit feature grew the validator wasm 2.2 → 3.3 MB (mostly the columnar
  `to_ags4_ipc` reader path), so the PWA `maximumFileSizeToCacheInBytes` went 3 → 4 MiB. The Export
  pane itself only uses the JSON `to_ags4`; the app would more typically consume the wasm directly.
- **Defaults — all ACCEPTED as proposed:** validity = **AutoFix**, UNIT/TYPE = **hybrid**, edition
  selectable default **4.1.1**, MVP scope **PROJ·LOCA·SAMP·GEOL·ISPT** (frames whose *columns are
  the AGS headings*, wired by `{group}_ID`).

## Update (2026-06-25): AutoFix synthesizes missing metadata groups

`emit_ags4`'s **AutoFix** mode (the default) now mints whichever mandatory metadata
catalog group is absent, so a *data-only* build — notably a typed PROJ graph
(`build_ags4(proj)`), which can't reach the parentless root-metadata groups — yields
a **valid file in one call** instead of Rule 14/15/17 findings:

- **UNIT / TYPE** — pure derivations of the data: one row per distinct unit/type
  actually used (the Rule 15/17 collection), `*_DESC` falling back to the symbol
  (the dictionary carries no unit/type description catalog).
- **TRAN** — a placeholder stub: `TRAN_AGS` = the edition's expected value,
  `TRAN_DLIM`/`TRAN_RCON` = the AGS standard `|`/`+`, and the REQUIRED transmission
  fields the build can't know (`TRAN_PROD`/`TRAN_RECV`/`TRAN_STAT`, the date) as
  `"TBC"`/placeholder values for the caller to overwrite (**owner decision,
  2026-06-25** — stub-with-placeholders over leave-as-a-finding).
- **ABBR** — synthesized too, but only when the data uses PA picklist codes (Rule
  16): one row per distinct `(heading, code)` (PA cells split on the `TRAN_RCON`
  concatenator like Rule 16a), `ABBR_DESC` from the standard ABBR table via
  `Dictionary::abbr_desc`, falling back to the code.
- **PROJ** is never synthesized (real project identity, not derivable) → a missing
  PROJ stays a Rule 13 finding.

One shared implementation in `repo:rust-packages/laterite-ags4-emit/src/emit.rs::synthesize_metadata`
(a new step before `write_ags4`, gated to AutoFix), so **all four surfaces** (PyO3,
Node, both wasm paths) inherit it with zero per-binding work; `"report"`/`"strict"`
are unchanged (they show/reject the gaps). Mirrors the existing forge synthesizer
(`repo:rust-packages/laterite-ags4-forge/src/synth/model.rs` `collect_unit`/`collect_type`/`tran`)
— a parallel flagged for convergence in [[reliquary]].

## Update (2026-07-24): metadata synthesis is OPT-IN — LANDED

**Reverses the "on by default" half of the 2026-06-25 update above.** The synthesis
*behaviour* is unchanged and stays available; what changes is that the caller must
ask for it.

**Reason — no unexpected magic.** A caller who hands `emit_ags4` their data should
get *their* data back as AGS4, not silently acquire TRAN/UNIT/TYPE/ABBR groups they
never supplied. Minting a placeholder `TRAN` with `"TBC"` fields is a meaningful act
performed on someone's behalf; the owner's position is that the user decides and
opts in. That argument stands on agency, not cost.

**Not a performance change.** The staged emit bench prices synthesis at ~0.13 ms —
**0.3%** of an export (see [[core-perf-baseline]]). Perf neither motivates nor
argues against this; it only means the change is free either way.

> [!note] This section originally added "the 48% `AutoFix` premium is
> validate-and-fix, i.e. the original 2026-06-12 default". **That was wrong.** The
> premium was mostly a *duplicate parse* of the emitted bytes, removed 2026-07-24;
> `AutoFix` now costs ~3% more than `Report`. So there is no performance argument
> on either side of this decision, which is the cleanest possible footing for one
> made on agency grounds.

**Consequence, accepted:** a data-only build (notably a typed PROJ graph) no longer
yields a valid file in one call by default — Rule 14/15/17 findings return unless
the caller opts in. The one-call-valid path remains, one argument away.

**Shape:** `EmitOpts::synthesise_metadata`, default `false`. British spelling for
the new public surface (owner decision); only the flag and its private helper were
renamed — the surrounding `-ize` prose was left alone rather than swept.

Exposed on the surfaces that construct emit options, because a surface that
silently opts in on the user's behalf is the same magic one layer down:

| surface | name |
|---|---|
| Rust | `EmitOpts::synthesise_metadata` |
| Python | `build_ags4(..., synthesise_metadata=True)` |
| Node | `buildAgs4(..., { synthesiseMetadata: true })` |
| wasm / merge | inherit the default — neither mints unasked |

**What "off" actually means.** `AutoFix` still repairs what the caller wrote; it
just stops adding groups they didn't. The gaps come back as Rule 14/15/17 findings,
which is the point — the caller can see what they declined. Pinned at both layers
(`autofix_does_not_synthesise_by_default` in Rust,
`test_synthesis_is_off_by_default` in Python), because the *absence* of a
behaviour needs a test as much as its presence does.

The derivable/authorial boundary is unchanged and worth restating: UNIT and TYPE
are pure functions of the data, ABBR comes from the standard table and only when PA
codes are used, TRAN is a placeholder stub. **PROJ and DICT are never synthesised**
— a project identity and a schema extension are authorial facts, and a guessed
`DICT_PGRP` would turn a loud Rule 18 error into a silent false statement about the
data model that Rule 10's relational checks then trust.

**Status: landed 2026-07-24**, ahead of 0.8.0 so the default does not move after a
release. The docs examples (`ex09a`/`ex09b`) were rewritten to demonstrate both
paths rather than narrate the old one — showing the findings first and the opt-in
second teaches the boundary better than the previous one-call version did.

## Update (2026-06-25): the typed-graph door emits only the headings you set

The typed-graph walk used to emit *every declared heading* of each class (null →
empty cell). Because the typed classes are generated from the **union** dictionary,
a sparse node (e.g. `LOCA(loca_id=…, loca_gl=…)`) dragged in ~45 blank columns —
and the unset **edition-specific** headings tripped Rule 9 at the default 4.1.1,
while the unset **PA** columns tripped Rule 16 (ABBR). So `build_ags4(typed graph)`
came back with findings even though the data was fine.

Fixed by pruning entirely-unset columns in **both walks** (Python
`repo:packages/laterite/python/laterite/__init__.py::_typed_graph_to_items`, Node
`repo:rust-packages/laterite-node/ts/index.ts walkTree`), so the typed-graph door
now emits only the headings you set — exactly like the frames door — and a sparse
graph builds **clean at the default edition**. KEY headings are always kept (a
missing key must be flagged, not dropped); a heading set to `""` survives (it's a
value); a deliberately-set edition-specific heading is kept and correctly flagged
(no silent data loss). The choice (default-prune; an opt-in "full template" mode is
deferred) was the owner's, 2026-06-25.

## Shape: one core, one orchestrator, two thin bindings

```
                         ┌───────────────────────────────────────────────────────┐
   PyO3:  Arrow tables ──┤  emit_ags4(groups, dict, opts)            ── NEW (small)│
   (polars/pandas)       │   ├─ format each typed cell -> AGS4 string             │
                         │   │     via laterite_types::ags4_str            ── SHIPPED  │
   wasm:  JSON / Arrow ──┤   ├─ fill UNIT/TYPE from laterite-ags4-core::registry ── SHIPPED │
   (browser)             │   │     where the caller omits them                    │
                         │   ├─ laterite-ags4-core::ags4_writer::write_ags4     ── SHIPPED  │
                         │   └─ opts: Strict | Report | AutoFix                    │
                         │        (AutoFix chains validate + apply_fixes ── SHIPPED)│
                         └───────────────────────────────────────────────────────┘
                                                  └──► valid AGS4 bytes (+ findings)
```

## What already exists (reuse — pinned 2026-06-12)

- **Emitter:** `laterite-ags4-core::ags4_writer::write_ags4(out, &[EmitGroup])` — byte-faithful (CRLF,
  every field quoted, `"`→`""`, UNIT padded to heading width, TYPE defaulting `"X"`). Wasm-safe.
  `EmitGroup { code, headings, units, types, rows: Vec<Vec<String>> }`.
- **Typed→string formatter:** `laterite_types::ags4_str(value: &serde_json::Value, ags_type) ->
  String` (lib.rs:128) — the **inverse of `parse_value`**: Null→`""`, YN→`Y`/`N`, DT
  date/precision normalization, `0DP`→int, `nDP`→`{:.n}`, `nSF`→sig-figs fixed-point,
  `nSCI`→scientific, else passthrough. Already in the **wasm-safe `laterite-types`** crate that
  `laterite-ags4-wasm` already depends on; well-tested. Plus `truncate_dt_to_unit` (lib.rs:234) for
  DT-vs-UNIT precision (Rule 8).
- **UNIT/TYPE source:** `laterite-ags4-core::registry` — standard headings/units/types per group.
- **Validity:** `validate`, `compute_fixes`, `apply_fixes` — all shipped (native core + wasm
  exports). The "validate-and-fix the *generated* output, not the raw file" approach chains
  these after the build.
- The only DuckDB-bound helper (`value_to_json`, ags5db/db.rs) is **not needed** — the emit
  path converts Arrow/JSON cells to `serde_json::Value` (a pure adapter), which `ags4_str`
  consumes. For wasm JSON input the cells *are* serde values → fed to `ags4_str` directly.

So the formatter **and** the emitter are done; the new code is the orchestrator + bindings.

## The new pieces (small)

1. **`emit_ags4(groups, dict, opts) -> EmitResult`** — builds `EmitGroup`s (formatting typed
   cells via `ags4_str`, filling UNIT/TYPE from the registry where omitted), calls `write_ags4`,
   then per `opts`. `EmitResult { bytes, findings, fixes_applied }`. (Crate placement open —
   see below.)
2. **PyO3:** `emit_ags4_from_arrow(tables: Vec<(String, PyTable)>, dict, mode) -> bytes` —
   frames in as Arrow (zero-copy capsule, the read path reversed), all-Rust out. Replaces
   `compat`'s `_matrix_from_df` Python loop; `Ags4File.write` on a *read* file keeps the cheaper
   `Reading::emit` fast path.
3. **wasm:** `to_ags4(groups, dict, mode) -> {bytes, findings}` — JSON (MVP) or Arrow IPC
   (columnar; the read path's IPC reversed). The browser wraps `bytes` in a Blob to download.

## Data-in

MVP / small (PROJ·LOCA·SAMP·GEOL·ISPT), JSON:
```jsonc
[ { "code": "LOCA",
    "headings": ["LOCA_ID", "LOCA_NATE", "LOCA_GL"],   // units/types optional -> dict fills
    "rows": [["BH01", 523145.10, 12.34], ["BH02", 523200.0, 13.0]] } ]
```
Big / columnar: **Arrow IPC** (browser) or the **Arrow capsule** (native, zero-copy) — symmetric
with the read boundary (read = AGS→Arrow→host; this = host-Arrow→AGS).

## Validity options (`opts`)

- **Strict** — enforce dict units/types/required-KEYs on construction; reject bad input up front.
- **Report** — build + validate, return findings, no mutation (the caller fixes).
- **AutoFix** — build + validate + apply the *safe* mechanical fixes, emit compliant (owner's
  lean). Risky fixers (e.g. DATETIME canonicalisation) stay opt-in. Not exclusive — a UI can do
  Report and offer an AutoFix button.

## Deployment — RESOLVED: both (2026-06-12)

**Both native (PyO3) and wasm.** Python does not run in wasm (no Pyodide here), so the two are
separate frontends over the *one* shared Rust emitter:
- **Server-side (native):** a Python backend holds the pandas/polars df → emits via PyO3 → `.ags`
  bytes (download / POST to the other system). Cheaper, faster, mostly built.
- **Client-side (wasm):** the browser builds AGS4 from JSON/Arrow via the new `to_ags4` — the
  offline / no-server-round-trip option.

The shared Rust core means "both" is barely more than one — same orchestrator, two ~20-line
bindings.

## Remaining open questions (proposed defaults — say otherwise)

- **Validity mode default:** `AutoFix` (build → validate → apply the *safe* fixes → compliant),
  `Report` available. (Owner's lean.)
- **UNIT/TYPE source:** **Hybrid** — dict-fill for known headings, caller's explicit values
  override (for custom / non-standard headings).
- **AGS4 edition:** selectable per call, default `4.1.1`; "valid" = passes the AGS4.x rule set
  for the chosen edition (strict), since the receiving system imports it.
- **MVP group scope:** PROJ · LOCA · SAMP · GEOL · ISPT (SPT); related groups supplied as flat
  per-group tables, wired by `{group}_ID` (the `at()`-style key convention).

## Build plan (phased)

> **P0–P4 ✅ DONE (2026-06-13)** — see *As built* above (P3 = wasm `to_ags4` JSON +
> `to_ags4_ipc` Arrow IPC, over a shared `laterite-ags4-emit` transpose; P4 = the web Export tab).
> **The feature is complete.**

- **Phase 0 ✅ — crate placement (decide first; see below).** Determines `laterite-ags4-wasm`'s dependency,
  so it gates everything. *Built: the `laterite-ags4-emit` leaf.*
- **Phase 1 — the shared orchestrator (Rust, host-agnostic).** `emit_ags4(groups, dict, opts)
  -> EmitResult` + the `GroupInput` / `RowData` (typed-columns | string-cells) model: format each
  cell via `ags4_str`, fill UNIT/TYPE from the registry where omitted, `write_ags4`, then
  `Strict | Report | AutoFix` (chain `validate` + `apply_fixes`). Flagship test: a build → parse
  → build round-trip property test + "the AutoFix output validates clean".
- **Phase 2 — PyO3 binding + native cleanup.** `emit_ags4_from_arrow(tables, dict, mode)` (Arrow
  capsule in, zero-copy). Rewire `compat.dataframe_to_AGS4` and `Ags4File.write` onto it, dropping
  the `_matrix_from_df` Python loop (all-Rust native write). Gate: byte-identical to today's
  compat write; python-ags4 parity stays 122/131.
- **Phase 3 ✅ — wasm binding.** `to_ags4(groups_json, …)` (JSON) **and** `to_ags4_ipc(groups, …)`
  (Arrow IPC, columnar) → `{text, findings, fixes_applied}`, over the shared `laterite-ags4-emit` transpose.
  Tests in the wasm crate (build → re-parse → validate, incl. an IPC round-trip); a real
  `wasm32-unknown-unknown` build proves `laterite-ags4-emit` + its `arrow` feature are wasm-safe; `to_ags4`
  verified live in Chrome.
- **Phase 4 ✅ — web app integration.** The validator site's **Export tab** (SolidJS `ExportPane`):
  JSON editor → `to_ags4` (via the worker bridge) → `downloadBlob`. e2e in `web/e2e/export.spec.ts`.
  Bumped the PWA precache limit 3 → 4 MiB for the grown wasm (see *As built* → P4).

Each phase lands green (Rust tests, clippy/ruff, python-ags4 parity, wiki lint). Performance is a
non-issue at site scale — the emitter is microseconds; the only cost is the data boundary,
minimised by Arrow for columnar input.

## Crate placement (Phase 0 — decide before code)

The orchestrator needs a wasm-safe home, and `laterite-ags4-wasm` currently depends only on
`laterite-ags4-validator` + `laterite-types` (NOT `laterite-ags4-core`, where `ags4_writer` + `registry` live). Two
options:
- **(recommended) Extract a small wasm-safe `laterite-ags4-emit` leaf** holding `write_ags4` (moved from
  `laterite-ags4-core`) + `emit_ags4`, depending on `laterite-types` (`ags4_str`) + the registry. `laterite-ags4-core`
  (its excel / db-export paths), `laterite-ags4-wasm`, and `laterite-py` all depend on it. Keeps
  `laterite-ags4-wasm`'s dep graph lean (no pulling all of `laterite-ags4-core`, whose `excel` / `transport` may
  not be wasm-safe) **and** is the first concrete step of resolving the `laterite-ags4-core` naming / scope
  smell (AGS4 machinery lifted into its own leaf).
- **Put `emit_ags4` in `laterite-ags4-core`** + make `laterite-ags4-wasm` depend on `laterite-ags4-core` — simplest, but
  needs *all* of `laterite-ags4-core` to compile to wasm (gate `excel` / `transport` behind features if
  not), and it deepens the naming smell rather than relieving it.

Recommendation: the leaf. Settle this first — it's the one architectural decision the rest hangs
on.
