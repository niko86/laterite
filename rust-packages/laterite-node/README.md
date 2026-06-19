# laterite

AGS4 geotechnical data for **Node.js** — read, validate, produce, and query, backed by a Rust engine (the Node port of the [`laterite`](https://pypi.org/project/laterite/) Python toolkit).

Born-typed: a `2DP` heading comes back as a JavaScript number, an `ID` as a string, a `DT` as a `Date` — decoded directly from the engine's typed Arrow, the same typing the Python and browser-wasm hosts produce.

```bash
npm install laterite
```

Prebuilt native binaries ship for linux-x64-gnu, darwin-arm64, and win32-x64-msvc (auto-selected via `optionalDependencies`).

## Part of the laterite suite

One clean-room Rust AGS4 engine, surfaced for every stack:

| Surface | Package | Get it |
|---|---|---|
| **Python** | [`laterite`](https://pypi.org/project/laterite/) — PyPI | `pip install laterite` |
| **Node.js** | [`laterite`](https://www.npmjs.com/package/laterite) — npm | `npm install laterite` |
| **Rust / CLI** | [`lat-db` + `lat-check`](https://github.com/niko86/laterite/releases) | GitHub Releases |
| **Browser** | [validator + data explorer](https://niko86.github.io/laterite/) — WASM | open in a browser |

## Read & validate

```ts
import { read, validate } from "laterite";

const ags = read("delivery.ags");        // path, or read(bytes) / read(undefined, { text })
ags.groups;                               // ["PROJ", "LOCA", "SAMP", …]
const loca = ags.table("LOCA");           // a born-typed apache-arrow Table
loca.getChild("LOCA_GL")?.get(0);         // → 12.3 (a number)

const report = validate("delivery.ags");
report.isValid;                           // boolean
report.findings;                          // [{ rule, line?, group, desc, severity? }]
report.toJson();                          // byte-identical to `lat-check --json`
```

## Produce AGS4

From per-group data (arrow-js Tables or plain row objects):

```ts
import { buildAgs4 } from "laterite";

const res = buildAgs4(new Map([
  ["PROJ", [{ PROJ_ID: "P1", PROJ_NAME: "Demo" }]],
  ["LOCA", [{ LOCA_ID: "BH01", LOCA_GL: 12.3 }]],
]), { edition: "4.1.1", mode: "autofix" });

res.text;          // the AGS4 document
res.save("out.ags");
```

…or from a **typed builder graph** (`import { PROJ, LOCA } from "laterite"`):

```ts
import { PROJ, LOCA, buildAgs4 } from "laterite";

const proj = new PROJ({
  PROJ_ID: "P1",
  PROJ_NAME: "Demo",
  locas: [new LOCA({ LOCA_ID: "BH01", LOCA_GL: 12.3 })],
});
buildAgs4(proj);    // walks the tree → valid AGS4
```

## SQL across groups (optional)

`sql()` / `at()` need the optional peer `@duckdb/node-api`:

```bash
npm install @duckdb/node-api
```

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

## Also exported

- `agsTypes` — `canonicalType` / `displayHint` / `parseValue`
- `registry` — `GROUPS`, `childGroups`, `ancestorChain`, …
- `transport` — `pack` / `unpack` (zstd) + `lock` / `unlock` (age passphrase)

## License

MIT
