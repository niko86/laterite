"""Coverage campaign (P1 residue): compat.py's scattered error / branch arms.

These are the paths no behavioural test happened to reach — the file-like-buffer
reader arms, the python-ags4 hard-raise mirrors (duplicate GROUP / ragged DATA /
duplicate headings), the ``convert_to_text`` dictionary-recovery branches, the
cross-backend ``get_TRAN_AGS`` / ``_columns_of`` fallbacks, ``sort_groups``'s three
strategies, and the ``write_error_report`` formatter.

Where the oracle exposes the same function (``write_error_report``) we assert
byte-for-byte parity against it rather than a hand-rolled expectation — the
strongest form of the drop-in contract. Genuinely-unreachable arms (native
raises ``Ags4Error`` directly, so the ``except RuntimeError`` translations never
fire) are documented ``# pragma: no cover`` in the source, not faked here.
"""

from __future__ import annotations

import io
import tempfile
from pathlib import Path

import pandas as pd
import polars as pl
import pytest
from laterite import compat as AGS4

# White-box access: the engine lives in `compat._impl`, and these tests reach
# for helpers that are deliberately NOT on the public `laterite.compat` surface.
from laterite.compat import _impl as compat_impl

try:
    from python_ags4 import AGS4 as up_AGS4

    _HAS_ORACLE = True
except Exception:  # pragma: no cover - oracle is a declared dev dependency
    _HAS_ORACLE = False

oracle = pytest.mark.skipif(
    not _HAS_ORACLE, reason="python-ags4 parity oracle not installed"
)

_ROOT = Path(__file__).resolve().parents[3]
_CLEAN = (
    _ROOT
    / "rust-packages"
    / "laterite-ags4-validator"
    / "tests"
    / "fixtures"
    / "clean_minimal.ags"
)


def _ags(*lines: str) -> str:
    """Join AGS4 record lines with CRLF + trailing newline."""
    return "\r\n".join(lines) + "\r\n"


# A small clean multi-group file with a blank separator line (exercises the
# strict pre-check's blank-line skip).
_CLEAN_TEXT = _ags(
    '"GROUP","PROJ"',
    '"HEADING","PROJ_ID"',
    '"UNIT",""',
    '"TYPE","ID"',
    '"DATA","P1"',
    "",
    '"GROUP","LOCA"',
    '"HEADING","LOCA_ID","LOCA_GL"',
    '"UNIT","","m"',
    '"TYPE","ID","2DP"',
    '"DATA","BH01","12.30"',
    '"DATA","BH02","13.40"',
)


# --- readers accept file-like buffers, not just paths -----------------------
#
# _primitives / _strict_pre_check / _compat_arrow / check_file each branch on
# `hasattr(obj, "read")`; the path arm is well-trodden, the buffer arm was not.


@pytest.mark.parametrize("as_bytes", [False, True])
def test_ags4_to_dict_accepts_buffer(as_bytes: bool) -> None:
    buf: io.IOBase = (
        io.BytesIO(_CLEAN_TEXT.encode()) if as_bytes else io.StringIO(_CLEAN_TEXT)
    )
    data, headings = AGS4.AGS4_to_dict(buf)
    # Same result the path arm produces (buffer read + decode is transparent).
    with tempfile.NamedTemporaryFile(
        "w", suffix=".ags", delete=False, newline=""
    ) as fh:
        fh.write(_CLEAN_TEXT)
        p = fh.name
    pdata, pheadings = AGS4.AGS4_to_dict(p)
    assert data == pdata and headings == pheadings
    assert set(data) == {"PROJ", "LOCA"}


@pytest.mark.parametrize("as_bytes", [False, True])
def test_ags4_to_dataframe_accepts_buffer(as_bytes: bool) -> None:
    buf = io.BytesIO(_CLEAN_TEXT.encode()) if as_bytes else io.StringIO(_CLEAN_TEXT)
    tables, headings = AGS4.AGS4_to_dataframe(buf)
    assert set(tables) == {"PROJ", "LOCA"}
    assert headings["LOCA"] == ["HEADING", "LOCA_ID", "LOCA_GL"]


