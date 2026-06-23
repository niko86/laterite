<p align="center">
  <img src="https://raw.githubusercontent.com/niko86/laterite/main/assets/laterite-social-preview-white.png" alt="laterite — a Rust-backed AGS4 reader, writer and validator" width="600" />
</p>

# laterite

A Rust-backed AGS4 reader, writer and validator for Python.

A faster drop-in for
[`python-ags4`](https://gitlab.com/ags-data-format-wg/ags-python-library)'s
`AGS4` module (1.2.0 parity-pinned, 122/131 tests) — swap
`from python_ags4 import AGS4` for `from laterite import compat as AGS4`.
The native API returns born-typed **polars** frames by default (or
**pandas** with `read(..., backend="pandas")`) — both **pyarrow-free**,
read back from a Python-owned in-memory DuckDB engine.

[![ci](https://github.com/niko86/laterite/actions/workflows/ci.yml/badge.svg)](https://github.com/niko86/laterite/actions/workflows/ci.yml)
[![rust cov](https://img.shields.io/codecov/c/github/niko86/laterite?flag=rust&label=rust%20cov)](https://codecov.io/gh/niko86/laterite)
[![python cov](https://img.shields.io/codecov/c/github/niko86/laterite?flag=python&label=python%20cov)](https://codecov.io/gh/niko86/laterite)
[![web cov](https://img.shields.io/codecov/c/github/niko86/laterite?flag=web&label=web%20cov)](https://codecov.io/gh/niko86/laterite)
[![PyPI](https://img.shields.io/pypi/v/laterite.svg)](https://pypi.org/project/laterite/)
[![Python versions](https://img.shields.io/pypi/pyversions/laterite.svg)](https://pypi.org/project/laterite/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

```bash
pip install laterite                  # base AGS4 (polars + duckdb, pyarrow-free)
pip install "laterite[compat]"        # + pandas (python-ags4 drop-in)
```

```python
import laterite

result = laterite.validate("delivery.ags")
for rule, findings in result.by_rule().items():
    print(rule, len(findings))

# python-ags4 drop-in
from laterite import compat as AGS4
tables, _ = AGS4.AGS4_to_dataframe("delivery.ags")

# Or a typed view of the file
from laterite.ags4 import read_typed
proj = read_typed("delivery.ags")
```

The validator engine is clean-room from the AGS4 spec. python-ags4
is LGPL-3.0; the clean-room separation lets laterite ship under MIT.

Requires Python ≥ 3.12.

Full docs, parity catalogue and observations at
<https://github.com/niko86/laterite>.
