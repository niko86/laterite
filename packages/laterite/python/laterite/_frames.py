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
from typing import Any

import polars as pl

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
            import pandas  # noqa: F401
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
