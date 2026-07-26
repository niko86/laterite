"""Differential-parity tests for `laterite.compat`'s python-ags4 dict-table
and numeric-formatting helpers (coverage campaign P1 — see
`ags-wiki/concepts/coverage-campaign.md`).

`get_{DICT,ABBR,TYPE,UNIT}_table_from_json_file`, `format_numeric_column`,
`_format_sf` and `count_errors` are drop-in mirrors of `python_ags4.{utils,
AGS4}`. Their contract *is* "behave identically to python-ags4", so the
strongest test is a **differential** one: feed identical input to laterite
and the oracle and assert the outputs are equal. Standalone structural
invariants back them up, so the core shape stays pinned even if the oracle
is ever absent.

pandas `read_json` coerces an all-numeric-looking `Version` column to float
(breaking the `.str.contains` filter) — real AGS-DFWG exports carry
comma-joined version lists, so the fixtures below use those (e.g.
``"4.0.4,4.1,4.2"``), which is also what keeps laterite and the oracle in
lockstep here.
"""

from __future__ import annotations

import json
from typing import Any

import pandas as pd
import pandas.testing as pt
import pytest
from laterite import compat as AGS4

try:
    from python_ags4 import AGS4 as up_AGS4
    from python_ags4 import utils as up_utils

    _HAS_ORACLE = True
except Exception:  # pragma: no cover - oracle is a declared dev dependency
    _HAS_ORACLE = False

oracle = pytest.mark.skipif(
    not _HAS_ORACLE, reason="python-ags4 parity oracle not installed"
)

# --- fixtures: minimal but branch-exercising AGS-DFWG JSON -----------------

_DICT_JSON = [
    {
        "group": "PROJ",
        "heading": "PROJ_ID",
        "suggested_type": "ID",
        "description": "Project ID",
        "suggested_unit": "",
        "example": "121415",
        "heading_status": "KEY",
        "in_group_order": 1,
        "group_order": 1,
        "group_description": "Project Information",
        "parent": "",
        "group_status": "Approved",
    },
    {
        "group": "LOCA",
        "heading": "LOCA_ID",
        "suggested_type": "ID",
        "description": "Location ID",
        "suggested_unit": "",
        "example": "BH01",
        "heading_status": "KEY",
        "in_group_order": 1,
        "group_order": 2,
        "group_description": "Location Details",
        "parent": "PROJ",
        "group_status": "Approved",
    },
    {
        # description carries an embedded double-quote AND a newline — both
        # must be scrubbed by the DICT_DESC hygiene pass.
        "group": "LOCA",
        "heading": "LOCA_NATE",
        "suggested_type": "2DP",
        "description": 'Easting with a "quoted" span\nand a line break',
        "suggested_unit": "m",
        "example": "523145.25",
        "heading_status": "REQUIRED",
        "in_group_order": 2,
        "group_order": 2,
        "group_description": "Location Details",
        "parent": "PROJ",
        "group_status": "Approved",
    },
    {
        # a Deprecated group — its GROUP row must carry DICT_STAT=DEPRECATED.
        "group": "XXXX",
        "heading": "XXXX_ID",
        "suggested_type": "ID",
        "description": "Legacy id",
        "suggested_unit": "",
        "example": "X",
        "heading_status": "KEY",
        "in_group_order": 1,
        "group_order": 3,
        "group_description": "A deprecated group",
        "parent": "PROJ",
        "group_status": "Deprecated",
    },
]

_ABBR_JSON = [
    {
        "Group": "LOCA_TYPE",
        "Code": "RC",
        "Description": "Rotary Core",
        "Version": "4.0.4,4.1,4.2",
        "Status": "Approved",
    },
    # 4.0.3-only → filtered out for version 4.1.
    {
        "Group": "LOCA_TYPE",
        "Code": "TP",
        "Description": "Trial Pit",
        "Version": "4.0.3",
        "Status": "Approved",
    },
    # not Approved → filtered by Status.
    {
        "Group": "SAMP_TYPE",
        "Code": "B",
        "Description": "Bulk",
        "Version": "4.1,4.2",
        "Status": "Proposed",
    },
]

_ELRG_JSON = [
    {
        "code": "AC",
        "description": "Acid soluble",
        "version": "4.0.4,4.1,4.2",
        "status": "Approved",
    },
    {"code": "OLD", "description": "legacy", "version": "4.0.3", "status": "Approved"},
]

_TYPE_JSON = [
    {"Type": "2DP", "Desc": "2 decimal places", "Version": "4.0.4,4.1,4.2"},
    {"Type": "ID", "Desc": "Identifier", "Version": "4.0.4,4.1,4.2"},
    {"Type": "OLD", "Desc": "legacy type", "Version": "4.0.3"},  # filtered for 4.1
]

