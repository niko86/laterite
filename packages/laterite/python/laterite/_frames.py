"""Frame construction + backend resolution.

The read path's native layer hands back typed Apache Arrow per group (the
pyo3-arrow ``PyTable``); ``frame_from_arrow`` ingests it into polars
zero-copy via the Arrow PyCapsule interface. ``compat`` works from
primitives and ``materialise``s to a configurable backend (pandas by
default — a true python-ags4 drop-in returns pandas). Polars is the
always-available substrate (mandatory dep); the core surface returns polars
/ pandas directly (no narwhals).
"""

from __future__ import annotations

import os
from typing import TYPE_CHECKING, Any

import polars as pl

if TYPE_CHECKING:
    from collections.abc import Callable

# Module-level compat backend default. `compat.set_backend(...)`
# mutates this; env LATERITE_COMPAT_BACKEND overrides the built-in
# default ("pandas") at import time. Per-call `backend=` beats both.
_DEFAULT_BACKEND = os.environ.get("LATERITE_COMPAT_BACKEND", "pandas").lower()

_VALID_BACKENDS = ("pandas", "polars", "pyarrow")


def get_default_backend() -> str:
    return _DEFAULT_BACKEND


def set_default_backend(name: str) -> None:
    global _DEFAULT_BACKEND
    name = name.lower()
    if name not in _VALID_BACKENDS:
        raise ValueError(
            f"unknown backend {name!r} (expected one of {_VALID_BACKENDS})"
        )
    _DEFAULT_BACKEND = name


def resolve_backend(explicit: str | None) -> str:
    name = (explicit or _DEFAULT_BACKEND).lower()
    if name not in _VALID_BACKENDS:
        raise ValueError(
            f"unknown backend {name!r} (expected one of {_VALID_BACKENDS})"
        )
    return name


# Module-level compat output string dtype (pandas backend only — polars/pyarrow
# have a single string type). Mirrors the backend knob exactly:
# `compat.set_string_dtype(...)` mutates this, env LATERITE_COMPAT_STRING_DTYPE
# overrides the built-in default at import, per-call `string_dtype=` beats both.
#   "object" — numpy object (today's python-ags4 baseline; the true drop-in).
#   "string" — pandas' Arrow-backed str dtype (na_value=NaN), which is what
#              python-ags4 itself returns once it runs on pandas 3. The default
#              flips to "string" in that era — a one-word change here.
_DEFAULT_STRING_DTYPE = os.environ.get("LATERITE_COMPAT_STRING_DTYPE", "object").lower()

_VALID_STRING_DTYPES = ("object", "string")


def get_default_string_dtype() -> str:
    return _DEFAULT_STRING_DTYPE


def set_default_string_dtype(name: str) -> None:
    global _DEFAULT_STRING_DTYPE
    name = name.lower()
    if name not in _VALID_STRING_DTYPES:
        raise ValueError(
            f"unknown string_dtype {name!r} (expected one of {_VALID_STRING_DTYPES})"
        )
    _DEFAULT_STRING_DTYPE = name


def resolve_string_dtype(explicit: str | None) -> str:
    name = (explicit or _DEFAULT_STRING_DTYPE).lower()
    if name not in _VALID_STRING_DTYPES:
        raise ValueError(
            f"unknown string_dtype {name!r} (expected one of {_VALID_STRING_DTYPES})"
        )
    return name


def frame_from_arrow(table: Any) -> pl.DataFrame:
    """Ingest a Rust-built Arrow table (the pyo3-arrow ``PyTable`` ``read()``
    hands back per group) into a polars frame — pyarrow-free and zero-copy via
    the Arrow PyCapsule interface. Columns arrive already typed from the file's
    TYPE row (a 2DP heading is ``Float64``, an ID ``String``); a cell the
    permissive cast rejects lands as null, never an error. Ragged-row safety
    is handled upstream in the Rust builder (a short row nulls its tail)."""
    df = pl.from_arrow(table)
    # from_arrow is typed DataFrame | Series; a table always yields a frame.
    return df if isinstance(df, pl.DataFrame) else df.to_frame()


