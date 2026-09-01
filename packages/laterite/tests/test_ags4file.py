"""Coverage for the `laterite.Ags4File` accessor surface.

`Ags4File` (the result of `laterite.read`) was entirely untested — the
highest-severity coverage gap per docs/test-suite-review.md (11 public
accessors with zero references). These tests exercise every accessor on
a real fixture plus a structure-preserving text round-trip property.
"""

from __future__ import annotations

from pathlib import Path

import duckdb
import laterite
import polars as pl
import pytest
from hypothesis import HealthCheck, given, settings
from hypothesis import strategies as st
from laterite import _laterite_native as _native

# Reuse the hand-authored clean fixture the rest of the suite uses.
_FIX = (
    Path(__file__).resolve().parents[3]
    / "rust-packages"
    / "laterite-ags4-validator"
    / "tests"
    / "fixtures"
)
_CLEAN = _FIX / "clean_minimal.ags"

# An inline AGS4 file with a born-typed numeric (2DP) column plus a non-numeric
# (ID) column — exercises the born-typed / backend / engine tests below.
_NUMERIC_SRC = (
    '"GROUP","LOCA"\r\n'
    '"HEADING","LOCA_ID","LOCA_FDEP"\r\n'
    '"UNIT","","m"\r\n'
    '"TYPE","ID","2DP"\r\n'
    '"DATA","BH1","10.50"\r\n'
    '"DATA","BH2","oops"\r\n'  # non-numeric cell → null under coercion
    '"DATA","BH3","3.25"\r\n'
)

