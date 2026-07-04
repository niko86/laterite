# Node API reference

The `laterite` npm package. Import the free functions, or chain on the `Ags4File`
a `read()` returns. Full surface — see the [Node surface page](../node/index.md)
for the guided tour.

## Free functions

| Function | Signature | Returns |
|---|---|---|
| `read` | `read(source?, opts?)` | `Ags4File` |
| `validate` | `validate(source?, opts?)` | `Report` |
| `buildAgs4` | `buildAgs4(groups, opts?)` | `Ags4File` |
| `fix` | `fix(source?, opts?)` | `FixResult` |
| `diff` | `diff(a, b, opts?)` | `RevisionDelta` |
| `listRules` | `listRules()` | `RuleMeta[]` |
| `toExcel` | `toExcel(agsPath, xlsxPath, { groups? })` | `ExcelStats` |
| `fromExcel` | `fromExcel(xlsxPath, agsPath, { formatNumericColumns? })` | `ExcelStats` |

`source` is a path (`string`) or `Uint8Array`; `read`/`validate` also take
`{ text }` in `opts` to read from an in-memory string. `validate`/`fix` accept
`{ warnings, fyi, checkFiles, encoding, dictVersion }`.

```js
import { read, validate, buildAgs4 } from "laterite";
```

## `Ags4File`

Returned by `read()` and `buildAgs4()`. The read handle is **fluent** — the
chained verbs return something you keep working on.

### Inspect

| Member | Returns | |
|---|---|---|
| `groups` | `string[]` | group codes present |
| `tranAgs` | `string \| null` | the declared edition |
| `has(code)` | `boolean` | is a group present |
| `headings(code)` · `units(code)` · `types(code)` · `sqlTypes(code)` | `string[]` | a group's column metadata |
| `lineNumbers(code)` | `number[]` | source lines of a group's rows |

### Data

| Member | Returns | |
|---|---|---|
| `table(code, { keys? })` | arrow-js `Table` | one group, born-typed columns |
| `text` | `string` | the AGS4 text |
| `bytes` | `Buffer` | the AGS4 bytes |
| `save(path)` | `string` | write to disk |
| `sql(query, { arrow? })` | `Promise<Row[] \| Table>` | SQL across groups (loads the `@duckdb/node-api` peer) |
| `at(group, values)` | `AgsSubset` | a group filtered to KEY values |

### Verbs (chained)

| Member | Returns | |
|---|---|---|
| `validate(opts?)` | `this` | run the engine; verdict on `.report` |
| `fix(opts?)` | `Ags4File` | a **new** repaired file; log on `.fixReport` |
| `diff(other, opts?)` | `RevisionDelta` | cell/row/group changes vs another file |
| `certify(path?)` | `string` | mint the `.ags.idx` certificate (after a clean `validate`) |
| `report` | `Report \| undefined` | the last `validate()` verdict |
| `fixReport` | `FixResult \| undefined` | the last `fix()` log |

```js
const file = read("delivery.ags").validate({ warnings: true });
if (!file.report.ok) file.fix().save("clean.ags");
```

!!! note "Python-only"
    The `python-ags4` compat shim is the one Python-only surface — see the
    [capability matrix](../surfaces/index.md#what-each-door-can-do). Excel I/O
    (`toExcel` / `fromExcel`) landed on Node in #358.