def test_check_file_accepts_buffer() -> None:
    from_buf = AGS4.check_file(io.StringIO(_CLEAN_TEXT))
    from_bytes = AGS4.check_file(io.BytesIO(_CLEAN_TEXT.encode()))
    # A buffer has no filename/size, so Metadata differs from the path case, but
    # the Summary-of-data section (data-derived) is identical across both buffers.
    assert from_buf["Summary of data"] == from_bytes["Summary of data"]
    assert "Metadata" in from_buf


# --- python-ags4's hard raises (via the native fast path) -------------------


def test_duplicate_group_raises() -> None:
    dup = _ags(
        '"GROUP","PROJ"',
        '"HEADING","PROJ_ID"',
        '"UNIT",""',
        '"TYPE","ID"',
        '"DATA","P1"',
        '"GROUP","PROJ"',
        '"HEADING","PROJ_ID"',
        '"UNIT",""',
        '"TYPE","ID"',
        '"DATA","P2"',
    )
    with pytest.raises(AGS4.AGS4Error, match="duplicated"):
        AGS4.AGS4_to_dataframe(io.StringIO(dup))


def test_ragged_data_row_raises() -> None:
    ragged = _ags(
        '"GROUP","LOCA"',
        '"HEADING","LOCA_ID","LOCA_GL"',
        '"UNIT","","m"',
        '"TYPE","ID","2DP"',
        '"DATA","BH01","12.30"',
        '"DATA","BH02"',  # one field short of the HEADING row
    )
    with pytest.raises(AGS4.AGS4Error, match="same number of entries"):
        AGS4.AGS4_to_dataframe(io.StringIO(ragged))


_DUP_HEADINGS = _ags(
    '"GROUP","LOCA"',
    '"HEADING","LOCA_ID","LOCA_ID"',  # duplicate heading
    '"UNIT","",""',
    '"TYPE","ID","ID"',
    '"DATA","BH01","BH01"',
)


def test_duplicate_headings_raise_when_rename_disabled() -> None:
    with pytest.raises(AGS4.AGS4Error, match="duplicate entries"):
        AGS4.AGS4_to_dataframe(
            io.StringIO(_DUP_HEADINGS), rename_duplicate_headers=False
        )


def test_duplicate_headings_renamed_by_default() -> None:
    _, headings = AGS4.AGS4_to_dataframe(io.StringIO(_DUP_HEADINGS))
    # python-ags4's scheme: the second LOCA_ID becomes LOCA_ID_1.
    assert headings["LOCA"] == ["HEADING", "LOCA_ID", "LOCA_ID_1"]


# --- get_line_numbers=True: the primitives-reshape fallback -----------------


def test_get_line_numbers_dict_returns_triple() -> None:
    data, _headings, line_numbers = AGS4.AGS4_to_dict(
        io.StringIO(_CLEAN_TEXT), get_line_numbers=True
    )
    # the third element maps each group to its GROUP/HEADING source lines
    assert line_numbers["PROJ"]["GROUP"] == 1
    assert line_numbers["PROJ"]["HEADING"] == 2
    # and a line_number column is appended to each group's headings
    assert data["PROJ"]["line_number"]  # non-empty pseudo-column


def test_get_line_numbers_dataframe_routes_via_dict() -> None:
    tables, _headings, line_numbers = AGS4.AGS4_to_dataframe(
        io.StringIO(_CLEAN_TEXT), get_line_numbers=True
    )
    assert set(tables) == {"PROJ", "LOCA"}
    assert "PROJ" in line_numbers


# --- convert_to_text dictionary recovery ------------------------------------


def _numeric_group_frame(
    col: str, values: list[str], type_: str = "2DP"
) -> pd.DataFrame:
    """A one-numeric-column AGS4-shaped frame (HEADING/UNIT/TYPE + DATA rows),
    ready for convert_to_numeric → convert_to_text(dictionary=...). The TYPE row
    marks the column numeric so convert_to_numeric casts it (convert_to_numeric
    then drops UNIT/TYPE; convert_to_text recovers them from the dictionary)."""
    return pd.DataFrame(
        {
            "HEADING": ["UNIT", "TYPE", *["DATA"] * len(values)],
            col: ["", type_, *values],
        }
    )


