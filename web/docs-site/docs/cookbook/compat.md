# Drop-in for python-ags4

**Available in:** Python (by design — the whole point is the pandas-shaped
python-ags4 surface; see the [capability matrix](../surfaces/index.md#what-each-door-can-do))

Already have code built on `python-ags4`? Swap the import for `laterite.compat`
and the existing `AGS4_to_dataframe` call works unchanged — same `(tables,
headings)` 2-tuple, same pandas frames.

```python
--8<-- "python/ex11_compat.py"
```

```text
<class 'tuple'> ['PROJ', 'TRAN', 'UNIT', 'TYPE', 'ABBR']
(16, 7)
```

`from laterite import compat as AGS4` aliases the shim to the name python-ags4
code already uses, so `AGS4.AGS4_to_dataframe(path)` returns the faithful
2-tuple: `result[0]` maps each group code to a pandas `DataFrame` (including the
metadata groups `TRAN`/`UNIT`/`TYPE`/`ABBR`), and `result[1]` carries the
heading metadata. The import-swap *is* the migration — no call-site edits.

The shim is faithful by design: it mirrors python-ags4's verdicts and is gated
against that library's own test suite. The pandas backend is the default, so
downstream `df.shape`, indexing, and `.to_numeric` code keeps working.

**Gotcha** — the pandas frames need the `[compat]` extra: `pip install
laterite[compat]`. Importing `laterite.compat` without it is safe, but a
pandas-backed call raises `ModuleNotFoundError` with install guidance. The base
`pip install laterite` ships polars + duckdb only.

When you're ready to leave the shim behind, [`laterite.read()`](../learn/read.md)
gives you born-typed frames directly — no `.cast()`, no `pd.to_numeric`.

See also: [Install & import](../learn/install.md) · [Home](../index.md)
