# DuckDB

The `laterite_ags4` extension puts the **same engine inside DuckDB** as SQL
table functions — validate and query AGS4 files *in place*, no import step, no
other language.

```sql
INSTALL laterite_ags4 FROM community;
LOAD laterite_ags4;
```

## Validate

```sql
-- one row per finding; a clean file returns zero rows
SELECT rule, line, "group", desc
FROM validate_ags('delivery.ags');
```

`validate_ags(path)` runs the numbered-rules engine and returns the findings as a
normal result set — filter, count, or aggregate them like any table. The AGS
edition is selected automatically from the file's `TRAN_AGS`.

## Read a group as typed columns

```sql
-- read_ags returns one group, already cast to AGS4 types
SELECT loca_id, loca_natn, loca_nate, loca_gl
FROM read_ags('delivery.ags', 'LOCA')
WHERE loca_gl < 0;
```

Because every group is a table function, **joins across groups are plain SQL** —
pull a borehole's samples and their lab results in one query, no glue code:

```sql
SELECT s.samp_id, s.samp_top, g.geol_leg
FROM read_ags('delivery.ags', 'SAMP') AS s
JOIN read_ags('delivery.ags', 'GEOL') AS g USING (loca_id);
```

`read_ags_text(content, group)` does the same from an AGS4 string instead of a
path.

## The function set

| Function | Returns |
|---|---|
| `validate_ags(path)` | the numbered-rule findings |
| `read_ags(path, group)` · `read_ags_text(text, group)` | one group, typed columns |
| `ags_groups(path)` | the groups present in the file |
| `ags_headings(path, group)` | a group's headings, units, and types |
| `ags_dictionary()` | the bundled AGS4 dictionary |
| `ags_relationships()` | the group parent/child (KEY) graph |
| `certify_ags(path)` | mints the `.ags.idx` validity certificate |

## Inspect the dictionary

The dictionary ships *inside* the extension — no download:

```sql
SELECT "group", heading, unit, data_type
FROM ags_dictionary()
WHERE "group" = 'LOCA';
```

!!! note "One engine, every stack"
    `validate_ags` is the identical rule engine behind
    [Python](../learn/validate.md), [Node](../node/index.md), and the
    [browser app](../surfaces/browser.md) — the cross-surface compliance matrix
    proves all six surfaces agree on findings.