def test_convert_to_text_scientific_and_sigfig_from_bundled_dict() -> None:
    # FGHG_IPRM is 1SCI and ICBR_ICBR is 2SF in the bundled 4.1.1 dictionary,
    # so convert_to_text(dictionary="4.1.1") recovers those TYPEs and formats
    # the numeric DATA cells accordingly (the SCI / SF arms of the inner fmt).
    sci = AGS4.convert_to_text(
        AGS4.convert_to_numeric(
            _numeric_group_frame("FGHG_IPRM", ["0.0012345"], "1SCI")
        ),
        dictionary="4.1.1",
    )
    sci_cell = sci.loc[sci.HEADING == "DATA", "FGHG_IPRM"].iloc[0]
    assert "E" in str(sci_cell)  # scientific notation

    sf = AGS4.convert_to_text(
        AGS4.convert_to_numeric(_numeric_group_frame("ICBR_ICBR", ["123.456"], "2SF")),
        dictionary="4.1.1",
    )
    sf_cell = sf.loc[sf.HEADING == "DATA", "ICBR_ICBR"].iloc[0]
    assert str(sf_cell) == "120"  # 2 significant figures


def test_convert_to_text_edition_4_0_maps_to_4_0_4() -> None:
    # "4.0" is remapped to the 4.0.4 patch before the dict lookup (O-30/O-42).
    out = AGS4.convert_to_text(
        AGS4.convert_to_numeric(_numeric_group_frame("LOCA_GL", ["12.3"])),
        dictionary="4.0",
    )
    assert (out.HEADING == "TYPE").any()  # UNIT/TYPE recovered, no raise


def test_convert_to_text_overwrites_existing_unit_type_rows() -> None:
    # A frame that still carries UNIT/TYPE rows: the dictionary's values win and
    # the old rows are dropped (the has_unit/has_type filter branch).
    frame = pd.DataFrame(
        {
            "HEADING": ["UNIT", "TYPE", "DATA"],
            "LOCA_GL": ["stale", "stale", "12.3"],
        }
    )
    out = AGS4.convert_to_text(frame, dictionary="4.1.1")
    assert (out.HEADING == "TYPE").sum() == 1  # exactly one TYPE row, not duplicated


def test_convert_to_text_unknown_dictionary_raises_bad_dict() -> None:
    with pytest.raises(compat_impl.BadDictError):
        AGS4.convert_to_text(
            _numeric_group_frame("LOCA_GL", ["1.0"]),
            dictionary="not-a-version-nor-a-file",
        )


def test_convert_to_text_no_group_prefix_leaves_frame_untouched() -> None:
    # Columns with no inferable 4-letter group code → the dict lookup returns {}
    # (no UNIT/TYPE injected), so a TYPE row must already be present or it raises.
    frame = pd.DataFrame(
        {"HEADING": ["UNIT", "TYPE", "DATA"], "plain": ["", "2DP", "1.0"]}
    )
    out = AGS4.convert_to_text(frame, dictionary="4.1.1")
    assert list(out["HEADING"]) == ["UNIT", "TYPE", "DATA"]


# --- check_file dictionary argument handling --------------------------------


def test_check_file_bundled_dict_basename() -> None:
    # A bundled-dict *basename* (not a version string) resolves to its edition.
    errs = AGS4.check_file(
        str(_CLEAN), standard_AGS4_dictionary="Standard_dictionary_v4_1.ags"
    )
    assert "Summary of data" in errs


def test_check_file_unknown_external_dict_raises() -> None:
    with pytest.raises(compat_impl.BadDictError):
        AGS4.check_file(str(_CLEAN), standard_AGS4_dictionary="/no/such/dict_v9.ags")


def test_try_dict_version_none_is_none() -> None:
    # The non-raising variant returns None for a missing dictionary (the arm the
    # convert_to_text guard never reaches, since it gates on `is not None`).
    assert compat_impl._try_dict_version(None) is None


# --- _columns_of cross-backend enumeration ----------------------------------


