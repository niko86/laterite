# Produce AGS4

```python
--8<-- "python/ex09a_build_from_frames.py"
```

```text
groups: ['PROJ', 'LOCA', 'TRAN', 'UNIT', 'TYPE']
findings: 0
```

`build_ags4` takes a `{code: frame}` mapping — the columns are your AGS
headings — and constructs a **valid** file. You handed it two data groups
(`PROJ`, `LOCA`); it handed back five. In `mode="autofix"` (the default) it
synthesizes the mandatory metadata catalogs — `TRAN`, `UNIT`, `TYPE`, plus
`ABBR` for any `PA` pick-list codes — so a data-only build passes the rule
engine in one call. Zero findings, no boilerplate.

## From a typed PROJ graph

```python
--8<-- "python/ex09b_build_from_typed_graph.py"
```

```text
groups: ['PROJ', 'LOCA', 'TRAN', 'UNIT', 'TYPE']
findings: 0
```

The other door takes a typed graph: a `PROJ` with `LOCA` children attached via
`.locas.append(...)` or the `locas=[...]` constructor kwarg. `build_ags4` walks
it depth-first and — like the frames door — emits **only the headings you
set**. That's why a sparse graph builds clean: nothing is invented in your data
columns, only the metadata catalogs around them. The managed child collection
is append-only, so reassigning `p.locas` raises `AttributeError` rather than
silently dropping the rows you built up.

!!! tip
Both doors return the same `BuildResult`. Inspect `res.text` / `res.bytes`
in memory, check `res.findings` for any caveats autofix couldn't resolve, or
`res.save("out.ags")` to persist a byte-faithful AGS4 file to disk.

→ See the whole fluent API assembled in the
[Chaining Showcase](../chaining/index.md).
