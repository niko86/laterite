"""The typed read of `DT` cells (#992).

`laterite.read` hands DT columns back as polars `Datetime`. Any value no parser
format matched became a null with no finding, and the `T`-separated minute form
(the dictionary UNIT of many 4.1+ DT headings) and fractional seconds were both
missing — so a file laterite itself had just built read back with its dates
blanked."""

from __future__ import annotations

import datetime as dt

import laterite
import polars as pl


def _samp(unit: str, value: str) -> str:
    return "\r\n".join(
        [
            '"GROUP","PROJ"',
            '"HEADING","PROJ_ID"',
            '"UNIT",""',
            '"TYPE","ID"',
            '"DATA","P1"',
            "",
            '"GROUP","LOCA"',
            '"HEADING","LOCA_ID"',
            '"UNIT",""',
            '"TYPE","ID"',
            '"DATA","BH1"',
            "",
            '"GROUP","SAMP"',
            '"HEADING","LOCA_ID","SAMP_TOP","SAMP_REF","SAMP_TYPE","SAMP_ID","SAMP_DTIM"',
            f'"UNIT","","m","","","","{unit}"',
            '"TYPE","ID","2DP","X","PA","ID","DT"',
            f'"DATA","BH1","1.00","1","B","","{value}"',
            "",
        ]
    )


def _dtim(unit: str, value: str) -> dt.datetime | None:
    return laterite.read(text=_samp(unit, value))["SAMP"]["SAMP_DTIM"][0]


def test_t_separated_minute_precision_reads():
    # The issue's reproduction.
    assert _dtim("yyyy-mm-ddThh:mm", "2026-03-02T09:15") == dt.datetime(
        2026, 3, 2, 9, 15
    )


def test_fractional_seconds_read_to_the_millisecond():
    assert _dtim("yyyy-mm-ddThh:mm:ss.sss", "2026-03-02T09:15:00.500") == dt.datetime(
        2026, 3, 2, 9, 15, 0, 500_000
    )


def test_a_malformed_value_still_reads_as_null():
    assert _dtim("yyyy-mm-ddThh:mm", "2026-03-02T9h15") is None


def test_a_built_4_2_file_reads_its_own_minute_dates_back():
    # "laterite cannot read back its own 4.2 build output": SAMP_DTIM's 4.2
    # UNIT is yyyy-mm-ddThh:mm, so the build writes minute precision.
    samp = pl.DataFrame(
        {
            "LOCA_ID": ["BH1"],
            "SAMP_TOP": [1.0],
            "SAMP_REF": ["1"],
            "SAMP_TYPE": ["B"],
            "SAMP_ID": pl.Series([None], dtype=pl.String),
            "SAMP_DTIM": ["2026-03-02T09:15"],
        }
    )
    frames = {
        "PROJ": pl.DataFrame({"PROJ_ID": ["P1"]}),
        "LOCA": pl.DataFrame({"LOCA_ID": ["BH1"]}),
        "SAMP": samp,
    }
    built = laterite.build_ags4(frames, dict_version="4.2")
    assert '"2026-03-02T09:15"' in built.text
    back = laterite.read(built.bytes)["SAMP"]["SAMP_DTIM"][0]
    assert back == dt.datetime(2026, 3, 2, 9, 15)
