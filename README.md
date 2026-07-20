<p align="center">
  <img src="https://raw.githubusercontent.com/niko86/laterite/main/assets/laterite-social-preview-white.png" alt="laterite — a Rust-backed AGS4 reader, writer and validator" width="600" />
</p>

# laterite

A Rust-backed **AGS4 toolchain** for the
[AGS4](https://www.ags.org.uk/data-format/) geotechnical data format —
**validate, read as typed data, query, build, fix, diff, certify, and convert
↔ Excel** — surfaced natively for **Python, Node.js, the CLI, DuckDB and the
browser**, all on one clean-room Rust engine.

Coming from [`python-ags4`](https://gitlab.com/ags-data-format-wg/ags-python-library)?
`laterite` is a faster, faithful drop-in for its `AGS4` module — swap
`from python_ags4 import AGS4` for `from laterite import compat as AGS4`, keep
your code — and a full toolchain beyond it ([what that means](#more-than-a-faster-python-ags4)).

[![ci](https://github.com/niko86/laterite/actions/workflows/ci.yml/badge.svg)](https://github.com/niko86/laterite/actions/workflows/ci.yml)
[![rust cov](https://img.shields.io/codecov/c/github/niko86/laterite?flag=rust&label=rust%20cov)](https://codecov.io/gh/niko86/laterite)
[![python cov](https://img.shields.io/codecov/c/github/niko86/laterite?flag=python&label=python%20cov)](https://codecov.io/gh/niko86/laterite)
[![web cov](https://img.shields.io/codecov/c/github/niko86/laterite?flag=web&label=web%20cov)](https://codecov.io/gh/niko86/laterite)
[![PyPI](https://img.shields.io/pypi/v/laterite.svg)](https://pypi.org/project/laterite/)
[![Python versions](https://img.shields.io/pypi/pyversions/laterite.svg)](https://pypi.org/project/laterite/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

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
pip install laterite                  # base AGS4 (polars + duckdb)
pip install "laterite[compat]"        # + pandas (python-ags4 drop-in)
```

Requires Python ≥ 3.12.

## Use

The same engine drives every surface — pick your stack.

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

## More than a faster `python-ags4`

[`python-ags4`](https://gitlab.com/ags-data-format-wg/ags-python-library) is the
reference **Python** library for AGS4 — validation plus pandas read/write.
`laterite` is a faithful, faster drop-in for that
([parity](#parity-with-python-ags4) · [speed](#performance)) **and** a
cross-surface toolchain on top:

| | `laterite` | `python-ags4` |
|---|:---:|:---:|
| Runs on | Python · Node · CLI · DuckDB · browser | Python |
| Validate — numbered AGS4 rules | ✅ | ✅ |
| Read → typed data | ✅ born-typed (polars) | pandas (all strings) |
| Build / write AGS4 | ✅ | ✅ |
| Excel ↔ AGS4 | ✅ | ✅ |
| Repair engine (`fix`) — CRLF / BOM / short-row pad / embedded-CR… | ✅ | — |
| SQL across groups | ✅ (DuckDB) | — |
| Revision diff | ✅ | — |
| Validity certificates (`.ags.idx`) | ✅ | — |
| Transport — compress + encrypt | ✅ | — |
| Typed PROJ → LOCA → SAMP graph | ✅ | — |
| Shipped as a standalone binary CLI | ✅ (`lat`) | — |

laterite reports the same findings as `python-ags4`, with 10 documented
exceptions (see [Parity](#parity-with-python-ags4)).

## Performance

Validation throughput vs `python-ags4` 1.2.0, on synthetic AGS4 files of
increasing size (wall-clock after warmup, macOS arm64). The files are
LOCA-heavy — a 40-column real-schema LOCA group with `ID`/`PA`/`2DP`/`DT`
columns — carrying floating-point noise in the numeric cells that fails
AGS4 Rule 8, so **every cell triggers a finding**. This is the worst case:
it exercises the validator's per-finding accumulation and output-rendering
paths in full. laterite reports the same findings as `python-ags4` here.

| Size | python-ags4 | `laterite.validate` | `lat` (CLI) | Findings | speedup |
|---:|---:|---:|---:|---:|---:|
|   512 KB |    90 ms |   **7 ms** |   13 ms |     1 129 | **13×** |
|     5 MB |   547 ms |  **67 ms** |   92 ms |    11 485 |  **8×** |
|    50 MB |   6.09 s |  **0.79 s** | 0.96 s |   116 415 |  **8×** |
|   500 MB |  62.45 s |   **8.1 s** | 10.1 s | 1 170 223 |  **8×** |
|     1 GB | 132.67 s |  **18.3 s** | 22.9 s | 2 396 471 |  **7×** |

Notes on the CLI:

- `lat validate --json` writes findings to stdout as JSON
  (~80 bytes/finding). On the worst-case 1 GB file (2.4 M findings)
  that's ~190 MB of JSON. The native PyO3 path skips this
  serialisation step — for "validate then process findings in
  Python", use `laterite.validate`; for "validate then pipe to
  downstream tools", use `lat`. On clean files the gap is
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

121 / 131 of python-ags4 1.2.0's own test suite passes through
`laterite.compat` (92 %). The 10 remaining are deliberate non-closures,
documented rule by rule:

- [COMPAT.md](COMPAT.md) — the rule-by-rule differences and why
- [docs/parity-coverage-map.md](docs/parity-coverage-map.md) — the
  test-level map of laterite ↔ python-ags4

The validator is **clean-room** — every rule written from the AGS4 spec,
not copied from python-ags4 (LGPL-3.0) source — which is what lets laterite
ship under MIT.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Short version: PRs welcome, CI
gates `cargo test` + `pytest` + the python-ags4 parity oracle.

## License

[MIT](LICENSE). The bundled AGS4 standard dictionaries remain ©
[AGS](https://www.ags.org.uk/data-format/) and are redistributed
under their published terms.
