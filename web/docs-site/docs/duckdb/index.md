# DuckDB

The `laterite_ags4` extension puts the **same engine inside DuckDB** as SQL
table functions — read and query AGS4 files _in place_, no import step, no
other language.

```sql
INSTALL laterite_ags4 FROM community;
LOAD laterite_ags4;
```

## Read-only — validation lives in the CLI & library

The extension is a **read-only reader**: it reads, joins and inspects, but it
doesn't validate or certify. Run the numbered rules and mint an `.ags.idx`
certificate with the [`lat` CLI](../reference/cli.md) (`lat validate` /
`lat certify`) or the [`laterite`](../surfaces/python.md) library — `read_ags`
then _consumes_ that `.ags.idx` beside the file for fast single-group reads.

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

| Function                                               | Returns                                                     |
| ------------------------------------------------------ | ----------------------------------------------------------- |
| `read_ags(path, group)` · `read_ags_text(text, group)` | one group, typed columns                                    |
| `ags_groups(path)`                                     | the groups present in the file                              |
| `ags_headings(path)`                                   | every group's headings, units, and types (a `group` column) |
| `ags_dictionary()`                                     | the bundled AGS4 dictionary                                 |
| `ags_relationships()`                                  | the group parent/child (KEY) graph                          |
| `ags_rules()`                                          | the AGS4 numbered-rule catalogue (severity, fixability)     |
| `load_ags(path)`                                       | CREATE TABLE DDL to materialise an indexed, keyed copy      |

## Inspect the dictionary

The dictionary ships _inside_ the extension — no download:

```sql
SELECT "group", heading, unit, ags_type, sql_type
FROM ags_dictionary()
WHERE "group" = 'LOCA';
```

## Recipes

A read-only reader plus DuckDB's own SQL is enough for multi-file, external-data
and spatial work. These are illustrative — swap in your own file names.

### Merge two deliveries, one row per location

Union two phases of a site. `_id` is content-addressed on a row's **identity** —
its AGS key plus its parent chain, _not_ its every value — so the same borehole
from either file shares an `_id`, and `DISTINCT ON (_id)` collapses it to one row
with no key columns to name:

```sql
SELECT DISTINCT ON (_id) *
FROM (SELECT * FROM read_ags('phase1.ags', 'LOCA')
      UNION ALL SELECT * FROM read_ags('phase2.ags', 'LOCA'));
```

Because `_id` keys on identity, a location that was _revised_ between phases
(same `LOCA_ID`, changed data) shares that `_id` too — so `DISTINCT ON` keeps an
**arbitrary** version. To keep a _specific_ one, dedup on the AGS key instead:
carry a version column and let `QUALIFY` pick the winner per key:

```sql
SELECT * FROM (
  SELECT *, 1 AS ver FROM read_ags('phase1.ags', 'LOCA')
  UNION BY NAME
  SELECT *, 2 AS ver FROM read_ags('phase2.ags', 'LOCA')
)
QUALIFY row_number() OVER (PARTITION BY loca_id ORDER BY ver DESC) = 1;
```

### Keep every distinct value with `_content_hash`

`_id` keys on identity, so the `DISTINCT ON (_id)` above collapses a _revised_
borehole (same `LOCA_ID`, changed data) to a single arbitrary row. To keep one
row per distinct **value** instead — folding away rows that are genuinely
identical, but keeping both the original and its revision — dedup on
`_content_hash`, the value twin of `_id`:

```sql
SELECT DISTINCT ON (_content_hash) *
FROM (SELECT * FROM read_ags('phase1.ags', 'LOCA')
      UNION ALL SELECT * FROM read_ags('phase2.ags', 'LOCA'))
ORDER BY _content_hash;
```

`_content_hash` fingerprints a row's values _through their declared type_, so
`10.0` and `10.00` under `2DP` share one fingerprint where a plain string compare
would not. Like `_id` and `_parent_id`, it is always present on every `read_ags`
row — drop all three with `SELECT * EXCLUDE (_id, _parent_id, _content_hash)` when
you want only the AGS columns.

### Join AGS4 to external data

`read_ags` is just another table, so it joins straight to a Parquet (or CSV,
JSON, database) table — here tagging each borehole with its planning zone:

```sql
SELECT l.loca_id, l.loca_gl, z.zone
FROM read_ags('site.ags', 'LOCA') l
JOIN 'planning_zones.parquet' z ON z.parcel = l.loca_id;
```

…and the reverse — export a typed group straight to Parquet for a warehouse:

```sql
COPY (SELECT * FROM read_ags('site.ags', 'LOCA')) TO 'loca.parquet';
```

### Spatial — boreholes near an alignment

With DuckDB's `spatial` extension the born-typed easting/northing become
geometry — find every borehole within 50 m of a route centre-line:

```sql
LOAD spatial;
SELECT loca_id
FROM read_ags('site.ags', 'LOCA')
WHERE ST_DWithin(
        ST_Point(loca_nate, loca_natn),
        ST_GeomFromText('LINESTRING(531000 181000, 531200 181150)'),
        50);
```

### Deepest sample and its plasticity, per borehole

The content keys make a three-group walk a plain join; `arg_max` reads a value
from the deepest sample's lab test in one pass:

```sql
SELECT l.loca_id,
       max(s.samp_top)                AS deepest,
       arg_max(t.llpl_pi, s.samp_top) AS pi_at_deepest
FROM read_ags('site.ags', 'LOCA') l
JOIN read_ags('site.ags', 'SAMP') s ON s._parent_id = l._id
JOIN read_ags('site.ags', 'LLPL') t ON t._parent_id = s._id
GROUP BY l.loca_id;
```

!!! note "One engine, every stack"
    `read_ags` is the identical born-typing engine behind
    [Python](../surfaces/python.md), [Node](../node/index.md), and the
    [browser app](../surfaces/browser.md) — the cross-surface compliance matrix
    proves every read surface agrees, so a column's type is the same wherever
    you read it.
