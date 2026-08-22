---
type: decision
title: "The reliquary — the decision record for anything that looks removable"
status: accepted
tags: [design, decision, process, relics]
decided: 2026-06-19
supersedes: []
from_gap: []
related: [api-surface-1.0, ags4-output, mutation-sweep]
sources: []
---

# The reliquary — the decision record for anything that looks removable

The single living register of code/doc relics, per the *"Surgical, targeted changes"*
convention — the ones **still present** and the ones **already gone**. A relic is
**inventoried here first**, then **expunged in the PR that retires the feature it belongs
to** — with its now-redundant tests, **individually, never an opportunistic sweep**.

**Three verdicts, and the middle one is not a holding pen.** `spotted` is removal debt
waiting on a trigger. `removed` is the receipt — kept, not pruned, because "we already
looked at this and it is gone" is the answer to a question somebody will ask again.
`keep` is a **reviewed retention**: an item that reads as dead to every scout and is not,
with the reason written down. That last class is the register's highest-value content and
the easiest to mistake for an oversight — `engine_fingerprint()` has no caller in this
tree and is required by [[cert-trust-v2]]; `version()` has no *web* caller and a live
harness one. Each of those rows exists so the next person does not re-chase a call graph
that has already been chased and answered.

So the register is not a list of pending deletions with some exceptions in it. It is the
**decision record for anything that looks removable**, whatever the decision was.

> **Discipline:** removing a relic is part of the work of the PR that supersedes it, not a
> later batch cleanup. Add a row when you *spot* one; flip its status when the superseding
> PR lands. See [[api-surface-1.0]] for the surface most of these live on.

## Register

