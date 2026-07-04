# Build AGS4 from frames

**Available in:** Python · Node · Browser

**What / when:** you have your data as per-group dataframes (one frame per group, columns named for the AGS headings) and want a **valid** AGS4 file back — without hand-writing the `TRAN` / `UNIT` / `TYPE` boilerplate.

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

See also: [Build from a typed graph](./build-from-typed-graph.md) · [Produce AGS4](../learn/produce.md)
