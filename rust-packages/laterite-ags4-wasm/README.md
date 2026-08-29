# laterite — WebAssembly

The **browser / edge** surface of the [laterite](https://github.com/niko86/laterite)
AGS4 engine. Same Rust core as the Python wheel, the Node addon, the `lat` CLI and
the DuckDB extension — same numbered rules, same findings, same emitted bytes.

Everything runs in the caller's process. Nothing is uploaded, and there is no
server to run.

> **laterite is in beta** — a statement about how much real-world use it has had, not
> how much testing. The API can still change; what it runs on is the steadier promise.
> [What that means](https://docs.laterite.dev/reference/support/) ·
> [tell us how it goes](https://docs.laterite.dev/feedback/).

## Read this first — what's different from the native surfaces

WebAssembly is not just "the library, compiled". Four things change:

| | native (Python / Node / CLI) | wasm |
|---|---|---|
| Startup | import and go | **`await init()` first** — every export throws before it |
| Input | a file path, or bytes | **bytes only** (`Uint8Array`) — no filesystem |
| Cross-group SQL | built in (`ags.sql(…)`) | **you** supply duckdb-wasm; laterite hands you Arrow IPC |
| Transport (`pack` / `lock`) | ✅ | not available — zstd + age don't cross the wasm boundary |

Validate, read-as-typed, build and fix are all present and behave identically.
Five more surfaces are **not in this package** — see below.

## What's in the package

A browser downloads what you ship it, so this package carries the surfaces a page
actually needs and leaves the rest to a from-source build:

| | here |
|---|---|
| `validate` — the full numbered-rule engine | ✅ |
| `read` → `group_codes` · `meta` · `rows_json` | ✅ |
| `build_ags4` — data → valid AGS4 | ✅ |
| `compute_fixes` · `apply_fixes` | ✅ |
| `list_rules` · `dictionary` | ✅ |
| `version` · `engine_version` · `engine_fingerprint` | ✅ |
| `arrow_ipc` · `build_ags4_ipc` — Arrow IPC for duckdb-wasm | [build from source](#building-a-bigger-engine) |
| `ags4_to_xlsx` · `xlsx_to_ags4` | [build from source](#building-a-bigger-engine) |
| `certify` · `diff` · `merge` · `censor` | [build from source](#building-a-bigger-engine) |

The whole read → validate → **fix** → write chain is here; nothing is missing
from the middle of it. Two of the omissions have a replacement rather than being
simply absent: `rows_json()` reads a group without Arrow, and `build_ags4` takes
as JSON what `build_ags4_ipc` takes as Arrow.

If you were using 0.10.x or earlier, note that `read` is where this bites: that
build returned Arrow IPC via `arrow_ipc()`, and this one returns JSON via
`rows_json()`. **It is not a drop-in for that call.** The values are the same —
same cast, same types — only the framing differs.

## Size

| | raw | gzip | brotli |
|---|---:|---:|---:|
| **this package** | **1.8 MiB** | **749 KiB** | **573 KiB** |
| everything, from source | 5.1 MiB | 1.71 MiB | 1.24 MiB |

Roughly 2.3× smaller on the wire than the full engine. Load it lazily either
way — the module is only needed once a user actually opens a file, so keep it
off the critical path with a dynamic `import()`.

Two wasm modules are built from this repo: this one and
`laterite-ags4-tokenizer-wasm`, a much smaller leaf that only tokenises lines —
enough for syntax highlighting in an editor without paying for the engine at all.

## Install it

```bash
npm i @laterite/ags4-wasm
```

An ES module built with `--target web`: `ags4_wasm.js` with a generated
`ags4_wasm.d.ts` beside it.

### Building a bigger engine

The five omitted surfaces are cargo features, and they are **on by default** — so
a plain source build gives you everything, and the published package is the
deliberately trimmed one:

```bash
cargo install wasm-pack

# everything: excel, arrow, certify, diff, merge, censor
wasm-pack build rust-packages/laterite-ags4-wasm --target web --release --out-dir pkg

# this package's shape, plus just the one you want back
wasm-pack build rust-packages/laterite-ags4-wasm --target web --release --out-dir pkg \
  -- --no-default-features --features arrow
```

Cargo flags go **after `--`**. wasm-pack forwards everything past it and exits
**zero** when they land in the wrong place, so check that the artifact appeared
rather than trusting the exit code.

`--target web` emits an ES module for browsers; `--target bundler` suits
webpack/rollup/vite, `--target nodejs` a CommonJS host. wasm-pack writes a
complete `.d.ts` next to the module — **that file is the authoritative API
reference**, generated from the Rust signatures, so it can't drift from what
you're calling, and it lists exactly the features you built with.

## Use it

```ts
import init, { validate, read, version } from "@laterite/ags4-wasm";

await init();                       // required, once, before anything else
version();                          // "0.12.0"

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

// One group's rows, values already correctly typed
const rows = JSON.parse(ds.rows_json("LOCA"));
// [["BH01", 451105.75], ["BH02", 451235.21], …]

ds.free();                                 // before the next parse
```

`meta` and `rows_json` are **positional against each other** — `headings[i]`
names `rows[r][i]` — and a short DATA row pads with `null` rather than coming
back narrow, so the two always zip.

Values are born typed, off the file's own `TYPE` row and through the same cast
the Python wheel and the DuckDB extension apply: a `2DP` heading is a JSON
number, a `DT` a `"yyyy-mm-dd hh:mm:ss"` string, a blank or unparseable cell
`null`. It is a JSON *string* rather than an array because one `JSON.parse` beats
building a boxed value per cell across the wasm boundary.

`rows_json` materialises **one group at a time** and drops it on return, so peak
memory is a single group rather than the whole file. The dataset is a handle into
wasm memory: hold it while you pull, and `free()` it before the next parse.

### Round-tripping into `build_ags4`

`meta` and `rows_json` together are exactly `build_ags4`'s input shape, so
read → edit → write needs no adapter:

```ts
const groups = ds.group_codes().map((code) => {
  const { headings, units, types } = ds.meta(code);
  return { code, headings, units, types, rows: JSON.parse(ds.rows_json(code)) };
});
const out = build_ags4(JSON.stringify(groups), { mode: "autofix" });
```

### Cross-group SQL, via duckdb-wasm

laterite doesn't embed a query engine in the browser — you bring
[duckdb-wasm](https://duckdb.org/docs/api/wasm/overview.html) and laterite feeds
it Arrow. That path needs the `arrow` feature, so
[build from source](#building-a-bigger-engine) for it; `arrow_ipc(code, keys)`
then prepends the content-addressed `_id` / `_parent_id` columns, which are **the
same UUIDv8s** the wheel, Node and the DuckDB extension mint, so joins resolve
across surfaces:

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

It is not in this package because Arrow is roughly a third of the compiled
engine, and a caller who is not driving duckdb-wasm would pay half a megabyte for
bytes they only parse back.

### Building AGS4

```ts
import { build_ags4, build_ags4_ipc } from "@laterite/ags4-wasm";

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
`[{ code, ipc }]` and skips the per-cell JSON round-trip entirely — with the
`arrow` feature, [from source](#building-a-bigger-engine).

### Also exported

`compute_fixes` / `apply_fixes` · `list_rules` · `dictionary` · `version` /
`engine_version` / `engine_fingerprint`.

With the matching feature, [from source](#building-a-bigger-engine): `certify` ·
`diff` · `merge` · `censor` · `ags4_to_xlsx` / `xlsx_to_ags4`.

### Two calling conventions, and which is which

The surface is **mid-migration**, so read this before guessing (the gated exports
are listed too — the convention travels with the export, not with the build):

| Export | Shape |
|---|---|
| `validate` · `certify` · `build_ags4` · `build_ags4_ipc` · `merge` | `(inputs…, opts?)` — **named options**, unknown keys refused |
| `compute_fixes` · `apply_fixes` · `diff` · `censor` · `read` · `ags4_to_xlsx` · `xlsx_to_ags4` | still **positional** |

The migrated exports call the text-encoding option `encoding`, matching Python,
Node and the CLI. The positional ones still name it `encoding_label` — the
browser was the only surface ever carrying that `_label` suffix, and the name
moves with each export rather than being changed underneath a signature that
hasn't. So a mixed spelling in one module is a **recorded state**, not an
oversight; it ends when the table's second row empties.

An arity gate (`test_modality_parity.py`) holds the migrated exports at their
current shape and lists the rest with the reason, because CI excludes this crate
from clippy — `too_many_arguments` has never fired here and would not fire on
the next export either.

## See it running

<https://app.laterite.dev/> is this engine — a full AGS4 validator and data
explorer, entirely client-side. It compiles the crate itself with every feature
on, so it shows the Excel, diff, merge and duckdb-wasm paths this package leaves
out. Its source is in
[`web/`](https://github.com/niko86/laterite/tree/main/web) and is the reference
integration: lazy module load, duckdb-wasm wiring, and one group at a time.

📖 [Documentation](https://docs.laterite.dev/) · [Browser guide](https://docs.laterite.dev/surfaces/browser/)

## License

MIT