class _NamesOnly:
    """A pyarrow-Table-like object: exposes `.column_names`, not `.columns`."""

    column_names = ("HEADING", "LOCA_ID")


class _PydictOnly:
    """Falls back to `.to_pydict().keys()` — neither `.columns` nor names."""

    def to_pydict(self) -> dict[str, list]:
        return {"HEADING": [], "LOCA_ID": []}


def test_columns_of_backend_variants() -> None:
    assert compat_impl._columns_of(_NamesOnly()) == ["HEADING", "LOCA_ID"]
    assert compat_impl._columns_of(_PydictOnly()) == ["HEADING", "LOCA_ID"]
    with pytest.raises(TypeError, match="cannot enumerate columns"):
        compat_impl._columns_of(12345)


# --- get_TRAN_AGS across backends -------------------------------------------


def _tran_frame(backend: str, edition: str | None) -> dict:
    rows = ["UNIT", "TYPE"] + (["DATA"] if edition is not None else [])
    vals = ["", ""] + ([edition] if edition is not None else [])
    if backend == "polars":
        return {"TRAN": pl.DataFrame({"HEADING": rows, "TRAN_AGS": vals})}
    return {"TRAN": pd.DataFrame({"HEADING": rows, "TRAN_AGS": vals})}


def test_get_tran_ags_pandas_and_polars() -> None:
    assert AGS4.get_TRAN_AGS(_tran_frame("pandas", "4.1.1")) == "4.1.1"
    assert AGS4.get_TRAN_AGS(_tran_frame("polars", "4.0.4")) == "4.0.4"


def test_get_tran_ags_missing_group_is_none() -> None:
    assert AGS4.get_TRAN_AGS({"PROJ": pd.DataFrame({"HEADING": ["DATA"]})}) is None


class _PydictTran:
    """pyarrow/unknown backend: reached via the `.to_pydict()` fallback."""

    def __init__(self, headings: list[str], trans: list[str]) -> None:
        self._d = {"HEADING": headings, "TRAN_AGS": trans}

    def to_pydict(self) -> dict[str, list]:
        return self._d


def test_get_tran_ags_pydict_fallback() -> None:
    assert (
        AGS4.get_TRAN_AGS({"TRAN": _PydictTran(["UNIT", "DATA"], ["", "4.2"])}) == "4.2"
    )
    # no DATA row → the fallback yields nothing → None
    assert AGS4.get_TRAN_AGS({"TRAN": _PydictTran(["UNIT", "TYPE"], ["", ""])}) is None


def test_get_tran_ags_unknown_object_is_none() -> None:
    # An object exposing neither loc/filter/to_pydict → AttributeError swallow → None.
    assert AGS4.get_TRAN_AGS({"TRAN": object()}) is None


# --- sort_groups: the three strategies + unknown-group handling -------------


def _dict_tables_with_project_groups() -> dict[str, pd.DataFrame]:
    """A tables dict whose DICT group declares two project-specific GROUP codes
    (XXXA child of PROJ, XXXB child of XXXA) so the dictionary/hierarchical
    strategies pick them up via _extract_project_group(_parents)."""
    dict_df = pd.DataFrame(
        {
            "HEADING": ["UNIT", "TYPE", "DATA", "DATA"],
            "DICT_TYPE": ["", "", "GROUP", "GROUP"],
            "DICT_GRP": ["", "", "XXXA", "XXXB"],
            "DICT_PGRP": ["", "", "PROJ", "XXXA"],
        }
    )
    stub = pd.DataFrame({"HEADING": ["DATA"]})
    return {"PROJ": stub, "DICT": dict_df, "XXXA": stub, "XXXB": stub, "LOCA": stub}


def test_sort_groups_alphabetical() -> None:
    tables = {"LOCA": 1, "ABBR": 2, "PROJ": 3}
    assert list(AGS4.sort_groups(tables, "alphabetical")) == ["ABBR", "LOCA", "PROJ"]


def test_sort_groups_dictionary_appends_project_groups() -> None:
    out = list(AGS4.sort_groups(_dict_tables_with_project_groups(), "dictionary"))
    assert out[0] == "PROJ"  # PROJ heads the dictionary order
    # the DICT-declared project groups are appended (present in the input)
    assert "XXXA" in out and "XXXB" in out
    assert out.index("LOCA") < out.index("XXXA")  # standard groups before project ones


