# Dependency shape (pyarrow-free by default)

`pip install laterite` pulls **polars + duckdb** only — no pandas, no pyarrow.
A base install stays lean.

<!-- doc-code: skip — installs packages — a gate that ran it would rewrite its own environment -->
```bash
pip install laterite                  # polars + duckdb only
pip install laterite[compat]          # + pandas<3 (the python-ags4 drop-in) — pyarrow-free
pip install laterite[compat,pyarrow]  # + pyarrow accelerator (faster pandas hop + string dtype)
pip install laterite[pyarrow]         # pyarrow for the explicit Arrow backend
pip install laterite[all]             # pandas + pyarrow
```

DuckDB is the pyarrow-free dataframe bridge. The `compat` drop-in's
`AGS4_to_dataframe` reads a **Rust-built all-Utf8 Arrow table** (no per-cell
Python boxing) and hands pandas an **object-dtype** frame through DuckDB's NumPy
`.df()` materialiser — the same trick the core uses — so pandas works _without_
pyarrow, and already runs **~3× faster than python-ags4**. The polars path
ingests the same Arrow via the C-stream capsule, also pyarrow-free.

## pyarrow is an optional accelerator

Add pyarrow (`[compat,pyarrow]` or `[all]`) and `compat` auto-detects it: the
pandas hop swaps to pyarrow's `to_pandas` (a touch faster), and you unlock the
`string_dtype="string"` output — pandas' Arrow-backed `str` dtype, which is what
python-ags4 itself returns once it runs on pandas 3. Without pyarrow, the
default `object` dtype path is used; `string_dtype="string"` raises an
actionable error rather than downgrading.

```python
--8<-- "python/ex22_string_dtype.py:code"
```

```text
--8<-- "python/ex22_string_dtype.out"
```

## `.sql()` hands you DuckDB, and DuckDB has its own dependencies

Everything above is about **laterite's** materialisers — `.frame()`,
`.to_polars()`, `.to_pandas()`, `compat`'s frames — and all of them are
pyarrow-free as described. [`.sql(...)`](../cookbook/sql-across-groups.md) is the
escape hatch, and it stops returning laterite objects: what comes back is a
**`DuckDBPyRelation`**, so the call that materialises it is DuckDB's, under
DuckDB's rules and not ours.

| Terminal on the relation | Needs                          |
| ------------------------ | ------------------------------ |
| `.df()` → pandas         | `[compat]` (pandas, via NumPy) |
| `.pl()` → polars         | `[pyarrow]`                    |
| `.arrow()` → Arrow table | `[pyarrow]`                    |

`.pl()` is the one that surprises people: polars is in the **base** install, so
the line looks like it cannot need an extra — but DuckDB routes it through Arrow
and imports pyarrow to do it. A base install therefore gets
`ModuleNotFoundError: No module named 'pyarrow'` from a script that never
mentions pyarrow. That is why the `.sql()` examples on this site pin
`laterite[pyarrow]` in their `uv run` headers.

Staying on laterite's own terminals avoids the question entirely:
`.query(...).to_polars()` needs nothing beyond the base install.

!!! note "Which extra do I want?"
    Most callers want none. Reach for **`[compat]`** to use the `laterite.compat`
    python-ags4 shim (pandas-backed by default); it adds `pandas<3` and nothing
    else. Add **`[compat,pyarrow]`** for the faster pandas hop and the
    Arrow-backed `string` dtype. Reach for **`[pyarrow]`** when you explicitly want
    the Arrow backend — e.g. handing native `pyarrow.Table` objects to another
    Arrow-native library, or calling `.pl()` / `.arrow()` on a `.sql()` relation.

So a base user gets polars + duckdb and an Arrow-capable bridge without dragging
in two heavyweight dataframe stacks. Importing `laterite.compat` without the
extra is safe — the lazy materialiser only raises (with a
`pip install laterite[compat]` hint) when a pandas-backed call is actually made.

See also: [The python-ags4 drop-in](../cookbook/compat.md).
