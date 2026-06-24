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

The same clean-room engine drives every surface — pick your stack.

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
report.toJson();                       // byte-identical to `lat-check --json`

// Produce valid AGS4 from plain rows (or a typed PROJ/LOCA graph)
const res = buildAgs4(new Map([
  ["PROJ", [{ PROJ_ID: "P1", PROJ_NAME: "Demo" }]],
  ["LOCA", [{ LOCA_ID: "BH01", LOCA_GL: 12.3 }]],
]), { mode: "autofix" });
res.save("out.ags");
```

Cross-group SQL (`ags.sql(...)` / `ags.at(...)`) needs the optional peer `@duckdb/node-api`.

### CLI (`lat-check`)

```bash
lat-check delivery.ags                 # human report; exit 0 clean / 1 findings
lat-check delivery.ags --json          # machine-readable findings (pretty JSON)
lat-check delivery.ags --no-warnings   # errors only (warnings show by default)
lat-check delivery.ags --fix           # repair → sibling .fixed.ags (safe fixes)
lat-check old.ags --diff new.ags       # KEY-aware revision delta (+added -removed ~changed)
lat-check --list-rules                 # the AGS4 rule catalogue (no input needed)
```

Exit codes: `0` clean · `1` findings · `3` unreadable · `4` not AGS4 · `5` bad args.
Run `lat-check --readme` for the full guide.

## Performance

Validation throughput vs `python-ags4` 1.2.0, on synthetic AGS4 files
of increasing size. Wall-clock after warmup, macOS arm64. All four
laterite paths agree on the **same findings** here, matching
`python-ags4`'s on these files — speed not at the cost of diagnostic
coverage. (The [Parity](#parity-with-python-ags4) section has the full
comparison: 122 / 131 of python-ags4's own test suite, divergences
documented.)

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
