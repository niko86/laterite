"""Backend-resolution tests for `laterite._frames` (coverage campaign P4).

`_frames` picks the cheapest hop from the Rust-built Arrow table to the
caller's chosen frame backend (polars / pandas / pyarrow), and its most
interesting code — the **pyarrow-free pandas fallback** (polars → pandas via
DuckDB's NumPy `.df()`) and the dep-missing guidance errors — only runs when a
dependency is *absent*. pyarrow and pandas are both installed in the dev env,
so those arms are exercised by *simulating* the missing dependency with an
`__import__` shim. That is not gaming coverage: `pip install laterite[compat]`
(pandas, no pyarrow) is a real, shipped install shape, and this is the path it
takes.
"""

from __future__ import annotations

import builtins
from typing import Any

import polars as pl
import pytest
from laterite import _frames


def _hide(monkeypatch: Any, *names: str) -> None:
    """Make `import <name>` raise ModuleNotFoundError for the given modules,
    delegating every other import to the real machinery."""
    real = builtins.__import__

    def fake(name: str, *a: Any, **k: Any) -> Any:
        if name in names or any(name.startswith(f"{n}.") for n in names):
            raise ModuleNotFoundError(f"No module named {name!r}")
        return real(name, *a, **k)

    monkeypatch.setattr(builtins, "__import__", fake)


def _frame() -> pl.DataFrame:
    return pl.DataFrame({"HEADING": ["DATA", "DATA"], "LOCA_ID": ["BH1", "BH2"]})


def _native_like() -> tuple[Any, list[str]]:
    """A stand-in for the native compat Arrow table (positional column names)
    plus the python-ags4 labels the materializer relabels them to."""
    import pyarrow as pa

    tbl = pa.table({"c0": ["DATA", "DATA"], "c1": ["BH1", "BH2"]})
    return tbl, ["HEADING", "LOCA_ID"]


# --- backend / string-dtype resolution + validation -----------------------


def test_resolve_backend_default_and_explicit() -> None:
    assert _frames.resolve_backend(None) == _frames.get_default_backend()
    assert _frames.resolve_backend("PoLaRs") == "polars"


def test_unknown_backend_raises() -> None:
    with pytest.raises(ValueError, match="unknown backend"):
        _frames.resolve_backend("numpy")
    with pytest.raises(ValueError, match="unknown backend"):
        _frames.set_default_backend("numpy")


def test_string_dtype_resolution_and_validation() -> None:
    assert _frames.resolve_string_dtype(None) == _frames.get_default_string_dtype()
    assert _frames.resolve_string_dtype("STRING") == "string"
    with pytest.raises(ValueError, match="unknown string_dtype"):
        _frames.resolve_string_dtype("utf8")
    with pytest.raises(ValueError, match="unknown string_dtype"):
        _frames.set_default_string_dtype("utf8")


def test_set_default_backend_round_trips(monkeypatch: Any) -> None:
    monkeypatch.setattr(_frames, "_DEFAULT_BACKEND", "pandas")
    _frames.set_default_backend("polars")
    assert _frames.get_default_backend() == "polars"


def test_set_default_string_dtype_round_trips(monkeypatch: Any) -> None:
    monkeypatch.setattr(_frames, "_DEFAULT_STRING_DTYPE", "object")
    _frames.set_default_string_dtype("string")
    assert _frames.get_default_string_dtype() == "string"


# --- materialize: every backend -------------------------------------------


def test_materialize_polars_is_identity() -> None:
    f = _frame()
    assert _frames.materialize(f, "polars") is f


def test_materialize_pyarrow() -> None:
    import pyarrow as pa

    out = _frames.materialize(_frame(), "pyarrow")
    assert isinstance(out, pa.Table)
    assert out.column_names == ["HEADING", "LOCA_ID"]


def test_materialize_pandas_is_pyarrow_free() -> None:
    import pandas as pd

    out = _frames.materialize(_frame(), "pandas")
    assert isinstance(out, pd.DataFrame)
    assert list(out.columns) == ["HEADING", "LOCA_ID"]


def test_materialize_unknown_backend_raises() -> None:
    with pytest.raises(ValueError, match="unknown backend"):
        _frames.materialize(_frame(), "numpy")


def test_materialize_pandas_missing_raises_guidance(monkeypatch: Any) -> None:
    _hide(monkeypatch, "pandas")
    with pytest.raises(ModuleNotFoundError, match="laterite\\[compat\\]"):
        _frames.materialize(_frame(), "pandas")


# --- compat_materializer: the per-backend closures ------------------------


def test_compat_materializer_polars() -> None:
    tbl, cols = _native_like()
    fn = _frames.compat_materializer("polars", "object")
    out = fn(tbl, cols)
    assert isinstance(out, pl.DataFrame)
    assert out.columns == cols


def test_compat_materializer_pyarrow() -> None:
    import pyarrow as pa

    tbl, cols = _native_like()
    fn = _frames.compat_materializer("pyarrow", "object")
    out = fn(tbl, cols)
    assert isinstance(out, pa.Table)
    assert out.column_names == cols


def test_compat_materializer_pandas_fast_path_when_pyarrow_present() -> None:
    import pandas as pd

    tbl, cols = _native_like()
    fn = _frames.compat_materializer("pandas", "object")
    out = fn(tbl, cols)
    assert isinstance(out, pd.DataFrame)
    assert list(out.columns) == cols


