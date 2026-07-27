"""Public-API surface tests (coverage campaign P3: `laterite/__init__.py`).

The read/validate/query/build/merge/excel doors carry the reprs and small
glue arms that no behavioural test happened to exercise. These drive the *public*
flow end-to-end and assert the observable result — a repr is a debugging contract
(it shows in every traceback), so we pin its shape, not merely that it doesn't
raise. Where an uncovered arm is genuinely unreachable on the shipped Python
floor it is left as a documented `# pragma: no cover` in the source, not faked
here.
"""

from __future__ import annotations

from pathlib import Path

import laterite as L
import pandas as pd
import polars as pl
import pytest

_ROOT = Path(__file__).resolve().parents[3]
_FIX = _ROOT / "rust-packages" / "laterite-ags4-validator" / "tests" / "fixtures"
_CLEAN = _FIX / "clean_minimal.ags"

# A multi-group file (PROJ / LOCA sharing LOCA_ID, plus an XN-typed FRAC) so the
# fan-out accessors and the SQL engine have something real to bite on.
_AGS = "\r\n".join(
    [
        '"GROUP","PROJ"',
        '"HEADING","PROJ_ID"',
        '"UNIT",""',
        '"TYPE","ID"',
        '"DATA","P1"',
        '"GROUP","LOCA"',
        '"HEADING","LOCA_ID","LOCA_GL"',
        '"UNIT","","m"',
        '"TYPE","ID","2DP"',
        '"DATA","BH01","12.30"',
        '"DATA","BH02","13.40"',
        '"GROUP","FRAC"',
        '"HEADING","LOCA_ID","FRAC_FI"',
        '"UNIT","","mm"',
        '"TYPE","ID","XN"',
        '"DATA","BH01","12.5"',
        '"DATA","BH01","NP"',
        "",
    ]
)


# --- Ags4File: repr + membership + subscript --------------------------------


def test_ags4file_repr_and_membership() -> None:
    f = L.read(_CLEAN)
    r = repr(f)
    assert r.startswith("<Ags4File groups=") and "tran_ags=" in r
    assert "PROJ" in f  # Ags4File.__contains__
    assert "NOPE" not in f
    assert isinstance(f["PROJ"], pl.DataFrame)  # __getitem__ materialises a group
    assert set(f.groups) >= {"PROJ", "TRAN"}


# --- Report: repr + .file ---------------------------------------------------


def test_report_repr_and_file() -> None:
    rep = L.validate(_CLEAN)
    assert rep.file == str(_CLEAN)
    text = repr(rep)
    assert text.startswith("<Report ")
    # a clean file reports "valid"; the repr encodes validity + dict edition
    assert ("valid" in text) or ("finding(s)" in text)
    assert "dict=" in text


# --- AgsQuery: the full-featured repr (every bit) + membership + query() ----


def test_agsquery_repr_covers_every_bit() -> None:
    f = L.read(text=_AGS)
    q = (
        f.at("LOCA", ["BH01"])
        .query("SELECT * FROM LOCA")
        .filter("LOCA_GL > 1")
        .select("LOCA_ID", "LOCA_GL")
    )
    r = repr(q)
    # each builder contributes its own clause to the repr
    assert r.startswith("<AgsQuery ")
    assert "at[" in r and "LOCA_ID" in r
    assert "query=" in r
    assert "filter[" in r
    assert "select" in r
    assert "LOCA" in q  # AgsQuery.__contains__ delegates to the parent handle


def test_query_and_frame_conversions() -> None:
    f = L.read(text=_AGS)
    base = f.query("SELECT * FROM LOCA")  # query() sets the single-result base
    assert isinstance(base.to_polars(), pl.DataFrame)
    assert isinstance(base.to_pandas(), pd.DataFrame)


# --- register(): user frame into the engine ---------------------------------


def test_register_user_frame() -> None:
    f = L.read(_CLEAN)
    # register()'s body runs on the call itself (no AGS-group registration needed);
    # a capsule-exposing frame routes through the pyarrow-free ArrowStream path.
    f.register("mine", pl.DataFrame({"k": ["a", "b"], "v": [1, 2]}))
    got = f.sql("SELECT COUNT(*) AS n FROM mine").fetchone()
    assert got[0] == 2


