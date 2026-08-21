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

Two functions turn a one-shot read into a durable, indexed store. Both **emit
DDL rather than running it** — the extension API cannot cleanly issue
`CREATE TABLE` or `ATTACH` from inside a table function — so each returns an
ordered `(seq, stmt)` script for you to execute.

| Function                     | Returns                                                                        |
| ---------------------------- | ------------------------------------------------------------------------------ |
| `load_ags(path)`             | ordered DDL materialising every group into the **current** database.           |
| `to_duckdb(path, out_db)`    | ordered DDL persisting every group into a **standalone** `.duckdb` at `out_db`. |

```sql
SELECT stmt FROM load_ags('delivery.ags') ORDER BY seq;
```

The difference is where the tables land. `load_ags` creates them here;
`to_duckdb` wraps the same per-group DDL in `ATTACH '<out_db>' AS _lat_out;` …
`DETACH _lat_out;`, so you can persist to any file from any session:

```sql
SELECT stmt FROM to_duckdb('delivery.ags', 'delivery.duckdb') ORDER BY seq;
```

Stitch the statements into one script and execute that — the ordering matters,
so keep the `ORDER BY`:

```sql
SELECT string_agg(stmt, e'\n' ORDER BY seq)
FROM to_duckdb('delivery.ags', 'delivery.duckdb');
```

Feed the result back to whichever host you're in: `con.execute(script)` from a
driver, or `duckdb -c "$script"` from a shell.

Each group becomes an `ags_<group>` table indexed on `_id`, and on `_parent_id`
too where the group has a dictionary parent. Those keys are byte-identical to
`read_ags`'s, so the resulting file matches what the Python and Node
`to_duckdb()` produce — one store shape that every surface agrees on.

!!! note "Read-only query surface"
    DuckDB is a **read** door — it reads and inspects, but doesn't validate,
    certify, `fix`, `diff`, or emit AGS4 (those are the library and CLI
    surfaces). See the [capability matrix](../surfaces/index.md#what-each-door-can-do).
