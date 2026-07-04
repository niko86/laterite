# Validate a delivery — any surface

One clean-room engine, one verdict. The edition selects itself from the file's
`TRAN_AGS` — you never pass it. Pick your stack (the choice **syncs across the
whole page**):

=== "Python"

    ```python
    import laterite

    rep = laterite.validate("delivery.ags")
    print(rep.is_valid, rep.count, rep.dict_version, rep.resolution)
    # True 0 4.1.1 exact
    ```

=== "Node"

    ```js
    import { validate } from "laterite";

    const rep = validate("delivery.ags", { warnings: true });
    console.log(rep.isValid, rep.count, rep.dictVersion, rep.resolution);
    // true 0 "4.1.1" "exact"
    ```

=== "DuckDB"

    ```sql
    INSTALL laterite_ags4 FROM community;
    LOAD laterite_ags4;

    SELECT rule, line, "group", desc
    FROM validate_ags('delivery.ags');
    ```

Read a group as typed data — same story, same synced choice:

=== "Python"

    ```python
    loca = laterite.read("delivery.ags").table("LOCA")   # polars, born-typed
    ```

=== "Node"

    ```js
    const loca = read("delivery.ags").table("LOCA");     // arrow-js Table
    ```

=== "DuckDB"

    ```sql
    SELECT loca_id, loca_gl FROM read_ags('delivery.ags', 'LOCA');
    ```

Because the tabs are **linked** (`content.tabs.link`), choosing "Node" on either
group switches *every* Python/Node/SQL group on the site to Node — the same
synced behaviour as a dedicated multi-stack site, right inside your existing
MkDocs.

> _Draft page — part of the #201 “surfaces” prototype to compare doc stacks._