class ArrowStream:
    """A minimal wrapper exposing ONLY ``__arrow_c_stream__``, so DuckDB takes
    its pyarrow-free Arrow-capsule ingest path rather than a frame-library
    special case (``con.register(polars_df)`` would call ``polars.to_arrow()``
    and pull in pyarrow, a ``[compat]`` extra). Wrap any frame that exposes the
    capsule (polars / pyarrow / arro3) before registering it into the engine."""

    __slots__ = ("_o",)

    def __init__(self, obj: Any) -> None:
        self._o = obj

    def __arrow_c_stream__(self, requested_schema: object = None) -> object:
        return self._o.__arrow_c_stream__(requested_schema)


def materialize(frame: pl.DataFrame, backend: str) -> Any:
    """Convert a Polars frame to the requested backend's native frame.

    pandas / pyarrow are imported lazily and ONLY when actually requested, so a
    `polars`-backend user never needs them installed. The **pandas path is
    pyarrow-free** — it goes polars → pandas via DuckDB's NumPy ``.df()``, NOT
    ``polars.to_pandas()`` (which would pull pyarrow). The pyarrow backend
    naturally needs pyarrow (it IS the table you asked for).
    """
    if backend == "polars":
        return frame
    if backend == "pyarrow":
        return frame.to_arrow()
    if backend == "pandas":
        try:
            import pandas as pd  # noqa: F401
        except ModuleNotFoundError as e:
            raise ModuleNotFoundError(
                "the laterite `compat` default backend is pandas but pandas "
                "is not installed. Install it with `pip install "
                "laterite[compat]` (or `uv add laterite[compat]`), or switch "
                "the backend: `laterite.compat.set_backend('polars')` / set "
                "LATERITE_COMPAT_BACKEND=polars (then pandas is not needed)."
            ) from e
        # polars -> pandas via DuckDB's NumPy `.df()` (pyarrow-free); ArrowStream
        # forces DuckDB's Arrow-capsule ingest, not polars.to_arrow() (pyarrow).
        import duckdb

        con = duckdb.connect()
        con.register("__f", ArrowStream(frame))
        return con.sql("SELECT * FROM __f").df()
    raise ValueError(f"unknown backend {backend!r}")


def _pandas_str_dtype() -> Any:
    """python-ags4's pandas-3 baseline string dtype: pandas' Arrow-backed `str`
    with a NaN missing sentinel — byte-for-byte what `future.infer_string=True`
    produces, and (unlike the strict `pd.NA` variant) still matched by
    `select_dtypes(include='object')`. Constructed explicitly so the pandas hop
    can target it regardless of the caller's global `infer_string` option."""
    import numpy as np
    import pandas as pd

    return pd.StringDtype(storage="pyarrow", na_value=np.nan)


def _pandas_missing_error() -> ModuleNotFoundError:
    return ModuleNotFoundError(
        "the laterite `compat` default backend is pandas but pandas is not "
        "installed. Install it with `pip install laterite[compat]` (or "
        "`uv add laterite[compat]`), or switch the backend: "
        "`laterite.compat.set_backend('polars')` / set "
        "LATERITE_COMPAT_BACKEND=polars (then pandas is not needed)."
    )


def _pyarrow_missing_error(what: str) -> ModuleNotFoundError:
    return ModuleNotFoundError(
        f"{what} needs pyarrow, which the default `[compat]` install omits. "
        "Install it with `pip install laterite[compat,pyarrow]` (or `[all]`). "
        "The pyarrow-free `[compat]` still gives object-dtype pandas frames "
        "(via DuckDB's NumPy `.df()`) and the full polars backend — pyarrow only "
        "accelerates the pandas hop a touch and unlocks string_dtype='string' "
        "(pandas' Arrow-backed str)."
    )


def _pyarrow_available() -> bool:
    """Is pyarrow importable? The optional accelerator — its absence routes the
    compat pandas hop through DuckDB's `.df()` instead of pyarrow's `to_pandas`."""
    try:
        import pyarrow as pa  # noqa: F401
    except ModuleNotFoundError:
        return False
    return True


