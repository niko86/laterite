"""Frame construction + backend resolution.

The native layer hands back plain primitives (string cells — AGS4 is
a text format). Frames are built *here* on the Python side: Polars is
the always-available substrate (mandatory dep), narwhals is the
public wrapper for the nice API, and ``compat`` materialises to a
configurable backend (pandas by default — a true python-ags4
drop-in returns pandas).
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


def _align(values: list[str], n: int) -> list[str]:
    """Pad short / truncate long rows to `n` columns. The Rust parser
    is deliberately tolerant of ragged lines (reporting raggedness is
    a rule's job); the nice API still needs a rectangular frame."""
    if len(values) == n:
        return values
    if len(values) < n:
        return values + [""] * (n - len(values))
    return values[:n]


def polars_string_frame(columns: list[str], rows: list[list[str]]) -> pl.DataFrame:
    """All-`str` Polars frame. Empty rows still yields the right
    columns with a String dtype (so downstream schema is stable)."""
    if not columns:
        return pl.DataFrame()
    data = {
        col: pl.Series(col, [_align(r, len(columns))[i] for r in rows], dtype=pl.String)
        for i, col in enumerate(columns)
    }
    return pl.DataFrame(data)


def materialize(frame: pl.DataFrame, backend: str) -> Any:
    """Convert a Polars frame to the requested backend's native frame.

    pandas / pyarrow are imported lazily and ONLY when actually
    requested, so a `polars`-backend user never needs them installed.
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
        return frame.to_pandas()
    raise ValueError(f"unknown backend {backend!r}")
