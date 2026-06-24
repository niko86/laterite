"""`laterite.build_ags4` — the data→AGS4 door (frames → valid AGS4).

Exercises the native `emit_ags4_from_arrow` binding through the public
Python entry point: the DuckDB-bridge boundary (pandas *and* polars,
pyarrow-free), dictionary UNIT/TYPE fill, the three validity modes, and a
build→read round-trip."""

from __future__ import annotations

import laterite
import pandas as pd
import polars as pl
import pytest


def _proj() -> pd.DataFrame:
    return pd.DataFrame({"PROJ_ID": ["P1"], "PROJ_NAME": ["Demo project"]})


def test_emit_fills_unit_and_type_from_dict():
    # Columns are the AGS headings; UNIT/TYPE come from the 4.1.1 dict.
    loca = pd.DataFrame({"LOCA_ID": ["BH01"], "LOCA_GL": [12.3]})
    res = laterite.build_ags4({"PROJ": _proj(), "LOCA": loca})
    text = res.text
    assert '"TYPE","ID","2DP"' in text
    assert '"UNIT","","m"' in text


def test_typed_float_is_canonical_by_construction():
    # A native float under a 2DP heading formats to "12.30" with no fixing.
    loca = pd.DataFrame({"LOCA_ID": ["BH01"], "LOCA_GL": [12.3]})
    res = laterite.build_ags4({"PROJ": _proj(), "LOCA": loca})
    assert '"DATA","BH01","12.30"' in res.text
    assert res.fixes_applied == 0


def test_polars_backend_works_pyarrow_free():
    # polars frames cross the same DuckDB bridge; no pyarrow needed.
    loca = pl.DataFrame({"LOCA_ID": ["BH01", "BH02"], "LOCA_GL": [12.3, 13.0]})
    res = laterite.build_ags4({"PROJ": _proj(), "LOCA": loca})
    assert '"DATA","BH01","12.30"' in res.text
    assert '"DATA","BH02","13.00"' in res.text


def test_autofix_pads_a_string_numeric():
    # A string "12.3" under 2DP is non-compliant; AutoFix's safe fix pads it.
    loca = pl.DataFrame({"LOCA_ID": ["BH01"], "LOCA_GL": ["12.3"]})
    res = laterite.build_ags4({"PROJ": _proj(), "LOCA": loca}, mode="autofix")
    assert res.fixes_applied >= 1
    assert '"12.30"' in res.text
    assert '"12.3"' not in res.text


def test_report_mode_keeps_strings_verbatim():
    loca = pl.DataFrame({"LOCA_ID": ["BH01"], "LOCA_GL": ["12.3"]})
    res = laterite.build_ags4({"PROJ": _proj(), "LOCA": loca}, mode="report")
    assert '"12.3"' in res.text
    assert res.fixes_applied == 0
    # The non-compliant cell is still surfaced as a finding.
    assert any("Rule 8" in f.get("rule", "") for f in res.findings)


def test_strict_mode_raises_on_invalid():
    # No PROJ / TRAN → error-severity rules → Strict refuses.
    loca = pl.DataFrame({"LOCA_ID": ["BH01"], "LOCA_GL": [12.3]})
    with pytest.raises(RuntimeError, match="strict mode rejected"):
        laterite.build_ags4({"LOCA": loca}, mode="strict")


def test_edition_is_selectable():
    loca = pd.DataFrame({"LOCA_ID": ["BH01"], "LOCA_GL": [12.3]})
    # A different edition still resolves + emits (smoke; dict differs internally).
    res = laterite.build_ags4({"PROJ": _proj(), "LOCA": loca}, dict_version="4.2")
    assert '"DATA","BH01","12.30"' in res.text


def test_unknown_edition_and_mode_raise():
    loca = pd.DataFrame({"LOCA_ID": ["BH01"], "LOCA_GL": [12.3]})
    with pytest.raises(RuntimeError, match="unknown edition"):
        laterite.build_ags4({"LOCA": loca}, dict_version="9.9")
    with pytest.raises(RuntimeError, match="unknown mode"):
        laterite.build_ags4({"LOCA": loca}, mode="banana")


def test_round_trips_through_read():
    loca = pl.DataFrame({"LOCA_ID": ["BH01", "BH02"], "LOCA_GL": [12.3, 13.0]})
    res = laterite.build_ags4({"PROJ": _proj(), "LOCA": loca})
    back = laterite.read(text=res.text)
    assert back.groups == ["PROJ", "LOCA"]
    df = back["LOCA"]
    assert df["LOCA_ID"].to_list() == ["BH01", "BH02"]
    assert df["LOCA_GL"].to_list() == [12.3, 13.0]


def test_emit_result_write(tmp_path):
    loca = pd.DataFrame({"LOCA_ID": ["BH01"], "LOCA_GL": [12.3]})
    res = laterite.build_ags4({"PROJ": _proj(), "LOCA": loca})
    out = res.save(tmp_path / "out.ags")
    assert out.read_bytes() == res.bytes
    assert out.read_bytes().startswith(b'"GROUP","PROJ"')


def test_accepts_ordered_pairs_too():
    # A list of (code, frame) pairs preserves order without a Mapping.
    res = laterite.build_ags4(
        [("PROJ", _proj()), ("LOCA", pd.DataFrame({"LOCA_ID": ["BH01"]}))]
    )
    assert res.text.index('"GROUP","PROJ"') < res.text.index('"GROUP","LOCA"')
