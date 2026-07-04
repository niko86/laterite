# Build AGS4 from a typed graph

**Available in:** Python · Node · Browser

Construct a `PROJ` typed-class tree, attach its children, and hand the whole
graph to `build_ags4` — use this when your data is already a graph in memory
(objects, not tables) rather than `{code: frame}` mappings.

```python
--8<-- "python/ex09b_build_from_typed_graph.py"
```

```text
groups: ['PROJ', 'LOCA', 'TRAN', 'UNIT', 'TYPE']
findings: 0
```

`build_ags4(PROJ(...))` walks the graph depth-first (#214) and emits a
**valid** file. You handed it a `PROJ` with one `LOCA` child; it handed back
five groups — autofix synthesizes the mandatory metadata catalogs (`TRAN`,
`UNIT`, `TYPE`, plus `ABBR` for any `PA` pick-list codes) around your data, so a
sparse graph builds clean in one call.

Children attach two ways, shown above: `.locas.append(LOCA(...))` after the
fact, or the `locas=[...]` constructor kwarg up front. Either way the typed-graph
door emits **only the headings you set** — nothing is invented in your data
columns — which is why a graph carrying just `loca_id` and `loca_gl` is enough.

The managed child collection is append-only: reassigning `p.locas = [...]`
raises `AttributeError` rather than silently dropping the rows you built up.
Mutate it through `.append` (and the list's own methods), never by rebinding.

!!! tip
    `build_ags4` returns a `BuildResult` whichever door you use. Inspect
    `res.text` / `res.bytes` in memory, check `res.findings` for any caveats
    autofix couldn't resolve, or `res.save("out.ags")` to persist a
    byte-faithful AGS4 file.

See also: [Build from frames](./build-from-frames.md) ·
[Produce AGS4](../learn/produce.md).
