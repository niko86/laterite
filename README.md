<p align="center">
  <img src="https://raw.githubusercontent.com/niko86/laterite/main/assets/laterite-social-preview-white.png" alt="laterite — a Rust-backed AGS4 reader, writer and validator" width="600" />
</p>

# laterite

A Rust-backed reader, writer and validator for the
[AGS4](https://www.ags.org.uk/data-format/) geotechnical data format.
A faster drop-in for [`python-ags4`](https://gitlab.com/ags-data-format-wg/ags-python-library)'s
`AGS4` module — swap `from python_ags4 import AGS4` for
`from laterite import compat as AGS4` — with a modern, born-typed polars API.

[![ci](https://github.com/niko86/laterite/actions/workflows/ci.yml/badge.svg)](https://github.com/niko86/laterite/actions/workflows/ci.yml)
[![rust cov](https://img.shields.io/codecov/c/github/niko86/laterite?flag=rust&label=rust%20cov)](https://codecov.io/gh/niko86/laterite)
[![python cov](https://img.shields.io/codecov/c/github/niko86/laterite?flag=python&label=python%20cov)](https://codecov.io/gh/niko86/laterite)
[![web cov](https://img.shields.io/codecov/c/github/niko86/laterite?flag=web&label=web%20cov)](https://codecov.io/gh/niko86/laterite)
[![PyPI](https://img.shields.io/pypi/v/laterite.svg)](https://pypi.org/project/laterite/)
[![Python versions](https://img.shields.io/pypi/pyversions/laterite.svg)](https://pypi.org/project/laterite/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Part of the laterite suite

One clean-room Rust AGS4 engine, surfaced for every stack:

| Surface | Package | Get it |
|---|---|---|
| **Python** | [`laterite`](https://pypi.org/project/laterite/) — PyPI | `pip install laterite` |
| **Node.js** | [`laterite`](https://www.npmjs.com/package/laterite) — npm | `npm install laterite` |
| **Rust / CLI** | [`lat-check`](https://github.com/niko86/laterite/releases) | GitHub Releases |
| **Browser** | [validator + data explorer](https://niko86.github.io/laterite/) — WASM | open in a browser |

## Install

```bash
pip install laterite                  # base AGS4 (polars + duckdb)
pip install "laterite[compat]"        # + pandas (python-ags4 drop-in)
```

Requires Python ≥ 3.12.

## Use

```python
import laterite

# Validate a file
result = laterite.validate("delivery.ags")
for rule, findings in result.by_rule().items():
    print(rule, len(findings))

# Or the python-ags4 drop-in
from laterite import compat as AGS4
tables, headings = AGS4.AGS4_to_dataframe("delivery.ags")
AGS4.dataframe_to_AGS4(tables, headings, "round-trip.ags")

# Typed view: PROJ → LOCA → SAMP → ...
from laterite.ags4 import read_typed
proj = read_typed("delivery.ags")
for loca in proj.locas:
    print(loca.loca_id, loca.loca_gl)
```

```bash
lat-check delivery.ags --json
```

## Performance

Validation throughput vs `python-ags4` 1.2.0, on synthetic AGS4 files
of increasing size. Wall-clock after warmup, macOS arm64. All four
laterite paths return **byte-identical findings** to python-ags4 —
the speed gain is not at the cost of diagnostic coverage.

The files are LOCA-heavy (40-column real-schema LOCA group, with
`ID`/`PA`/`2DP`/`DT` TYPE columns) generated in two profiles:

**Clean** — values pre-formatted to match their declared TYPE
exactly (2DP rounded to 2 decimals, valid `yyyy-mm-dd` dates).
Real-world files look closer to this. ~15 baseline findings from
fixed-cost rules:

| Size | python-ags4 | `laterite.validate` | `lat-check` (CLI) | speedup |
|---:|---:|---:|---:|---:|
|   512 KB |    84 ms |   **5 ms** |   10 ms | **17×** |
|     5 MB |   489 ms |  **57 ms** |   61 ms |  **9×** |
|    50 MB |   5.27 s |  **0.62 s** | 0.64 s |  **8×** |
|   500 MB |  53.43 s |  **6.5 s** |  6.6 s |  **8×** |
|     1 GB | 117.27 s | **15.1 s** |  16.5 s |  **8×** |

**Worst case** — same files but with floating-point noise in
numeric cells that fails AGS4 Rule 8 (TYPE precision). Every cell
triggers a finding; exercises the validator's per-finding
accumulation + output-rendering paths fully:

| Size | python-ags4 | `laterite.validate` | `lat-check` (CLI) | Findings | speedup |
|---:|---:|---:|---:|---:|---:|
|   512 KB |    90 ms |   **7 ms** |   13 ms |     1 129 | **13×** |
|     5 MB |   547 ms |  **67 ms** |   92 ms |    11 485 |  **8×** |
|    50 MB |   6.09 s |  **0.79 s** | 0.96 s |   116 415 |  **8×** |
|   500 MB |  62.45 s |   **8.1 s** | 10.1 s | 1 170 223 |  **8×** |
|     1 GB | 132.67 s |  **18.3 s** | 22.9 s | 2 396 471 |  **7×** |

Notes on the CLI:

- `lat-check --json` writes findings to stdout as JSON
  (~80 bytes/finding). On the worst-case 1 GB file (2.4 M findings)
  that's ~190 MB of JSON. The native PyO3 path skips this
  serialisation step — for "validate then process findings in
  Python", use `laterite.validate`; for "validate then pipe to
  downstream tools", use `lat-check`. On clean files the gap is
  single-digit %.
- Native PyO3 returns findings as native Python objects (no Arrow
  boundary — findings are few); `rep.findings` assembles them into a
  polars frame and `rep.by_rule()` into a dict, both ~0% over the bare
  validation pass (PyO3's `IntoPyObject` is well-tuned for our `Finding`
  struct). The zero-copy Arrow capsule is the separate *data* read path
  (`read()` / `ags[code]`), not this validation path.

Both validators scale linearly: ~17 ns/byte for laterite, ~110
ns/byte for python-ags4.

## Parity with python-ags4

122 / 131 of python-ags4 1.2.0's own test suite passes through
`laterite.compat` (93 %). The 9 remaining are deliberate non-closures.
Full breakdown:

- [COMPAT.md](COMPAT.md) — rule-by-rule differences with rationale
- [OBSERVATIONS.md](OBSERVATIONS.md) — the engineering record (every
  observation, 5-field house style)
- [docs/parity-coverage-map.md](docs/parity-coverage-map.md) — test-
  level map of laterite ↔ python-ags4

The validator is **clean-room**: every rule is written from the AGS4
spec, not copied from python-ags4 source. python-ags4 is LGPL-3.0; the
clean-room separation lets laterite ship under MIT.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Short version: PRs welcome, CI
gates `cargo test` + `pytest` + the python-ags4 parity oracle.

## License

[MIT](LICENSE). The bundled AGS4 standard dictionaries remain ©
[AGS](https://www.ags.org.uk/data-format/) and are redistributed
under their published terms.