| # | Relic | Location | Superseded by | Remove in | Status |
|---|---|---|---|---|---|
| 1 | `.sql()` returns a raw DuckDB relation (can't fluent-chain) | `laterite/__init__.py` | the `AgsQuery` lazy view (`.query()`) | **PR B (#178)** — `.query()` added; `.sql()` kept for BC by design | mitigated |
| 2 | `Ags4File.write()` | `laterite/__init__.py` | `.save(path)` | **feat/api-vocab-python** | removed |
| 3 | `_AgsSubset` proto view | `laterite/__init__.py` | unified `AgsQuery` | **PR B (#178)** | removed |
| 4 | `read_ags_text` "PROTOTYPE/experiment" doc | `niko86/laterite-duckdb` `src/read_ags.rs` (ext repo) | it's a shipped stable-subset fn | resolved in the native-only rework (pre-PR-D) — doc no longer present | removed |
| 5 | `Ags4File.to_numeric` (redundant; born-typed) | `laterite/__init__.py` | born-typed columns | **PR B (#178)** | removed |
| 6 | Two parsers (`ags4_codec` vs validator `parse.rs`) | core vs `laterite-ags4-validator/src/parse.rs` | parser convergence (shared `laterite-ags4-parse` leaf, #168) | Phases 0–6 shipped: leaf built; validator + py/node/wasm/perf parse through it; core's `index` (P4, O-40) AND read path `ags4_codec` (P5, `from_shared`) source from it; **both historical parsers retired, `csv` gone from core**; **P6 = the DuckDB ext already runs on the converged core (its pinned core has `from_shared`; laterite-duckdb #10 built it green)**. **P7 shipped (#348, green) — #168 COMPLETE:** the tautological `parse_parity.rs` retired → `from_shared_trim.rs` (keeps only the real trim-asymmetry guard, fork 1); the lean non-UTF-8 wording pinned (`error_mapping.rs`) + **ratified as O-46** (owner-confirmed 2026-07-03: lean read path rejects, validator decodes lossily→Rule 1 per O-32); python-ags4 parity UNCHANGED (122/9); corpus dogfood clearance green (801-file censored set: 0 hard-errors, 0 panics — behaviour-neutral P7 so representative). | removed |
| 7 | `read_typed(attachments_dir=)` no-op arg | `laterite/ags4.py` | intentional signature-stability anchor | next signature-breaking major | anchor (keep) |
| 9 | Module-level `laterite.write(obj, path)` | `laterite/__init__.py` | `.save()` method + the chain | **feat/api-vocab-python** | removed |
| 10 | `source()` load alias | `laterite/__init__.py` | `read()` (the cross-surface load verb) | **feat/api-vocab-python** | removed |
| 11 | `emit_ags4` / `EmitResult` names | `laterite/__init__.py` | `build_ags4` / `BuildResult` | **feat/api-vocab-python** | removed |
| 12 | Cross-surface naming drift: node `.write` / module `write`; wasm `parse`, `load_ags_script`(?); duckdb/wasm `to_ags4` | `laterite-node`, `web/` wasm, duckdb ext | the locked vocab — `read` / `save` / `build_ags4` / `.text` / `.bytes` | node PR (#180), wasm PR (#181), duckdb-ext (read_ags/validate_ags/certify_ags) | **removed** — verified 2026-06-20: node public `read`/`validate`/`buildAgs4`→`BuildResult`(`.save`/`.text`/`.bytes`); wasm `read`/`validate`/`build_ags4`; duckdb SQL idiom `read_ags`/`validate_ags`/`certify_ags` **at the time**. Superseded 2026-07-08 (PR #446): the duckdb-ext's `validate_ags`/`certify_ags` were themselves removed in the extension's 0.7.0 read-only rework — duckdb's settled surface is now `read_ags`/`ags_groups`/`ags_headings`/`ags_dictionary`/`ags_relationships`/`load_ags` only. The napi `parseArrow`/`emitAgs4FromIpc`/`EmitResult` names are auto-generated native plumbing the curated TS layer wraps — not public vocab | <!-- historical -->
| 13 | One-time wiki bootstrap pipeline: `generate.py` scaffolder + `ingest_phase{A,A_cont,B,C,D,F}.py` + `ingest_headings.py` + `phase{A,A_cont,B,C,D,F}.json` (14 files) | `ags-wiki/.bootstrap/` | vault fully bootstrapped (Campaign A–E complete 2026-05-16, 372 pages); steady-state = `reindex.py` + `lint.py` only. All 14 orphaned (only `log.md` history) and unrunnable (pre-rebrand `rust-packages/ags5db/` path, retired `ags5-models`/`ags4-validator` crates) | **docs/wiki-portable-blueprint** — `git rm` + AGS-WIKI §14 retirement note | removed | <!-- historical -->
| 14 | Two metadata-group synthesizers: emit's `synthesize_metadata`/`collect_units`/`collect_types`/`collect_abbreviations` (UNIT/TYPE/TRAN/ABBR, shipped) vs forge's `collect_unit`/`collect_type`/`tran`/`collect_abbr` (fuzz fixtures) | `laterite-ags4-emit/src/emit.rs` + `laterite-ags4-forge/src/synth/model.rs` | a shared catalog-synthesis helper (a future leaf extraction) | not yet — emit's fills gaps in a real build, forge's generates whole synthetic files; converge only if they drift (see [[ags4-output]]) | spotted |
| 15 | `tools/coverage-combined.sh` + the `ci/combined-coverage` local branch (instrumented the maturin `.so` so pytest credited the PyO3 glue → one honest combined Rust coverage number) | local branch only (never on `master`; head `094e4c5`, reflog-recoverable ~90d) | the `coverage` job **excludes** `laterite-py` from the Rust floor instead of instrument-and-credit; its main driver (`ags5db` Python-tested Rust) was decoupled to `ags5/` (#177) | **chore/retire-combined-coverage** (2026-06-26) — stale (218k behind master; `cd packages/laterite-ags5` path moved to `ags5/`), unused on master, owner confirmed not useful | removed |
| 16 | Two byte-identical zstd+age transport copies (`pack`/`unpack`/`lock`/`unlock` + `encrypt`/`decrypt_with_passphrase`) | `laterite-ags4-core/src/transport.rs` + `laterite-node/src/transport_fns.rs` | the shared `laterite-transport` leaf (#327) — one copy of the age/zstd logic + its own `TransportError`; core re-exports it (mapping `→CliError`) so `laterite-py` is unchanged, node binds it directly (`→napi::Error`). Same convergence-leaf pattern as #6 | **feat/transport-leaf (#327)** — the age 0.10→0.11 migration having to touch **both** copies was the dup coming due; extracted, node's now-untestable+redundant Rust `#[cfg(test)]` block dropped (leaf owns the round-trip tests; vitest `p3-transport` owns the napi boundary). The issue's "lat `commands/lock.rs`" was stale — no CLI lock command exists | removed |
| 17 | `validate-anywhere.md` (the #201 synced-tabs prototype page) with three **hand-inlined**, ungated snippets | `web/docs-site/docs/validate-anywhere.md` | the cookbook's `validate-a-delivery.md` — 5 gated tabs (Python/Node/DuckDB/CLI/Browser), every snippet single-sourced from the `examples/` trees | **feat/docs-380-recipe-tabs (#380)** — its hand-inlined snippets were the drift source #379 had to fix (2 of the 3 drift bugs lived here *because* they weren't gated); the cookbook recipe fully supersedes it; its one inbound link (`surfaces/index.md`) repointed, nav entry removed | removed |
| 18 | `.row-link` / `.row-note` component classes (styled the destination-rows shim list) | `web/landing/landing.css` `@layer components` | #395 replaced the shim list with the real landing page; repo-wide grep finds no reference outside the definitions | **feat/landing-relic-535 (#535)** — inventoried by the issue, removed on its own per the relic discipline | removed |

## Machine-checked register

Unlike the narrative register above (a historical log), rows here are **verified
by `lint.py` against the tree** (HARD check `reliquary register out of sync`). Each
carries a `Verify` anchor `path::token` (a literal declaration-fragment scoped to
one file). The gate asserts **`removed` ⇒ token absent** at that path, and
**`spotted`/`keep`/`mitigated` ⇒ token present** — so a relic can't be marked
removed while its code lingers, nor silently vanish while still listed present.
Seeded from the 2026-07-11 `workspace-bloat-audit` (find-only). Status vocab:
`spotted` = inventoried, still present, scheduled for removal; `keep` = reviewed,
deliberately retained; `removed` = expunged (flip the row in the PR that removes
it, and name that PR in *Removed-in*). Verify `—` = display-only.

<!-- BEGIN GENERATED: reliquary-register — from ags-wiki/design/reliquary.json; regenerate with `uv run --no-project python tools/gen_wiki_tables.py` (DO NOT EDIT THE TABLE BY HAND) -->
| Symbol | Verify | Axis | Status | Removed-in | Evidence |
|---|---|---|---|---|---|
| `truncate_dt_to_unit()` | `rust-packages/laterite-ags4-types/src/lib.rs::fn truncate_dt_to_unit` | code | removed | misc-deadcode PR | AGS5 DT-precision writer; 0 shipped callers (only ags5/) |
| `LineTerminator::as_bytes()` | `rust-packages/laterite-ags4-parse/src/lib.rs::as_bytes(self)` | code | removed | misc-deadcode PR | unused ergonomic twin of `as_str()`; 0 call sites |
| `tempfile` (dev-dep) | `rust-packages/laterite-transport/Cargo.toml::tempfile` | dependency | removed | dep-hygiene PR | declared, never used; tests use `env::temp_dir()` |
| `chrono` (dep) | `rust-packages/laterite-ags4-core/Cargo.toml::chrono` | dependency | removed | dep-hygiene PR | 0 chrono/DateTime uses in core src/tests |
| `CliError::NotImpl` | `rust-packages/laterite-ags4-core/src/error.rs::NotImpl` | code | removed | core-relics PR | ags5db stub variant; last use retired even in ags5 |
| `GroupDescriptor::non_key_headings()` | `rust-packages/laterite-ags4-core/src/registry.rs::fn non_key_headings` | code | removed | core-relics PR | ergonomic pair of `key_headings()`; 0 shipped callers |
| `Heading::py_name()` | `rust-packages/laterite-ags4-core/src/registry.rs::fn py_name` | code | removed | core-relics PR | ags5 DuckDB column-alias helper; 0 shipped callers |
| `count_by_group()` | `rust-packages/laterite-ags4-validator/src/findings.rs::fn count_by_group` | code | removed | misc-deadcode PR | only its own unit test calls it |
| `comfy-table` (dep) | `rust-packages/laterite-cli/Cargo.toml::comfy-table =` | dependency | removed | dep-hygiene PR | 0 direct use; served via `laterite-cliutil` |
| `indicatif` (dep) | `rust-packages/laterite-cli/Cargo.toml::indicatif =` | dependency | removed | dep-hygiene PR | 0 direct use; served via `laterite-cliutil::Spinner` |
| `SelfCheck.ok` field | `rust-packages/laterite-ags4-parity/src/oracle.rs::pub ok: bool` | code | removed | misc-deadcode PR | hardcoded `true`; never read |
| `rayon` (dep) | `rust-packages/laterite-ags4-forge/Cargo.toml::rayon` | dependency | removed | dep-hygiene PR | full parallelism crate, never called |
| `registry_get_group()` | `rust-packages/laterite-py/src/registry_fns.rs::fn registry_get_group` | code | removed | python-registry PR | dead PyO3 export; py loads via bulk JSON |
| `Heading.indexed` field | `packages/laterite/python/laterite/registry.py::indexed` | code | removed | python-registry PR | hardcoded `None` in registry.rs; never populated |
| `GroupDescriptor.index_parent` field | `packages/laterite/python/laterite/registry.py::index_parent` | code | removed | python-registry PR | hardcoded `None` in registry.rs; never populated |
| `CanonicalType::Date`/`Time` variants | `rust-packages/laterite-ags4-types/src/lib.rs::CanonicalType::Time` | code | keep | — (owner call) | unreachable via `canonical_type()` but public + matched across 3 bindings — breaking to remove |
| `laterite-ags4-parse` `serde` feature | `rust-packages/laterite-ags4-parse/Cargo.toml::serde` | dependency | removed | ags5-relics PR | dead #168 scaffold, 0 activators anywhere; dispense-with-ags5 sweep (#177 concept) |
| `CliError` ags5-compat variants | `rust-packages/laterite-ags4-core/src/error.rs::PreVersion65` | code | removed | ags5-relics PR | `PreVersion65`/`UnknownGroup`/`Predicate`/`UnsupportedFeature`/`Sql`/`Validation` — 0 shipped construction (ags5/ only); `CliError` now `FileNotFound`+`Schema` |
| `Registry::extended_with()` | `rust-packages/laterite-ags4-core/src/registry.rs::fn extended_with` | code | removed | ags5-relics PR | passthrough-registration; 0 shipped callers (2 in ags5/) |
| `heading_storage_index()` | `rust-packages/laterite-ags4-core/src/registry.rs::fn heading_storage_index` | code | removed | ags5-relics PR | ags5 parent-overlap-dedup helper; 0 shipped callers (2 in ags5/) |
| `parse.rs` `pub use` block (6 dead names) | `rust-packages/laterite-ags4-validator/src/parse.rs::parse_bytes_opts` | code | removed | ags5-relics PR | `InvalidUtf8`/`LineSpan`/`LineSpans`/`LineTerminator`/`ParseOptions`/`parse_bytes_opts` re-exported but imported by nobody; live names kept |
| `Dictionary::group_count()`/`heading_count()` | `rust-packages/laterite-ags4-validator/src/dict.rs::fn group_count` | code | removed | ags5-relics PR | test-only; call-sites inlined to `.groups.len()`/`.headings.len()` |
| `crossterm` (opt dep) | `rust-packages/laterite-cli/Cargo.toml::crossterm` | dependency | keep | — (owner call) | reached via `ratatui::crossterm`; needed by the `tui` feature gate |
| `DuckParse::surface` field | `rust-packages/laterite-ags4-compliance/src/bin/duckdb_parse_check.rs::surface: String` | code | removed | ags5-relics PR | `#[allow(dead_code)]`; deserialized, never read |
| `_ordered()` (modality helper) | `tools/gen_modality.py::def _ordered` | code | removed | the modality drift-gate PR | vulture-surfaced 2026-07-20, verified dead: 0 references repo-wide (module *or* test), superseded by `_columns()`/`_grid()`. Trivial standalone leftover — safe to drop in any pass that touches `gen_modality.py` — dropped 2026-08-04, in the pass its own note nominated |
| `include_warnings: true` (diff) | `rust-packages/laterite-cli/src/commands/diff.rs::include_warnings: true` | code | removed | mutation-sweep cleanup (coverage-rust) | inert survivor 2026-07-27: `diff` never runs the rule engine, so `opts.include_warnings` was never read — cargo-culted from `validate`/`fix`. Collapsed to `CheckOptions::default()`. See [[mutation-sweep]] |
| `include_warnings: true` (merge) | `rust-packages/laterite-cli/src/commands/merge.rs::include_warnings: true` | code | removed | mutation-sweep cleanup (coverage-rust) | same inert pattern as `diff` — `merge` parses/reconciles but never runs the rule engine, so `opts.include_warnings` was never read. Collapsed to `CheckOptions::default()` |
| `GroupDescriptor::table()`/`view()` | `rust-packages/laterite-ags4-reference/src/union.rs::fn table` | code | removed | #175, re-removed in #189 | DuckDB `g_<code>`/`v_<code>` name builders on the reference-data leaf. Verified dead 2026-07-29: the only occurrences repo-wide were the two fns and their own unit test, and `laterite-duckdb` never references `GroupDescriptor` — it builds no `g_`/`v_` names at all. Publishing would have frozen an unrelated product's naming into a dictionary crate's public API. Deleted in #175; **#178 restored them** (it branched before #175 and squash-merged after, so its diff undid the removal — the same revert that took out the #176 crate rename). Re-deleted in #189. See [[dec-rust-api-crates-io]], laterite#161 |
| `_template-requirement.md` — the `requirement` page class | `ags-wiki/templates/_template-requirement.md::AGS5 req` | doc | removed | the PR closing #500 | AGS5-strand page class with **0 live pages** (`type: requirement` outside `templates/`: 0). Not a plain delete: `lint.py` builds `TEMPLATE_SCHEMA` *from* each `_template-<class>.md`, so removing the file removes the schema for its `type:` — a vault-wide change, not a file removal. The AGS5 strand is dormant and lives in the private satellite, so its page classes are residue here. Spotted 2026-08-12 while removing the AGS5 title from `_template-decision.md`; templates are invisible to the linter (`SKIP_DIRS` includes `templates`, deliberately — they are its schema source), which is why this survived. See [[dec-monorepo-structure]] **Retired #500:** template deleted, and with it the `TEMPLATE_SCHEMA` entry it was the source of. The hand-declared halves went too — `STATUS_VOCAB` in `lint.py`, `CLASS_ORDER` in `librarian.py`, the §3 class list and §4 vocabulary in `AGS-WIKI.md`, and `design/_README.md`'s page-kind list, dataview block and traceability section. Live pages of the class at the time: none. |
| `_template-experiment.md` — the `experiment` page class | `ags-wiki/templates/_template-experiment.md::AGS5 experiment` | doc | removed | the PR closing #500 | Same shape as the `requirement` class above: **0 live pages** (`type: experiment` outside `templates/`: 0), AGS5-worded throughout (its `evidence` field still points into the decoupled `packages/ags5-` tree), and load-bearing as a `TEMPLATE_SCHEMA` source until the class itself is retired. Retire both together or neither. Spotted 2026-08-12 **Retired #500:** template deleted, and with it the `TEMPLATE_SCHEMA` entry it was the source of. The hand-declared halves went too — `STATUS_VOCAB` in `lint.py`, `CLASS_ORDER` in `librarian.py`, the §3 class list and §4 vocabulary in `AGS-WIKI.md`, and `design/_README.md`'s page-kind list, dataview block and traceability section. Live pages of the class at the time: none. |
| `list_rules()` (wasm) | `rust-packages/laterite-ags4-wasm/src/metadata.rs::pub fn list_rules` | code | keep | — (published surface) | Inventoried by #349 as having **no web caller** (0 hits in `web/src`, `web/e2e`, `web/scripts` — verified 2026-08-18; the Rule explainer reads `public/rules-catalogue.json`, synced by `scripts/sync-rules.mjs`). Not a relic: it is part of the **published** `@laterite/ags4-wasm` surface, named in `EXPECTED_FUNCTIONS` in BOTH `tools/release/check-wasm-slim.mjs` and `check-wasm-tier1.mjs`, documented available in `web/docs-site/docs/reference/wasm-api.md`, with a cookbook page and node/py twins. "No UI caller" is not "no caller" — removing it is an npm breaking change, not a cleanup. |
| `version()` (wasm) | `rust-packages/laterite-ags4-wasm/src/metadata.rs::pub fn version` | code | keep | — (live caller) | Inventoried by #349 as having no web caller — **true, and misleading**: it has a live NON-web caller. The wasm compliance runner calls it (`glue.version?.()`, the satellite's `tools/compliance/emit_js.mjs`), which is the exact purpose its own docstring records — it exists because that harness had hard-coded `version: "0.5.1"` and printed a false cross-surface identity (laterite-dev#556). Also in both release gates' `EXPECTED_FUNCTIONS` and the published API docs. Deleting it would re-open the bug it was added to close. |
| `engine_version()` (wasm) | `rust-packages/laterite-ags4-wasm/src/metadata.rs::pub fn engine_version` | code | keep | — (cert-trust, prospective) | No web caller (verified 2026-08-18). Retained on two independent grounds. (1) **Prospective feature, not dead code**: [[cert-trust-v2]] defines engine identity as a tuple `EngineId = (validator, validator_version, engine_fingerprint, compat)`, and this export is the browser's route to the validator's own version — a browser-minted certificate needs it. (2) It is in `EXPECTED_FUNCTIONS` in both release gates and the published API docs. Its docstring is candid that it is "useful for humans, useless as an identity" on its own — the identity claim belongs to the fingerprint below; the two are recorded together because a future cert-trust build reads them as a pair. |
| `engine_fingerprint()` (wasm) | `rust-packages/laterite-ags4-wasm/src/metadata.rs::pub fn engine_fingerprint` | code | keep | — (cert-trust, prospective) | No web caller (verified 2026-08-18). **A missing feature, not a dead one** — the case #349 asked to be checked before removal, and the check says keep. [[cert-trust-v2]] names `engine_fingerprint` in `EngineId` and requires its equality for a cert to be trusted; it is a build-time SHA-256 over every rule source, the dictionary and the rules catalogue, so it is the only thing that can show two surfaces agree on the RULES rather than merely shipping together. Deleting it would have to be walked back the moment browser-minted certs land. Also in both release gates' `EXPECTED_FUNCTIONS`, the published API docs, and `tools/xcheck/`. |
| `build_ags4_ipc()` (wasm) | `rust-packages/laterite-ags4-wasm/src/build.rs::pub fn build_ags4_ipc` | code | keep | — (published, feature-gated) | Inventoried by #349 as never imported by either worker — true (the app's Export pane uses the JSON door `build_ags4`). Not a relic: it is the **Arrow** door of the same verb, gated behind the `arrow` feature, documented in `wasm-api.md` as "build from source" and in `surfaces/browser.md`, and carried in the cross-surface modality register (`modality.json`, held by `packages/laterite/tests/test_modality_parity.py`) — so removing it would drop a modality row and fail that gate. `check-wasm-tier1.mjs` names it among the exports whose ABSENCE defines tier 1, which is a dependency on the name too. |
| `dictionary()` (wasm) | `rust-packages/laterite-ags4-wasm/src/dictionary.rs::pub fn dictionary` | code | keep | — (published surface) | Made caller-less on the web by **this** change (#349): the app's Dictionary browser and Template generator read the static union `ags_dictionary.json` via `lib/dict.ts`, which #349 settled as the web's single dict source, so the worker op and client fn were removed. Recorded here per the relic discipline — add the row in the PR that strands the export — but `keep`, on the same grounds as its four siblings above: it is named in `EXPECTED_FUNCTIONS` in BOTH `tools/release/check-wasm-slim.mjs` and `check-wasm-tier1.mjs` and documented available in `web/docs-site/docs/reference/wasm-api.md`, so it is published npm surface. Its TYPE is still load-bearing web-side even with no caller: `web/src/lib/validator.ts` re-exports `StandardDict` from the engine so the local projection must keep conforming to what this export returns. |
| `laterite_ags4_validator::is_valid()` | `rust-packages/laterite-ags4-validator/src/lib.rs::pub fn is_valid` | code | spotted | the release that bumps the engine tier off 0.9.0 | Renamed to `is_clean` in #321, when a warning stopped failing a file and "zero findings" stopped being the verdict — two public `is_valid` in one crate that disagree on a warning-carrying file. Kept as a `#[deprecated]` delegating alias so the rename reaches a crates.io consumer as a warning naming both replacements rather than a build failure; 0 callers in this tree. |
| `.github/workflows/allocator-bench.yml` + `tools/bench/allocator_ab.py` | `.github/workflows/allocator-bench.yml::allocator-bench (temporary, #448)` | code | removed | the PR that recorded the #448 measurement (runs 32418268428 / 32421189772) | Added deliberately temporary and inventoried in the same change; removed in the PR its own row named. It was an INSTRUMENT, not a gate — `workflow_dispatch`-only, never on a push, a PR or a schedule — and it answered exactly one question: on Linux and Windows, does mimalloc v3 beat the v2 pin by enough to narrow that pin to `cfg(target_os = "macos")`? Measured on both self-hosted runners at the 25 MB rung: no. The system-allocator control separated by 36.2% on Linux and 62.6% on Windows, so the harness demonstrably resolved allocator effects — and against that, the v2/v3 gap sat inside the noise floor on both, pointing in OPPOSITE directions (v3 3.3% slower on Linux, 1.7% faster on Windows). The answer is on #448; the harness that produced it has nothing left to measure. Kept past that point it would be a build cost nobody could justify and nobody would dare delete. |
| `tests/p1-flagship.cjs` (node) | rust-packages/laterite-node/tests/p1-flagship.cjs::P1 — the native engine core | code | spotted | not yet — the PR that next touches the node surface's test wiring | A pre-vitest CommonJS smoke script, sole occupant of `tests/` — and `vitest.config.ts` reads `test/`, one letter away, which is why nothing has run it and nothing noticed. Zero references repo-wide (no `package.json` script, no workflow, no doc); it arrived in a pre-flip `sync: public tree` commit and no commit has touched it since. What it asserts — parse→Arrow IPC, validate, byte-faithful emit, data→AGS4 — is covered by the `test/` suite that replaced it, so this is a superseded copy rather than lost coverage. Found by `knip` (the only genuine orphan in the 2026-08-21 three-scout pass; vulture's four `tools/` hits all had live callers in `tests/`, and web's 70 knip hits are overwhelmingly the #418 Solid primitives, built ahead of the app restyle they are for). **Confirm before deleting**: a file vitest cannot see is exactly the shape that reads as dead while being someone's manual smoke test. |
| `feeds_ags5_req` (insight + strategy frontmatter) | ags-wiki/templates/_template-insight.md::feeds_ags5_req | doc | removed | the PR retiring the field with its last reader | Stranded by #500: the field fed the AGS5 requirement register, which that change retired. Empty on every page that carried it, so it had never pointed at anything. Deferred at the time as a separate unit of work, then done once the deferral was re-examined and found to rest on a wrong count AND a missed reader. THE COUNT: this row said "31 insight pages". It was 29 pages across TWO classes — 18 `insight`, 11 `strategy` — plus two templates, and `_template-strategy.md` carried the field as well, which the row never mentioned. 31 was the file count read as a page count. THE READER: `insights/_README.md` had a live dataview block, "Confirmed gaps feeding AGS5", selecting the column — so the field was not inert, it had a query pointing at it, and #500 missed it by grepping that file for the retired CLASS names rather than for the mechanism. Removed with it. `feeds_strategy`, its sibling, IS live and stays. |
<!-- END GENERATED: reliquary-register -->

## Finding relics — tooling

The register above is curated by hand, but three **scouts** surface candidates so a
relic can't hide just because nobody went looking. Each is a *scout, not a gate*:
it prints candidates, a human confirms against the real call graph, and only a
confirmed relic earns a row above. **Never delete on a tool's say-so** — every one
of these tools is blind to some caller, so a hit is a *question*, not a verdict.

| Language | Tool | Run | Built-in? |
|---|---|---|---|
| Rust | `dead_code` lint | `cargo build` / `clippy` (warns by default) | yes (compiler) |
| Python | `vulture` | `uv run vulture` (config: `[tool.vulture]`) | no — vulture fills the gap |
| TS | `knip` | `npm run knip` in `web/` and `rust-packages/laterite-node/` | partial — `tsc --noUnusedLocals` catches *locals*; knip catches unused *exports / files / deps* |

**Each tool's blind spot (why a hit needs human confirmation):**

- **Rust `dead_code`** — sees only *within* a crate build: a `pub` item assumed
  reachable by a downstream crate never warns even when genuinely dead, and the
  codebase carries deliberate `#[allow(dead_code)]` (e.g. `DuckParse::surface`).
  So `0 warnings` means "no *intra-crate* dead code", not "no dead code".
  It is also **silent on any name starting with `_`** — that leading underscore
  is the language's own "intentionally unused" marker, so `fn _helper()` is
  invisible to this scout however dead it is. Worth knowing twice over: it is a
  real hiding place, and it silently invalidates a hand-rolled positive control
  named `_probe`, which reads as "the lint is off" rather than "the probe was
  wrong". **Verify the scout on a normally-named dead item before trusting a
  clean run** — a green scan and a broken scan look identical.
- **Python `vulture`** — scoped (`[tool.vulture]`) to `packages/…/laterite` +
  `tools`, so it **cannot see `tests/`**. Two false-positive classes follow:
  1. *Public API* — most `packages/…/laterite` hits are methods/properties/enum
     members called from **outside Python** (the PyO3, wasm, or JS surfaces, or a
     sibling package). Cross-check against `__all__` + `concepts/crate-map.md` before believing one.
  2. *Test-only callers* — a `tools/` helper whose **only** caller is a
     faithfulness/coverage test in `tests/` reads as "unused" because that
     directory is out of scope. These are **live oracles, not relics** — confirmed
     examples (do not re-chase): `parse_subcommands` (`tools/gen_wiki_cli.py`, the
     independent cli.rs parser behind `test_readme_verbs_match_clap_subcommands`),
     `OP_VERBS` (`tools/xcheck/emit_cli.py`, the verb-coverage authority for
     `test_xcheck_verb_coverage.py`), and `crate_deps` (`tools/gen_crate_graph.py`,
     the crate-graph oracle for `test_crate_graph_faithful.py`). The single genuine
     orphan from the 2026-07-20 sweep was `_ordered` (recorded above).
- **TS `knip`** — configured per-project (`web/knip.json`,
  `rust-packages/laterite-node/knip.json`). Silenced structural noise: the
  napi-rs generated loader (`index.js` → per-platform `@laterite/native-*`
  optional packages + the wasi loader) in node, and the co-located MkDocs project
  (`web/docs-site/**`, which has its own docs-faithfulness gate) in web. Remaining
  hits (unused exports/types, a redundant `@types/proj4`, an orphaned test file)
  are real candidates for a future deliberate review, not yet promoted to rows.

**Workflow:** run a scout → for each hit, grep the symbol repo-wide *including
`tests/`* → if a live caller exists it's a false positive (note the class above);
if truly unreferenced, add a `spotted` row here → remove it deliberately in the PR
that retires the feature it belongs to, never as a sweep.

## Notes

- **Vocabulary source of truth:** the locked cross-surface vocabulary (load `read`, persist
  `save`, data→AGS4 `build_ags4`→`BuildResult`, in-memory `.text`/`.bytes`) is described in
  [[api-surface-1.0]]; `compat` is exempt (it mirrors python-ags4 names by contract).
- **Not relics:** the `.filter()` SQL-string form has a *future-enhancement* note (typed
  `column=value` / expression forms) in the `AgsQuery` docstring — an addition, not a removal,
  so it is **not** tracked here.
