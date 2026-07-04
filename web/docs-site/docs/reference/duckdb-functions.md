# DuckDB function reference

The `laterite_ags4` extension registers these table functions. See the [DuckDB
surface page](../duckdb/index.md) for worked examples.

```sql
INSTALL laterite_ags4 FROM community;
LOAD laterite_ags4;
```

## Validate & certify

| Function | Returns |
|---|---|
| `validate_ags(path)` | one row per finding: `rule, line, group, severity, desc`. Zero rows = clean. |
| `certify_ags(path)` | mints `<path>.ags.idx` (a validity certificate) for a clean file. |

Both accept `encoding := 'windows-1252'` to decode legacy files.

## Read

| Function | Returns |
|---|---|
| `read_ags(path, group)` | one group, columns cast to their AGS4 types. |
| `read_ags_text(content, group)` | the same, from an AGS4 string instead of a path. |

```sql
SELECT loca_id, loca_gl FROM read_ags('delivery.ags', 'LOCA') WHERE loca_gl < 0;
```

Every group is a table function, so **joins are plain SQL**:

```sql
SELECT s.samp_id, g.geol_leg
FROM read_ags('delivery.ags', 'SAMP') s
JOIN read_ags('delivery.ags', 'GEOL') g USING (loca_id);
```

## Inspect the file

| Function | Returns |
|---|---|
| `ags_groups(path)` | the group codes present in the file. |
| `ags_headings(path, group)` | a group's headings, units, and types. |

## Inspect the dictionary

The AGS4 dictionary ships *inside* the extension — no download.

| Function | Returns |
|---|---|
| `ags_dictionary()` | every group/heading with its unit and data type. |
| `ags_relationships()` | the group parent/child (KEY) graph. |

```sql
SELECT heading, unit, data_type FROM ags_dictionary() WHERE "group" = 'LOCA';
```

!!! note "Read/validate/query surface"
    DuckDB is a **query** door — it reads, validates, certifies, and inspects, but
    doesn't `fix`, `diff`, or emit AGS4 (those are the library and CLI surfaces).
    See the [capability matrix](../surfaces/index.md#what-each-door-can-do).
