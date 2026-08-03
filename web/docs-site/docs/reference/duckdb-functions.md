# DuckDB function reference

The `laterite_ags4` extension registers these table functions. See the [DuckDB
surface page](../duckdb/index.md) for worked examples.

```sql
INSTALL laterite_ags4 FROM community;
LOAD laterite_ags4;
```

## Validation & certification live elsewhere

This extension is a **read-only reader** — there is no `validate_ags` /
`certify_ags` in SQL. Run the numbered rules and mint an `.ags.idx` certificate
with the [`lat` CLI](cli.md) (`lat validate` / `lat certify`) or the `laterite`
Python/Node library. `read_ags` then **consumes** an externally-minted `.ags.idx`
beside the file to range-read a single group's bytes instead of parsing the
whole file.

`read_ags` accepts `encoding := 'windows-1252'` to decode a legacy file.

## Read

| Function                        | Returns                                          |
| ------------------------------- | ------------------------------------------------ |
| `read_ags(path, group)`         | one group, columns cast to their AGS4 types.     |
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

| Function             | Returns                                                                                               |
| -------------------- | ----------------------------------------------------------------------------------------------------- |
| `ags_groups(path)`   | the groups present in the file, with per-group row and heading counts.                                |
| `ags_headings(path)` | every group's headings — unit, `ags_type`, `sql_type`, `is_key` — with a `group` column to filter on. |

## Inspect the dictionary

The AGS4 dictionary ships _inside_ the extension — no download.

| Function              | Returns                                                                                                                                  |
| --------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| `ags_dictionary()`    | every group/heading with its unit and data type.                                                                                         |
| `ags_relationships()` | the group parent/child (KEY) graph.                                                                                                      |
| `ags_rules()`         | the AGS4 numbered-rule catalogue (`rule`, `title`, `severity`, `fixable`) — the extension _lists_ the rules; the CLI/library _run_ them. |

```sql
SELECT heading, unit, ags_type, sql_type FROM ags_dictionary() WHERE "group" = 'LOCA';
```

## Materialise a store

`load_ags` emits the `CREATE TABLE` DDL to persist every group as an indexed,
repeat-/remote-queryable store — turning a one-shot read into a durable database.

| Function         | Returns                                                            |
| ---------------- | ------------------------------------------------------------------ |
| `load_ags(path)` | `(seq, stmt)` — ordered `CREATE TABLE` DDL, one statement per row. |

```sql
SELECT stmt FROM load_ags('delivery.ags') ORDER BY seq;
```

!!! note "Read-only query surface"
    DuckDB is a **read** door — it reads and inspects, but doesn't validate,
    certify, `fix`, `diff`, or emit AGS4 (those are the library and CLI
    surfaces). See the [capability matrix](../surfaces/index.md#what-each-door-can-do).
