<p align="center">
  <img src="https://raw.githubusercontent.com/niko86/laterite/main/assets/laterite-social-preview-white.png" alt="laterite — a modern AGS4 toolkit" width="600" />
</p>

# laterite

A modern **AGS4 toolkit** for the [AGS4](https://www.ags.org.uk/data-format/)
geotechnical data format: validate, read as born-typed data, query, build, fix,
diff, certify, and convert ↔ Excel.

**One Rust engine, surfaced natively for Python, Node.js, the CLI, DuckDB and the
browser** — the same rules, the same bytes, the same findings on every one.

Files come back born-typed — a `2DP` heading is a float, a `DT` a datetime, an
`ID` a string — so polars, SQL and the typed graph see real types, not text.

[![ci](https://github.com/niko86/laterite/actions/workflows/ci.yml/badge.svg)](https://github.com/niko86/laterite/actions/workflows/ci.yml)
[![PyPI](https://img.shields.io/pypi/v/laterite.svg)](https://pypi.org/project/laterite/)
[![npm](https://img.shields.io/npm/v/laterite.svg)](https://www.npmjs.com/package/laterite)
[![DuckDB](https://img.shields.io/badge/DuckDB-laterite__ags4-FFF000?logo=duckdb&logoColor=black)](https://community-extensions.duckdb.org/extensions/laterite_ags4.html)
[![Python versions](https://img.shields.io/pypi/pyversions/laterite.svg)](https://pypi.org/project/laterite/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![rust cov](https://img.shields.io/codecov/c/github/niko86/laterite?flag=rust&label=rust%20cov)](https://codecov.io/gh/niko86/laterite)
[![python cov](https://img.shields.io/codecov/c/github/niko86/laterite?flag=python&label=python%20cov)](https://codecov.io/gh/niko86/laterite)
[![node cov](https://img.shields.io/codecov/c/github/niko86/laterite?flag=node&label=node%20cov)](https://codecov.io/gh/niko86/laterite)
[![web cov](https://img.shields.io/codecov/c/github/niko86/laterite?flag=web&label=web%20cov)](https://codecov.io/gh/niko86/laterite)
[![wasm cov](https://img.shields.io/codecov/c/github/niko86/laterite?flag=wasm&label=wasm%20cov)](https://codecov.io/gh/niko86/laterite)

📖 **[Documentation](https://niko86.github.io/laterite/docs/)** · 🌐 **[Browser validator + data explorer](https://niko86.github.io/laterite/)** · 📓 **[Cookbook](https://niko86.github.io/laterite/docs/cookbook/)**

## Pick your surface

Every surface is the same engine — no surface re-implements a rule, and
scriptable output is byte-identical across them.

| Surface | Get it | Use it for | Docs |
|---|---|---|---|
| **Python** | `pip install laterite` | polars frames, the typed graph, the `python-ags4` drop-in | [guide](https://niko86.github.io/laterite/docs/surfaces/python/) |
| **Node.js** | `npm install laterite` | apache-arrow tables, server-side validation | [guide](https://niko86.github.io/laterite/docs/node/) |
| **CLI — `lat`** | [Releases](https://github.com/niko86/laterite/releases), or `pip install laterite` / `npx laterite` | pipelines, CI gates, one-off checks | [guide](https://niko86.github.io/laterite/docs/surfaces/cli/) |
| **DuckDB** | `INSTALL laterite_ags4 FROM community;` | SQL straight over `.ags` files, no conversion step | [guide](https://niko86.github.io/laterite/docs/duckdb/) |
| **Browser** | `npm i @laterite/ags4-wasm` | validate + explore in the page, nothing uploaded anywhere | [guide](https://niko86.github.io/laterite/docs/reference/wasm-api/) |

All five are **[in beta](https://niko86.github.io/laterite/docs/reference/support/)** —
the engine is tested; what it hasn't had is your files. The Rust crate
(`cargo add laterite`) is the one surface that isn't: same engine, but not yet at
parity with these five.

The **[web app](https://niko86.github.io/laterite/)** is built on the browser package
and nothing else — a worked example of that surface rather than a sixth one. Go and
use it; there's nothing to install.

## Quickstart

```python
import laterite

report = laterite.validate("delivery.ags")   # errors + warnings (FYI is opt-in)
report.is_valid

ags = laterite.read("delivery.ags")          # born-typed polars frames
ags["LOCA"]["LOCA_GL"][0]                    # → 12.3, a float — not "12.30"
```

<details>
<summary><b>Node.js</b></summary>

```ts
import { read, validate, buildAgs4 } from "laterite";

const ags = read("delivery.ags");                  // or read(bytes) / read(undefined, { text })
ags.table("LOCA").getChild("LOCA_GL")?.get(0);     // → 12.3, born-typed apache-arrow

validate("delivery.ags").toJson();                 // byte-identical to `lat validate --json`

buildAgs4(new Map([
  ["PROJ", [{ PROJ_ID: "P1", PROJ_NAME: "Demo" }]],
  ["LOCA", [{ LOCA_ID: "BH01", LOCA_GL: 12.3 }]],
])).save("out.ags");
```

Cross-group SQL (`ags.sql(...)` / `ags.at(...)`) needs the optional peer `@duckdb/node-api`.
</details>

<details>
<summary><b>CLI — <code>lat</code></b></summary>

```bash
uvx --from laterite lat validate delivery.ags   # try it with no install at all

lat delivery.ags                       # shorthand for `lat validate`
lat validate delivery.ags --json       # machine-readable findings
lat validate delivery.ags --no-warnings   # errors only
lat fix delivery.ags                   # repair → sibling .fixed.ags (safe fixes)
lat diff old.ags new.ags               # KEY-aware revision delta
lat certify delivery.ags               # mint delivery.ags.idx if clean
lat rules                              # the AGS4 rule catalogue
```

Exit codes: `0` clean · `1` findings · `3` unreadable · `4` not AGS4 · `5` bad args · `6` schema.
`lat --readme` prints the full guide.
</details>

<details>
<summary><b>DuckDB</b></summary>

```sql
INSTALL laterite_ags4 FROM community;
LOAD laterite_ags4;

-- every row carries an `_id` / `_parent_id` (UUIDv8 over the AGS key), so
-- parent↔child joins need no hand-written key list
SELECT l.loca_id, s.samp_ref, s.samp_top
FROM read_ags('delivery.ags', 'SAMP') s
JOIN read_ags('delivery.ags', 'LOCA') l ON s._parent_id = l._id;
```
</details>

<details>
<summary><b>Python — typed graph and the <code>python-ags4</code> drop-in</b></summary>

```python
# Typed graph: PROJ → LOCA → SAMP → …
from laterite.ags4 import read_typed
for loca in read_typed("delivery.ags").locas:
    print(loca.loca_id, loca.loca_gl)

# python-ags4 drop-in — swap the import, keep your code
from laterite import compat as AGS4
tables, headings = AGS4.AGS4_to_dataframe("delivery.ags")
```

The drop-in mirrors python-ags4's **library** API name for name. Its CLI is
deliberately not mirrored — see [Parity](#parity-with-python-ags4).
</details>

The [cookbook](https://niko86.github.io/laterite/docs/cookbook/) shows each task
side by side across Python, Node, DuckDB and the CLI, with synced tabs.

## What it does

The closest open-source tool, [`python-ags4`](https://gitlab.com/ags-data-format-wg/ags-python-library),
focuses on Python validation and pandas I/O — and inspired this project. laterite
matches that and adds a cross-surface toolchain on top:

| | `laterite` | `python-ags4` |
|---|:---:|:---:|
| Runs on | Python · Node · CLI · DuckDB · browser | Python |
| Validate — numbered AGS4 rules | ✅ | ✅ |
| Read → typed data | ✅ born-typed | strings; opt-in `convert_to_numeric` |
| Build / write AGS4 · Excel ↔ AGS4 | ✅ | ✅ |
| Repair engine (`fix`) | ✅ | — |
| SQL across groups · revision diff | ✅ | — |
| Validity certificates (`.ags.idx`) | ✅ | — |
| Transport — compress + encrypt | ✅ | — |
| Typed PROJ → LOCA → SAMP graph | ✅ | — |

## Performance

Synthetic, spec-valid AGS4 from `ags4-forge` — the `wide` scaffold: **123
groups**, realistic type mix, zero findings. macOS arm64, hot files, mean of 5
warm runs, against `python-ags4` 1.2.0. Each cell is laterite's time and its
speedup.

| File (123 groups) | `validate` | read → typed | read → strings (`compat`) |
|---:|---:|---:|---:|
| 4.9 MB · 459 BH | 50 ms · **30.0×** | 26 ms · **7.2×** | 49 ms · **2.9×** |
| 24.9 MB · 2,219 BH | 266 ms · **13.9×** | 136 ms · **6.0×** | 206 ms · **3.5×** |
| 102.7 MB · 8,872 BH | 1.1 s · **11.7×** | 541 ms · **6.3×** | 870 ms · **3.2×** |
| 275.5 MB · 22,813 BH | 2.6 s · **13.0×** | 1.4 s · **6.4×** | 2.2 s · **3.3×** |
| 549.7 MB · 45,107 BH | 5.4 s · **12.9×** | 2.9 s · **6.0×** | 4.6 s · **3.3×** |

Reproduce with `uv run python tools/bench-vs-python-ags4.py`. It generates the
rungs, verifies each against a pinned SHA-256 so a change to the generator can't
move the numbers unnoticed, and prints these tables. `node tools/bench-node.mjs`
does the same for the Node surface.

Read-typed is the honest comparison for real work: python-ags4 needs
`AGS4_to_dataframe` + `convert_to_numeric` on every group to get there, and still
skips dates — `laterite.read` is born-typed, dates included.

## Parity with python-ags4

121 / 131 of python-ags4 1.2.0's own test suite passes through
`laterite.compat` (92 %). The 10 remaining are deliberate non-closures,
documented rule by rule in [COMPAT.md](COMPAT.md) and the
[parity coverage map](docs/parity-coverage-map.md). A weekly job compares the two
public surfaces, so a function added upstream can't quietly go missing here.
<!-- cadence: parity -->


**The CLI is deliberately not mirrored.** laterite ships `lat` instead, with its
own JSON / NDJSON shapes — so a *script* that shells out to `ags4_cli` needs
porting rather than an import swap. Your Python code is unaffected.

Every validator rule is written from the published AGS4 specification, not
adapted from another library's source — which is what lets laterite ship under a
permissive MIT licence.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Short version: PRs welcome; CI gates
`cargo test` + `pytest` + the python-ags4 parity oracle.
