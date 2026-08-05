# Node

```bash
npm install laterite
```

`laterite` on npm is the **same clean-room engine** as the Python wheel, with a
born-typed **arrow-js** decode. The verbs match the [shared
vocabulary](../surfaces/index.md#the-shared-vocabulary) — `read`, `validate`,
`buildAgs4` — and, like Python, the read handle **chains**.

## Validate

```js
import { validate } from "laterite";

const report = validate("delivery.ags", { warnings: true });
console.log(report.isValid, report.count, report.dictVersion, report.resolution);
// true 0 "4.1.1" "exact"
```

`validate(path)` runs the numbered-rules engine and returns a `Report`. `isValid`
is the headline verdict (zero findings), `count` the number of findings, and
`dictVersion` / `resolution` tell you which AGS edition the rules came from —
selected automatically from the file's `TRAN_AGS`, never passed in. Group them by
rule (the same shape as Python's `by_rule()`):

```js
for (const [rule, findings] of Object.entries(report.byRule())) {
  console.log(rule, findings.length);
}
```

## Read a group as typed data

`read()` returns an `Ags4File`; `.table(code)` hands back an **arrow-js `Table`**
whose columns are already cast to their AGS4 types.

```js
import { read } from "laterite";

const file = read("delivery.ags");
file.groups; // ["PROJ", "LOCA", "SAMP", …]
const loca = file.table("LOCA"); // arrow-js Table, born-typed
console.log(loca.numRows, file.headings("LOCA"));
```

## The chain — validate · fix · diff · certify

The read handle is fluent, so a whole workflow stays on one object:

```js
const file = read("delivery.ags").validate({ warnings: true }); // returns the file; verdict on .report

if (!file.report.isValid) {
  const repaired = file.fix(); // returns a NEW repaired Ags4File
  const delta = file.diff(repaired); // what changed, group by group
  repaired.save("clean.ags");
}
```

- **`.fix(opts)`** → a new repaired `Ags4File` (the fix log on `.fixReport`).
- **`.diff(other)`** → a `RevisionDelta` of cell/row/group changes.
- **`.certify(path?)`** → mints the `.ags.idx` validity certificate beside the
  file (after a clean `.validate()`).

## Query with SQL

`.sql()` loads the optional `@duckdb/node-api` peer on demand (the addon itself is
DuckDB-free) and runs SQL across the file's groups:

```js
const rows = await file.sql("SELECT loca_id FROM LOCA WHERE loca_gl < 0");
```

## Produce AGS4

```js
import { buildAgs4 } from "laterite";

const out = buildAgs4({ PROJ: [{ PROJ_ID: "P1" }] /* … */ });
out.save("out.ags"); // or out.text / out.bytes
```

!!! note "Same engine, proven"
    Node's findings are asserted **byte-identical** to Python, wasm, and DuckDB
    by the cross-surface compliance harness — see [One engine, every
    stack](../surfaces/index.md).
