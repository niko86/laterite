"""The compat write's two doors agree, and both keep the refusal contract.

`dataframe_to_AGS4` routes a fresh UTF-8 write through the streaming door
(group-at-a-time straight into a temp file, renamed on success — #805) and
everything else (append, transcode) through the byte-building door. The two
must produce identical bytes, and BOTH must preserve the contract the
build-then-write shape always gave: a refused cell (embedded newline, #423)
leaves whatever was at the destination untouched — and leaves no temp litter
beside it.
"""

from __future__ import annotations

import polars as pl
import pytest
from laterite import compat as AGS4


def _frames(name_cell: str = "Anytown Redevelopment"):
    """A minimal PROJ delivery in python-ags4's dataframe shape."""
    proj = pl.DataFrame(
        {
            "HEADING": ["UNIT", "TYPE", "DATA"],
            "PROJ_ID": ["", "ID", "P1"],
            "PROJ_NAME": ["", "X", name_cell],
        }
    )
    return {"PROJ": proj}, {"PROJ": ["HEADING", "PROJ_ID", "PROJ_NAME"]}


def test_streaming_and_byte_doors_write_identical_bytes(tmp_path):
    """mode="w" takes the streaming door; mode="a" onto a fresh path takes the
    byte-building door. Same input → the same file, byte for byte."""
    t, h = _frames()
    streamed = tmp_path / "streamed.ags"
    appended = tmp_path / "appended.ags"
    AGS4.dataframe_to_AGS4(t, h, str(streamed))
    AGS4.dataframe_to_AGS4(t, h, str(appended), mode="a")
    assert streamed.read_bytes() == appended.read_bytes()


def test_append_mode_appends(tmp_path):
    t, h = _frames()
    p = tmp_path / "grow.ags"
    AGS4.dataframe_to_AGS4(t, h, str(p))
    first = p.read_bytes()
    AGS4.dataframe_to_AGS4(t, h, str(p), mode="a")
    assert p.read_bytes() == first + first


def test_non_utf8_encoding_transcodes(tmp_path):
    """A non-UTF-8 ``encoding=`` takes the byte-building door and transcodes;
    the µ survives as cp1252's single 0xB5 byte, not UTF-8's pair."""
    t, h = _frames(name_cell="5µm sieve")
    p = tmp_path / "legacy.ags"
    AGS4.dataframe_to_AGS4(t, h, str(p), encoding="cp1252")
    raw = p.read_bytes()
    assert b"5\xb5m sieve" in raw
    assert "5µm sieve" in raw.decode("cp1252")


@pytest.mark.parametrize("mode", ["w", "a"])
def test_a_refused_cell_leaves_the_destination_untouched(tmp_path, mode):
    """The #423 refusal (a cell carrying CR/LF cannot be written faithfully)
    must not damage what is already at the destination — the streaming door
    stages to a temp file and renames only on success, the byte door builds
    before it opens. Also: no temp litter left beside the target."""
    t, h = _frames(name_cell="torn\nvalue")
    p = tmp_path / "precious.ags"
    p.write_bytes(b"the original bytes")
    with pytest.raises(Exception, match=r"CR/LF|newline|Rule 6|embedded"):
        AGS4.dataframe_to_AGS4(t, h, str(p), mode=mode)
    assert p.read_bytes() == b"the original bytes"
    assert [f.name for f in tmp_path.iterdir()] == ["precious.ags"]
