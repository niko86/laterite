"""Round-trip tests for the compat numeric coercion pair.

`convert_to_numeric` (string DATA → typed floats, drops UNIT/TYPE) and
`convert_to_text` (floats → AGS4-precision text, recovers UNIT/TYPE from
the dictionary) are inverse-ish: numeric values must survive
`convert_to_text(convert_to_numeric(frame), dictionary=...)` value-equal
to float precision, with non-numeric columns untouched. Kept on the
default (pandas) backend per the compat contract.
"""

from __future__ import annotations

import math

import narwhals.stable.v1 as nw
import polars as pl
import pytest
from hypothesis import HealthCheck, given, settings
from hypothesis import strategies as st
from laterite import compat as AGS4

# LOCA_FDEP is a real 2DP heading in the bundled dictionary, so
# convert_to_text(dictionary=...) can recover its UNIT/TYPE rows.
_DICT = "4.1.1"


def _native(frame):
    return nw.from_native(frame, eager_only=True).to_native()


def _data_rows(frame, col):
    """The DATA-row values of `col` as a plain list (drops UNIT/TYPE)."""
    pf = _native(frame)
    if not isinstance(pf, pl.DataFrame):
        pf = pl.from_pandas(pf)
    return (
        pf.filter(pl.col("HEADING") == "DATA")[col].to_list()
    )


def _make_string_frame(fdep_cells):
    """A LOCA frame: HEADING/UNIT/TYPE + DATA rows. LOCA_FDEP is 2DP,
    LOCA_ID is an untyped string column that must pass through."""
    n = len(fdep_cells)
    ids = [f"BH{i}" for i in range(n)]
    return pl.DataFrame({
        "HEADING": ["UNIT", "TYPE"] + ["DATA"] * n,
        "LOCA_ID": ["", "ID"] + ids,
        "LOCA_FDEP": ["m", "2DP"] + list(fdep_cells),
    })


def _round_trip(frame):
    numeric = AGS4.convert_to_numeric(frame)
    return AGS4.convert_to_text(numeric, dictionary=_DICT)


# --- example-based edge cases ---------------------------------------------

def test_round_trip_preserves_numeric_values_and_string_column():
    frame = _make_string_frame(["10.50", "3.25", "0.00"])
    out = _round_trip(frame)
    # Numeric column reformatted to 2DP text — value-equal as floats.
    assert _data_rows(out, "LOCA_FDEP") == ["10.50", "3.25", "0.00"]
    # Non-numeric column untouched.
    assert _data_rows(out, "LOCA_ID") == ["BH0", "BH1", "BH2"]


def test_round_trip_all_null_numeric_column():
    """An all-empty 2DP column survives as empty cells (no spurious 0.00)."""
    frame = _make_string_frame(["", "", ""])
    out = _round_trip(frame)
    assert _data_rows(out, "LOCA_FDEP") == ["", "", ""]
    assert _data_rows(out, "LOCA_ID") == ["BH0", "BH1", "BH2"]


def test_round_trip_mixed_numeric_and_unparseable():
    """Mixed numeric / non-numeric cells: numbers reformat, junk → empty
    (convert_to_numeric coerces bad cells to null → empty text)."""
    frame = _make_string_frame(["10.50", "oops", "3.25"])
    out = _round_trip(frame)
    assert _data_rows(out, "LOCA_FDEP") == ["10.50", "", "3.25"]
    assert _data_rows(out, "LOCA_ID") == ["BH0", "BH1", "BH2"]


def test_convert_to_numeric_no_type_row_is_noop():
    """With no TYPE row, no column is treated as numeric — values stay as
    strings and convert_to_text(dictionary=...) recovers the TYPE so the
    round-trip still closes."""
    frame = pl.DataFrame({
        "HEADING": ["DATA", "DATA"],
        "LOCA_ID": ["BH1", "BH2"],
        "LOCA_FDEP": ["10.50", "3.25"],
    })
    numeric = AGS4.convert_to_numeric(frame)
    # Untouched (no UNIT/TYPE to drop, nothing flagged numeric).
    assert _data_rows(numeric, "LOCA_FDEP") == ["10.50", "3.25"]
    # And the dictionary still lets us text-ify it.
    out = AGS4.convert_to_text(numeric, dictionary=_DICT)
    assert _data_rows(out, "LOCA_FDEP") == ["10.50", "3.25"]


def test_convert_to_text_without_dict_after_numeric_raises():
    """convert_to_numeric drops UNIT/TYPE; convert_to_text without a
    dictionary then has no TYPE row and must raise (not silently pass)."""
    numeric = AGS4.convert_to_numeric(_make_string_frame(["10.50"]))
    with pytest.raises(AGS4.AGS4Error, match="UNIT and/or TYPE"):
        AGS4.convert_to_text(numeric)


# --- property: numeric values survive the round-trip to 2DP precision -----

@settings(max_examples=80, suppress_health_check=[HealthCheck.too_slow])
@given(
    values=st.lists(
        st.floats(
            min_value=-1e6, max_value=1e6,
            allow_nan=False, allow_infinity=False,
        ),
        min_size=1, max_size=6,
    ),
)
def test_round_trip_numeric_values_equal_to_2dp_precision(values):
    # Feed pre-rendered 2DP strings so the input is already at the
    # column's declared precision — the round-trip must then be a fixed
    # point (idempotent) to 2DP.
    cells = [f"{v:.2f}" for v in values]
    frame = _make_string_frame(cells)
    out = _round_trip(frame)
    got = _data_rows(out, "LOCA_FDEP")
    assert len(got) == len(cells)
    for original, rendered in zip(cells, got, strict=True):
        assert math.isclose(float(original), float(rendered), abs_tol=1e-9)
