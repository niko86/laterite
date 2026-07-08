# Build AGS4 from frames

**Available in:** Python · Node · Browser

**What / when:** you have your data as per-group tables (one per group, columns named
for the AGS headings) and want a **valid** AGS4 file back — without hand-writing the
`TRAN` / `UNIT` / `TYPE` boilerplate.

=== "Python"

    ```python
    --8<-- "python/ex09a_build_from_frames.py"
    ```

    ```text
    groups: ['PROJ', 'LOCA', 'TRAN', 'UNIT', 'TYPE']
    findings: 0
    ```

    `build_ags4` takes a `{code: frame}` mapping — each frame's columns *are* the AGS
    headings for that group — and returns a `BuildResult`. You handed it two data
    groups (`PROJ`, `LOCA`); it handed back five. In `mode="autofix"` (the default)
    it synthesizes the mandatory metadata catalogs — `TRAN`, `UNIT`, `TYPE`, plus
    `ABBR` for any `PA` pick-list codes — so a data-only build passes the rule
    engine in one call. The result carries the file three ways:

    - `res.text` — the AGS4 as a `str` (in memory)
    - `res.bytes` — the byte-faithful encoding (what `read(data=…)` consumes above)
    - `res.save("out.ags")` — persist to disk

    **Variation:** pass `mode="strict"` to skip the metadata synthesis — then you
    own the `TRAN`/`UNIT`/`TYPE` groups and findings will flag anything missing.
    **Gotcha:** the emitter writes **only the headings (columns) you supply** — it
    never invents data columns, only the catalog groups around them — so a sparse
    frame builds clean rather than padding out the full dictionary.

=== "Node"

    ```js
    --8<-- "node/ex09a_build_from_frames.mjs"
    ```

    ```text
    groups: [ 'PROJ', 'LOCA', 'TRAN', 'UNIT', 'TYPE' ]
    findings: 0
    ```

    `buildAgs4` takes a `Map` (or array) of `[code, rows]` entries — each row a
    plain object whose **keys are the AGS headings** — or an arrow-js `Table` per
    group. Group order is preserved, so put `PROJ` first. It returns the same
    `BuildResult` as Python: `res.bytes` / `res.text` carry the document and
    `res.findings` is the residual the mode couldn't clear. The default
    `{ mode: "autofix" }` synthesizes `TRAN`/`UNIT`/`TYPE` (and `ABBR` for `PA`
    codes); pass `{ mode: "strict" }` to own that metadata yourself. No DuckDB
    peer needed — emit is pure.

=== "Browser"

    Open the [web app](../surfaces/browser.md)'s **Export** pane: assemble or
    paste your per-group data and export a valid AGS4 file. The same emitter runs
    compiled to WebAssembly and synthesizes the `TRAN`/`UNIT`/`TYPE` metadata
    client-side, so the file you download passes the rule engine — and nothing is
    uploaded to build it.

Every door runs the same emitter over the same dictionary: hand it data groups,
get back a valid file with the metadata catalogs filled in. Reach for
`mode="strict"` on any surface when you want to own the `TRAN`/`UNIT`/`TYPE`
groups yourself.

See also: [Build from a typed graph](./build-from-typed-graph.md) · [Produce AGS4](../learn/produce.md)
