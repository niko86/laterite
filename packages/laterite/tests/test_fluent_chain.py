"""PR B — the fluent chain on :class:`Ags4File` + the unified :class:`AgsQuery`.

`read(...)` now chains: `.pipe()` (functional escape hatch), `.save()` (the
file terminal), chainable `.validate() -> self` (outcome on
`.report`), and `.query()`/`.at()` returning a lazy `AgsQuery`. `AgsQuery` carries
two modes — the multi-group fan-out from `.at()` (`q[code]`, `.frames()`, `.groups`,
preserved from the old `_AgsSubset`) and the single-result builder from `.query()`
(`.filter(sql)`, `.select(*cols)`, terminals `.frame()`/`.to_polars()`/`.to_pandas()`/
`.relation()`). `.sql()` is unchanged (still a raw DuckDB relation) for back-compat.
"""

from __future__ import annotations

import laterite as lat
import polars as pl
import pytest

# LOCA carries a 2DP LOCA_GL (so SQL `.filter()` has a numeric to bite on); SAMP
# shares LOCA_ID (fan-out); PROJ has no LOCA_ID (passes through .at()).
AGS = "\r\n".join(
    [
        '"GROUP","PROJ"',
        '"HEADING","PROJ_ID"',
        '"UNIT",""',
        '"TYPE","ID"',
        '"DATA","P1"',
        '"GROUP","LOCA"',
        '"HEADING","LOCA_ID","LOCA_GL"',
        '"UNIT","","m"',
        '"TYPE","ID","2DP"',
        '"DATA","BH01","12.30"',
        '"DATA","BH02","13.40"',
        '"DATA","BH03","9.10"',
        '"GROUP","SAMP"',
        '"HEADING","LOCA_ID","SAMP_REF"',
        '"UNIT","",""',
        '"TYPE","ID","ID"',
        '"DATA","BH01","S1"',
        '"DATA","BH01","S2"',
        '"DATA","BH02","S3"',
        '"DATA","BH03","S4"',
        "",
    ]
)


# --- AgsQuery type + the fan-out (preserved from _AgsSubset) ---------------


def test_at_and_query_return_agsquery():
    f = lat.read(text=AGS)
    assert isinstance(f.at("LOCA", ["BH01"]), lat.AgsQuery)
    assert isinstance(f.query("SELECT * FROM LOCA"), lat.AgsQuery)


def test_fanout_indexing_and_frames_preserved():
    sub = lat.read(text=AGS).at("LOCA", ["BH01"])
    assert sub["SAMP"]["SAMP_REF"].to_list() == ["S1", "S2"]
    assert sub.groups == ["LOCA", "SAMP"]  # PROJ has no LOCA_ID
    assert set(sub.frames()) == {"LOCA", "SAMP"}


# --- single-result builder: query / filter / select / terminals -----------


def test_query_filter_select_frame():
    df = (
        lat.read(text=AGS)
        .query("SELECT * FROM LOCA")
        .filter("LOCA_GL > 12")
        .select("LOCA_ID")
        .frame()
    )
    assert df.columns == ["LOCA_ID"]
    assert sorted(df["LOCA_ID"].to_list()) == ["BH01", "BH02"]  # 9.10 excluded


def test_query_at_narrows_the_base_relation():
    df = lat.read(text=AGS).query("SELECT * FROM SAMP").at("LOCA", ["BH01"]).frame()
    assert df["SAMP_REF"].to_list() == ["S1", "S2"]


def test_relation_is_a_raw_duckdb_relation():
    rel = lat.read(text=AGS).query("SELECT * FROM LOCA").relation()
    assert not isinstance(rel, (pl.DataFrame,))
    assert rel.fetchall() and len(rel.fetchall()) == 3  # lazy duckdb relation


def test_to_polars_forces_polars_even_on_pandas_handle():
    pytest.importorskip("pandas")
    out = lat.read(text=AGS, backend="pandas").query("SELECT * FROM LOCA").to_polars()
    assert isinstance(out, pl.DataFrame)


def test_to_pandas_forces_pandas_even_on_polars_handle():
    pd = pytest.importorskip("pandas")
    out = lat.read(text=AGS).query("SELECT * FROM LOCA").to_pandas()
    assert isinstance(out, pd.DataFrame)


def test_frame_uses_handle_backend():
    pd = pytest.importorskip("pandas")
    assert isinstance(
        lat.read(text=AGS).query("SELECT * FROM LOCA").frame(), pl.DataFrame
    )
    assert isinstance(
        lat.read(text=AGS, backend="pandas").query("SELECT * FROM LOCA").frame(),
        pd.DataFrame,
    )


# --- the modes don't silently blur ----------------------------------------


def test_fanout_accessor_after_single_result_op_raises():
    q = lat.read(text=AGS).at("LOCA", ["BH01"]).filter("LOCA_GL > 1")
    with pytest.raises(TypeError, match="single-result"):
        _ = q["SAMP"]


def test_terminal_without_a_base_raises():
    q = lat.read(text=AGS).at("LOCA", ["BH01"])
    with pytest.raises(TypeError, match="no base"):
        q.frame()


def test_agsquery_is_immutable_builder():
    base = lat.read(text=AGS).query("SELECT * FROM LOCA")
    narrowed = base.filter("LOCA_GL > 12")
    # the original is untouched (chaining returns a new query)
    assert base.frame().height == 3
    assert narrowed.frame().height == 2


# --- pipe / validate / report / save --------------------------------------


def test_pipe_applies_function_and_returns_its_result():
    assert lat.read(text=AGS).pipe(lambda f: sorted(f.groups)) == [
        "LOCA",
        "PROJ",
        "SAMP",
    ]
    # pipe also exists on AgsQuery
    n = (
        lat.read(text=AGS)
        .query("SELECT * FROM LOCA")
        .pipe(lambda q: q.frame().height)
    )
    assert n == 3


def test_validate_is_chainable_and_sets_report():
    f = lat.read(text=AGS)
    assert f.validate() is f  # chainable
    assert isinstance(f.report, lat.Report)
    # same engine/verdict as the free validate() on the same source
    assert f.report.count == lat.validate(text=AGS).count


def test_report_before_validate_raises():
    with pytest.raises(AttributeError, match="validate"):
        _ = lat.read(text=AGS).report


def test_save_round_trips(tmp_path):
    f = lat.read(text=AGS)
    out = tmp_path / "o.ags"
    assert f.save(out) == out
    assert out.exists()
    assert set(lat.read(str(out)).groups) == set(f.groups)


def test_end_to_end_chain_source_validate_query():
    # the headline fluent flow: source -> validate -> query -> filter -> frame
    df = (
        lat.read(text=AGS)
        .validate()
        .query("SELECT * FROM LOCA")
        .filter("LOCA_GL > 12")
        .frame()
    )
    assert df.height == 2
