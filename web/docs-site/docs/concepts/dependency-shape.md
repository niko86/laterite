# Dependency shape (pyarrow-free)

`pip install laterite` pulls **polars + duckdb** only — no pandas, no pyarrow.
A base install stays lean.

```bash
pip install laterite            # polars + duckdb only
pip install laterite[compat]    # + pandas<3 (the python-ags4 drop-in) — still pyarrow-free
pip install laterite[pyarrow]   # the only extra that pulls pyarrow
```

DuckDB is the pyarrow-free dataframe bridge. When you ask for a pandas frame
(`.to_pandas()`, `.df()`) the path goes polars → pandas through DuckDB's NumPy
materialiser — the same trick the core uses — so pandas works *without* pyarrow.
The polars path uses the Arrow C-stream capsule, also pyarrow-free.

!!! note "Which extra do I want?"
    Most callers want none. Reach for **`[compat]`** only to use the
    `laterite.compat` python-ags4 shim (pandas-backed by default); it adds
    `pandas<3` and nothing else. Reach for **`[pyarrow]`** only when you
    explicitly want the Arrow backend — e.g. handing native `pyarrow.Table`
    objects to another Arrow-native library.

So a base user gets polars + duckdb and an Arrow-capable bridge without dragging
in two heavyweight dataframe stacks. Importing `laterite.compat` without the
extra is safe — the lazy materialiser only raises (with a
`pip install laterite[compat]` hint) when a pandas-backed call is actually made.

See also: [The python-ags4 drop-in](../cookbook/compat.md).
