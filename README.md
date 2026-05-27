# laterite

A Rust-backed reader, writer and validator for the
[AGS4](https://www.ags.org.uk/data-format/) geotechnical data format.
Drop-in for [`python-ags4`](https://gitlab.com/ags-data-format-wg/ags-python-library),
faster, with a narwhals-native API.

[![ci](https://github.com/niko86/laterite/actions/workflows/ci.yml/badge.svg)](https://github.com/niko86/laterite/actions/workflows/ci.yml)
[![PyPI](https://img.shields.io/pypi/v/laterite.svg)](https://pypi.org/project/laterite/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Install

```bash
pip install laterite                  # base AGS4 (polars + narwhals)
pip install "laterite[compat]"        # + pandas (python-ags4 drop-in)
pip install "laterite[ags5]"          # + experimental .ags5db surface
```

Requires Python ≥ 3.14.

## Use

```python
import laterite

# Validate a file
result = laterite.validate("delivery.ags")
for rule, findings in result.findings.items():
    print(rule, len(findings))

# Or the python-ags4 drop-in
from laterite import compat as AGS4
tables, headings = AGS4.AGS4_to_dataframe("delivery.ags")
AGS4.write_AGS4_file(tables, "round-trip.ags")

# Typed view: PROJ → LOCA → SAMP → ...
from laterite.ags4 import read_typed
proj = read_typed("delivery.ags")
for loca in proj.locas:
    print(loca.loca_id, loca.loca_gl)
```

```bash
ags4-check delivery.ags --json
```

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
