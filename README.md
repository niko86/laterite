<p align="center">
  <img src="https://raw.githubusercontent.com/niko86/laterite/main/assets/laterite-social-preview-white.png" alt="laterite — a modern AGS4 toolkit" width="600" />
</p>

# laterite

A modern **AGS4 toolkit** for the [AGS4](https://www.ags.org.uk/data-format/)
geotechnical data format: validate, read as born-typed data, query, build, fix,
diff, certify, and convert ↔ Excel. One fast Rust engine drives it, surfaced
natively for **Python, Node.js, the CLI, DuckDB and the browser**.

Files come back born-typed — a `2DP` heading is a float, a `DT` a datetime, an
`ID` a string — so polars, SQL and the typed graph see real types, not text.

The closest open-source tool is
[`python-ags4`](https://gitlab.com/ags-data-format-wg/ags-python-library), and
it's what inspired laterite. laterite takes that idea, rebuilds it on a Rust
core for speed, and uses that core to bring AGS4 to more languages. Already on
python-ags4? There's a drop-in — swap `from python_ags4 import AGS4` for
`from laterite import compat as AGS4` and keep your code.

[![ci](https://github.com/niko86/laterite/actions/workflows/ci.yml/badge.svg)](https://github.com/niko86/laterite/actions/workflows/ci.yml)
[![rust cov](https://img.shields.io/codecov/c/github/niko86/laterite?flag=rust&label=rust%20cov)](https://codecov.io/gh/niko86/laterite)
[![python cov](https://img.shields.io/codecov/c/github/niko86/laterite?flag=python&label=python%20cov)](https://codecov.io/gh/niko86/laterite)
[![web cov](https://img.shields.io/codecov/c/github/niko86/laterite?flag=web&label=web%20cov)](https://codecov.io/gh/niko86/laterite)
[![node cov](https://img.shields.io/codecov/c/github/niko86/laterite?flag=node&label=node%20cov)](https://codecov.io/gh/niko86/laterite)
[![PyPI](https://img.shields.io/pypi/v/laterite.svg)](https://pypi.org/project/laterite/)
[![npm](https://img.shields.io/npm/v/laterite.svg)](https://www.npmjs.com/package/laterite)
[![Python versions](https://img.shields.io/pypi/pyversions/laterite.svg)](https://pypi.org/project/laterite/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

📖 **[Full documentation](https://niko86.github.io/laterite/docs/)** · 🌐 **[Browser validator + data explorer](https://niko86.github.io/laterite/)**

## Part of the laterite suite

One Rust AGS4 engine, surfaced for every stack:

| Surface | Package | Get it |
|---|---|---|
| **Python** | [`laterite`](https://pypi.org/project/laterite/) — PyPI | `pip install laterite` |
| **Node.js** | [`laterite`](https://www.npmjs.com/package/laterite) — npm | `npm install laterite` |
| **Rust / CLI** | [`lat`](https://github.com/niko86/laterite/releases) | GitHub Releases |
| **DuckDB** | [`laterite_ags4`](https://community-extensions.duckdb.org/extensions/laterite_ags4.html) — community extension | `INSTALL laterite_ags4 FROM community;` |
| **Browser** | [validator + data explorer](https://niko86.github.io/laterite/) — WASM | open in a browser |

## Install

```bash
pip install laterite                     # base AGS4 (polars + duckdb)
pip install "laterite[compat]"           # + pandas (python-ags4 drop-in) — pyarrow-free
pip install "laterite[compat,pyarrow]"   # + optional pyarrow accelerator (or [all])
```

Requires Python ≥ 3.12. The `[compat]` drop-in is pyarrow-free and fast on its
own; adding `pyarrow` swaps the pandas step for pyarrow's `to_pandas` and
unlocks the Arrow-backed `string` dtype — an optional accelerator, never
required.

## Use

The same engine drives every surface — pick your stack. Full guides and the API
reference live in the [documentation](https://niko86.github.io/laterite/docs/).

### Python

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

### Node.js

```ts
import { read, validate, buildAgs4 } from "laterite";

const ags = read("delivery.ags");      // path, or read(bytes) / read(undefined, { text })
ags.groups;                            // ["PROJ", "LOCA", "SAMP", …]
ags.table("LOCA").getChild("LOCA_GL")?.get(0);   // → 12.3 (born-typed apache-arrow)

const report = validate("delivery.ags");
report.isValid;                        // boolean
report.toJson();                       // byte-identical to `lat validate --json`

// Produce valid AGS4 from plain rows (or a typed PROJ/LOCA graph)
const res = buildAgs4(new Map([
  ["PROJ", [{ PROJ_ID: "P1", PROJ_NAME: "Demo" }]],
  ["LOCA", [{ LOCA_ID: "BH01", LOCA_GL: 12.3 }]],
]), { mode: "autofix" });
res.save("out.ags");
```

Cross-group SQL (`ags.sql(...)` / `ags.at(...)`) needs the optional peer `@duckdb/node-api`.

### CLI (`lat`)

```bash
lat validate delivery.ags              # human report; exit 0 clean / 1 findings
lat delivery.ags                       # shorthand for `lat validate delivery.ags`
lat validate delivery.ags --json       # machine-readable findings (pretty JSON)
lat validate delivery.ags --no-warnings   # errors only (warnings show by default)
lat fix delivery.ags                   # repair → sibling .fixed.ags (safe fixes)
lat diff old.ags new.ags               # KEY-aware revision delta (+added -removed ~changed)
lat certify delivery.ags               # mint delivery.ags.idx if clean
lat rules                              # the AGS4 rule catalogue (no input needed)
```

Exit codes: `0` clean · `1` findings · `3` unreadable · `4` not AGS4 · `5` bad args · `6` schema.
Run `lat --readme` for the full guide.

## A full AGS4 toolkit

laterite covers the whole AGS4 workflow, not just read and write. The closest
open-source tool, `python-ags4`, focuses on Python validation and pandas I/O;
laterite matches that and adds a cross-surface toolchain on top:

| | `laterite` | `python-ags4` |
|---|:---:|:---:|
| Runs on | Python · Node · CLI · DuckDB · browser | Python |
| Validate — numbered AGS4 rules | ✅ | ✅ |
| Read → typed data | ✅ born-typed (polars) | strings; opt-in `convert_to_numeric` |
| Build / write AGS4 | ✅ | ✅ |
| Excel ↔ AGS4 | ✅ | ✅ |
| Repair engine (`fix`) — CRLF / BOM / short-row pad / embedded-CR… | ✅ | — |
| SQL across groups | ✅ (DuckDB) | — |
| Revision diff | ✅ | — |
| Validity certificates (`.ags.idx`) | ✅ | — |
| Transport — compress + encrypt | ✅ | — |
| Typed PROJ → LOCA → SAMP graph | ✅ | — |
| Command-line interface | ✅ standalone binary (`lat`) | ✅ Python (`ags4_cli`) |

laterite reports the same findings as `python-ags4`, with 10 documented
exceptions (see [Parity](#parity-with-python-ags4)).

## Performance

Timings on synthetic, spec-valid AGS4 files generated by `ags4-forge` — the
`wide` scaffold: **123 groups**, a realistic type mix, clean (zero findings).
Every rung is reproducible (`ags4-forge scale --size 100MB --scaffold wide
--seed 0`); each cell is the mean of 10 warm runs (8 at 265 MB, 5 at 524 MB —
a single run there is already tens of seconds), files read hot, on macOS arm64,
against `python-ags4` 1.2.0.

**Validation** (`AGS4.check_file` vs `laterite.validate`, all rules):

| File (123 groups) | `python-ags4` | `laterite.validate` | speedup |
|---:|---:|---:|:---:|
| 4.9 MB · 459 BH | 1.5 s | 0.11 s | **14.0×** |
| 24.9 MB · 2,219 BH | 3.7 s | 0.51 s | **7.2×** |
| 102 MB · 8,872 BH | 12.2 s | 2.1 s | **5.8×** |
| 265 MB · 21,952 BH | 30.5 s | 5.3 s | **5.8×** |
| 524 MB · 43,042 BH | 63.9 s | 11.4 s | **5.6×** |

**Read, strings** — like-for-like, both sides return the same all-string pandas
frames: `compat.AGS4_to_dataframe` vs `python-ags4`'s `AGS4_to_dataframe`:

| File | `python-ags4` `AGS4_to_dataframe` | `compat` | speedup |
|---:|---:|---:|:---:|
| 4.9 MB | 144 ms | 61 ms | **2.4×** |
| 24.9 MB | 669 ms | 249 ms | **2.7×** |
| 102 MB | 2.7 s | 1.0 s | **2.7×** |
| 265 MB | 7.4 s | 2.5 s | **2.9×** |
| 524 MB | 14.9 s | 5.2 s | **2.8×** |

**Read, typed** — real typed columns (floats, dates). python-ags4 gets there with
`AGS4_to_dataframe` + `convert_to_numeric` on every group; `laterite.read` is
born-typed, so the typing is inline — and it types dates too, which
`convert_to_numeric` skips:

| File | `python-ags4` + `convert_to_numeric` | `laterite.read` (born-typed) | speedup |
|---:|---:|---:|:---:|
| 4.9 MB · 459 BH | 187 ms | 52 ms | **3.6×** |
| 24.9 MB · 2,219 BH | 771 ms | 250 ms | **3.1×** |
| 102 MB · 8,872 BH | 3.0 s | 1.0 s | **3.0×** |
| 265 MB · 21,952 BH | 7.8 s | 2.6 s | **3.1×** |

`laterite.read` hands back born-typed polars frames (a `2DP` heading is a
`Float64`, a `DT` a `Datetime`) ready for `.sql()` — the same data python-ags4
gives you only after a separate conversion pass.

## Parity with python-ags4

121 / 131 of python-ags4 1.2.0's own test suite passes through
`laterite.compat` (92 %). The 10 remaining are deliberate non-closures,
documented rule by rule:

- [COMPAT.md](COMPAT.md) — the rule-by-rule differences and why
- [docs/parity-coverage-map.md](docs/parity-coverage-map.md) — the
  test-level map of laterite ↔ python-ags4

Every validator rule is written from the published AGS4 specification, not
adapted from another library's source — which is what lets laterite ship under
a permissive MIT licence.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Short version: PRs welcome, CI
gates `cargo test` + `pytest` + the python-ags4 parity oracle.
