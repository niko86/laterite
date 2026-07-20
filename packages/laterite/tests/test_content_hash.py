"""`read(..., content_hash=True)` — the typed, blank-insensitive `_content_hash`
column (#448).

`_id` fingerprints a row's IDENTITY (its KEY chain); `_content_hash` fingerprints
its VALUE. Two deliveries of borehole BH01 with a corrected level share an `_id`
and differ here — which is exactly the distinction the DuckDB cookbook once got
wrong by claiming `_id` did value-dedup.

Every test below asserts a clause of the contract this feature was specified
against, on REAL output values (hashes, row counts, column sets) — never merely
"it runs".
"""

import duckdb
import laterite
import pytest

# Two deliveries of one project.
#   d1: BH01 (GL 10.00), BH02 (GL 12.00).            NO LOCA_REM column at all.
#   d2: BH01 UNCHANGED but the level is re-emitted "10.0" (formatting only),
#       BH02 REVISED (12.00 -> 12.75), BH03 new.     PLUS a blank LOCA_REM column.
# So d2 differs from d1 in three ways that must NOT count as value changes
# (reformat, new blank column) and one that MUST (BH02's level).
_D1 = "\r\n".join(
    [
        '"GROUP","PROJ"',
        '"HEADING","PROJ_ID","PROJ_NAME"',
        '"UNIT","",""',
        '"TYPE","ID","X"',
        '"DATA","P100","Demo"',
        '"GROUP","LOCA"',
        '"HEADING","LOCA_ID","LOCA_NATE","LOCA_GL"',
        '"UNIT","","m","m"',
        '"TYPE","ID","2DP","2DP"',
        '"DATA","BH01","523400.00","10.00"',
        '"DATA","BH02","523500.00","12.00"',
        "",
    ]
)
_D2 = "\r\n".join(
    [
        '"GROUP","PROJ"',
        '"HEADING","PROJ_ID","PROJ_NAME"',
        '"UNIT","",""',
        '"TYPE","ID","X"',
        '"DATA","P100","Demo"',
        '"GROUP","LOCA"',
        '"HEADING","LOCA_ID","LOCA_NATE","LOCA_GL","LOCA_REM"',
        '"UNIT","","m","m",""',
        '"TYPE","ID","2DP","2DP","X"',
        '"DATA","BH01","523400.00","10.0",""',
        '"DATA","BH02","523500.00","12.75",""',
        '"DATA","BH03","523600.00","9.25",""',
        "",
    ]
)


def _loca(text: str):
    return laterite.read(text=text, content_hash=True, keys=True)["LOCA"]


def _hash_of(frame, loca_id: str) -> str:
    row = frame.filter(frame["LOCA_ID"] == loca_id)
    return row["_content_hash"][0]


def _id_of(frame, loca_id: str) -> str:
    row = frame.filter(frame["LOCA_ID"] == loca_id)
    return row["_id"][0]


def test_value_dedup_collapses_identical_rows_and_keeps_revisions():
    """THE headline claim of #448, in the exact shape the issue specified:
    DISTINCT ON (_content_hash) yields 4 rows where DISTINCT ON (_id) yields 3."""
    con = duckdb.connect()
    con.register("A", _loca(_D1))
    con.register("B", _loca(_D2))
    con.execute(
        """CREATE TABLE U AS
             SELECT LOCA_ID, LOCA_GL, _id, _content_hash FROM A
             UNION ALL
             SELECT LOCA_ID, LOCA_GL, _id, _content_hash FROM B"""
    )
    assert con.sql("SELECT count(*) FROM U").fetchone()[0] == 5
    # Three BOREHOLES across the two deliveries — identity.
    assert con.sql("SELECT count(DISTINCT _id) FROM U").fetchone()[0] == 3
    # Four distinct row VALUES: BH01 (once — the reformat + new blank column did
    # NOT make it a different row), BH02 twice (12.00 and the 12.75 revision),
    # BH03 once.
    assert con.sql("SELECT count(DISTINCT _content_hash) FROM U").fetchone()[0] == 4


def test_a_revised_row_shares_its_id_but_not_its_hash():
    a, b = _loca(_D1), _loca(_D2)
    assert _id_of(a, "BH02") == _id_of(b, "BH02"), "same borehole → same identity"
    assert _hash_of(a, "BH02") != _hash_of(b, "BH02"), "changed level → changed value"


def test_blank_and_absent_columns_hash_alike_so_deliveries_still_dedup():
    """d2 carries a LOCA_REM column d1 doesn't have, blank on BH01. The contract
    says blank ≡ absent — without this, two deliveries whose heading sets differ
    at all (the normal case) could never dedup."""
    assert _hash_of(_loca(_D1), "BH01") == _hash_of(_loca(_D2), "BH01")


def test_formatting_only_reemit_is_not_a_value_change():
    """d2 writes BH01's level as `10.0` where d1 wrote `10.00`. Under a numeric
    TYPE these are the same value — parse_value canonicalises before hashing, so
    a producer reformatting its output does not manufacture a false revision."""
    a, b = _loca(_D1), _loca(_D2)
    assert a.filter(a["LOCA_ID"] == "BH01")["LOCA_GL"][0] == 10.0
    assert b.filter(b["LOCA_ID"] == "BH01")["LOCA_GL"][0] == 10.0
    assert _hash_of(a, "BH01") == _hash_of(b, "BH01")


@pytest.mark.parametrize(
    ("cell", "changed"),
    [('"523400.00"', '"523400.01"'), ('"10.00"', '"10.01"')],
)
def test_any_real_value_change_changes_the_hash(cell, changed):
    """Perturb each non-key cell of BH01 in turn; every one must move the hash.
    (A hash that misses a changed cell would silently drop a correction — the
    exact harm the merge-semantics note warns about.)"""
    base = _hash_of(_loca(_D1), "BH01")
    mutated = _D1.replace(cell, changed, 1)
    assert mutated != _D1, "the fixture must actually contain the cell"
    assert _hash_of(_loca(mutated), "BH01") != base


def test_hashes_are_deterministic_across_independent_reads():
    """No clock, no RNG, no shared state — two reads of the same bytes must agree,
    which is what lets two surfaces (or two machines) dedup without coordinating."""
    assert list(_loca(_D1)["_content_hash"]) == list(_loca(_D1)["_content_hash"])


def test_default_read_is_unchanged_no_hash_column_anywhere():
    """The opt-in must be a true opt-in: without it there is no `_content_hash`,
    in the frame OR in the relational layer. Hashing is a build-time cost and a
    caller who never asks pays nothing."""
    plain = laterite.read(text=_D1)
    assert "_content_hash" not in plain["LOCA"].columns
    # The relational `.sql()` layer carries _id/_parent_id but must NOT have
    # gained a third synthetic column.
    cols = [d[0] for d in plain.sql("SELECT * FROM LOCA").description]
    assert "_content_hash" not in cols
    assert "_id" in cols, "sanity: the always-keyed relational layer is intact"


def test_type_disagreement_across_the_typed_text_boundary_does_not_dedup():
    """The documented SHARP EDGE, asserted rather than hand-waved. The hash is
    computed from one file using THAT file's TYPE row. A delivery that re-declares
    LOCA_GL as free text `X` canonicalises `10.00` as a string, not a number — so
    identical bytes do NOT dedup. `lat merge` is the tool for that case; this test
    exists so the limit can never regress into a silent wrong answer."""
    retyped = _D1.replace('"TYPE","ID","2DP","2DP"', '"TYPE","ID","2DP","X"', 1)
    assert _hash_of(_loca(_D1), "BH01") != _hash_of(_loca(retyped), "BH01")