def test_compat_materializer_pandas_string_dtype_needs_pyarrow() -> None:
    tbl, cols = _native_like()
    fn = _frames.compat_materializer("pandas", "string")
    out = fn(tbl, cols)
    # pandas' Arrow-backed string dtype only reachable via pyarrow.
    assert str(out["LOCA_ID"].dtype) in ("string", "large_string[pyarrow]", "str")


def test_compat_materializer_unknown_backend_raises() -> None:
    with pytest.raises(ValueError, match="unknown backend"):
        _frames.compat_materializer("numpy", "object")


# --- the pyarrow-FREE pandas fallback (real [compat]-only install) --------


def test_pyarrow_available_false_when_absent(monkeypatch: Any) -> None:
    _hide(monkeypatch, "pyarrow")
    assert _frames._pyarrow_available() is False


def test_compat_materializer_pandas_duckdb_fallback(monkeypatch: Any) -> None:
    """With pyarrow absent, the pandas hop must still produce an object-dtype
    frame — via DuckDB's `.df()`, over one shared connection."""
    monkeypatch.setattr(_frames, "_pyarrow_available", lambda: False)
    import pandas as pd

    tbl, cols = _native_like()
    fn = _frames.compat_materializer("pandas", "object")
    out = fn(tbl, cols)
    assert isinstance(out, pd.DataFrame)
    assert list(out.columns) == cols
    assert out["LOCA_ID"].tolist() == ["BH1", "BH2"]


def test_compat_materializer_pandas_duckdb_skips_polars_intermediate(
    monkeypatch: Any,
) -> None:
    """The shipped pyarrow-free hop must register the native table's own Arrow
    capsule — never copy it into polars just to rename (#834): the copy was
    per-group avoidable work the fix removed. (The shipped hop's larger memory
    premium over the pyarrow hop proved to live in the DuckDB bridge leg
    itself — the perf ledger's M5 row carries the attribution.)"""
    monkeypatch.setattr(_frames, "_pyarrow_available", lambda: False)
    monkeypatch.setattr(
        _frames,
        "frame_from_arrow",
        lambda table: pytest.fail("the duckdb hop took the polars intermediate"),
    )
    tbl, cols = _native_like()
    fn = _frames.compat_materializer("pandas", "object")
    out = fn(tbl, cols)
    assert list(out.columns) == cols
    assert out["LOCA_ID"].tolist() == ["BH1", "BH2"]


def test_compat_materializer_pandas_duckdb_hostile_heading_names(
    monkeypatch: Any,
) -> None:
    """A heading name is file-supplied text. An embedded `"` must land in the
    output verbatim rather than terminate the projection's quoted identifier —
    the rename happens in SQL now, so the label is an identifier, not data."""
    monkeypatch.setattr(_frames, "_pyarrow_available", lambda: False)
    import pyarrow as pa

    tbl = pa.table({"c0": ["DATA"], "c1": ["x"]})
    cols = ["HEADING", 'WEIRD"NAME, "c0" --']
    fn = _frames.compat_materializer("pandas", "object")
    out = fn(tbl, cols)
    assert list(out.columns) == cols
    assert out[cols[1]].tolist() == ["x"]


def test_compat_materializer_pandas_duckdb_col_count_mismatch_raises(
    monkeypatch: Any,
) -> None:
    """A label list that doesn't match the table's column count keeps raising
    ValueError (the strict zip), never silently dropping or padding columns."""
    monkeypatch.setattr(_frames, "_pyarrow_available", lambda: False)
    tbl, _ = _native_like()
    fn = _frames.compat_materializer("pandas", "object")
    with pytest.raises(ValueError, match="shorter"):
        fn(tbl, ["HEADING"])


def test_pandas_hops_agree(monkeypatch: Any) -> None:
    """The shipped pyarrow-free hop and the accelerated pyarrow hop hand back
    the same pandas frame — the dev venv has pyarrow, so the wider compat suite
    only ever exercises the fast hop; this is the cross-hop pin."""
    import pandas.testing as pdt

    tbl, cols = _native_like()
    fast = _frames.compat_materializer("pandas", "object")(tbl, cols)
    monkeypatch.setattr(_frames, "_pyarrow_available", lambda: False)
    shipped = _frames.compat_materializer("pandas", "object")(tbl, cols)
    pdt.assert_frame_equal(fast, shipped)


def test_string_dtype_string_without_pyarrow_raises(monkeypatch: Any) -> None:
    monkeypatch.setattr(_frames, "_pyarrow_available", lambda: False)
    with pytest.raises(ModuleNotFoundError, match="pyarrow"):
        _frames.compat_materializer("pandas", "string")


def test_compat_materializer_pyarrow_backend_missing_raises(monkeypatch: Any) -> None:
    _hide(monkeypatch, "pyarrow")
    with pytest.raises(ModuleNotFoundError, match="pyarrow"):
        _frames.compat_materializer("pyarrow", "object")


def test_compat_materializer_pandas_missing_raises(monkeypatch: Any) -> None:
    _hide(monkeypatch, "pandas")
    with pytest.raises(ModuleNotFoundError, match="laterite\\[compat\\]"):
        _frames.compat_materializer("pandas", "object")


# --- the error-message helpers --------------------------------------------


def test_missing_dependency_error_helpers() -> None:
    assert isinstance(_frames._pandas_missing_error(), ModuleNotFoundError)
    err = _frames._pyarrow_missing_error("backend='pyarrow'")
    assert isinstance(err, ModuleNotFoundError)
    assert "pyarrow" in str(err)
