# Query across groups

A single AGS file is a graph: a `LOCA` borehole owns its `SAMP` samples, which
own their test results. `laterite` gives you four ways to walk that graph —
from a one-line fan-out to raw SQL.

## Fan out to a borehole's group set

```python
--8<-- "python/ex03_at_fanout_groups.py"
```

```text
['LOCA', 'SAMP', 'LLPL']
```

`ags.at("LOCA", ["BH01", "BH02"])` follows the dictionary's parent/child links
and returns a query scoped to just those boreholes. `.groups` tells you which
group codes came along for the ride — here the locations plus the samples and
plasticity tests that hang off them.

## Materialise the record set as frames

```python
--8<-- "python/ex04_at_frames.py"
```

```text
['LLPL', 'LOCA', 'SAMP']
4
```

`.frames()` turns that scoped query into a plain `dict` of born-typed polars
frames, keyed by group code. Pull one out with `frames["SAMP"]` (the dict, not
the query, is what you subscript) and you have BH01's four samples ready to work
with — already typed, no casting.

## Build a query lazily

```python
--8<-- "python/ex05_query_builder.py"
```

```text
shape: (7, 3)
┌─────────┬───────────┬─────────┐
│ LOCA_ID ┆ LOCA_TYPE ┆ LOCA_GL │
│ ---     ┆ ---       ┆ ---     │
│ str     ┆ str       ┆ f64     │
╞═════════╪═══════════╪═════════╡
│ BH02    ┆ RC        ┆ 32.49   │
│ BH03    ┆ RC        ┆ 28.54   │
│ BH04    ┆ RC        ┆ 29.04   │
│ BH05    ┆ RC        ┆ 31.62   │
│ BH07    ┆ RC        ┆ 31.33   │
│ BH08    ┆ CP        ┆ 28.67   │
│ BH09    ┆ CP        ┆ 30.98   │
└─────────┴───────────┴─────────┘
```

`.query(sql)` returns a lazy `AgsQuery`. Chain `.filter(...)` and `.select(...)`
to refine it — each call hands back a new `AgsQuery`, and **nothing runs** until
a terminal is called. There are four: `.frame()` (the handle's default backend),
`.to_polars()`, `.to_pandas()`, and `.relation()` (the raw `DuckDBPyRelation`,
still lazy). Because the type IS the AGS type, `LOCA_GL > 28` compares numbers,
not strings.

## Drop to SQL for a cross-group join

```python
--8<-- "python/ex06_sql_join.py"
```

```text
shape: (14, 2)
┌─────────┬─────┐
│ LOCA_ID ┆ n   │
│ ---     ┆ --- │
│ str     ┆ i64 │
╞═════════╪═════╡
│ BH01    ┆ 4   │
│ BH02    ┆ 2   │
│ BH03    ┆ 3   │
│ BH04    ┆ 4   │
│ BH05    ┆ 2   │
│ …       ┆ …   │
│ BH10    ┆ 3   │
│ BH11    ┆ 3   │
│ BH12    ┆ 3   │
│ BH13    ┆ 4   │
│ BH14    ┆ 4   │
└─────────┴─────┘
```

When you need a real join, `.sql(...)` exposes every group as a DuckDB table by
its code — so `SAMP s JOIN LOCA l USING (LOCA_ID)` just works. It returns a
`DuckDBPyRelation` (a terminal); call `.pl()`, `.df()`, or `.arrow()` to land it
as polars, pandas, or Arrow.

!!! tip
    `.query()` is the guard-railed builder for single-group work; `.sql()` is
    the escape hatch for anything DuckDB can express. Both share the same engine.

Next → [Produce AGS4](./produce.md)
