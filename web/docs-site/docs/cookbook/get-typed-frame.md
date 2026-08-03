# Get one group as a typed frame

**Available in:** Python · Node · DuckDB · [Browser](../surfaces/browser.md)

Pull a single AGS group straight into a dataframe whose dtypes already match
the group's `TYPE` row — no casting at the call site.

=== "Python"

    ```python
    --8<-- "python/ex01_read_typed.py"
    ```

    ```text
    --8<-- "python/ex01_read_typed.out"
    ```

    `ags["LOCA"]` is a born-typed polars `DataFrame`: the dtype *is* the `TYPE`
    row. The `2DP` columns (`LOCA_NATE`, `LOCA_GL`) arrive as `Float64`; the
    `ID` column (`LOCA_ID`) stays `String`. You can sort, add, and join
    immediately — no `.cast()`, no `pd.to_numeric`.

    `ags.table("LOCA")` is the same call by another name — use whichever reads
    better at the call site.

    **The three input doors** — `read` takes a path, or read from memory:

    ```python
    laterite.read("delivery.ags")          # a path
    laterite.read(text=ags_string)         # text already in hand
    laterite.read(data=ags_bytes)          # raw bytes (e.g. an upload)
    ```

    A missing group raises `KeyError`; check `"LOCA" in ags` first if the group
    may be absent.

=== "Node"

    ```js
    --8<-- "node/ex01_read_typed.mjs"
    ```

    ```text
    --8<-- "node/ex01_read_typed.out"
    ```

    `file.table("LOCA")` is a born-typed **arrow-js** `Table` — the Node
    counterpart of the polars frame, from the same Arrow columns the engine
    builds for every surface. `2DP` columns arrive as `Float64` (real JS
    numbers from `.get()`), `ID` stays `Utf8`. The three input doors mirror
    Python: `read(path)`, `read(bytes)`, or `read(undefined, { text })`.

=== "DuckDB"

    ```sql
    --8<-- "duckdb/ex01_read_typed.sql"
    ```

    `read_ags(path, group)` exposes the group as a table function with the
    same born-typed columns — `2DP` is `DOUBLE`, `ID` is `VARCHAR` — so
    numeric predicates (`WHERE loca_gl < 0`) mean what they say with no
    `CAST`. Feed it straight into `CREATE TABLE … AS` to materialise, or join
    it against other groups (see [SQL across groups](./sql-across-groups.md)).

See also: [Read](../learn/read.md) · [Born-typed reads](../concepts/born-typed.md)
