# laterite

AGS4 geotechnical data for **Node.js** — read, validate, produce, and query, backed by a Rust engine (the Node surface of the [`laterite`](https://github.com/niko86/laterite) toolkit).

Born-typed: a `2DP` heading comes back as a JavaScript number, an `ID` as a string, a `DT` as a `Date` — decoded directly from the engine's typed Arrow, the same typing the Python and browser-wasm hosts produce.

```bash
npm install laterite
```

> **laterite is in beta** — a statement about how much real-world use it has had, not
> how much testing. The API can still change; what it runs on is the steadier promise.
> [What that means](https://laterite.dev/docs/reference/support/) ·
> [tell us how it goes](https://laterite.dev/docs/feedback/).

Prebuilt native binaries ship for linux-x64-gnu, darwin-arm64, and win32-x64-msvc (auto-selected via `optionalDependencies`). No build step, no Python, no toolchain.

## Read & validate

```ts
import { read, validate } from "laterite";

const ags = read("delivery.ags"); // path, or read(bytes) / read(undefined, { text })
ags.groups; // ["PROJ", "LOCA", "SAMP", …]
const loca = ags.table("LOCA"); // a born-typed apache-arrow Table
loca.getChild("LOCA_GL")?.get(0); // → 12.3 (a number)

const report = validate("delivery.ags");
report.isValid; // boolean
report.findings; // [{ rule, line?, group, desc, severity? }]
report.toJson(); // byte-identical to `lat validate --json`
```

## Produce AGS4

From per-group data (arrow-js Tables or plain row objects):

```ts
import { buildAgs4 } from "laterite";

const res = buildAgs4(
  new Map([
    ["PROJ", [{ PROJ_ID: "P1", PROJ_NAME: "Demo" }]],
    ["LOCA", [{ LOCA_ID: "BH01", LOCA_GL: 12.3 }]],
  ]),
  { edition: "4.1.1", mode: "autofix" },
);

res.text; // the AGS4 document
res.save("out.ags");
```

The engine formats each cell from the heading's declared AGS4 TYPE — a `2DP` column emits `12.30`, a `3SF` column `1.23e4`, a `DT` a spec-shaped timestamp — so pass raw typed values and let it render. Pre-stringified cells are written through verbatim.

…or from a **typed builder graph** (`import { PROJ, LOCA } from "laterite"`):

```ts
import { PROJ, LOCA, buildAgs4 } from "laterite";

const proj = new PROJ({
  PROJ_ID: "P1",
  PROJ_NAME: "Demo",
  locas: [new LOCA({ LOCA_ID: "BH01", LOCA_GL: 12.3 })],
});
buildAgs4(proj); // walks the tree → valid AGS4
```

## SQL across groups (optional)

`sql()` / `at()` need the optional peer `@duckdb/node-api`:

```bash
npm install @duckdb/node-api
```

<!-- doc-snippet: skip — `{ arrow: true }` downloads DuckDB's `arrow` community extension at call time; gating on a network fetch would make this red whenever the registry is unreachable, the same reason the duckdb example tree runs monthly rather than per-PR -->

```ts
const ags = read("delivery.ags");

// cross-group JOIN — plain JS row objects by default
const rows = await ags.sql(
  "SELECT * FROM SAMP JOIN LOCA USING (LOCA_ID) WHERE LOCA_GL > 50",
);

// key-filter a location's whole related record set
const frames = await ags.at("LOCA", ["BH01", "BH02"]).frames();

// opt into arrow-js Table output (loads DuckDB's `arrow` community extension)
const table = await ags.sql("SELECT * FROM LOCA", { arrow: true });
```

## Command line — `npx laterite`

The package ships the **`lat`** CLI too — the same AGS4 tool as the standalone Rust binary and `uvx --from laterite lat`, one launcher per ecosystem:

```bash
npx laterite validate delivery.ags          # or `lat …` after a global install
npx laterite read delivery.ags LOCA --csv    # dump a group (raw file cells)
npx laterite diff old.ags new.ags            # revision delta
npx laterite pack delivery.ags delivery.ags.zst
npx laterite excel delivery.ags delivery.xlsx
```

Verbs: `validate` (the default — a bare `lat <file>` runs it) · `read` · `fix` ·
`diff` · `certify` · `rules` · `pack` / `unpack` / `lock` / `unlock` · `excel`. The
scriptable outputs (`validate --json` / `--ndjson`, `read --json` / `--csv`,
`rules --json`) are **byte-identical** to the Rust binary; `lock` / `unlock` take
the passphrase from `--password-file` or `$LAT_TRANSPORT_PASSWORD` (never a flag).

## Performance

Node 24, macOS arm64, hot files, mean of 5 warm runs. Fixtures are synthetic,
spec-valid AGS4 from `ags4-forge` — the `wide` scaffold: **123 groups**,
realistic type mix, zero findings.

|    File (123 groups) |            `read` |        `validate` | `read` + all typed tables |
| -------------------: | ----------------: | ----------------: | ------------------------: |
|      4.9 MB · 459 BH |  24 ms · 203 MB/s |   53 ms · 92 MB/s |          32 ms · 153 MB/s |
|   24.9 MB · 2,219 BH | 121 ms · 205 MB/s | 229 ms · 109 MB/s |         135 ms · 184 MB/s |
|  102.7 MB · 8,872 BH | 451 ms · 227 MB/s | 921 ms · 111 MB/s |         619 ms · 166 MB/s |
| 275.5 MB · 22,813 BH |  1.2 s · 238 MB/s |  2.4 s · 117 MB/s |          1.8 s · 151 MB/s |
| 549.7 MB · 45,107 BH |  2.7 s · 206 MB/s |  5.0 s · 111 MB/s |          3.9 s · 141 MB/s |

`read` parses and holds the file; the third column adds materialising **every**
group to an apache-arrow Table, which is what a consumer actually pays. Throughput
holds across the whole range — a half-gigabyte delivery is the 4.9 MB rate, not a
cliff.

Reproduce with `node tools/bench-node.mjs` in the repo. It generates the rungs and
verifies each against a pinned SHA-256, so a change to the fixture generator can't
move these numbers unnoticed.

## Also exported

- `agsTypes` — `canonicalType` / `displayHint` / `parseValue`
- `registry` — `GROUPS`, `childGroups`, `ancestorChain`, …
- `transport` — `pack` / `unpack` (zstd) + `lock` / `unlock` (age passphrase)

## Part of the laterite suite

One clean-room Rust AGS4 engine, surfaced for every stack. Scriptable output is byte-identical across all of them, so a CI gate and a notebook can't disagree.

| Surface     | Package                                                                                                        | Get it                                  |
| ----------- | -------------------------------------------------------------------------------------------------------------- | --------------------------------------- |
| **Node.js** | [`laterite`](https://www.npmjs.com/package/laterite) — npm                                                     | `npm install laterite`                  |
| **Python**  | [`laterite`](https://pypi.org/project/laterite/) — PyPI                                                        | `pip install laterite`                  |
| **CLI**     | [`lat`](https://github.com/niko86/laterite/releases)                                                           | bundled here, or GitHub Releases        |
| **DuckDB**  | [`laterite_ags4`](https://community-extensions.duckdb.org/extensions/laterite_ags4.html) — community extension | `INSTALL laterite_ags4 FROM community;` |
| **Browser** | [validator + data explorer](https://laterite.dev/) — WASM                                         | open in a browser                       |

**Running in a browser?** This package is Node-only — it loads a native addon via
`optionalDependencies` and touches the filesystem. Use the
[wasm build](https://github.com/niko86/laterite/tree/main/rust-packages/laterite-ags4-wasm)
instead: same engine, same rules, no `fs`.

📖 [Documentation](https://laterite.dev/docs/) · [Node guide](https://laterite.dev/docs/node/) · [Cookbook](https://laterite.dev/docs/cookbook/)

## License

MIT
