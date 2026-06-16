# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
