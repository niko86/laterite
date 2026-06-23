# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

The next line settles the AGS4 surface and decouples the experimental AGS5 strand. A
**breaking** change (the `[ags5]` extra and the `.ags5db` companion are gone from the
shipped package); the 0.x version signals the surface is still settling.

### Added

- **`lat-check <a> --diff <b>`.** The revision diff on the CLI — the Rust `lat-check`
  binary and the Python `_cli` (byte-identical output): a per-group
  `+added −removed ~changed` summary, or the full `RevisionDelta` with `--json`. Completes
  `revision-diff` across Python + CLI (#204).
- **`laterite.diff(a, b)` + `Ags4File.diff(other)`.** The KEY-aware, type-aware revision
  diff — previously browser-only — now on the Python surface. Compares two AGS4 documents
  (each a path / text / bytes / `Ags4File`), matching rows by the group's dictionary KEY
  headings and comparing cells through the **typed** value (so a formatting-only
  `"1.0"`→`"1.00"` isn't reported); returns a `RevisionDelta` dict (per-group deltas + total
  added/removed/changed). Backed by the shared `laterite-ags4-diff` leaf — the same diff the
  browser's Tools tab uses (the CLI `lat-check --diff` follows). (#204)
- **wasm `list_rules()` + the `warnings` knob.** The browser/wasm surface gains the rule
  catalogue (`list_rules()`, the gated `rules_meta.json`) and an `include_warnings` parameter
  on `validate` — so Rule 18 WARNINGs now reach the web validator (its severity filter already
  had a warning lane; the engine just never produced any). Completes the cross-surface
  warnings/fyi knob (#204). The wasm `build_ags4`/`build_ags4_ipc`/`dictionary` params also
  moved `edition`→`dict_version` (positional, caller-transparent), finishing the #241
  vocabulary alignment on wasm.
- **Node `fix()` + `listRules()`.** The Node surface gains the two verbs it was missing
  versus Python/CLI: headless mechanical repair — `fix(source, { risky })` → a `FixResult`
  (`.bytes` / `.text` / `.save(path)`; `findings` is what couldn't be mechanically fixed) —
  and the rule catalogue `listRules()` → typed `RuleMeta[]` (the gated `rules_meta.json`,
  no phantom `12`/`16a`). Both share the engine's `fix_document` / `rule_metadata_json`, so
  `applied[]` and the catalogue are byte-identical across Python / CLI / Node. (#204
  gap-closures.)
- **Rule 18 — opt-in WARNING for a malformed DICT group (the first WARNING-tier
  producer).** A file that declares custom groups/headings through a structurally
  broken DICT — a missing `DICT_TYPE`/`DICT_GRP`/`DICT_HDNG` column, a blank
  `DICT_GRP`, or a `HEADING` row with no heading name — now surfaces a `Warning
  (Related to Rule 18)` under `validate(warnings=True)` / `lat-check --show-warnings`.
  The engine only *consumed* DICT before, so a malformed one silently degraded every
  downstream check. It's **WARNING, not Error** — the file is broken but the spec is
  silent and python-ags4 doesn't check it, so an error would break parity; opt-in + a
  separate label keep the default verdict and the python-ags4 path unchanged (O-44).
  This makes the previously-inert `--show-warnings` / `warnings=True` flag live.
- **`lat-check --list-rules` + `laterite.list_rules()`.** A read-only AGS4 rule
  catalogue — one entry per rule with title, severity (`error` / `fyi` / `mixed`),
  whether `fix` can repair it, and the cited `O-N` divergence notes. The CLI prints a
  table (or `--json`, no input file needed); the Python verb returns `list[dict]`.
  Backed by a new **gated single source** (`rules_meta.json`, compile-time-embedded so
  there's no runtime parse): a faithfulness test asserts it covers *exactly* the rules
  the engine emits and that `fixable` matches the fix engine — so it can't list a rule
  that doesn't exist (the old hand-curated web catalogue had drifted, carrying a no-op
  `12` and a non-existent `16a`). Repointing the web RuleExplainer onto this source — it
  still serves its old, now-divergent copy until then — and a companion `--edition-delta`,
  follow.
- **Headless mechanical fix/repair.** `laterite.fix(source)` / `Ags4File.fix()` and
  `lat-check --fix` apply the same fix engine the browser uses — without a UI — to an
  existing delivery: the **safe** fixes (CRLF / BOM / embedded-CR / short-row pad /
  numeric reformat / TRAN delimiter+concatenator rows) always, the intent-guessing
  ones (duplicate-heading rename, `dd/mm` date, typography) under `risky=` /
  `--fix-risky`. The result is re-validated, so `FixResult.findings` is what could not
  be mechanically fixed. Non-destructive: the Python verb returns the fixed bytes
  (write with `in_place=`/`out=`/`.save()`); the CLI writes a sibling
  `<file>.fixed.ags` by default (`--in-place` / `--fix-out` to redirect). The single
  fix orchestration lives in the validator (`fix_document`), shared by the PyO3 verb
  and the CLI.
- **Rule 16 — non-standard abbreviation FYI.** A `PA` value self-declared in the file's
  own ABBR group but not in the bundled standard picklist for its heading (a typo like
  `"Borng"`, or an invented code) now surfaces as an opt-in `FYI (Related to Rule 16)`
  (`validate(fyi=True)` / `lat-check --show-fyi`). The file stays spec-legal — Rule 16 is
  satisfied by the ABBR declaration — so it is informational, never an error or warning;
  bounded to headings that have a standard picklist (custom/DICT-defined `PA` headings are
  skipped). A laterite-originated check python-ags4 lacks (OBSERVATIONS O-43).
- **First-class Excel verbs on the modern surface** — `laterite.to_excel(source, out)`
  / `laterite.from_excel(xlsx)` (plus `Ags4File.to_excel(out)`). AGS4↔XLSX round-trip
  was already shipped Rust-side but reachable only through the legacy `compat` shim;
  it is now a first-class verb (one sheet per group, pyarrow/openpyxl-free). `to_excel`
  accepts anything `read` does (path / text / bytes / `Ags4File`); `from_excel` returns
  a parsed `Ags4File` by default, or writes an AGS4 file when given an output path.
- **`Report.findings` / `Report.by_rule()` now carry severity + location.** The frame
  gains `severity` (`"error"` / `"warning"` / `"fyi"`), `target`, `heading`,
  `field_index` and `data_row` columns; `by_rule()` items carry the same plus
  `char_span`. The data already crossed the PyO3 boundary (and `to_json` / `to_ndjson`
  exposed it) — the dataframe path simply dropped it, so `validate(warnings=True,
  fyi=True)` couldn't tell the tiers apart or reach the offending heading/field.
  Additive — the existing `rule` / `line` / `group` / `desc` columns and `by_rule`
  keys are unchanged.
- **`read(..., xn="numeric")`** — an opt-in read-side numeric view of AGS `XN`-typed
  columns ("numeric, may carry a non-numeric qualifier" — `NP` / `<5` / `>100`). Every
  XN heading materialises as `Float64` across the handle (`ags[code]` / `sql` / `at`),
  non-numeric tokens becoming null. Default stays byte-faithful text; write-back is
  unaffected. A fuller bidirectional/born-typed XN treatment is future work.
- **`laterite.source`** — the fluent-chain entry name, an alias of `laterite.read`
  (`laterite.source(x).validate()…`); same callable, exported in `__all__`.
- **A published source distribution.** The release builds + publishes an sdist (once, on
  the Linux leg) alongside the wheels, so a platform without a published wheel (e.g. ARM
  Linux) installs from source. The sdist vendors all the path-dep Rust crates and is
  verified buildable + installable in isolation.

### Changed

- **`Ags4File.fix()` now returns a repaired `Ags4File`** (was a `FixResult`) — the fluent
  capstone, so `read(path).fix().validate().save(out)` reads as one chain. The repaired
  handle inherits the source's `backend` / `xn`; the `FixResult` (what was applied + the
  residual findings) rides on its new `.fix_report` attribute. **Breaking** for callers of
  the *method* — the free `laterite.fix(source)` is unchanged and still returns a
  `FixResult` (with its `in_place=` / `out=` write options), so reach for that when you want
  the report. Completes the fluent surface (#204).
- **New internal `laterite-ags4-diff` leaf.** The KEY-aware/type-aware revision-diff core
  (`diff_parsed` + its row/cell deltas) is extracted out of the `laterite-ags4-wasm` crate
  into a wasm-safe leaf — the browser `diff()` output is unchanged. It's the shared engine
  that brings `revision-diff` to Python + the CLI next (#204).
- **`build_ags4`: `edition` → `dict_version`.** The data→AGS4 builder's edition argument is
  renamed to `dict_version` (Python `build_ags4(dict_version=...)`, Node
  `buildAgs4(groups, {dictVersion})`) so the whole surface speaks one word for the AGS
  edition (matching `validate` / `read`). `edition` stays a **deprecated alias** — a
  `DeprecationWarning` in Python, a `@deprecated` JSDoc in Node — so existing callers keep
  working. First step of the cross-surface vocabulary alignment (#204); wasm + the DuckDB
  extension follow.
- **Pre-GROUP orphan rows are now reported (Rule 2).** A `HEADING`/`UNIT`/`TYPE`/`DATA`
  row appearing before any `GROUP` was previously dropped silently (the lenient parser);
  it is now reported as an `AGS Format Rule 2` finding, so the validator gives a complete
  report where python-ags4 hard-fails on the same input (O-41).
- **`TRAN_AGS="4.0"` resolves to the newer 4.0.4 dictionary** (it is a strict superset of
  4.0.3), and an exact `"4.0.3"` is upgraded to 4.0.4 when the file uses a 4.0.4-only
  heading — with a transparency FYI. This avoids the false **Rule 10c** orphans a stale
  `"4.0"`→4.0.3 mapping produces on a file that is actually 4.0.4 (O-30 / O-42).
- **One faithful, multi-edition dictionary.** The registry is now generated from a single
  `ags_dictionary.json` projecting the official AGS dictionaries across editions
  4.0.3 → 4.2, consumed as the latest-edition **union**. The AGS4 dictionary drops the
  AGS-L draft groups (CONL / TREL / TRIL) and the `is_high_volume` flag. (Also fixed a
  KEY-detection bug: combined statuses like `KEY+REQUIRED` are now recognised as KEY.)

### Removed

- **The experimental `.ags5db` / `.agsx` (AGS5) surface is decoupled from the shipped
  package** — the `[ags5]` extra, the `laterite-ags5` companion wheel, and the `lat-db`
  CLI are no longer built or published (moved to a dormant holding folder for a future,
  separately-published AGS5 strand). `from laterite.ags5db import …` no longer resolves;
  `pip install laterite[ags5]` no longer exists. This reverses the 0.4.0 "`laterite-ags5`
  now publishes to PyPI". The shipped `laterite` is AGS4-only and links no bundled DuckDB.

### Fixed

- **Stale package metadata** — the PyPI `description`, README and keywords claimed a
  "narwhals-native API" and advertised the removed `[ags5]` extra. Both have been gone
  since 0.4.0 (the API returns polars/pandas directly, pyarrow-free; the AGS5 surface was
  decoupled). Corrected so the package page reflects the shipped surface.
- **Cross-surface CLI / consistency fixes (0.5.0 audit).** The Python `lat-check` gained
  `--encoding` (the Rust binary already had it) and now rejects `--json` + `--ndjson`
  together with exit 5 (it previously preferred JSON silently); the Python CLI README was
  resynced to the Rust one (it had drifted — missing the `--fix` / `--list-rules` docs).
  The DuckDB perf tool referenced a non-existent `ags_validate` (the registered function
  is `validate_ags`), so its validate timing never ran. Transport/Excel errors no longer
  carry a stale `"ags5db error"` brand (now `"laterite error"`). Comment/doc cleanups: the
  typed-graph codegen said "92 classes" (it emits 174); the DuckDB-extension design page
  used the old `ags_validate` name.

## [0.4.0] — 2026-06-16

The **0.4.0** line — the DuckDB-engine redesign of the read / data-science surface, the
`laterite-*` crate rebrand, and version unification. This is a **breaking** change (frame
return types move from narwhals to polars / pandas; the validator CLI is renamed to
`lat-check`); the 0.x version signals the surface is still settling.

### Added

- **`laterite-ags5` now publishes to PyPI.** `pip install laterite[ags5]` resolves the
  `.ags5db` companion wheel from PyPI (previously the companion shipped via GitHub release
  only). The base `laterite` and `laterite-ags5` are separate PyPI projects with separate
  trusted publishers; the `[ags5]` extra pulls the companion.
- **In-memory DuckDB engine.** `laterite.read(path)` loads each AGS4 group into a
  born-typed table in a Python-owned in-memory DuckDB (the pip `duckdb` package, *not*
  bundled in the wheel), populated lazily per group on first touch:
  - `ags["LOCA"]` → an eager **polars** `DataFrame` by default, born typed (a 2DP heading
    is `Float64`, an ID `str`, a non-conforming numeric cell `null`).
  - `read(path, backend="pandas")` → the same accessor returns **pandas**; both paths are
    **pyarrow-free** (polars via the Arrow C-stream capsule, pandas via DuckDB's NumPy
    `.df()`).
  - `ags.sql("SELECT … WHERE …")` → a **DuckDB relation**: real cross-group joins and
    **filter pushdown** (the `WHERE` runs in the engine, so a huge file materialises only
    the rows you keep); finish with `.df()` / `pl.from_arrow(rel)` or chain more SQL.
  - `ags.at("LOCA", ["BH01", "BH02"])` → a **filtered subset view**: pull a parent entity's
    related records across groups (everything keyed by `LOCA_ID`), materialising only the
    matching rows. `sub.groups` lists the related groups; chain `sub["SAMP"]`.
  - `ags.register(name, frame)` joins your own frames against the groups; `ags.connection`
    exposes the raw DuckDB connection (parquet export, Arrow via `.arrow()`, …). Read
    handles are context managers — `with laterite.read(p) as ags: …` — with `close()`.
- **Born-typed reads on a zero-copy Apache Arrow boundary.** The Rust parser builds typed
  Arrow per group (`pyo3-arrow`, pyarrow-free) to seed the engine; `to_numeric` is now
  redundant for typed columns (kept for the compat shape). Writes stay **byte-faithful** —
  re-emitted from the retained Rust parse, independent of which groups were read.
- `laterite.transport` (`pack` / `unpack` / `lock` / `unlock`) documented as
  **content-agnostic** — any file, not only `.ags5db`; a non-path arg now raises an
  actionable `TypeError`.
- **Data → valid AGS4.** `laterite.emit_ags4(groups, *, edition, mode)` builds a
  spec-correct AGS4 document from your own per-group polars / pandas frames (columns =
  AGS headings) — the inverse of `read`, with `autofix` / `report` / `strict` modes and
  per-edition UNIT/TYPE fill. A **base** feature, pyarrow-free for polars. The browser
  gets the same producer via the wasm `to_ags4` / `to_ags4_ipc` exports and a web Export
  tab.

### Changed

- **BREAKING — frame return types are now polars / pandas, not narwhals.** The public
  surface (`ags[code]`, `Report.findings`, `compat`) returns native polars (default) or
  pandas frames directly; **narwhals is dropped**. A narwhals user wraps the result in one
  line — `nw.from_native(ags["LOCA"])`. No deprecation shim (nothing depends on 0.2's
  narwhals returns yet).
- **Dependencies.** Base is now **`polars + duckdb`** (was `polars + narwhals`); **narwhals
  and pyarrow are no longer base dependencies**. The `[compat]` extra is **`pandas +
  pyarrow`** — compat's python-ags4 pandas output goes through `polars.to_pandas()`, which
  needs pyarrow; compat-polars and the whole core are pyarrow-free. The single
  `cp312-abi3` wheel is preserved.
- `Ags4File(...)` given a bad argument (e.g. a file path) now raises a clear `TypeError`
  pointing at `laterite.read()`, instead of failing several calls later with a cryptic
  `'PosixPath' object is not subscriptable`.
- **BREAKING — the validator CLI command is renamed `ags4-check` → `lat-check`.** The wheel
  console-script and the standalone Rust binary attached to the GitHub release are now
  `lat-check` (the flags, JSON/NDJSON shape, and exit codes are unchanged). Re-invoke as
  `lat-check delivery.ags --json`. Part of the crate-suite rebrand (the Rust crate is now
  `laterite-ags4-check`); there is no `ags4-check` alias.

### Deprecated

- The first parameter of `transport.pack` / `lock` (was `db=`), `unpack` (was `zst=`)
  and `unlock` (was `file=`) is unified to `src=`. The old keyword names still work but
  emit a `DeprecationWarning` and will be removed in a future release.

### Fixed

- **Base install no longer reaches into the optional extras** (#111). On a plain
  `pip install laterite` (no `[ags5]` / `[compat]`):
  - `laterite.ags4.read_typed(path)` builds the typed PROJ graph on the base path — it
    no longer routes through a temporary `.ags5db`, so it no longer pulls the `[ags5]`
    (DuckDB) companion.
  - `laterite.emit_ags4(...)` with a polars frame is pyarrow-free — it no longer round-
    trips through DuckDB (whose polars ingest called `.to_arrow()`, pulling `[compat]`'s
    pyarrow).
  A regression guard exercises the whole base surface under a simulated base-only install.

### Changed

- **Python support widened to ≥ 3.12** (previously ≥ 3.14). Both `laterite` and
  `laterite-ags5` now build as a single `cp312-abi3` wheel per platform —
  one artefact, installable on Python 3.12, 3.13, and 3.14.

### Added

- Validator findings now carry character-level spans and rule-aware severity,
  enabling precise in-place error highlighting.
- **AGS4 fix engine** — compute and apply mechanical corrections (pad short rows
  to a group's field count, typographic→ASCII normalisation), with an opt-in
  *risky* DATETIME-canonicalisation fixer kept separate from the safe set.
- Edition-selectable dictionary / templates, sourced from the per-edition AGS
  standard dictionary.

### Fixed

- Correct the TRIL group classification and flag CONL/TREL/TRIL as AGS-L.

## [0.1.0] — 2026-05-27

Initial public release.

### Added

- **AGS4 reader / writer / validator**, Rust-backed via PyO3.
  - `laterite.validate(text=...)` / `laterite.read(...)` / `laterite.write(...)` — narwhals-native return types.
  - `ags4-check` CLI binary (bundled in the wheel as a Python entry point; standalone Rust binary attached to the GitHub release).
  - 122/131 of the python-ags4 parity suite passes; see [`docs/parity-coverage-map.md`](docs/parity-coverage-map.md).
- **`laterite.compat`** — drop-in replacement for `python-ags4`'s public surface (pandas frames by default; `set_backend("polars")` opts out of the pandas dependency).
- **92 typed-graph classes** auto-generated from the AGS dictionary: `from laterite import PROJ, LOCA, SAMP, …`. Field names, units, and types match the AGS4 standard dictionary.
- **`laterite.transport`** — zstd + age envelope helpers (`pack` / `unpack` / `lock` / `unlock`). Compression and password-protected secure transport for AGS data files.
- **`laterite[ags5]` extra** *(experimental, opt-in)* — installs the
  `laterite-ags5` companion wheel and unlocks `laterite.ags5db`:
  DuckDB-backed `.ags5db` read / write / convert / export / query /
  diff / peek, plus binary blob attachments. Adds ~50 MB to the install
  footprint (bundles DuckDB), kept out of the base wheel so AGS4-only
  users stay light.
- **`lat-db` CLI binary** *(experimental)* — attached to the GitHub release for users who want the `.ags5db` toolchain without installing the Python wheel.

### Known limitations

- Python 3.14 only (the wheel is pinned to a specific CPython build
  via pyo3-polars).
- `.ags5db` is experimental — format and API may change in v0.2. AGS4
  surface is stable.
- 9 python-ags4 parity tests fail by design; see
  [`docs/parity-coverage-map.md`](docs/parity-coverage-map.md).

[Unreleased]: https://github.com/niko86/laterite/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/niko86/laterite/releases/tag/v0.4.0
[0.2.0]: https://github.com/niko86/laterite/releases/tag/v0.2.0
[0.1.0]: https://github.com/niko86/laterite/releases/tag/v0.1.0
