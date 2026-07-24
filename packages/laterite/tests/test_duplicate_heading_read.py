"""Duplicate HEADING names on the read path (AGS4 Rule 7).

The read surfaces (`lat read`, the excel export, node, `read_groups_raw`) never
run the rule engine, so Rule 7 does not fire for them. Rows are keyed by heading
name, which made a repeat worse than lossy: the second occurrence overwrote the
first, and consumers that walk `headings` positionally then read the survivor at
*both* positions. This file read back as ``["SECOND", "1.00", "SECOND"]`` --
``FIRST`` gone AND ``SECOND`` duplicated into its column, silently, so the column
looked fully populated and was wrong.

Fatal by default now; the recovery mode suffixes the repeats so nothing is lost.
"""

import json

import pytest
from laterite import _cli
from laterite import _laterite_native as _native

DUP = (
    "\r\n".join(
        [
            '"GROUP","LOCA"',
            '"HEADING","LOCA_ID","LOCA_GL","LOCA_ID"',
            '"UNIT","","m",""',
            '"TYPE","ID","2DP","ID"',
            '"DATA","FIRST","1.00","SECOND"',
        ]
    )
    + "\r\n"
)


@pytest.fixture
def dup_file(tmp_path):
    p = tmp_path / "dup.ags"
    p.write_text(DUP, encoding="utf-8")
    return p


def test_read_groups_raw_refuses_a_duplicate_heading(dup_file):
    with pytest.raises(ValueError, match="duplicate heading"):
        _native.read_groups_raw(str(dup_file))


def test_recovery_keeps_every_cell_in_its_own_column(dup_file):
    raw = _native.read_groups_raw(str(dup_file), True)
    g = raw["groups"]["LOCA"]
    assert g["headings"] == ["LOCA_ID", "LOCA_GL", "LOCA_ID__2"]
    # The regression guard: positionally, both values survive.
    assert g["rows"] == [["FIRST", "1.00", "SECOND"]]


def test_cli_read_refuses_then_recovers(dup_file, capsys):
    assert _cli.main(["read", str(dup_file), "LOCA", "--json"]) != 0
    assert "duplicate heading" in capsys.readouterr().err

    assert (
        _cli.main(
            [
                "read",
                str(dup_file),
                "LOCA",
                "--json",
                "--recover-duplicate-headings",
            ]
        )
        == 0
    )
    body = json.dumps(json.loads(capsys.readouterr().out))
    assert "LOCA_ID__2" in body
    assert "FIRST" in body and "SECOND" in body


def test_a_clean_file_is_unaffected_by_the_flag(tmp_path):
    """The guard must cost nothing for the overwhelmingly common case."""
    p = tmp_path / "clean.ags"
    p.write_text(
        '"GROUP","LOCA"\r\n"HEADING","LOCA_ID"\r\n"DATA","BH01"\r\n',
        encoding="utf-8",
    )
    strict = _native.read_groups_raw(str(p))
    recover = _native.read_groups_raw(str(p), True)
    assert strict["order"] == recover["order"]
    assert strict["groups"]["LOCA"] == recover["groups"]["LOCA"]