def test_sort_groups_hierarchical_descends_project_parents() -> None:
    out = list(AGS4.sort_groups(_dict_tables_with_project_groups(), "hierarchical"))
    assert out[0] == "PROJ"
    # XXXA (child of PROJ) then XXXB (child of XXXA) via recursive descent
    assert out.index("XXXA") < out.index("XXXB")


def test_sort_groups_unknown_group_appended_with_warning() -> None:
    tables = {"PROJ": 1, "ZZZZ": 2}  # ZZZZ is in neither the registry nor a DICT
    with pytest.warns(UserWarning, match="ZZZZ"):
        out = AGS4.sort_groups(tables, "dictionary")
    assert "ZZZZ" in out


def test_sort_groups_unknown_strategy_raises() -> None:
    with pytest.raises(ValueError, match="unknown sorting_strategy"):
        AGS4.sort_groups({"PROJ": 1}, "bananas")


# --- write_error_report: byte-for-byte parity with the oracle ---------------
#
# Feed the SAME synthetic ags_errors dict to compat and python-ags4 and diff the
# written bytes — the strongest form of the "same report" drop-in contract. The
# synthetic dict carries every section the writer branches on.


def _full_ags_errors() -> dict:
    return {
        "Metadata": [
            {"line": "File Name", "group": "", "desc": "demo.ags"},
            {"line": "Checker", "group": "", "desc": "laterite"},
        ],
        "General": [
            {"line": "", "group": "", "desc": "A long general note " * 8},
        ],
        "Summary of data": [
            {
                "line": "",
                "group": "",
                "desc": "3 groups identified in file: PROJ LOCA TRAN",
            },
        ],
        "AGS Format Rule 5": [
            {"line": 12, "group": '"LOCA"', "desc": "Contains a semicolon."},
        ],
        "Validator Process Error": [
            {"line": "-", "group": "", "desc": "engine note"},
        ],
        "Warning (Related to Rule 16)": [
            {"line": 4, "group": '"ABBR"', "desc": "unlisted abbreviation"},
        ],
        "FYI (Related to Rule 16)": [
            {"line": 5, "group": '"TYPE"', "desc": "informational"},
        ],
    }


def _write(fn, ags_errors, **kw) -> bytes:
    with tempfile.NamedTemporaryFile(
        "w", suffix=".txt", delete=False, newline=""
    ) as fh:
        path = fh.name
    fn(ags_errors, path, **kw)
    return Path(path).read_bytes()


@oracle
def test_write_error_report_matches_oracle_full() -> None:
    errs = _full_ags_errors()
    ours = _write(AGS4.write_error_report, errs, show_warnings=True, show_fyi=True)
    theirs = _write(up_AGS4.write_error_report, errs, show_warnings=True, show_fyi=True)
    assert ours == theirs


@oracle
def test_write_error_report_matches_oracle_clean() -> None:
    # error_count == 0 → the "All checks passed!" branch.
    errs = {"Metadata": [{"line": "Checker", "group": "", "desc": "laterite"}]}
    assert _write(AGS4.write_error_report, errs) == _write(
        up_AGS4.write_error_report, errs
    )


@oracle
def test_write_error_report_matches_oracle_ags3_abort() -> None:
    # A Rule 3 finding mentioning AGS3 → the "Checking aborted" branch.
    errs = {
        "Metadata": [{"line": "Checker", "group": "", "desc": "laterite"}],
        "AGS Format Rule 3": [
            {"line": 1, "group": "", "desc": "This looks like an AGS3 file."}
        ],
    }
    assert _write(AGS4.write_error_report, errs) == _write(
        up_AGS4.write_error_report, errs
    )


def test_write_error_report_none_output_is_noop() -> None:
    # output_file=None returns without writing (python-ags4's TypeError swallow).
    assert AGS4.write_error_report(_full_ags_errors(), None) is None


