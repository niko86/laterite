# laterite — WebAssembly

The **browser / edge** surface of the [laterite](https://github.com/niko86/laterite)
AGS4 engine. Same Rust core as the Python wheel, the Node addon, the `lat` CLI and
the DuckDB extension — same numbered rules, same findings, same emitted bytes.

Everything runs in the caller's process. Nothing is uploaded, and there is no
server to run.

## Read this first — what's different from the native surfaces

WebAssembly is not just "the library, compiled". Four things change:

| | native (Python / Node / CLI) | wasm |
|---|---|---|
| Startup | import and go | **`await init()` first** — every export throws before it |
| Input | a file path, or bytes | **bytes only** (`Uint8Array`) — no filesystem |
| Cross-group SQL | built in (`ags.sql(…)`) | **you** supply duckdb-wasm; laterite hands you Arrow IPC |
| Transport (`pack` / `lock`) | ✅ | not available — zstd + age don't cross the wasm boundary |

Validate, read-as-typed, build, fix, diff, merge, certify and Excel ↔ AGS4 are all
present and behave identically.

## Size

| | bytes |
|---|---:|
| `ags4_wasm_bg.wasm` raw | 7.1 MB |
| gzip | 1.9 MB |
| **brotli** | **1.35 MB** |

Serve it with brotli and it is a one-time 1.35 MB fetch, cached thereafter. Load
it lazily — the module is only needed once a user actually opens a file, so keep
it off the critical path with a dynamic `import()`.

Two wasm modules are built from this repo: this one (the full engine) and
`laterite-ags4-tokenizer-wasm`, a much smaller leaf that only tokenises lines —
enough for syntax highlighting in an editor without paying for the engine.

## Build it

There is no npm package yet — build from the repo:

```bash
cargo install wasm-pack
wasm-pack build rust-packages/laterite-ags4-wasm --target web --release --out-dir pkg
```

`--target web` emits an ES module for browsers; `--target bundler` suits
webpack/rollup/vite, `--target nodejs` a CommonJS host. wasm-pack writes a
complete `.d.ts` next to the module — **that file is the authoritative API
reference**, generated from the Rust signatures, so it can't drift from what
you're calling.

## Use it

```ts
import init, { validate, read, version } from "./pkg/ags4_wasm.js";

await init();                       // required, once, before anything else
version();                          // "0.8.2"

const bytes = new Uint8Array(await file.arrayBuffer());

// Validate — same findings as every other surface, shaped for a UI.
// Every option is named and optional; `warnings` defaults ON, `fyi` OFF.
const report = validate(bytes);

// …or with options. An unrecognised key is REFUSED by name (with a suggestion),
// rather than silently taking its default:
const report = validate(bytes, {
  dictVersion: "4.1.1",   // "auto" (or omitted) reads the file's TRAN_AGS
  fyi: true,
  maxPerRule: 50,         // clip what CROSSES the boundary; totals stay true
});

report.ok;                 // the verdict: no error and no findings
report.dict_version;       // the edition judged against, e.g. "4.1.1"
report.finding_count;      // the TRUE total, never clipped
report.findings;           // grouped by rule: [{ rule, total, items: [{ line, group, desc, … }] }]
report.error;              // { kind, message } when the input wasn't validatable at all
```

`validate`'s positional arguments mirror the CLI's flags, in order:
`dict_version`, `include_warnings`, `include_fyi`, `encoding_label`,
`max_per_rule`, `dict_bytes`, `dict_replace`. Pass `undefined` for the defaults.

The **findings** are the same on every surface; the **envelope** is not. This
report is shaped for a UI — findings pre-grouped by rule, with counts — where
`lat validate --json` emits `{ file, findings: { "<rule>": [...] } }`. Don't
write a parser expecting one to be the other.

Two things to get right. **`error` is not a finding** — it means the input wasn't
validatable (wrong format, unsupported edition), no rules ran, and every other
field is empty; branch on it before reading `findings`. And when you pass
`max_per_rule`, each rule group's `items` is clipped but its `total` is not, so
render "showing N of `total`" rather than reporting the clipped length.

### Reading typed data

`read` is permissive on purpose — it builds typed columns from whatever parsed, so
an explorer still works on a file with findings. Only genuinely-unparseable input
throws.

```ts
const ds = read(bytes, undefined);        // ParsedDataset
ds.group_codes();                          // ["PROJ", "LOCA", "SAMP", …]
ds.meta("LOCA");                           // { headings, units, types, sql_types }

// One group as an Arrow IPC stream — columns already correctly typed
const ipc = ds.arrow_ipc("LOCA", false, false);
```

`arrow_ipc` materialises **one group at a time** and drops it on return, so peak
memory is a single group rather than the whole file. Feed the `Uint8Array` to
[apache-arrow](https://www.npmjs.com/package/apache-arrow) or straight into
duckdb-wasm.

### Cross-group SQL, via duckdb-wasm

laterite doesn't embed a query engine in the browser — you bring
[duckdb-wasm](https://duckdb.org/docs/api/wasm/overview.html) and laterite feeds
it. Pass `keys = true` to prepend the content-addressed `_id` / `_parent_id`
columns, which are **the same UUIDv8s** the wheel, Node and the DuckDB extension
mint, so joins resolve across surfaces:

```ts
for (const code of ds.group_codes()) {
  await conn.insertArrowFromIPCStream(ds.arrow_ipc(code, true, false), {
    name: code,
    create: true,
  });
}
await conn.query(`
  SELECT l.LOCA_ID, s.SAMP_REF
  FROM LOCA l JOIN SAMP s ON s._parent_id = l._id
`);
```

### Building AGS4

```ts
import { build_ags4, build_ags4_ipc } from "./pkg/ags4_wasm.js";

// an ARRAY of groups, columnar — `units` / `types` are optional per-heading
// overrides, and the dictionary fills in whatever you leave out
const res = build_ags4(JSON.stringify([
  {
    code: "PROJ",
    headings: ["PROJ_ID", "PROJ_NAME"],
    rows: [["P1", "Demo"]],
  },
  {
    code: "LOCA",
    headings: ["LOCA_ID", "LOCA_GL"],
    rows: [["BH01", 12.3], ["BH02", 9.87]],
  },
]), { dictVersion: "4.1.1", mode: "autofix" });

res.text;             // the AGS4 document (UTF-8, CRLF) — wrap in a Blob
res.findings;
res.fixes_applied;
```

The engine formats every cell from the heading's declared AGS4 TYPE — a `2DP`
column emits `12.30`, a `3SF` column `1.23e4`, a `DT` a spec-shaped timestamp — so
pass raw JSON numbers and dates and let it render. A cell you pass as a **string**
is written through verbatim; don't pre-format.

`mode` is `"autofix"` (default) · `"report"` · `"strict"`. Note that `autofix`
repairs what the input contains but does **not** mint the mandatory UNIT / TYPE /
ABBR catalogues — a data-only build reports Rules 14/15/17 rather than silently
inventing metadata. Opt in with `synthesiseMetadata`:

```ts
build_ags4(groupsJson, { synthesiseMetadata: true });
```

`TRAN` is **never** synthesised, even with that on: only you know who sent what
to whom, and a placeholder that *satisfies* Rule 14 asserts a transmission that
never happened. State it instead — all five are REQUIRED headings, so they are
required together:

```ts
build_ags4(groupsJson, {
  synthesiseMetadata: true,
  tran: {
    issue: "1",
    date: "2026-07-30",
    producer: "Your Firm",
    recipient: "The Client",
    status: "FINAL",
  },
});
```

Omit `tran` and no `TRAN` is written; Rule 14 reports the gap.

For large, already-columnar data (a duckdb-wasm result), `build_ags4_ipc` takes
`[{ code, ipc }]` and skips the per-cell JSON round-trip entirely.

### Also exported

`certify` · `compute_fixes` / `apply_fixes` · `diff` · `merge` ·
`ags4_to_xlsx` / `xlsx_to_ags4` · `list_rules` · `dictionary` · `censor`.

## See it running

<https://niko86.github.io/laterite/> is this module — a full AGS4 validator and
data explorer, entirely client-side. Its source is in
[`web/`](https://github.com/niko86/laterite/tree/main/web) and is the reference
integration: lazy module load, duckdb-wasm wiring, and one group at a time.

📖 [Documentation](https://niko86.github.io/laterite/docs/) · [Browser guide](https://niko86.github.io/laterite/docs/surfaces/browser/)

## License

MIT
