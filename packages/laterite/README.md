<p align="center">
  <img src="https://raw.githubusercontent.com/niko86/laterite/main/assets/laterite-social-preview-white.png" alt="laterite — a Rust-backed AGS4 reader, writer and validator" width="600" />
</p>

# laterite

A Rust-backed **AGS4 toolchain** for the
[AGS4](https://www.ags.org.uk/data-format/) geotechnical data format —
validate, read as typed data, query, build, fix, diff, certify, and convert ↔
Excel — with a modern, born-typed **polars** API.

Coming from [`python-ags4`](https://gitlab.com/ags-data-format-wg/ags-python-library)?
`laterite.compat` is a faithful, faster stand-in for its `AGS4` and `AGS4.utils`
modules — swap `from python_ags4 import AGS4` for
`from laterite import compat as AGS4` and keep your code.

[![ci](https://github.com/niko86/laterite/actions/workflows/ci.yml/badge.svg)](https://github.com/niko86/laterite/actions/workflows/ci.yml)
[![python cov](https://img.shields.io/codecov/c/github/niko86/laterite?flag=python&label=python%20cov)](https://codecov.io/gh/niko86/laterite)
[![PyPI](https://img.shields.io/pypi/v/laterite.svg)](https://pypi.org/project/laterite/)
[![Python versions](https://img.shields.io/pypi/pyversions/laterite.svg)](https://pypi.org/project/laterite/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/niko86/laterite/blob/main/LICENSE)

## Install

```bash
pip install laterite                     # base AGS4 (polars + duckdb, pyarrow-free)
pip install "laterite[compat]"           # + pandas (python-ags4 drop-in) — still pyarrow-free
pip install "laterite[compat,pyarrow]"   # + the optional pyarrow accelerator (or [all])
```

Requires Python ≥ 3.12. The wheel is abi3, so one binary covers 3.12 / 3.13 /
3.14. Installing it also puts the **`lat`** CLI on your `PATH`.

The `[compat]` drop-in is pyarrow-free and fast on its own; adding `pyarrow`
swaps the pandas step for pyarrow's `to_pandas` and unlocks the Arrow-backed
`string` dtype — an accelerator, never a requirement.

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

# SQL across groups, no conversion step
ags.sql("SELECT loca_id, count(*) FROM SAMP GROUP BY 1")

# Repair a dirty file into a fresh handle, then keep working with it
fixed = ags.fix(risky=True)      # pads short rows, transliterates non-ASCII, …

# Typed graph: PROJ → LOCA → SAMP → …
from laterite.ags4 import read_typed
for loca in read_typed("delivery.ags").locas:
    print(loca.loca_id, loca.loca_gl)

# python-ags4 drop-in — swap the import, keep your code
from laterite import compat as AGS4
tables, headings = AGS4.AGS4_to_dataframe("delivery.ags")
AGS4.dataframe_to_AGS4(tables, headings, "round-trip.ags")
```

`read` returns born-typed **polars** frames by default (or **pandas** with
`read(..., backend="pandas")`) — both **pyarrow-free**, read back from a
Python-owned in-memory DuckDB engine.

## More than a faster `python-ags4`

[`python-ags4`](https://gitlab.com/ags-data-format-wg/ags-python-library) is the
reference **Python** library for AGS4 — validation plus pandas read/write — and
it inspired this project. `laterite` matches that surface and adds a toolchain
on top:

| | `laterite` | `python-ags4` |
|---|:---:|:---:|
| Validate — numbered AGS4 rules | ✅ | ✅ |
| Read → data frames | ✅ born-typed polars **or** pandas | pandas, all strings |
| Build / write AGS4 · Excel ↔ AGS4 | ✅ | ✅ |
| Repair engine (`fix`) | ✅ | — |
| SQL across groups · revision diff | ✅ | — |
| Validity certificates (`.ags.idx`) | ✅ | — |
| Transport — zstd compress + age encrypt | ✅ | — |
| Typed PROJ → LOCA → SAMP graph | ✅ | — |
| pyarrow required | no (optional accelerator) | via pandas' own deps |

## Performance

Synthetic, spec-valid AGS4 from `ags4-forge` — the `wide` scaffold: **123
groups**, realistic type mix, zero findings. macOS arm64, hot files, mean of 5
warm runs, `python-ags4` 1.2.0 vs `laterite` 0.8.0. Both agree on the findings.

**Validation**

| File | `python-ags4 check_file` | `laterite.validate` | speedup |
|---:|---:|---:|:---:|
| 4.9 MB · 459 BH | 1.5 s | 50 ms | **30.0×** |
| 24.9 MB · 2,219 BH | 3.7 s | 266 ms | **13.9×** |
| 102.7 MB · 8,872 BH | 12.3 s | 1.1 s | **11.7×** |
| 275.5 MB · 22,813 BH | 34.1 s | 2.6 s | **13.0×** |
| 549.7 MB · 45,107 BH | 70.0 s | 5.4 s | **12.9×** |

**Read → typed** — the honest comparison for real work. python-ags4 needs
`AGS4_to_dataframe` + `convert_to_numeric` on every group to get numbers, and
still leaves dates as text; `laterite.read` is born-typed, dates included.

| File | `python-ags4` + `convert_to_numeric` | `laterite.read` | speedup |
|---:|---:|---:|:---:|
| 4.9 MB | 187 ms | 26 ms | **7.2×** |
| 24.9 MB | 811 ms | 136 ms | **6.0×** |
| 102.7 MB | 3.4 s | 541 ms | **6.3×** |
| 275.5 MB | 8.9 s | 1.4 s | **6.4×** |
| 549.7 MB | 17.5 s | 2.9 s | **6.0×** |

**Read → strings** — like for like, both returning pandas frames of text.

| File | `python-ags4 AGS4_to_dataframe` | `laterite.compat` | speedup |
|---:|---:|---:|:---:|
| 4.9 MB | 144 ms | 49 ms | **2.9×** |
| 24.9 MB | 718 ms | 206 ms | **3.5×** |
| 102.7 MB | 2.8 s | 870 ms | **3.2×** |
| 275.5 MB | 7.3 s | 2.2 s | **3.3×** |
| 549.7 MB | 15.2 s | 4.6 s | **3.3×** |

The ratio holds as files grow — the gap is a constant factor, not a head start
that erodes. Reproduce any of this with
`uv run python tools/bench-vs-python-ags4.py` in the repo: it generates the
rungs, verifies each against a pinned SHA-256 so a change to the generator can't
move the numbers unnoticed, and prints these exact tables.

## Parity + clean-room

121 / 131 of python-ags4 1.2.0's own test suite passes through
`laterite.compat` (92 %); the 10 remaining are deliberate non-closures,
documented rule by rule. A weekly job compares the two public surfaces, so a
function added upstream can't quietly go missing here.
<!-- cadence: parity -->


**Two caveats worth knowing before you swap the import.** `compat` mirrors the
*library* API — python-ags4's `ags4_cli` command is not mirrored, because
laterite ships `lat` instead with its own JSON shapes; and `compat` is one flat
module rather than a package, so `from laterite.compat import AGS4` (a submodule
import) is not the shape — use `from laterite import compat as AGS4`.

The validator is **clean-room** from the published AGS4 specification, not
adapted from another library's source — python-ags4 is LGPL-3.0, and that
separation is what lets laterite ship under MIT. Details:
[COMPAT.md](https://github.com/niko86/laterite/blob/main/COMPAT.md) ·
[OBSERVATIONS.md](https://github.com/niko86/laterite/blob/main/OBSERVATIONS.md).

## One engine, every stack

`laterite` on PyPI is the **Python** surface of one Rust AGS4 engine, shared
across:

| Surface | Package | Get it |
|---|---|---|
| **Python** | [`laterite`](https://pypi.org/project/laterite/) — PyPI | `pip install laterite` |
| **Node.js** | [`laterite`](https://www.npmjs.com/package/laterite) — npm | `npm install laterite` |
| **CLI** | [`lat`](https://github.com/niko86/laterite/releases) | bundled with this wheel |
| **DuckDB** | [`laterite_ags4`](https://community-extensions.duckdb.org/extensions/laterite_ags4.html) — community extension | `INSTALL laterite_ags4 FROM community;` |
| **Browser** | [validator + data explorer](https://niko86.github.io/laterite/) — WASM | open in a browser |

Scriptable output is byte-identical across all of them, so a CI gate and a
notebook can't disagree.

## Docs

Full documentation — Learn, Cookbook, Concepts, and the Python API reference —
at **<https://niko86.github.io/laterite/docs/>**.
