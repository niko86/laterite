# Pull one borehole record set

**Available in:** Python · Node · DuckDB · [Browser](../surfaces/browser.md)

Fan out from a `LOCA` location to everything that hangs off it (samples, tests,
the lot) as typed frames keyed by group code. Reach for this when you want a
single borehole's whole story, not one group at a time.

=== "Python"

    First, see *which* groups fan out for the boreholes you picked.
    `.at(code, ids)` returns a query whose `.groups` lists the related group set:

    ```python
    --8<-- "python/ex03_at_fanout_groups.py:code"
    ```

    ```text
    --8<-- "python/ex03_at_fanout_groups.out"
    ```

    `.at("LOCA", ["BH01", "BH02"])` filters on the `LOCA_ID` key column, and
    `q.groups` lists every group in the file that carries it: `LOCA` itself,
    `SAMP` (samples), `LLPL` (Atterberg limits). That way you know what's coming
    before you materialise anything.

    If no group in the file carries the key column, `.at()` raises `ValueError`
    instead of handing back every row. `GEOL` has no `GEOL_ID`, so
    `.at("GEOL", ...)` raises. Where the ID heading is optional or blank (as
    `SAMP_ID` often is), filter on row identity instead.
    `ags.at("SAMP", ids, by="_id")` keeps the `SAMP` rows whose `_id` is in
    `ids`, plus the rows of its direct children (`LLPL`, …) whose `_parent_id`
    is. Read the ids off `ags.table("SAMP", keys=True)`. See
    [content-addressed keys](../concepts/content-addressed-keys.md). Once a
    query filters by `_id`, asking it for a group no filter reaches (`LOCA`,
    or a grandchild) raises rather than returning every row. Passing `None` to
    `.at("SAMP", ...)` warns, because a blank `SAMP_ID` matches no row.

    Then call `.frames()` to materialise the record set as `{group_code: frame}`:

    ```python
    --8<-- "python/ex04_at_frames.py:code"
    ```

    ```text
    --8<-- "python/ex04_at_frames.out"
    ```

    `frames` is a plain dict of **born-typed** polars frames. Pull one out by its
    4-letter code (`frames["SAMP"]`, not `q["SAMP"]`). Each frame is already typed
    straight from the AGS TYPE row, and each is row-filtered to just the boreholes
    you asked for, so `frames["SAMP"]` here is the four samples taken on `BH01`.

=== "Node"

    ```js
    --8<-- "node/ex04_at_frames.mjs"
    ```

    ```text
    --8<-- "node/ex04_at_frames.out"
    ```

    Same fan-out, same manifest: `at("LOCA", ids)` returns a subset whose
    `.groups` lists the related codes, and `frames()` materialises them as
    `{ group: rows }`: plain JS row objects (or arrow-js `Table`s with
    `{ arrow: true }`), each filtered to the boreholes you asked for. It's `async`
    and rides the same **optional peer** as `sql()` (`npm i @duckdb/node-api`);
    `close()` releases the connection when you're done.

    The key rules match Python. `at()` throws when no group in the file carries
    the key column, and a `null` value warns. `at("SAMP", ids, { by: "_id" })`
    keeps the `SAMP` rows whose `_id` is in `ids`, plus the rows of its direct
    children whose `_parent_id` is. Then `table(code)` rejects for a group no
    filter reaches.

=== "DuckDB"

    ```sql
    --8<-- "duckdb/ex04_borehole.sql"
    ```

    DuckDB has no fan-out helper, so you name the related groups and join them
    on the shared KEY yourself. Each group is a `read_ags()` table function and the
    columns are born-typed, so `samp_top` sorts numerically. To pull every
    related group at once, add a `read_ags(...)` and `JOIN` per code, or reach
    for `load_ags(path)` to emit the DDL that materialises them all as
    `ags_<code>` tables first.

When to use it: building a per-location report, exporting one hole's data, or
feeding a downstream model that wants the whole record set at once. Gotcha:
`.at(...)`/`frames()` is a **fan-out, not a join**. Each group stays a separate
frame keyed on its own KEY heading. If you want the groups _joined_ into one wide
result, run SQL across them instead (the DuckDB tab shows the join form).

See also: [SQL across groups](./sql-across-groups.md) ·
[Born-typed](../concepts/born-typed.md)
