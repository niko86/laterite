# Dependency shape (pyarrow-free by default)

`pip install laterite` pulls **polars + duckdb** only — no pandas, no pyarrow.
A base install stays lean.

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
from laterite import compat as AGS4

# object dtype (numpy) — today's python-ags4 baseline, the default
tables, _ = AGS4.AGS4_to_dataframe("delivery.ags")

# string dtype (pandas' Arrow-backed str) — needs [compat,pyarrow]
tables, _ = AGS4.AGS4_to_dataframe("delivery.ags", string_dtype="string")
AGS4.set_string_dtype("string")            # process-wide (or LATERITE_COMPAT_STRING_DTYPE)
```

!!! note "Which extra do I want?"
    Most callers want none. Reach for **`[compat]`** to use the `laterite.compat`
    python-ags4 shim (pandas-backed by default); it adds `pandas<3` and nothing
    else. Add **`[compat,pyarrow]`** for the faster pandas hop and the
    Arrow-backed `string` dtype. Reach for **`[pyarrow]`** when you explicitly want
    the Arrow backend — e.g. handing native `pyarrow.Table` objects to another
    Arrow-native library.

So a base user gets polars + duckdb and an Arrow-capable bridge without dragging
in two heavyweight dataframe stacks. Importing `laterite.compat` without the
extra is safe — the lazy materialiser only raises (with a
`pip install laterite[compat]` hint) when a pandas-backed call is actually made.

See also: [The python-ags4 drop-in](../cookbook/compat.md).
