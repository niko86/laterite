<p align="center">
  <img src="https://raw.githubusercontent.com/niko86/laterite/main/assets/laterite-social-preview-white.png" alt="laterite — a Rust-backed AGS4 reader, writer and validator" width="600" />
</p>

# laterite

A Rust-backed **AGS4 toolchain** for the
[AGS4](https://www.ags.org.uk/data-format/) geotechnical data format —
validate, read as typed data, query, build, fix, diff, certify, and convert ↔
Excel — with a modern, born-typed **polars** API.

Coming from [`python-ags4`](https://gitlab.com/ags-data-format-wg/ags-python-library)?
`laterite` is a faster, faithful drop-in for its `AGS4` module — swap
`from python_ags4 import AGS4` for `from laterite import compat as AGS4`, keep
your code — and a full toolchain beyond it.

[![ci](https://github.com/niko86/laterite/actions/workflows/ci.yml/badge.svg)](https://github.com/niko86/laterite/actions/workflows/ci.yml)
[![python cov](https://img.shields.io/codecov/c/github/niko86/laterite?flag=python&label=python%20cov)](https://codecov.io/gh/niko86/laterite)
[![PyPI](https://img.shields.io/pypi/v/laterite.svg)](https://pypi.org/project/laterite/)
[![Python versions](https://img.shields.io/pypi/pyversions/laterite.svg)](https://pypi.org/project/laterite/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/niko86/laterite/blob/main/LICENSE)

## Install

```bash
pip install laterite                  # base AGS4 (polars + duckdb, pyarrow-free)
pip install "laterite[compat]"        # + pandas (python-ags4 drop-in)
```

Requires Python ≥ 3.12.

## Use

```python
import laterite

# Validate — errors + warnings by default (FYI is opt-in)
report = laterite.validate("delivery.ags")
report.is_valid
for rule, findings in report.by_rule().items():
    print(rule, len(findings))

# Read born-typed columns: a 2DP heading is a float, a DT a datetime
ags = laterite.read("delivery.ags")
ags.groups                       # ['PROJ', 'LOCA', 'SAMP', …]
ags["LOCA"]["LOCA_GL"][0]        # → 12.3  (a polars DataFrame per group)

# Repair a dirty file into a fresh handle, then keep working with it
fixed = ags.fix(risky=True)      # pads short rows, transliterates non-ASCII, …

# Typed graph: PROJ → LOCA → SAMP → …
from laterite.ags4 import read_typed
proj = read_typed("delivery.ags")
for loca in proj.locas:
    print(loca.loca_id, loca.loca_gl)

# python-ags4 drop-in — swap the import, keep your code
from laterite import compat as AGS4
tables, headings = AGS4.AGS4_to_dataframe("delivery.ags")
AGS4.dataframe_to_AGS4(tables, headings, "round-trip.ags")
```

The native API returns born-typed **polars** frames by default (or **pandas**
with `read(..., backend="pandas")`) — both **pyarrow-free**, read back from a
Python-owned in-memory DuckDB engine.

## One engine, every stack

`laterite` on PyPI is the **Python** surface of one Rust AGS4 engine, shared
across:

| Surface | Package | Get it |
|---|---|---|
| **Python** | [`laterite`](https://pypi.org/project/laterite/) — PyPI | `pip install laterite` |
| **Node.js** | [`laterite`](https://www.npmjs.com/package/laterite) — npm | `npm install laterite` |
| **Rust / CLI** | [`lat`](https://github.com/niko86/laterite/releases) | GitHub Releases |
| **DuckDB** | [`laterite_ags4`](https://community-extensions.duckdb.org/extensions/laterite_ags4.html) — community extension | `INSTALL laterite_ags4 FROM community;` |
| **Browser** | [validator + data explorer](https://niko86.github.io/laterite/) — WASM | open in a browser |

## More than a faster `python-ags4`

[`python-ags4`](https://gitlab.com/ags-data-format-wg/ags-python-library) is the
reference **Python** library for AGS4 — validation plus pandas read/write.
`laterite` is a faithful, faster drop-in for that, and a cross-surface toolchain
on top:

| | `laterite` | `python-ags4` |
|---|:---:|:---:|
| Runs on | Python · Node · CLI · DuckDB · browser | Python |
| Validate — numbered AGS4 rules | ✅ | ✅ |
| Read → typed data | ✅ born-typed (polars) | pandas (all strings) |
| Build / write · Excel ↔ AGS4 | ✅ | ✅ |
| Repair engine (`fix`) · revision diff · `.ags.idx` certificates | ✅ | — |
| SQL across groups (DuckDB) · zstd+age transport · typed graph | ✅ | — |

**And faster:** validation agrees on the same findings but runs **~17× on a
512 KB file, ~8× from 50 MB to 1 GB** (a 557 MB file in ~6.5 s vs ~53 s) — full
tables + methodology in the
[repo README](https://github.com/niko86/laterite#performance).

## Parity + clean-room

121 / 131 of python-ags4 1.2.0's own test suite passes through
`laterite.compat` (the 10 remaining are deliberate non-closures). The validator
is **clean-room** from the AGS4 spec — python-ags4 is LGPL-3.0, and the
separation is what lets laterite ship under MIT. Details:
[COMPAT.md](https://github.com/niko86/laterite/blob/main/COMPAT.md) ·
[OBSERVATIONS.md](https://github.com/niko86/laterite/blob/main/OBSERVATIONS.md).

## Docs

Full documentation — Learn, Cookbook, Concepts, and the Python API reference —
at **<https://niko86.github.io/laterite/docs/>**.
