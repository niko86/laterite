# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] — TBD

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
- **`ags5db` CLI binary** *(experimental)* — attached to the GitHub release for users who want the `.ags5db` toolchain without installing the Python wheel.

### Known limitations

- Python 3.14 only (the wheel is pinned to a specific CPython build
  via pyo3-polars).
- `.ags5db` is experimental — format and API may change in v0.2. AGS4
  surface is stable.
- 9 python-ags4 parity tests fail by design; see
  [`docs/parity-coverage-map.md`](docs/parity-coverage-map.md).

[Unreleased]: https://github.com/niko86/laterite/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/niko86/laterite/releases/tag/v0.1.0
