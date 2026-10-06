"""#1011 — DICT rows that redeclare a standard group or heading, and pruning them.

The effective dictionary reads the standard entry first, so a DICT row for a
standard name changes nothing validation checks; a reader that takes DICT
literally can still be misled by it. Pinned on the issue's own repro:

* one ``FYI (Related to Rule 18)`` per such row, naming the edition and every
  field that differs; a WARNING instead when it drops KEY or re-parents;
* user-defined rows are silent, and neither tier moves ``is_valid``;
* the edition judged against is the one validation resolved;
* ``dict_rows="prune"`` on ``build_ags4`` and ``merge`` drops the redundant
  rows, never adds a finding, and ``"keep"`` is byte-identical to the default.
"""

from __future__ import annotations

import typing

import laterite
import polars as pl
import pytest
from laterite import _laterite_native as _native

STAMP = laterite.TranStamp(
    issue="1", date="2026-01-01", producer="A", recipient="B", status="Draft"
)
FYI18 = "FYI (Related to Rule 18)"
WARN18 = "Warning (Related to Rule 18)"
DICT_COLS = (
    "DICT_TYPE",
    "DICT_GRP",
    "DICT_HDNG",
    "DICT_STAT",
    "DICT_DTYP",
    "DICT_DESC",
    "DICT_UNIT",
    "DICT_PGRP",
)


def _dict(*rows: tuple[str, ...]) -> pl.DataFrame:
    return pl.DataFrame(
        {c: [r[i] for r in rows] for i, c in enumerate(DICT_COLS)},
        schema=dict.fromkeys(DICT_COLS, pl.String),
    )


def _frames(dict_frame: pl.DataFrame | None = None, *, xtra: bool = False) -> dict:
    k = {
        "LOCA_ID": ["BH1", "BH1"],
        "SAMP_TOP": [1.0, 2.0],
        "SAMP_REF": ["1", "2"],
        "SAMP_TYPE": ["U", "D"],
        "SAMP_ID": ["S1", "S2"],
    }
    spec = {"SPEC_REF": ["1", "1"], "SPEC_DPTH": [1.0, 2.0]}
    lnmc = {**k, **spec, "LNMC_MC": [11.3, 14.1]}
    if xtra:
        lnmc["LNMC_XTRA"] = ["a", "b"]
    frames = {
        "PROJ": pl.DataFrame({"PROJ_ID": ["P1"]}),
        "LOCA": pl.DataFrame({"LOCA_ID": ["BH1"]}),
        "SAMP": pl.DataFrame(k),
        "LNMC": pl.DataFrame(lnmc),
    }
    if dict_frame is not None:
        frames["DICT"] = dict_frame
    return frames


def _build(dict_frame: pl.DataFrame | None = None, **kw) -> bytes:
    xtra = kw.pop("xtra", False)
    return laterite.build_ags4(
        _frames(dict_frame, xtra=xtra),
        dict_version="4.1.1",
        synthesise_metadata=True,
        tran=STAMP,
        **kw,
    ).bytes


MC_ROW = ("HEADING", "LNMC", "LNMC_MC", "OTHER", "X", "Water content", "%", "")
MC_DIFFERS = ("HEADING", "LNMC", "LNMC_MC", "OTHER", "2DP", "Water content", "kg", "")
TOP_NOT_KEY = (
    "HEADING",
    "SAMP",
    "SAMP_TOP",
    "OTHER",
    "2DP",
    "Depth to top of sample",
    "m",
    "",
)
SAMP_REPARENTED = ("GROUP", "SAMP", "", "", "", "Sample Information", "", "PROJ")
XTRA_ROW = ("HEADING", "LNMC", "LNMC_XTRA", "OTHER", "X", "An extra heading", "", "")


def _rows(report, rule: str) -> list[str]:
    return [d["desc"] for d in report.findings.to_dicts() if d["rule"] == rule]


def test_the_issue_repro_names_the_heading_and_the_edition():
    rep = laterite.validate(_build(_dict(MC_ROW)), fyi=True)
    assert _rows(rep, FYI18) == [
        'DICT declares LNMC.LNMC_MC, a standard heading in 4.1.1 (DICT_DESC "Water '
        'content" vs standard "Water/moisture content"); the standard definition '
        "applies."
    ]
    assert rep.is_valid


def test_a_contradicting_type_and_unit_are_both_listed():
    rep = laterite.validate(_build(_dict(MC_DIFFERS)), fyi=True)
    (desc,) = _rows(rep, FYI18)
    assert 'DICT_DTYP "2DP" vs standard "X"' in desc
    assert 'DICT_UNIT "kg" vs standard "%"' in desc
    assert rep.is_valid


