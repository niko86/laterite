"""A DATA row carrying more fields than its group declares headings (#776).

Rows bind to headings by position, so the surplus binds to nothing. It used to
be dropped, and the row came back looking complete -- which every later step
believed: a shortened row still satisfies Rule 4, so the file validates clean,
certifies clean, and re-emits as conforming AGS4 that no longer says what its
author wrote.

The usual cause is AGS4 Rule 5 -- a value containing a comma whose quotes were
lost. Nothing can say which side of the comma the heading wanted, so the read
refuses. ``--truncate-excess-fields`` opts back into the old, lossy behaviour
for salvage.
"""

import json

import pytest
from laterite import _cli
from laterite import _laterite_native as _native

# `PROJ_NAME` was one authored value, `"Acme, Bloggs and Co"`, and lost its
# quotes. Every other line is well-formed, which is exactly what let this
# survive: the file looks ordinary and the loss is one field wide.
AMBIGUOUS = (
    "\r\n".join(
        [
            '"GROUP","PROJ"',
            '"HEADING","PROJ_ID","PROJ_NAME"',
            '"UNIT","",""',
            '"TYPE","ID","X"',
            '"DATA","P1",Acme, Bloggs and Co',
        ]
    )
    + "\r\n"
)


@pytest.fixture
def ambiguous_file(tmp_path):
    p = tmp_path / "ambiguous.ags"
    p.write_text(AMBIGUOUS, encoding="utf-8")
    return p


def test_read_groups_raw_refuses_the_unbindable_field(ambiguous_file):
    with pytest.raises(ValueError, match="belong to no heading"):
        _native.read_groups_raw(str(ambiguous_file))


def test_truncation_is_available_and_is_lossy(ambiguous_file):
    raw = _native.read_groups_raw(str(ambiguous_file), False, True)
    g = raw["groups"]["PROJ"]
    assert g["headings"] == ["PROJ_ID", "PROJ_NAME"]
    # Stated rather than implied: `Bloggs and Co` is gone, and the column reads
    # as a complete-looking `Acme`. That is what the opt-in costs.
    assert g["rows"] == [["P1", "Acme"]]


def test_cli_read_refuses_then_truncates(ambiguous_file, capsys):
    assert _cli.main(["read", str(ambiguous_file), "PROJ", "--json"]) != 0
    err = capsys.readouterr().err
    assert "belong to no heading" in err
    # The line is what a reader has to act on, so it must be named.
    assert "line 5" in err

    assert (
        _cli.main(
            [
                "read",
                str(ambiguous_file),
                "PROJ",
                "--json",
                "--truncate-excess-fields",
            ]
        )
        == 0
    )
    body = json.loads(capsys.readouterr().out)
    assert body == [{"PROJ_ID": "P1", "PROJ_NAME": "Acme"}]


def test_a_quoted_comma_is_one_field_and_still_reads(tmp_path):
    """The control: the guard keys on the LOST QUOTES, not on the comma."""
    p = tmp_path / "quoted.ags"
    p.write_text(
        '"GROUP","PROJ"\r\n'
        '"HEADING","PROJ_ID","PROJ_NAME"\r\n'
        '"DATA","P1","Acme, Bloggs and Co"\r\n',
        encoding="utf-8",
    )
    raw = _native.read_groups_raw(str(p))
    assert raw["groups"]["PROJ"]["rows"] == [["P1", "Acme, Bloggs and Co"]]


def test_a_short_row_still_pads_rather_than_failing(tmp_path):
    """The other direction is NOT symmetrical and must not become so.

    A row with FEWER fields than headings loses nothing -- the missing tail is
    knowable -- so it still pads. Only the unbindable direction is fatal.
    """
    p = tmp_path / "short.ags"
    p.write_text(
        '"GROUP","PROJ"\r\n"HEADING","PROJ_ID","PROJ_NAME"\r\n"DATA","P1"\r\n',
        encoding="utf-8",
    )
    raw = _native.read_groups_raw(str(p))
    assert raw["groups"]["PROJ"]["rows"] == [["P1", ""]]
