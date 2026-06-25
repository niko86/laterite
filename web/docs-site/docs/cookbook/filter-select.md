# Filter & select one group

Narrow one group to the rows and columns you want — lazily, with nothing run until you ask for a frame.

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

`.query(sql)` returns a lazy `AgsQuery`. Chain `.filter(...)` to drop rows and
`.select(...)` to pick columns; **nothing runs** until a terminal pulls the
result. The four terminals materialise the same plan: `.frame()` (the handle's
default backend, polars), `.to_polars()`, `.to_pandas()`, and `.relation()` (the
raw `DuckDBPyRelation`, still lazy). Because the dtype IS the AGS type,
`LOCA_GL > 28` compares numbers, not strings.

Each builder call returns a **new immutable `AgsQuery`** — `.filter(...)` and
`.select(...)` never mutate in place, so you can branch off a shared base query
without one fork bleeding into another:

```python
base = ags.query("SELECT * FROM LOCA")
rotary = base.filter("LOCA_TYPE = 'RC'")     # base is untouched
deep   = base.filter("LOCA_GL > 30")         # independent of `rotary`
```

Use `.query()` for guard-railed single-group work. When you need a real join
across groups, reach for `.sql(...)` instead — see
[SQL across groups](./sql-across-groups.md).

See also: [SQL across groups](./sql-across-groups.md) ·
[Chaining](../chaining/index.md).
