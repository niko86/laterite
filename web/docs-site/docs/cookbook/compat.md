# Drop-in for python-ags4

**Available in:** Python (by design — the whole point is the pandas-shaped
python-ags4 surface; see the [capability matrix](../surfaces/index.md#what-each-door-can-do))

Already have code built on `python-ags4`? Swap the import for `laterite.compat`
and the existing `AGS4_to_dataframe` call works unchanged — same `(tables,
headings)` 2-tuple, same pandas frames.

!!! tip "Deciding rather than doing?"

    This page is task-shaped. If the question is whether to move at all — what is
    mirrored, which upstream version you get, and what your CI will do afterwards
    — start at
    [Coming from python-ags4](../reference/coming-from-python-ags4.md).

```python
--8<-- "python/ex11_compat.py:code"
```

```text
--8<-- "python/ex11_compat.out"
```

`from laterite import compat as AGS4` aliases the shim to the name python-ags4
code already uses, so `AGS4.AGS4_to_dataframe(path)` returns the faithful
2-tuple: `result[0]` maps each group code to a pandas `DataFrame` (including the
metadata groups `TRAN`/`UNIT`/`TYPE`/`ABBR`), and `result[1]` carries the
heading metadata. The import-swap _is_ the migration — no call-site edits.

The shim is faithful by design: it mirrors python-ags4's verdicts and is gated
against that library's own test suite. The pandas backend is the default, so
downstream `df.shape`, indexing, and `.to_numeric` code keeps working.

**It's also faster.** `AGS4_to_dataframe` reads a Rust-built Arrow table (no
per-cell Python boxing) and materialises pandas through DuckDB — **~3× faster
than python-ags4** on the pyarrow-free `[compat]` install, more with the pyarrow
accelerator. The frames are **object dtype** by default, byte-identical to
python-ags4 today. Want pandas' Arrow-backed `str` dtype (what python-ags4
returns on pandas 3)? Install `[compat,pyarrow]` and pass `string_dtype="string"`
(or `set_string_dtype("string")` / `LATERITE_COMPAT_STRING_DTYPE=string`). See
[Dependency shape](../concepts/dependency-shape.md).

**Gotcha** — the pandas frames need the `[compat]` extra: `pip install
laterite[compat]`. Importing `laterite.compat` without it is safe, but a
pandas-backed call raises `ModuleNotFoundError` with install guidance. The base
`pip install laterite` ships polars + duckdb only.

When you're ready to leave the shim behind, [`laterite.read()`](../learn/read.md)
gives you born-typed frames directly — no `.cast()`, no `pd.to_numeric`.

See also: [Install & import](../learn/install.md) · [Home](../index.md)
