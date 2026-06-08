"""Coverage for the `laterite.Ags4File` accessor surface.

`Ags4File` (the result of `laterite.read`) was entirely untested — the
highest-severity coverage gap per docs/test-suite-review.md (11 public
accessors with zero references). These tests exercise every accessor on
a real fixture plus a structure-preserving text round-trip property.
"""

from __future__ import annotations

from pathlib import Path

import laterite
import pytest
from hypothesis import HealthCheck, given, settings
from hypothesis import strategies as st

# Reuse the hand-authored clean fixture the rest of the suite uses.
_FIX = (
    Path(__file__).resolve().parents[3]
    / "rust-packages" / "ags4-validator" / "tests" / "fixtures"
)
_CLEAN = _FIX / "clean_minimal.ags"

# An inline AGS4 file carrying a numeric (2DP) column so to_numeric() has
# something to coerce, plus a non-numeric (ID) column it must leave alone.
_NUMERIC_SRC = (
    '"GROUP","LOCA"\r\n'
    '"HEADING","LOCA_ID","LOCA_FDEP"\r\n'
    '"UNIT","","m"\r\n'
    '"TYPE","ID","2DP"\r\n'
    '"DATA","BH1","10.50"\r\n'
    '"DATA","BH2","oops"\r\n'   # non-numeric cell → null under coercion
    '"DATA","BH3","3.25"\r\n'
)


@pytest.fixture(scope="module")
def clean() -> laterite.Ags4File:
    return laterite.read(str(_CLEAN))


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


# --- table / __getitem__ / to_numeric -------------------------------------

def test_getitem_and_table_are_equivalent(clean):
    via_item = clean["PROJ"].to_native()
    via_table = clean.table("PROJ").to_native()
    assert via_item.equals(via_table)
    assert clean["PROJ"].columns == ["PROJ_ID", "PROJ_NAME"]


def test_getitem_frame_holds_data_rows(clean):
    df = clean["PROJ"].to_native()
    assert df["PROJ_ID"].to_list() == ["P1"]


def test_to_numeric_coerces_numeric_columns_leaves_others():
    f = laterite.read(text=_NUMERIC_SRC)
    df = f.to_numeric("LOCA").to_native()
    # 2DP column cast to float; bad cell → null (errors='coerce' parity).
    assert df["LOCA_FDEP"].to_list() == [10.5, None, 3.25]
    # ID column untouched (still strings).
    assert df["LOCA_ID"].to_list() == ["BH1", "BH2", "BH3"]


def test_to_numeric_no_numeric_columns_returns_frame_unchanged(clean):
    """A group with no DP/SF/SCI/MC columns returns the same frame."""
    out = clean.to_numeric("PROJ").to_native()
    assert out.equals(clean["PROJ"].to_native())


# --- _g KeyError on absent group ------------------------------------------

@pytest.mark.parametrize(
    "accessor",
    ["headings", "units", "types", "line_numbers", "to_numeric"],
)
def test_accessors_raise_keyerror_on_absent_group(clean, accessor):
    with pytest.raises(KeyError, match="not in file"):
        getattr(clean, accessor)("NOPE")


def test_getitem_raises_keyerror_on_absent_group(clean):
    with pytest.raises(KeyError, match="not in file"):
        _ = clean["NOPE"]


# --- write / to_ags4_text round-trip --------------------------------------

def test_write_returns_path_and_reads_back(clean, tmp_path):
    out = tmp_path / "rt.ags"
    returned = clean.write(out)
    assert returned == out
    assert out.exists()
    f2 = laterite.read(str(out))
    assert set(f2.groups) == set(clean.groups)


def test_module_write_function_rejects_non_ags4file(tmp_path):
    """laterite.write() guards against non-Ags4File input (TypeError)."""
    with pytest.raises(TypeError, match="Ags4File"):
        laterite.write({"not": "an Ags4File"}, tmp_path / "x.ags")


def test_module_write_function_round_trips(clean, tmp_path):
    out = tmp_path / "via_module.ags"
    laterite.write(clean, out)
    f2 = laterite.read(str(out))
    assert set(f2.groups) == set(clean.groups)


# --- structure-preserving round-trip property -----------------------------
#
# to_ags4_text() must reconstruct a file whose group set and per-group
# heading lists survive a re-read. Byte-equality may NOT hold (every field
# is re-quoted, CRLF is normalised), so we assert *structural* equality.

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
                    min_codepoint=32, max_codepoint=126,
                    blacklist_characters='"',
                ),
                min_size=0, max_size=8,
            ),
            min_size=1, max_size=4,
        ),
        min_size=1, max_size=3,
    ),
)
def test_to_ags4_text_round_trip_preserves_structure(group, headings, rows):
    # Build a minimal valid-shaped AGS4 file: GROUP / HEADING / UNIT / TYPE
    # then DATA rows, every field padded/truncated to the heading width.
    n = len(headings)
    src = (
        f'"GROUP","{group}"\r\n'
        + '"HEADING",' + ",".join(f'"{h}"' for h in headings) + "\r\n"
        + '"UNIT",' + ",".join('""' for _ in headings) + "\r\n"
        + '"TYPE",' + ",".join('"X"' for _ in headings) + "\r\n"
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
    text = f.to_ags4_text()
    f2 = laterite.read(text=text)

    # Group set is preserved.
    assert set(f.groups) == set(f2.groups)
    # Per-group headings are preserved.
    for g in f.groups:
        assert f.headings(g) == f2.headings(g)
