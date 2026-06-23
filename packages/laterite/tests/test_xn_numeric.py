"""`read(..., xn="numeric")` — a read-side Float64 view of AGS `XN`-typed columns.

AGS `XN` ("numeric, may carry a non-numeric qualifier") is parsed byte-faithfully
as text by default. `xn="numeric"` casts every XN-typed column to Float64 across
the whole handle (`ags[code]` / `sql` / `at`); non-numeric tokens (NP, <5, >100,
blank) become null via DuckDB `TRY_CAST`. It is read-side only — write-back stays
byte-faithful from the retained parse. (A fuller bidirectional XN treatment is
future work.)
"""

from __future__ import annotations

import laterite as lat
import polars as pl
import pytest

# FRAC_FI / FRAC_IMAX are XN-typed in the dictionary; LOCA_ID is the parent KEY.
AGS = (
    '"GROUP","FRAC"\n'
    '"HEADING","LOCA_ID","FRAC_FI","FRAC_IMAX"\n'
    '"UNIT","","",""\n'
    '"TYPE","ID","XN","XN"\n'
    '"DATA","BH01","12.5","3.0"\n'
    '"DATA","BH02","NP","7.25"\n'
    '"DATA","BH03","<0.1","NP"\n'
)


def test_default_is_byte_faithful_string():
    frac = lat.read(text=AGS)["FRAC"]
    assert frac["FRAC_FI"].dtype == pl.String
    assert frac["FRAC_FI"].to_list() == ["12.5", "NP", "<0.1"]


def test_numeric_casts_xn_columns_to_float_with_null_qualifiers():
    frac = lat.read(text=AGS, xn="numeric")["FRAC"]
    assert frac["FRAC_FI"].dtype == pl.Float64
    assert frac["FRAC_IMAX"].dtype == pl.Float64
    # numeric → float; non-numeric qualifier (NP) and censored (<0.1) → null.
    assert frac["FRAC_FI"].to_list() == [12.5, None, None]
    assert frac["FRAC_IMAX"].to_list() == [3.0, 7.25, None]


def test_numeric_view_is_consistent_across_sql_and_at():
    ags = lat.read(text=AGS, xn="numeric")
    # the engine table is numeric, so SQL aggregates work without per-call casts.
    avg = ags.sql('SELECT AVG("FRAC_FI") AS a FROM "FRAC"').fetchone()[0]
    assert avg == pytest.approx(12.5)  # only BH01 is numeric
    # at() rides the same engine table → same numeric typing.
    sub = ags.at("LOCA", ["BH02"])["FRAC"]
    assert sub["FRAC_IMAX"].dtype == pl.Float64
    assert sub["FRAC_IMAX"].to_list() == [7.25]


def test_write_back_stays_byte_faithful_under_numeric():
    # `save`/`text`/`bytes` come from the retained parse, not the engine, so the
    # numeric view must not perturb round-trip fidelity.
    assert lat.read(text=AGS, xn="numeric").text == lat.read(text=AGS).text


def test_invalid_xn_mode_rejected():
    with pytest.raises(ValueError, match="xn must be one of"):
        lat.read(text=AGS, xn="float")


def test_source_alias_accepts_xn():
    # the `source` alias is the same callable, so it takes xn= too.
    frac = lat.source(text=AGS, xn="numeric")["FRAC"]
    assert frac["FRAC_FI"].dtype == pl.Float64