_UNIT_JSON = [
    {
        "Unit": "mm",
        "Description": "millimetre",
        "Version": "4.0.4,4.1,4.2",
        "Status": "Approved",
    },
    # exact duplicate on UNIT_UNIT → dropped (keep-first).
    {
        "Unit": "mm",
        "Description": "duplicate millimetre",
        "Version": "4.1",
        "Status": "Approved",
    },
    # differs only by case → the case-insensitive sort must group it with `mm`.
    {
        "Unit": "MM",
        "Description": "mega-molar",
        "Version": "4.1,4.2",
        "Status": "Approved",
    },
    # 4.0.3-only → filtered out for version 4.1.
    {
        "Unit": "kN",
        "Description": "kilonewton",
        "Version": "4.0.3",
        "Status": "Approved",
    },
]


def _write(tmp_path: Any, name: str, obj: Any) -> str:
    p = tmp_path / name
    p.write_text(json.dumps(obj), encoding="utf-8")
    return str(p)


# --- differential parity vs the python-ags4 oracle ------------------------


@oracle
def test_get_DICT_table_matches_oracle(tmp_path: Any) -> None:
    p = _write(tmp_path, "dict.json", _DICT_JSON)
    pt.assert_frame_equal(
        AGS4.get_DICT_table_from_json_file(p),
        up_utils.get_DICT_table_from_json_file(p),
    )


@oracle
def test_get_ABBR_table_matches_oracle(tmp_path: Any) -> None:
    p = _write(tmp_path, "abbr.json", _ABBR_JSON)
    pt.assert_frame_equal(
        AGS4.get_ABBR_table_from_json_file(p, version="4.1"),
        up_utils.get_ABBR_table_from_json_file(p, None, "4.1"),
    )


@oracle
def test_get_ABBR_table_with_elrg_matches_oracle(tmp_path: Any) -> None:
    """The optional ELRG-codes join is its own branch — exercise it."""
    p = _write(tmp_path, "abbr.json", _ABBR_JSON)
    elrg = _write(tmp_path, "elrg.json", _ELRG_JSON)
    pt.assert_frame_equal(
        AGS4.get_ABBR_table_from_json_file(p, filepath_ELRG=elrg, version="4.1"),
        up_utils.get_ABBR_table_from_json_file(p, elrg, "4.1"),
    )


@oracle
def test_get_TYPE_table_matches_oracle(tmp_path: Any) -> None:
    p = _write(tmp_path, "type.json", _TYPE_JSON)
    pt.assert_frame_equal(
        AGS4.get_TYPE_table_from_json_file(p, version="4.1"),
        up_utils.get_TYPE_table_from_json_file(p, "4.1"),
    )


@oracle
def test_get_UNIT_table_matches_oracle(tmp_path: Any) -> None:
    p = _write(tmp_path, "unit.json", _UNIT_JSON)
    pt.assert_frame_equal(
        AGS4.get_UNIT_table_from_json_file(p, version="4.1"),
        up_utils.get_UNIT_table_from_json_file(p, "4.1"),
    )


# --- standalone structural invariants (no oracle needed) ------------------

_DICT_COLS = [
    "HEADING",
    "DICT_TYPE",
    "DICT_GRP",
    "DICT_HDNG",
    "DICT_STAT",
    "DICT_DTYP",
    "DICT_DESC",
    "DICT_UNIT",
    "DICT_EXMP",
    "DICT_PGRP",
    "DICT_REM",
    "FILE_FSET",
]


def test_get_DICT_table_shape_invariants(tmp_path: Any) -> None:
    dict_df = AGS4.get_DICT_table_from_json_file(_write(tmp_path, "d.json", _DICT_JSON))
    # Exact column set + order.
    assert list(dict_df.columns) == _DICT_COLS
    # The AGS4 pseudo-rows lead the frame.
    assert dict_df.iloc[0]["HEADING"] == "UNIT"
    assert dict_df.iloc[1]["HEADING"] == "TYPE"
    # Every input group produces exactly one GROUP row.
    groups = dict_df.loc[dict_df["DICT_TYPE"] == "GROUP", "DICT_GRP"].tolist()
    assert sorted(groups) == ["LOCA", "PROJ", "XXXX"]
    # The deprecated group is marked, and only on its GROUP row.
    dep = dict_df[(dict_df["DICT_GRP"] == "XXXX") & (dict_df["DICT_TYPE"] == "GROUP")]
    assert dep["DICT_STAT"].item() == "DEPRECATED"


def test_get_DICT_table_scrubs_description(tmp_path: Any) -> None:
    """Rule-5 hygiene: no embedded double-quote or newline survives in
    DICT_DESC (they would break AGS4 quoting on re-emit)."""
    dict_df = AGS4.get_DICT_table_from_json_file(_write(tmp_path, "d.json", _DICT_JSON))
    joined = "".join(dict_df["DICT_DESC"].tolist())
    assert '"' not in joined
    assert "\n" not in joined and "\r" not in joined
    # the scrubbed easting description keeps its text, single-quoted.
    assert any("'quoted'" in d for d in dict_df["DICT_DESC"])