def compat_materializer(
    backend: str, string_dtype: str
) -> Callable[[Any, list[str]], Any]:
    """Resolve — ONCE, before the per-group loop — the cheapest hop from a native
    compat Arrow table (a leading `HEADING` tag column then one `Utf8` column per
    heading, positional field names) to `backend`, returning a
    ``(table, cols) -> frame`` callable that relabels the columns to `cols`.

    Resolving once is deliberate: the hop's shared state is captured in the closure
    and paid ONCE, not per group. The pyarrow hop holds the pyarrow module; the
    pyarrow-free pandas hop holds a single DuckDB connection (a fresh
    ``duckdb.connect()`` per group was the original `AGS4_to_dataframe` regression).

    pyarrow is an OPTIONAL accelerator, not a hard dep. Absent, the pandas backend
    still materialises object-dtype frames via DuckDB's NumPy ``.df()`` (~the same
    speed — the Rust builder already removed the per-cell boxing that was the real
    cost). pyarrow's ``to_pandas`` is a touch faster AND the only route to pandas'
    Arrow-backed `str` dtype, so ``string_dtype='string'`` requires it."""
    if backend == "polars":

        def _polars(table: Any, cols: list[str]) -> Any:
            f = frame_from_arrow(table)
            # Positional native names → the python-ags4 labels (HEADING + headings).
            return f.rename(dict(zip(f.columns, cols, strict=True)))

        return _polars

    if backend == "pyarrow":
        try:
            import pyarrow as pa
        except ModuleNotFoundError as e:
            raise _pyarrow_missing_error("backend='pyarrow'") from e

        def _pyarrow(table: Any, cols: list[str]) -> Any:
            return pa.table(table).rename_columns(cols)

        return _pyarrow

    if backend == "pandas":
        try:
            import pandas as pd  # noqa: F401
        except ModuleNotFoundError as e:
            raise _pandas_missing_error() from e
        if _pyarrow_available():
            # Fast path: pyarrow's `to_pandas` is the cheapest object hop and the
            # only way to reach pandas' Arrow-backed `str` dtype. pyarrow is
            # imported inside the closure — cached, so ~free per call, and it keeps
            # `pa` a plain module binding (no Optional to thread through).
            if string_dtype == "string":
                dt = _pandas_str_dtype()

                def _pandas_arrow_str(table: Any, cols: list[str]) -> Any:
                    import pyarrow as pa

                    pat = pa.table(table).rename_columns(cols)
                    return pat.to_pandas(
                        types_mapper=lambda t: dt if pa.types.is_string(t) else None
                    )

                return _pandas_arrow_str

            def _pandas_arrow_obj(table: Any, cols: list[str]) -> Any:
                import pyarrow as pa

                # numpy object dtype — byte-identical to python-ags4 today.
                return pa.table(table).rename_columns(cols).to_pandas()

            return _pandas_arrow_obj

        # pyarrow-free fallback: object dtype via DuckDB's NumPy `.df()`, over ONE
        # shared connection (register per group, `.df()`, unregister). The
        # Arrow-backed `str` dtype is unreachable without pyarrow.
        if string_dtype == "string":
            raise _pyarrow_missing_error("string_dtype='string'")
        import duckdb

        con = duckdb.connect()

        def _ident(name: str) -> str:
            # A heading name is file-supplied text; doubling `"` keeps it an
            # identifier however hostile the file.
            return '"' + name.replace('"', '""') + '"'

        def _pandas_duckdb(table: Any, cols: list[str]) -> Any:
            # The native table's own capsule goes straight into the engine — a
            # frame-library intermediate here would copy every group's data
            # purely to rename positional columns, avoidable work #834
            # removed. (The shipped hop's larger memory premium over the
            # pyarrow hop proved to live in the DuckDB bridge leg itself —
            # the perf ledger's M5 row carries that attribution.) The rename
            # rides the projection instead; source names come from the
            # engine's own view of the registration, so they cannot fail to
            # resolve, and the strict zip keeps the column-count-mismatch
            # ValueError.
            con.register("__f", ArrowStream(table))
            try:
                src = con.sql("SELECT * FROM __f").columns
                sel = ", ".join(
                    f"{_ident(s)} AS {_ident(c)}"
                    for s, c in zip(src, cols, strict=True)
                )
                return con.sql(f"SELECT {sel} FROM __f").df()
            finally:
                con.unregister("__f")

        return _pandas_duckdb

    raise ValueError(f"unknown backend {backend!r}")