def test_write_error_report_missing_parent_dir_warns() -> None:
    missing = str(Path(tempfile.mkdtemp()) / "no_such_dir" / "report.txt")
    with pytest.warns(UserWarning, match="could not write"):
        AGS4.write_error_report(_full_ags_errors(), missing)


# --- AGS4 <-> Excel error surfaces ------------------------------------------


def test_ags4_to_excel_rejects_invalid_input() -> None:
    d = Path(tempfile.mkdtemp())
    (d / "garbage.ags").write_text("not ags4 at all\r\n")
    with pytest.raises(AGS4.AGS4Error, match="No valid AGS4 data"):
        AGS4.AGS4_to_excel(str(d / "garbage.ags"), str(d / "out.xlsx"))


def test_excel_to_ags4_reraises_non_xlsx() -> None:
    d = Path(tempfile.mkdtemp())
    (d / "garbage.xlsx").write_text("not a workbook\r\n")
    # A genuine open error surfaces as RuntimeError (not the no-data Ags4Error),
    # so the translation arm is skipped and the error re-raises unchanged.
    with pytest.raises(RuntimeError):
        AGS4.excel_to_AGS4(str(d / "garbage.xlsx"), str(d / "out.ags"))


# --- remaining odds and ends ------------------------------------------------


def test_backend_getter() -> None:
    assert AGS4.get_backend() in {"pandas", "polars", "pyarrow"}


def _write_ags(text: str) -> str:
    with tempfile.NamedTemporaryFile(
        "w", suffix=".ags", delete=False, newline=""
    ) as fh:
        fh.write(text)
    return fh.name


def test_convert_to_text_external_dict_file_with_empty_heading() -> None:
    # An external AGS4 DICT file (not a bundled version): convert_to_text reads
    # its UNIT/TYPE via _unit_type_from_external_dict_file. The empty-heading DATA
    # row is skipped; LOCA_GL's (unit, type) is recovered.
    dict_file = _write_ags(
        _ags(
            '"GROUP","DICT"',
            '"HEADING","DICT_HDNG","DICT_UNIT","DICT_DTYP"',
            '"UNIT","","",""',
            '"TYPE","X","X","X"',
            '"DATA","LOCA_GL","m","2DP"',
            '"DATA","","",""',  # empty heading -> `if not h: continue`
        )
    )
    frame = pd.DataFrame(
        {"HEADING": ["UNIT", "TYPE", "DATA"], "LOCA_GL": ["", "", "12.3"]}
    )
    out = AGS4.convert_to_text(frame, dictionary=dict_file)
    assert out.loc[out.HEADING == "TYPE", "LOCA_GL"].iloc[0] == "2DP"


def test_convert_to_text_external_dict_file_without_dict_group() -> None:
    # An AGS4 file with no DICT group -> the recovery map is empty ({}), and the
    # inject still runs (no raise), leaving UNIT/TYPE rows blank.
    no_dict = _write_ags(
        _ags(
            '"GROUP","PROJ"',
            '"HEADING","PROJ_ID"',
            '"UNIT",""',
            '"TYPE","ID"',
            '"DATA","P1"',
        )
    )
    frame = pd.DataFrame(
        {"HEADING": ["UNIT", "TYPE", "DATA"], "LOCA_GL": ["", "", "12.3"]}
    )
    out = AGS4.convert_to_text(frame, dictionary=no_dict)
    assert (out.HEADING == "TYPE").sum() == 1


def test_sort_groups_polars_dict_falls_back_to_standard_order() -> None:
    # A polars DICT table has no `.loc`, so both _extract_* helpers return [] and
    # sort_groups falls back to the standard order rather than crashing.
    dict_pl = pl.DataFrame(
        {
            "HEADING": ["DATA"],
            "DICT_TYPE": ["GROUP"],
            "DICT_GRP": ["XXXA"],
            "DICT_PGRP": ["PROJ"],
        }
    )
    tables = {"PROJ": 1, "DICT": dict_pl, "LOCA": 2}
    assert next(iter(AGS4.sort_groups(tables, "dictionary"))) == "PROJ"
    assert next(iter(AGS4.sort_groups(tables, "hierarchical"))) == "PROJ"
