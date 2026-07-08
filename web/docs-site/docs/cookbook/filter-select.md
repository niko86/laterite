# Filter & select one group

**Available in:** Python · Node · DuckDB · Browser

Narrow one group to the rows and columns you want. In Python it's a lazy query
builder; on every other surface it's the same idea said in SQL — and because the
dtype *is* the AGS type, a numeric filter compares numbers, not strings.

=== "Python"

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

=== "Node"

    ```js
    --8<-- "node/ex05_query.mjs"
    ```

    ```text
    [
      { LOCA_ID: 'BH02', LOCA_TYPE: 'RC', LOCA_GL: 32.49 },
      { LOCA_ID: 'BH03', LOCA_TYPE: 'RC', LOCA_GL: 28.54 },
      { LOCA_ID: 'BH04', LOCA_TYPE: 'RC', LOCA_GL: 29.04 },
      { LOCA_ID: 'BH05', LOCA_TYPE: 'RC', LOCA_GL: 31.62 },
      { LOCA_ID: 'BH07', LOCA_TYPE: 'RC', LOCA_GL: 31.33 },
      { LOCA_ID: 'BH08', LOCA_TYPE: 'CP', LOCA_GL: 28.67 },
      { LOCA_ID: 'BH09', LOCA_TYPE: 'CP', LOCA_GL: 30.98 }
    ]
    ```

    Node has one query door — `sql()` — not a lazy `.query()`/`.filter()`/`.select()`
    chain: the filter and the projection live in the statement. It's `async` and
    returns plain row objects; `LOCA_GL` arrives as a real JS number, so the
    `> 28` in the SQL is a numeric comparison. `sql()` is the one Node feature
    behind an **optional peer** — `npm i @duckdb/node-api` — so services that only
    read/validate/fix never pull a database in; `close()` releases the connection.
    For a real cross-group join it's the same `sql()` — see
    [SQL across groups](./sql-across-groups.md).

=== "DuckDB"

    ```sql
    --8<-- "duckdb/ex05_filter_select.sql"
    ```

    `read_ags(file, 'LOCA')` is a table function; the filter and the projection
    are plain SQL over it. Columns come back born-typed, so `loca_gl > 28`
    compares numbers exactly as the other surfaces do. To narrow across groups,
    add more `read_ags(...)` calls and join — see
    [SQL across groups](./sql-across-groups.md).

=== "Browser"

    Open the [web app](../surfaces/browser.md) and load your file into the
    **Explore** pane. Pick a group, then filter its rows and choose columns
    interactively — the grid is fed by the same born-typed columns, so a numeric
    filter sorts and compares as numbers. The file never leaves your machine,
    which keeps confidential ground-investigation data local.

Every door narrows the same born-typed group: Python's builder stays lazy and
composable, Node and DuckDB say it in SQL, the browser does it by point-and-click.
Reach for `.query()`/`sql()` on one group; step up to a join across groups with
[SQL across groups](./sql-across-groups.md).

See also: [SQL across groups](./sql-across-groups.md) ·
[Chaining](../chaining/index.md).
