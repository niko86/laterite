"""File-declared groups mint `_id`/`_parent_id` from their DICT declarations.

The Rule 18 effective dictionary is the identity authority for a group the
standard registry does not know (#815): its declared KEY tuple mints `_id`,
its declared parent (`DICT_PGRP`) mints `_parent_id`, and the join
`child._parent_id == parent._id` holds across the registry/file boundary —
the same contract every standard group has always had. The validator already
keyed Rule 10a/10c off these declarations; the readers now agree with it.

A group declaring no KEY headings — or not declared at all — stays unkeyed
(NULL ids): an empty key chain would stamp every row identically, which is
worse than none.
"""

from __future__ import annotations

import laterite as lat

# The bespoke-group delivery from the certificate suite (#768's fixture
# shape): XMON declared in DICT with KEY PROJ_ID + XMON_ID under parent PROJ.
BESPOKE = "\r\n".join(
    [
        '"GROUP","PROJ"',
        '"HEADING","PROJ_ID","PROJ_NAME"',
        '"UNIT","",""',
        '"TYPE","ID","X"',
        '"DATA","P1","Effective-dictionary keys fixture"',
        "",
        '"GROUP","DICT"',
        '"HEADING","DICT_TYPE","DICT_GRP","DICT_HDNG","DICT_STAT","DICT_DTYP","DICT_DESC","DICT_UNIT","DICT_PGRP"',
        '"UNIT","","","","","","","",""',
        '"TYPE","X","X","X","X","X","X","X","X"',
        '"DATA","GROUP","XMON","","","","Monitoring bespoke group","","PROJ"',
        '"DATA","HEADING","XMON","PROJ_ID","KEY","ID","Project key","",""',
        '"DATA","HEADING","XMON","XMON_ID","KEY","ID","Monitoring point id","",""',
        '"DATA","HEADING","XMON","XMON_DESC","OTHER","X","Description","",""',
        '"DATA","GROUP","XNOT","","","","Keyless bespoke group","",""',
        '"DATA","HEADING","XNOT","XNOT_TXT","OTHER","X","Free text","",""',
        "",
        '"GROUP","XMON"',
        '"HEADING","PROJ_ID","XMON_ID","XMON_DESC"',
        '"UNIT","","",""',
        '"TYPE","ID","ID","X"',
        '"DATA","P1","M1","Standpipe"',
        '"DATA","P1","M2","Piezometer"',
        "",
        '"GROUP","XNOT"',
        '"HEADING","XNOT_TXT"',
        '"UNIT",""',
        '"TYPE","X"',
        '"DATA","free text"',
        "",
    ]
)


def test_a_declared_group_mints_ids_and_joins_to_its_parent():
    h = lat.read(text=BESPOKE)
    joined = h.sql(
        "SELECT count(*) AS n FROM XMON x JOIN PROJ p ON x._parent_id = p._id"
    ).pl()
    assert joined["n"].to_list() == [2], "both XMON rows join their PROJ parent"
    ids = h.sql("SELECT _id FROM XMON").pl()["_id"].to_list()
    assert all(ids), f"declared KEY tuple must mint non-null ids: {ids}"
    assert len(set(ids)) == 2, "distinct XMON_ID → distinct _id"


def test_ids_are_deterministic_across_reads():
    a = lat.read(text=BESPOKE).sql("SELECT _id FROM XMON").pl()["_id"].to_list()
    b = lat.read(text=BESPOKE).sql("SELECT _id FROM XMON").pl()["_id"].to_list()
    assert a == b


def test_a_keyless_declared_group_stays_unkeyed():
    """No declared KEY headings → the unkeyed batch: the id columns are
    ABSENT (the flagship's unkeyed shape), not NULL-filled."""
    h = lat.read(text=BESPOKE)
    cols = h.sql("SELECT * FROM XNOT").pl().columns
    assert "_id" not in cols and "_parent_id" not in cols, cols
    assert "XNOT_TXT" in cols


def test_standard_group_ids_are_untouched_by_the_file_dict():
    """The file's DICT never re-keys a standard group — PROJ's id here equals
    PROJ's id in a DICT-free file with the same row."""
    plain = "\r\n".join(
        [
            '"GROUP","PROJ"',
            '"HEADING","PROJ_ID","PROJ_NAME"',
            '"UNIT","",""',
            '"TYPE","ID","X"',
            '"DATA","P1","Effective-dictionary keys fixture"',
            "",
        ]
    )
    with_dict = lat.read(text=BESPOKE).sql("SELECT _id FROM PROJ").pl()["_id"].to_list()
    without = lat.read(text=plain).sql("SELECT _id FROM PROJ").pl()["_id"].to_list()
    assert with_dict == without