@pytest.mark.parametrize(
    ("row", "why"),
    [(TOP_NOT_KEY, "drops KEY"), (SAMP_REPARENTED, "different parent")],
)
def test_dropping_key_or_reparenting_is_a_warning_not_an_fyi(row, why):
    data = _build(_dict(row))
    rep = laterite.validate(data, fyi=True)
    (warning,) = _rows(rep, WARN18)
    assert why in warning
    assert _rows(rep, FYI18) == []
    # Shown by default, and never the verdict.
    assert _rows(laterite.validate(data), WARN18) == [warning]
    assert rep.is_valid


def test_a_user_defined_row_is_never_named():
    rep = laterite.validate(_build(_dict(XTRA_ROW), xtra=True), fyi=True)
    assert _rows(rep, FYI18) == []
    assert _rows(rep, WARN18) == []


def test_without_fyi_only_the_warning_cases_report():
    data = _build(_dict(MC_DIFFERS, TOP_NOT_KEY))
    default = laterite.validate(data)
    assert _rows(default, FYI18) == []
    assert len(_rows(default, WARN18)) == 1
    assert default.is_valid == laterite.validate(data, fyi=True).is_valid


def test_the_edition_is_the_one_validation_resolved():
    # LNMC_DEV is standard from 4.1; under 4.0.4 the same row is user-defined.
    dev = ("HEADING", "LNMC", "LNMC_DEV", "OTHER", "X", "Deviation", "", "")
    data = _build(_dict(dev))
    (desc,) = _rows(laterite.validate(data, fyi=True, dict_version="4.1"), FYI18)
    assert "a standard heading in 4.1 " in desc
    assert _rows(laterite.validate(data, fyi=True, dict_version="4.0.4"), FYI18) == []


def _dict_rows_of(data: bytes) -> list[list[str]]:
    lines = data.decode().split("\r\n")
    try:
        start = lines.index('"GROUP","DICT"')
    except ValueError:
        return []
    out = []
    for line in lines[start + 1 :]:
        if not line:
            break
        if line.startswith('"DATA"'):
            out.append([c.strip('"') for c in line.split('","')][1:])
    return out


def _reported(data: bytes) -> set[tuple[str, str]]:
    rep = laterite.validate(data, fyi=True)
    return {(d["rule"], d["desc"]) for d in rep.findings.to_dicts()}


def test_merge_prune_keeps_only_the_user_defined_row():
    a = _build(_dict(MC_ROW, TOP_NOT_KEY, XTRA_ROW), xtra=True)
    b = _build(_dict(MC_DIFFERS, XTRA_ROW), xtra=True)
    keep = laterite.merge(a, b, tran=STAMP).bytes
    prune = laterite.merge(a, b, tran=STAMP, dict_rows="prune").bytes
    assert len(_dict_rows_of(keep)) > 1
    (row,) = _dict_rows_of(prune)
    assert row[:3] == ["HEADING", "LNMC", "LNMC_XTRA"]
    assert _reported(prune) <= _reported(keep)


def test_build_prune_drops_an_absent_heading_and_then_dict_itself():
    gone = ("HEADING", "LNMC", "LNMC_GONE", "OTHER", "X", "Never written", "", "")
    data = _build(_dict(MC_ROW, gone), dict_rows="prune")
    assert _dict_rows_of(data) == []
    assert b'"GROUP","DICT"' not in data
    assert _reported(data) <= _reported(_build(_dict(MC_ROW, gone)))


def test_keep_is_byte_identical_to_the_default():
    frame = _dict(MC_ROW, TOP_NOT_KEY)
    assert _build(frame, dict_rows="keep") == _build(frame)
    a, b = _build(frame), _build(_dict(MC_DIFFERS))
    assert laterite.merge(a, b, dict_rows="keep").bytes == laterite.merge(a, b).bytes


def test_an_unknown_dict_rows_is_a_value_error():
    with pytest.raises(ValueError, match="unknown dict_rows 'trim'"):
        _build(_dict(MC_ROW), dict_rows="trim")
    a = _build(_dict(MC_ROW))
    with pytest.raises(ValueError, match="unknown dict_rows 'trim'"):
        laterite.merge(a, a, dict_rows="trim")


def test_the_literal_and_the_cli_choices_match_the_engine():
    """`DictRowsMode` cannot derive itself, so it is pinned to `DictRows::ALL`."""
    from laterite import _cli

    authority = list(_native.registry_dict_rows_modes())
    assert authority == ["keep", "prune"]
    assert list(typing.get_args(laterite.DictRowsMode)) == authority
    assert list(_cli._DICT_ROWS_CHOICES) == authority
