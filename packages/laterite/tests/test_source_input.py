"""`laterite.read()` input modes — path / text / bytes / file-like parse
identically, with explicit `path=`/`text=`/`data=` overrides + a content sniff.

The Rust boundary has a `data` door routed to the existing `parse::parse_bytes`,
and `read()` resolves any input shape to it. The invariant under test: the SAME
bytes reach the parser whatever shape the caller hands them in, so every input
mode yields the same parse.
"""

from __future__ import annotations

import io

import laterite as lat
import polars as pl
import pytest

AGS = (
    '"GROUP","PROJ"\n"HEADING","PROJ_ID"\n"UNIT",""\n"TYPE","ID"\n"DATA","P1"\n\n'
    '"GROUP","LOCA"\n"HEADING","LOCA_ID","LOCA_GL"\n"UNIT","","m"\n"TYPE","ID","2DP"\n'
    '"DATA","BH01","100.50"\n"DATA","BH02","98.00"\n'
)


@pytest.fixture
def ags_path(tmp_path):
    p = tmp_path / "site.ags"
    p.write_text(AGS, encoding="utf-8")
    return p


def _loca(ags) -> pl.DataFrame:
    return ags["LOCA"].sort("LOCA_ID")


def test_every_input_mode_parses_identically(ags_path):
    raw = AGS.encode("utf-8")
    ref = _loca(lat.read(ags_path))  # path (positional, exists → path)
    variants = {
        "explicit text": lat.read(text=AGS),
        "explicit data": lat.read(data=raw),
        "positional bytes": lat.read(raw),
        "positional ags-text": lat.read(AGS),
        "file-like bytes": lat.read(io.BytesIO(raw)),
        "file-like text": lat.read(io.StringIO(AGS)),
        "explicit path": lat.read(path=str(ags_path)),
    }
    for name, ags in variants.items():
        assert sorted(ags.groups) == ["LOCA", "PROJ"], name
        assert _loca(ags).equals(ref), f"{name}: LOCA frame differs from path parse"
    # born-typed survives the bytes path (a 2DP heading is a real double)
    assert ref["LOCA_GL"].dtype == pl.Float64
    assert ref.filter(pl.col("LOCA_ID") == "BH01")["LOCA_GL"][0] == 100.5


def test_validate_accepts_bytes(ags_path):
    by_path = lat.validate(ags_path)
    by_bytes = lat.validate(AGS.encode("utf-8"))
    # same engine, same verdict, byte-faithful JSON whichever way the bytes arrived
    assert by_bytes.count == by_path.count
    assert by_bytes.to_ndjson() == by_path.to_ndjson()


def test_dict_for_accepts_bytes():
    assert lat.dict_for(AGS.encode("utf-8")) == lat.dict_for(text=AGS)


def test_encoding_applies_to_bytes():
    # é is 0xE9 in windows-1252; decoding as UTF-8 would mojibake/replace it.
    raw = '"GROUP","PROJ"\n"HEADING","PROJ_ID"\n"UNIT",""\n"TYPE","ID"\n"DATA","Pré"\n'.encode(
        "windows-1252"
    )
    proj = lat.read(data=raw, encoding="windows-1252")["PROJ"]
    assert proj["PROJ_ID"][0] == "Pré"


def test_explicit_inputs_are_mutually_exclusive(ags_path):
    with pytest.raises(TypeError, match="only one of"):
        lat.read(path=str(ags_path), text=AGS)


def test_no_source_raises():
    with pytest.raises(TypeError):
        lat.read()


def test_str_sniff_prefers_existing_path(ags_path):
    # a str that is an existing path → path; an AGS-content str → text.
    assert sorted(lat.read(str(ags_path)).groups) == ["LOCA", "PROJ"]
    assert sorted(lat.read(AGS).groups) == ["LOCA", "PROJ"]


def test_source_is_an_alias_of_read(ags_path):
    # `source` is the fluent-chain entry name; `read` is the plain verb. Same
    # callable — one surface, two vocabularies — and both are exported.
    assert lat.source is lat.read
    assert "source" in lat.__all__
    # exercises the alias end-to-end across the input shapes read() accepts.
    assert sorted(lat.source(ags_path).groups) == ["LOCA", "PROJ"]
    assert _loca(lat.source(data=AGS.encode("utf-8"))).equals(_loca(lat.read(text=AGS)))
