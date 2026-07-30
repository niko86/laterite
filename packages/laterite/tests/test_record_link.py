"""`RL` (Record Link) is TEXT, not a number (#503).

An RL cell is a delimited reference — `GROUP|KEY1|KEY2`, split on `TRAN_DLIM`
(AGS Rule 11). `canonical_type("RL")` returned `decimal`, so `sql_type` was
DOUBLE and `parse_value` returned Null: every record link was **silently
destroyed on read**, coming back as an all-null f64 column.

Two unit tests in laterite-ags4-types pinned the wrong answer, which is how it
survived. These pin the right one, at the surface where the damage showed.
"""

import laterite
from laterite.ags_types import canonical_type, parse_value

_SRC = "\r\n".join(
    [
        '"GROUP","PROJ"',
        '"HEADING","PROJ_ID"',
        '"UNIT",""',
        '"TYPE","ID"',
        '"DATA","P1"',
        '"GROUP","SAMP"',
        '"HEADING","LOCA_ID","SAMP_ID","SAMP_LINK"',
        '"UNIT","","",""',
        '"TYPE","ID","ID","RL"',
        '"DATA","BH01","S1","{link}"',
        "",
    ]
)


def _samp(link: str, **kw):
    return laterite.read(text=_SRC.format(link=link), **kw)["SAMP"]


def test_rl_canonicalises_as_text_not_a_number():
    assert canonical_type("RL") == "string"
    # The exact failure: parsing a link as a float yields Null, and Null is what
    # nulled the column and (later) got omitted from _content_hash.
    assert parse_value("SAMP|BH01|1.00", "RL") == "SAMP|BH01|1.00"


def test_a_record_link_survives_a_read_it_used_to_come_back_null():
    """The headline: the column was f64 and every value was null. Assert the real
    link text is there — not merely that the column exists."""
    df = _samp("SAMP|BH01|S1")
    assert df["SAMP_LINK"][0] == "SAMP|BH01|S1"
    assert str(df["SAMP_LINK"].dtype) == "String", "an RL column must not be numeric"


def test_content_hash_distinguishes_rows_that_differ_only_by_record_link():
    """The knock-on. `_content_hash` omits Null cells (blank == absent), so while
    RL parsed to Null, two rows differing ONLY in their link hashed identically —
    a false dedup, i.e. silent row loss."""
    a = _samp("SAMP|BH01|S1", content_hash=True)["_content_hash"][0]
    b = _samp("SAMP|BH01|S2", content_hash=True)["_content_hash"][0]
    assert a != b, "different record links must not collapse into one row"