# LOCA + SAMP share LOCA_ID; PROJ has no LOCA_ID, so at() passes it through.
_RELATED_SRC = "\r\n".join(
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


@pytest.fixture(scope="module")
def clean() -> laterite.Ags4File:
    return laterite.read(str(_CLEAN))


# --- constructor guard (#112) ---------------------------------------------


def test_ags4file_rejects_a_path_with_actionable_typeerror():
    # The #112 footgun: `Ags4File(path)` used to construct silently and fail three
    # calls later with a cryptic `'PosixPath' object is not subscriptable`. It must
    # now fail early, naming `laterite.read()` as the fix.
    with pytest.raises(TypeError, match=r"laterite\.read"):
        laterite.Ags4File(_CLEAN)
    with pytest.raises(TypeError, match=r"laterite\.read"):
        laterite.Ags4File("site.ags")
    # A mapping missing the load-bearing keys is also rejected (not silently stored).
    with pytest.raises(TypeError, match=r"laterite\.read"):
        laterite.Ags4File({"not": "parsed"})


def test_ags4file_accepts_engine_parsed_mapping(clean):
    # The legitimate construction path (via read()) still produces a usable handle.
    assert isinstance(clean, laterite.Ags4File)
    assert clean.groups  # the mapping passed the guard and is queryable


# --- groups / membership --------------------------------------------------


def test_groups_lists_file_order(clean):
    assert clean.groups == ["PROJ", "TRAN", "UNIT", "TYPE"]


def test_contains_true_and_false(clean):
    assert "PROJ" in clean
    assert "TRAN" in clean
    assert "NOPE" not in clean


def test_tran_ags_value(clean):
    assert clean.tran_ags == "4.2"


def test_tran_ags_none_when_no_tran_group():
    """A file with no TRAN group reports tran_ags=None (not a raise)."""
    src = (
        '"GROUP","PROJ"\r\n'
        '"HEADING","PROJ_ID"\r\n'
        '"UNIT",""\r\n'
        '"TYPE","ID"\r\n'
        '"DATA","P1"\r\n'
    )
    f = laterite.read(text=src)
    assert f.tran_ags is None


# --- per-group metadata accessors -----------------------------------------


def test_headings_units_types_lengths_and_content(clean):
    headings = clean.headings("PROJ")
    units = clean.units("PROJ")
    types = clean.types("PROJ")
    assert headings == ["PROJ_ID", "PROJ_NAME"]
    assert units == ["", ""]
    assert types == ["ID", "X"]
    # All three accessors agree in length (one entry per heading).
    assert len(headings) == len(units) == len(types)


def test_line_numbers_are_ints_and_match_row_count(clean):
    lines = clean.line_numbers("TYPE")
    # TYPE group has 3 DATA rows (ID, X, DT) in the fixture.
    assert len(lines) == 3
    assert all(isinstance(n, int) for n in lines)
    # File-order: strictly ascending line numbers.
    assert lines == sorted(lines)


# --- table / __getitem__ ---------------------------------------------------


def test_getitem_and_table_are_equivalent(clean):
    via_item = clean["PROJ"]
    via_table = clean.table("PROJ")
    assert via_item.equals(via_table)
    assert clean["PROJ"].columns == ["PROJ_ID", "PROJ_NAME"]


def test_getitem_frame_holds_data_rows(clean):
    df = clean["PROJ"]
    assert df["PROJ_ID"].to_list() == ["P1"]


def test_getitem_is_born_typed():
    """read()[group] is typed straight from the file's TYPE row (loaded into the
    DuckDB engine) — a 2DP heading is Float64, 0DP Int64, ID String, and a
    non-conforming numeric cell is null. No to_numeric needed."""
    src = (
        '"GROUP","LOCA"\r\n'
        '"HEADING","LOCA_ID","LOCA_FDEP","LOCA_NUM"\r\n'
        '"UNIT","","m",""\r\n'
        '"TYPE","ID","2DP","0DP"\r\n'
        '"DATA","BH1","10.50","7"\r\n'
        '"DATA","BH2","bad","9"\r\n'
    )
    df = laterite.read(text=src)["LOCA"]
    assert df.schema["LOCA_ID"] == pl.String
    assert df.schema["LOCA_FDEP"] == pl.Float64
    assert df.schema["LOCA_NUM"] == pl.Int64
    assert df["LOCA_FDEP"].to_list() == [10.5, None]  # dirty cell -> null
    assert df["LOCA_NUM"].to_list() == [7, 9]


# --- backend selection + the in-memory DuckDB engine ----------------------
#
# read() -> Ags4File; ags[code] is a polars DataFrame by default (or pandas
# with backend="pandas" — both pyarrow-free). Each group loads into the
# in-memory DuckDB engine on first touch; ags.sql() returns a DuckDB relation
# (the filter-pushdown path).


def test_read_returns_polars_by_default():
    df = laterite.read(text=_NUMERIC_SRC)["LOCA"]
    assert isinstance(df, pl.DataFrame)


def test_read_backend_pandas():
    pd = pytest.importorskip("pandas")
    f = laterite.read(text=_NUMERIC_SRC, backend="pandas")
    assert f.backend == "pandas"
    df = f["LOCA"]
    assert isinstance(df, pd.DataFrame)
    # Born-typed + pyarrow-free (DuckDB's NumPy .df()): bad cell -> NaN.
    vals = df["LOCA_FDEP"].tolist()
    assert vals[0] == 10.5
    assert pd.isna(vals[1])


def test_read_rejects_unknown_backend():
    with pytest.raises(ValueError, match="backend must be"):
        laterite.read(text=_NUMERIC_SRC, backend="arrow")


def test_frame_reads_bypass_engine_keyed_path_spins_it_up():
    # The engine is lazy AND the default frame path is engine-free: read() spins
    # up nothing, and a plain keys-stripped frame read goes Arrow -> frame
    # directly (the fast path), never touching DuckDB — the content-addressed
    # keys it would strip anyway are simply not built. Only the keyed/relational
    # path (keys=True, .sql(), .connection, .at()) loads groups as native DuckDB
    # tables. A fresh handle (not the shared module-scoped `clean`, whose engine
    # other tests populate).
    f = laterite.read(str(_CLEAN))
    assert f._con is None  # read() did not create the engine
    _ = f["PROJ"]  # fast frame path...
    assert f._con is None  # ...touches no engine
    _ = f.table("PROJ", keys=True)  # a keyed frame needs the relational engine
    assert f._con is not None and "PROJ" in f._registered
    _ = f.sql('SELECT * FROM "TRAN"')  # .sql() registers the rest
    assert "TRAN" in f._registered


def test_sql_returns_a_duckdb_relation_with_pushdown():
    f = laterite.read(text=_NUMERIC_SRC)
    rel = f.sql("SELECT * FROM LOCA WHERE LOCA_FDEP > 5")
    assert "duckdb" in type(rel).__module__.lower()
    # The WHERE pushed into the engine — only the matching row materialises.
    assert pl.DataFrame(rel)["LOCA_FDEP"].to_list() == [10.5]


def test_sql_one_liner_survives_unbound_handle():
    # No __del__ close: the relation keeps the connection alive, so a one-liner
    # where the handle is never bound to a variable still materialises.
    df = pl.DataFrame(laterite.read(text=_NUMERIC_SRC).sql("SELECT * FROM LOCA"))
    assert len(df) == 3


def test_connection_exposes_raw_duckdb_seeded_with_groups():
    f = laterite.read(text=_NUMERIC_SRC)
    con = f.connection
    assert con.sql("SELECT COUNT(*) FROM LOCA").fetchone()[0] == 3


def test_context_manager_closes_engine():
    with laterite.read(text=_NUMERIC_SRC) as f:
        _ = (
            f.connection
        )  # spin up the relational engine (a plain frame read no longer does)
        assert f._con is not None
    assert f._con is None  # __exit__ closed it


def test_frame_read_does_not_leak_unkeyed_table_into_sql():
    # GUARDRAIL for the fast frame path: reading a group as a frame (which skips
    # the content-addressed keys the frame strips anyway) must NOT register an
    # un-keyed table into the shared SQL connection — else a later cross-group
    # JOIN on _parent_id/_id would silently return wrong/empty rows. The fast path
    # goes Arrow -> frame OFF the engine, so the engine only ever holds keyed
    # tables. Read the frames FIRST, then assert the join still resolves.
    f = laterite.read(text=_RELATED_SRC)
    _ = f["LOCA"]  # fast frame reads first...
    _ = f["SAMP"]
    rel = f.sql("SELECT s.SAMP_REF FROM SAMP s JOIN LOCA l ON s._parent_id = l._id")
    assert rel.df().shape[0] > 0  # the join still resolves via correct keys


# --- at(): location-subset view -------------------------------------------


def test_at_filters_to_the_requested_subset():
    sub = laterite.read(text=_RELATED_SRC).at("LOCA", ["BH01", "BH03"])
    assert sub["LOCA"]["LOCA_ID"].to_list() == ["BH01", "BH03"]
    # Related rows across groups: BH01 has S1+S2, BH03 has S4 (BH02's S3 excluded).
    assert sub["SAMP"]["SAMP_REF"].to_list() == ["S1", "S2", "S4"]


def test_at_groups_lists_related_groups_only():
    sub = laterite.read(text=_RELATED_SRC).at("LOCA", ["BH01"])
    # PROJ has no LOCA_ID, so it isn't "related".
    assert sub.groups == ["LOCA", "SAMP"]


def test_at_passes_through_groups_without_the_key():
    sub = laterite.read(text=_RELATED_SRC).at("LOCA", ["BH01"])
    assert sub["PROJ"]["PROJ_ID"].to_list() == ["P1"]  # unfiltered


def test_at_empty_values_selects_nothing():
    sub = laterite.read(text=_RELATED_SRC).at("LOCA", [])
    assert sub["SAMP"].height == 0


def test_at_honours_pandas_backend():
    pd = pytest.importorskip("pandas")
    sub = laterite.read(text=_RELATED_SRC, backend="pandas").at("LOCA", ["BH02"])
    df = sub["SAMP"]
    assert isinstance(df, pd.DataFrame)
    assert df["SAMP_REF"].tolist() == ["S3"]


def test_at_absent_group_raises_keyerror():
    sub = laterite.read(text=_RELATED_SRC).at("LOCA", ["BH01"])
    with pytest.raises(KeyError, match="not in file"):
        _ = sub["NOPE"]


def test_at_chaining_accumulates_filters():
    # Chained .at() filters AND together: BH01,BH02 then BH01 -> only BH01.
    sub = (
        laterite.read(text=_RELATED_SRC)
        .at("LOCA", ["BH01", "BH02"])
        .at("LOCA", ["BH01"])
    )
    assert sub["SAMP"]["SAMP_REF"].to_list() == ["S1", "S2"]  # BH01's samples only


def test_at_frames_pulls_all_related_groups():
    frames = laterite.read(text=_RELATED_SRC).at("LOCA", ["BH01"]).frames()
    assert set(frames) == {"LOCA", "SAMP"}  # related only (PROJ excluded)
    assert frames["LOCA"]["LOCA_ID"].to_list() == ["BH01"]
    assert frames["SAMP"]["SAMP_REF"].to_list() == ["S1", "S2"]


# --- _g KeyError on absent group ------------------------------------------


@pytest.mark.parametrize(
    "accessor",
    ["headings", "units", "types", "line_numbers"],
)
def test_accessors_raise_keyerror_on_absent_group(clean, accessor):
    with pytest.raises(KeyError, match="not in file"):
        getattr(clean, accessor)("NOPE")


def test_getitem_raises_keyerror_on_absent_group(clean):
    with pytest.raises(KeyError, match="not in file"):
        _ = clean["NOPE"]


# --- save / text round-trip -----------------------------------------------


def test_save_returns_path_and_reads_back(clean, tmp_path):
    out = tmp_path / "rt.ags"
    returned = clean.save(out)
    assert returned == out
    assert out.exists()
    f2 = laterite.read(str(out))
    assert set(f2.groups) == set(clean.groups)


# --- to_duckdb: persist the keyed relational store ------------------------


def test_to_duckdb_writes_keyed_tables_that_still_join(tmp_path):
    out = tmp_path / "store.duckdb"
    stats = laterite.read(text=_RELATED_SRC).to_duckdb(out)
    assert stats == {"path": out, "tables_written": 3, "rows_written": 1 + 3 + 4}
    assert out.exists()

    con = duckdb.connect(str(out))
    try:
        assert {r[0] for r in con.execute("SHOW TABLES").fetchall()} == {
            "PROJ",
            "LOCA",
            "SAMP",
        }
        # A KNOWN group's table leads with the two content-addressed key columns...
        cols = [r[0] for r in con.execute('DESCRIBE "SAMP"').fetchall()]
        assert cols[:2] == ["_id", "_parent_id"]
        # ...and the persisted keys still resolve a cross-group JOIN — the whole
        # point of persisting them (and what the read_ags extension diffs on).
        joined = con.execute(
            "SELECT s.SAMP_REF FROM SAMP s JOIN LOCA l ON s._parent_id = l._id "
            "ORDER BY s.SAMP_REF"
        ).fetchall()
        assert [r[0] for r in joined] == ["S1", "S2", "S3", "S4"]
    finally:
        con.close()


def test_to_duckdb_is_faithful_to_the_in_memory_relational_layer(tmp_path):
    out = tmp_path / "faithful.duckdb"
    laterite.read(text=_RELATED_SRC).to_duckdb(out)
    persisted = duckdb.connect(str(out))
    mem = laterite.read(text=_RELATED_SRC).connection
    try:
        for code in ("PROJ", "LOCA", "SAMP"):
            want = mem.execute(f'SELECT * FROM "{code}" ORDER BY ALL').fetchall()
            got = persisted.execute(f'SELECT * FROM "{code}" ORDER BY ALL').fetchall()
            assert got == want, code
    finally:
        persisted.close()


def test_to_duckdb_refuses_to_overwrite(tmp_path):
    out = tmp_path / "once.duckdb"
    laterite.read(text=_RELATED_SRC).to_duckdb(out)
    with pytest.raises(FileExistsError, match="fresh database"):
        laterite.read(text=_RELATED_SRC).to_duckdb(out)


def test_to_duckdb_groups_selects_a_subset(tmp_path):
    out = tmp_path / "subset.duckdb"
    stats = laterite.read(text=_RELATED_SRC).to_duckdb(out, groups=["SAMP", "PROJ"])
    assert stats["tables_written"] == 2
    con = duckdb.connect(str(out))
    try:
        assert {r[0] for r in con.execute("SHOW TABLES").fetchall()} == {"SAMP", "PROJ"}
    finally:
        con.close()


def test_to_duckdb_unknown_group_raises_keyerror(tmp_path):
    with pytest.raises(KeyError, match="not in file"):
        laterite.read(text=_RELATED_SRC).to_duckdb(
            tmp_path / "x.duckdb", groups=["NOPE"]
        )


def test_free_to_duckdb_matches_the_fluent_method(tmp_path):
    a = tmp_path / "free.duckdb"
    b = tmp_path / "fluent.duckdb"
    laterite.to_duckdb(text=_RELATED_SRC, output=a)
    laterite.read(text=_RELATED_SRC).to_duckdb(b)
    ca, cb = duckdb.connect(str(a)), duckdb.connect(str(b))
    try:
        for code in ("PROJ", "LOCA", "SAMP"):
            assert (
                ca.execute(f'SELECT * FROM "{code}" ORDER BY ALL').fetchall()
                == cb.execute(f'SELECT * FROM "{code}" ORDER BY ALL').fetchall()
            ), code
    finally:
        ca.close()
        cb.close()


def test_free_to_duckdb_accepts_an_already_read_handle(tmp_path):
    # The positional-source branch: pass an already-read Ags4File and the
    # functional form delegates to its method (it does not own/close it).
    out = tmp_path / "handle.duckdb"
    handle = laterite.read(text=_RELATED_SRC)
    try:
        stats = laterite.to_duckdb(handle, out)
        assert stats["tables_written"] == 3
        assert "SAMP" in handle  # not closed out from under the caller
    finally:
        handle.close()


def test_free_to_duckdb_requires_an_output_path():
    with pytest.raises(TypeError, match="output path"):
        laterite.to_duckdb(text=_RELATED_SRC)


# --- structure-preserving round-trip property -----------------------------
#
# .text must reconstruct a file whose group set and per-group heading lists
# survive a re-read. Byte-equality may NOT hold (every field is re-quoted,
# CRLF is normalised), so we assert *structural* equality.


def _ags_ident() -> st.SearchStrategy[str]:
    """A safe-ish AGS4 heading token: uppercase letter + alnum/underscore.
    Kept conservative so generated headings don't trip Rule-19a parse
    behaviour that would drop/rename a column on re-read."""
    first = st.sampled_from("ABCDEFGHIJKLMNOPQRSTUVWXYZ")
    rest = st.text(
        alphabet="ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_",
        min_size=0,
        max_size=6,
    )
    return st.builds(lambda a, b: a + b, first, rest)


@settings(max_examples=60, suppress_health_check=[HealthCheck.too_slow])
@given(
    # A 4-letter group code + 1..4 distinct headings + 1..3 data rows of
    # plain (quote-free, CR/LF-free) text values.
    group=st.text(alphabet="ABCDEFGHIJKLMNOPQRSTUVWXYZ", min_size=4, max_size=4),
    headings=st.lists(_ags_ident(), min_size=1, max_size=4, unique=True),
    rows=st.lists(
        st.lists(
            st.text(
                alphabet=st.characters(
                    min_codepoint=32,
                    max_codepoint=126,
                    blacklist_characters='"',
                ),
                min_size=0,
                max_size=8,
            ),
            min_size=1,
            max_size=4,
        ),
        min_size=1,
        max_size=3,
    ),
)
def test_text_round_trip_preserves_structure(group, headings, rows):
    # Build a minimal valid-shaped AGS4 file: GROUP / HEADING / UNIT / TYPE
    # then DATA rows, every field padded/truncated to the heading width.
    n = len(headings)
    src = (
        f'"GROUP","{group}"\r\n'
        + '"HEADING",'
        + ",".join(f'"{h}"' for h in headings)
        + "\r\n"
        + '"UNIT",'
        + ",".join('""' for _ in headings)
        + "\r\n"
        + '"TYPE",'
        + ",".join('"X"' for _ in headings)
        + "\r\n"
        + "".join(
            '"DATA",' + ",".join(f'"{c}"' for c in (list(r) + [""] * n)[:n]) + "\r\n"
            for r in rows
        )
    )

    f = laterite.read(text=src)
    # Parser may legitimately reject a generated group/heading; only the
    # files it accepts are in-scope for the round-trip invariant.
    if group not in f:
        return
    text = f.text
    f2 = laterite.read(text=text)

    # Group set is preserved.
    assert set(f.groups) == set(f2.groups)
    # Per-group headings are preserved.
    for g in f.groups:
        assert f.headings(g) == f2.headings(g)


# --- flagship byte-fidelity round-trip (phase-1 Arrow engine) -------------
#
# read -> write -> read must preserve every DATA cell value, INCLUDING dirty
# numerics the typed columns cannot hold: detection limits ("<0.01"),
# non-canonical decimals ("1.1" in a 2DP column), junk ("n/a"). The typed
# Arrow frame nulls those; fidelity rides on the raw-override path (#13). We
# assert at the raw-string layer (parse_primitives keeps cells verbatim) so
# the property is representation-independent — it passes on today's raw-rows
# write AND guards the typed write-back (task 114) the same way.

# Values where format(parse(v)) != v for a numeric column — the override path.
_DIRTY_NUMERIC = (
    "<0.01",
    ">100",
    "1.1",
    "5",
    "10.500",
    "-0.00",
    "1e3",
    "n/a",
    "",
    "NaN",
)
_TYPE_CHOICES = ("ID", "X", "2DP", "3DP", "0DP")


def _cell_for(ags_type: str) -> st.SearchStrategy[str]:
    """A DATA cell for a column of this TYPE: numeric columns draw a mix of
    conforming and dirty values (so the override path is exercised); text
    columns draw printable ASCII tokens WIDENED with the Unicode dimension the
    ASCII-only alphabet missed — accents, symbols, CJK, emoji — each of which must
    survive read→write→read byte-identically. `"` and CR/LF stay out: they
    re-quote / re-split and are pinned by the Rust writer round-trip, not this
    read-side value-preservation test."""
    if ags_type in ("2DP", "3DP", "0DP"):
        return st.one_of(
            st.sampled_from(_DIRTY_NUMERIC),
            st.integers(-9999, 9999).map(str),
            st.floats(
                allow_nan=False, allow_infinity=False, min_value=-1e6, max_value=1e6
            ).map(lambda x: f"{x:.2f}"),
        )
    return st.text(
        alphabet=st.one_of(
            st.characters(
                min_codepoint=33, max_codepoint=126, blacklist_characters='"'
            ),
            st.sampled_from("éàüßÇµ°±§日本語漢字→★🌍🚀"),
        ),
        min_size=0,
        max_size=8,
    )


@settings(max_examples=80, suppress_health_check=[HealthCheck.too_slow])
@given(
    group=st.text(alphabet="ABCDEFGHIJKLMNOPQRSTUVWXYZ", min_size=4, max_size=4),
    headings=st.lists(_ags_ident(), min_size=1, max_size=4, unique=True),
    type_choices=st.lists(st.sampled_from(_TYPE_CHOICES), min_size=4, max_size=4),
    data=st.data(),
)
def test_read_write_read_preserves_data_values(group, headings, type_choices, data):
    n = len(headings)
    types = type_choices[:n]
    rows = data.draw(
        st.lists(st.tuples(*[_cell_for(t) for t in types]), min_size=1, max_size=4)
    )
    src = (
        f'"GROUP","{group}"\r\n'
        + '"HEADING",'
        + ",".join(f'"{h}"' for h in headings)
        + "\r\n"
        + '"UNIT",'
        + ",".join('""' for _ in headings)
        + "\r\n"
        + '"TYPE",'
        + ",".join(f'"{t}"' for t in types)
        + "\r\n"
        + "".join('"DATA",' + ",".join(f'"{c}"' for c in row) + "\r\n" for row in rows)
    )
    # Ground truth: the raw-string view of the source.
    s1 = _native.parse_primitives(text=src)
    if not s1.get("ok") or group not in s1.get("groups", {}):
        return  # parser legitimately rejected the generated file — out of scope
    # read -> write -> re-parse, then compare raw cell values + types.
    f = laterite.read(text=src)
    s2 = _native.parse_primitives(text=f.text)
    assert s1["group_order"] == s2["group_order"]
    for code in s1["group_order"]:
        g1, g2 = s1["groups"][code], s2["groups"][code]
        assert g1["headings"] == g2["headings"], code
        assert g1["types"] == g2["types"], code
        v1 = [r["values"] for r in g1["rows"]]
        v2 = [r["values"] for r in g2["rows"]]
        assert v1 == v2, (code, v1, v2)
