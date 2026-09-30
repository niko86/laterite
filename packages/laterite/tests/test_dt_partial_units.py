"""`DT` cells under partial-precision UNITs (#999).

A `DT` heading may declare a year (`yyyy`), a month (`yyyy-mm`) or a time of
day (`hh:mm`, `hh:mm:ss`). The validator accepts those values, but the typed
read turned every one of them into a null. The decided mapping: a year or month
reads as the start of the period it names, still a `Datetime`; a time of day
reads as a `Time`, so no date nobody recorded is invented."""

from __future__ import annotations

import datetime as dt

import laterite
import polars as pl
import pytest

_HEADINGS = ["PROJ_ID", "PROJ_YEAR", "PROJ_MNTH", "PROJ_TMIN", "PROJ_TSEC"]
_UNITS = ["", "yyyy", "yyyy-mm", "hh:mm", "hh:mm:ss"]
_VALUES = ["P1", "2026", "2026-03", "09:15", "09:15:30"]


def _quoted(cells: list[str]) -> str:
    return ",".join(f'"{c}"' for c in cells)


def _file(values: list[str]) -> str:
    return "\r\n".join(
        [
            '"GROUP","PROJ"',
            f'"HEADING",{_quoted(_HEADINGS)}',
            f'"UNIT",{_quoted(_UNITS)}',
            f'"TYPE",{_quoted(["ID", "DT", "DT", "DT", "DT"])}',
            f'"DATA",{_quoted(values)}',
            "",
        ]
    )


def _proj() -> pl.DataFrame:
    return laterite.read(text=_file(_VALUES))["PROJ"]


def test_period_units_read_as_the_start_of_the_period():
    proj = _proj()
    assert proj.schema["PROJ_YEAR"] == pl.Datetime("us")
    assert proj["PROJ_YEAR"][0] == dt.datetime(2026, 1, 1)
    assert proj["PROJ_MNTH"][0] == dt.datetime(2026, 3, 1)


def test_time_of_day_units_read_as_a_time():
    proj = _proj()
    assert proj.schema["PROJ_TMIN"] == pl.Time
    assert proj["PROJ_TMIN"][0] == dt.time(9, 15)
    assert proj["PROJ_TSEC"][0] == dt.time(9, 15, 30)


@pytest.mark.parametrize(
    ("column", "value"),
    [
        ("PROJ_TMIN", "2026-03"),  # a month under a time-of-day UNIT
        ("PROJ_MNTH", "2026-03-02"),  # a real day under a month UNIT
        ("PROJ_YEAR", "2026-03"),  # a month under a year UNIT
    ],
)
def test_a_value_that_does_not_fit_its_unit_still_reads_as_null(column, value):
    values = list(_VALUES)
    values[_HEADINGS.index(column)] = value
    assert laterite.read(text=_file(values))["PROJ"][column][0] is None


def test_read_then_build_writes_each_value_back_in_its_units_form():
    units = {"PROJ": dict(zip(_HEADINGS[1:], _UNITS[1:], strict=True))}
    types = {"PROJ": dict.fromkeys(_HEADINGS[1:], "DT")}
    out = laterite.build_ags4_unchecked(
        {"PROJ": _proj()}, units=units, types=types
    ).decode()
    assert f'"DATA",{_quoted(_VALUES)}' in out
