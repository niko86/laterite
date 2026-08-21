# SQL across groups

**Available in:** Python · Node · DuckDB · [Browser](../surfaces/browser.md)

**When:** you need a real join across two or more groups — counts, lookups,
aggregates — that a single-group query can't express. Drop to SQL.

=== "Python"

    ```python
    --8<-- "python/ex06_sql_join.py:code"
    ```

    ```text
    --8<-- "python/ex06_sql_join.out"
    ```

    `.sql(...)` exposes every group in the file as a DuckDB table named by its
    four-letter code, so `SAMP s JOIN LOCA l USING (LOCA_ID)` joins on the
    shared key with no setup — the columns are already born-typed, so
    `count(*)`, `GROUP BY`, and numeric comparisons mean what they say.

    `.sql()` is a **terminal** — it returns a `DuckDBPyRelation`, not an
    `AgsQuery`. Finish it by materialising into the frame library you want:

    - `.pl()` → polars (shown above)
    - `.df()` → pandas
    - `.arrow()` → an Arrow table

    One variation — fold a third group in with another `JOIN` and pull real
    columns instead of a count:

    ```python
    ags = laterite.read("delivery.ags")
    rel = ags.sql(
        "SELECT l.LOCA_ID, s.SAMP_REF, g.GEOL_GEOL "
        "FROM SAMP s JOIN LOCA l USING (LOCA_ID) "
        "JOIN GEOL g ON g.LOCA_ID = s.LOCA_ID "
        "AND s.SAMP_TOP >= g.GEOL_TOP AND s.SAMP_TOP < g.GEOL_BASE"
    )
    df = rel.df()
    ```

=== "Node"

    ```js
    --8<-- "node/ex06_sql_join.mjs"
    ```

    ```text
    --8<-- "node/ex06_sql_join.out"
    ```

    Identical SQL, identical table names. `sql()` is `async` and returns plain
    row objects (note DuckDB's `BIGINT` arrives as JS `BigInt` — the `4n`);
    pass `{ arrow: true }` for an arrow-js `Table` instead. The SQL door is
    the one Node feature behind an **optional peer** — `npm i @duckdb/node-api`
    — so services that only read/validate/fix never pull a database in.
    `close()` (or `using`) releases the connection.

=== "DuckDB"

    ```sql
    --8<-- "duckdb/ex06_sql_join.sql"
    ```

    In DuckDB itself there's nothing to leave — each group is a `read_ags()`
    table function and the join is plain SQL. The same born-typed columns
    apply, and `load_ags(path)` emits the DDL to materialise every group as an
    `ags_<code>` table when you'd rather query a warehouse than a file.

### Join without knowing the keys

Every group also carries two synthetic **content-addressed** columns in the
engine — `_id` and `_parent_id` — so a parent/child join is the _same_ column
pair for every edge, with no `USING (…)` to look up per group:

```python
ags = laterite.read("delivery.ags")
rel = ags.sql("SELECT * FROM SAMP s JOIN LOCA l ON s._parent_id = l._id")
```

These live only in the relational layer; `ags["SAMP"]` frames drop them unless
you ask with `ags.table("SAMP", keys=True)` (Node:
`file.table("SAMP", { keys: true })`). See
[content-addressed keys](../concepts/content-addressed-keys.md) for the full
model.

**Gotcha:** `.sql()` is the escape hatch, not the guard-railed builder. Use
[`.query()`](./filter-select.md) for narrowing and selecting within one group —
it keeps you in the typed `AgsQuery` chain and stays lazy until a terminal.
Reach for `.sql()` only when you need a cross-group join or aggregate. Both
share the same DuckDB engine, so the type fidelity is identical.

See also: [Filter & select](./filter-select.md) ·
[Borehole record set](./borehole-record-set.md)