# --- build_ags4: BuildResult repr + the synth-key drop --------------------
#
# NB: build from a *polars* frame here, not pandas. A modern pandas frame
# (>=2.2) exposes __arrow_c_stream__ so it takes the same capsule path as
# polars anyway (the DuckDB fallback at build_ags4's 2136-2142 is pandas<2.2
# only, hence unreachable on the CI/dev pandas — a documented gap, not a test
# target). Building from pandas here ALSO tripped a native crash when a later
# native write ran in the same process — an arrow-rs use-after-free consuming
# pandas' pyarrow-backed stream, now guarded (niko86/laterite#122, apache/arrow-rs#10439).


def test_build_result_repr() -> None:
    br = L.build_ags4(
        {"PROJ": pl.DataFrame({"PROJ_ID": ["P1"], "PROJ_NAME": ["X"]})},
        synthesise_metadata=True,
    )
    r = repr(br)
    assert r.startswith("<BuildResult ") and "byte" in r and "finding(s)" in r
    assert '"GROUP","PROJ"' in br.text


def test_build_drops_synth_key_columns_polars() -> None:
    # a read(keys=True) frame carries synthetic _id/_parent_id; build must never
    # emit them — the polars drop-branch of _drop_synth_keys.
    frame = pl.DataFrame({"PROJ_ID": ["P1"], "PROJ_NAME": ["X"], "_id": [0]})
    br = L.build_ags4({"PROJ": frame}, synthesise_metadata=True)
    assert "_id" not in br.text


def test_pandas_build_then_native_write_is_safe() -> None:
    """Regression for niko86/laterite#122 (arrow-rs UAF, apache/arrow-rs#10439):
    a pandas frame's pyarrow-backed capsule handed to the native emit corrupted a
    later in-process native write. The guard normalises pandas ->2.2 to a polars
    capsule first. Assert the pandas build equals the polars build (proof the
    normalisation ran) AND that a native write after it stays intact."""
    import gc

    pandas_text = L.build_ags4(
        {"PROJ": pd.DataFrame({"PROJ_ID": ["P1"], "PROJ_NAME": ["ACME"]})},
        synthesise_metadata=True,
    ).text
    polars_text = L.build_ags4(
        {"PROJ": pl.DataFrame({"PROJ_ID": ["P1"], "PROJ_NAME": ["ACME"]})},
        synthesise_metadata=True,
    ).text
    assert pandas_text == polars_text  # the guard converted pandas -> polars

    gc.collect()
    assert L.to_excel(L.read(_CLEAN))[:2] == b"PK"  # native write uncorrupted


# --- merge: MergeResult repr ------------------------------------------------


def test_merge_result_repr() -> None:
    mr = L.merge(_CLEAN, _CLEAN, on_type_clash="widen")
    r = repr(mr)
    assert r.startswith("MergeResult(") and "byte" in r
    assert "warning(s)" in r and "revision(s)" in r


# --- to_excel: Ags4File source + in-memory bytes + the not-AGS error --------


def test_to_excel_from_ags4file_and_bytes_mode() -> None:
    f = L.read(_CLEAN)
    from_handle = L.to_excel(f)  # Ags4File branch
    assert isinstance(from_handle, bytes) and from_handle[:2] == b"PK"
    from_path = L.to_excel(source=str(_CLEAN))  # bytes mode (output=None)
    assert isinstance(from_path, bytes) and from_path[:2] == b"PK"


def test_to_excel_rejects_non_ags() -> None:
    with pytest.raises(L.NotAgs4Error):
        L.to_excel(data=b"this is not ags4 at all\r\n")


# --- _resolve_source: the unsupported-type door -----------------------------


def test_resolve_source_rejects_unsupported_type() -> None:
    from laterite import _resolve_source

    with pytest.raises(TypeError, match="unsupported source type"):
        _resolve_source(12345)