def test_get_UNIT_table_dedups_and_filters(tmp_path: Any) -> None:
    unit_df = AGS4.get_UNIT_table_from_json_file(
        _write(tmp_path, "u.json", _UNIT_JSON), version="4.1"
    )
    data = unit_df[unit_df["HEADING"] == "DATA"]["UNIT_UNIT"].tolist()
    assert data.count("mm") == 1  # exact duplicate dropped
    assert "kN" not in data  # 4.0.3-only filtered out for 4.1
    assert "MM" in data  # distinct casing kept


def test_get_TYPE_table_filters_by_version(tmp_path: Any) -> None:
    type_df = AGS4.get_TYPE_table_from_json_file(
        _write(tmp_path, "t.json", _TYPE_JSON), version="4.1"
    )
    data = type_df[type_df["HEADING"] == "DATA"]["TYPE_TYPE"].tolist()
    assert set(data) == {"2DP", "ID"}  # "OLD" (4.0.3-only) excluded


@pytest.mark.parametrize(
    "fn",
    [
        AGS4.get_ABBR_table_from_json_file,
        AGS4.get_TYPE_table_from_json_file,
        AGS4.get_UNIT_table_from_json_file,
    ],
)
def test_invalid_version_raises(tmp_path: Any, fn: Any) -> None:
    """`_valid_dict_version` rejects anything outside 4.0/4.1/4.2."""
    p = _write(tmp_path, "x.json", _TYPE_JSON)
    with pytest.raises(AGS4.AGS4Error):
        fn(p, version="9.9")


# --- format_numeric_column: parity across every TYPE branch ---------------


def _numeric_frame() -> pd.DataFrame:
    return pd.DataFrame(
        {
            "HEADING": ["UNIT", "TYPE", "DATA", "DATA", "DATA"],
            "SAMP_TOP": [None, None, 1.239, 12.0, 0.0],
        }
    )


@oracle
@pytest.mark.parametrize("type_spec", ["0DP", "2DP", "4DP", "3SCI", "3SF", "2SF"])
def test_format_numeric_column_matches_oracle(type_spec: str) -> None:
    frame = _numeric_frame()
    pt.assert_frame_equal(
        AGS4.format_numeric_column(frame, "SAMP_TOP", type_spec),
        up_AGS4.format_numeric_column(frame, "SAMP_TOP", type_spec),
    )


@oracle
def test_format_numeric_column_passthrough_type(tmp_path: Any) -> None:
    """A non-numeric TYPE (e.g. 'X') leaves the column untouched — same as
    the oracle."""
    frame = _numeric_frame()
    pt.assert_frame_equal(
        AGS4.format_numeric_column(frame, "SAMP_TOP", "X"),
        up_AGS4.format_numeric_column(frame, "SAMP_TOP", "X"),
    )


@oracle
def test_format_numeric_column_non_numeric_data_matches_oracle() -> None:
    """A column with non-numeric DATA hits the except arm: both laterite and
    the oracle log-and-return the frame unmodified rather than raising."""
    frame = pd.DataFrame(
        {"HEADING": ["UNIT", "TYPE", "DATA"], "SAMP_TOP": ["", "2DP", "not-a-number"]}
    )
    pt.assert_frame_equal(
        AGS4.format_numeric_column(frame, "SAMP_TOP", "2DP"),
        up_AGS4.format_numeric_column(frame, "SAMP_TOP", "2DP"),
    )


def test_format_numeric_column_does_not_mutate_input() -> None:
    """The docstring promises the original frame is untouched (works on a
    copy)."""
    frame = _numeric_frame()
    frame_copy = frame.copy(deep=True)
    AGS4.format_numeric_column(frame, "SAMP_TOP", "2DP")
    pt.assert_frame_equal(frame, frame_copy)  # NaN-safe equality


# --- _format_sf and count_errors: direct parity ---------------------------


@oracle
@pytest.mark.parametrize(
    ("value", "type_spec"),
    [
        (1.239, "3SF"),
        (0.0, "3SF"),
        (1234.5, "2SF"),
        (-0.00456, "2SF"),
        (999.9, "2SF"),
        (0.001, "1SF"),
        (-12345.0, "3SF"),
    ],
)
def test_format_sf_matches_oracle(value: float, type_spec: str) -> None:
    assert AGS4._format_sf(value, type_spec) == up_AGS4._format_SF(value, type_spec)


@oracle
def test_count_errors_matches_oracle() -> None:
    errs = {
        "AGS Format Rule 1": [{"line": 1}, {"line": 2}],
        "Validator Process Error": [{"line": 3}],
        "Warning (General)": [{"line": 4}],
        "FYI (General)": [{"line": 5}, {"line": 6}],
    }
    assert AGS4.count_errors(errs) == up_AGS4.count_errors(errs)


def test_count_errors_categorises() -> None:
    """Standalone: errors (Rule + Process) / warnings / FYI counted by key."""
    errs = {
        "AGS Format Rule 2a": [{"line": 1}],
        "Validator Process Error": [{"line": 2}],
        "Warning (Rule 16)": [{"line": 3}, {"line": 4}],
        "FYI (General)": [{"line": 5}],
    }
    assert AGS4.count_errors(errs) == (2, 2, 1)
