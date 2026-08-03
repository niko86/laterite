# Build AGS4 from a typed graph

**Available in:** Python · Node · [Browser](../surfaces/browser.md)

Construct a `PROJ` typed-class tree, attach its children, and hand the whole
graph to the emitter — use this when your data is already a graph in memory
(objects, not tables) rather than `{code: frame}` mappings.

=== "Python"

    ```python
    --8<-- "python/ex09b_build_from_typed_graph.py"
    ```

    ```text
    --8<-- "python/ex09b_build_from_typed_graph.out"
    ```

    `build_ags4(PROJ(...))` walks the graph depth-first (#214) and emits exactly
    the groups you built. You handed it a `PROJ` with one `LOCA` child; you got
    those two back, plus three findings naming the mandatory metadata catalogs your
    graph doesn't carry — Rules 14 (`TRAN`), 15 (`UNIT`) and 17 (`TYPE`). Pass
    `synthesise_metadata=True` to derive `UNIT` and `TYPE` (`ABBR` too, when `PA`
    pick-list codes are used). `TRAN` is not derivable — state it with the five
    `tran_*` arguments — so a sparse graph builds clean in one call once you supply
    the transmission it represents.

    Children attach two ways, shown above: `.locas.append(LOCA(...))` after the
    fact, or the `locas=[...]` constructor kwarg up front. Either way the typed-graph
    door emits **only the headings you set** — nothing is invented in your data
    columns — which is why a graph carrying just `loca_id` and `loca_gl` is enough.

    The managed child collection is append-only: reassigning `p.locas = [...]`
    raises `AttributeError` rather than silently dropping the rows you built up.
    Mutate it through `.append` (and the list's own methods), never by rebinding.

=== "Node"

    ```js
    --8<-- "node/ex09b_build_from_typed_graph.mjs"
    ```

    ```text
    --8<-- "node/ex09b_build_from_typed_graph.out"
    ```

    The same typed classes are named exports — `import { PROJ, LOCA } from
    "laterite"` — with **uppercase** heading fields (`PROJ_ID`, `LOCA_GL`) and a
    child array per parent (`p.locas`). Build the tree with the constructor's
    `locas: [...]` field or `push` onto it, then hand the root to `buildAgs4`; it
    walks the graph depth-first and returns the same `BuildResult` the frames door
    does, with `{ synthesiseMetadata: true }` deriving the catalogs the same way.
    (Node's `locas` is a plain array — the append-only guard is a Python-only
    nicety.)

!!! tip
    `build_ags4` / `buildAgs4` returns a `BuildResult` whichever door you use.
    Inspect `res.text` / `res.bytes` in memory, check `res.findings` for any
    caveats autofix couldn't resolve, or `res.save("out.ags")` to persist a
    byte-faithful AGS4 file.

See also: [Build from frames](./build-from-frames.md) ·
[Produce AGS4](../learn/produce.md).
