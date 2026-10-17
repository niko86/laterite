"""`.at()` never filters silently-nothing, and can key on row identity (#1093).

`.at(group, values)` filters on `{group}_ID`. A file in which no group carries
that heading used to come back whole — every GEOL row for "the rows at X" — so
it raises instead. And because `{group}_ID` is often optional (SAMP_ID blank),
`by="_id"` keys on the synthetic identity: the group's own rows by `_id`, its
direct children by `_parent_id`.
"""

from __future__ import annotations

import laterite as lat
import pytest

# SAMP_ID left blank (it is optional); LLPL, a DICT-declared XSMP and a
# DICT-declared keyless XNKY hang off SAMP; GEOL carries no GEOL_ID and is a
# child of LOCA, not SAMP.
AGS = "\r\n".join(
    [
        '"GROUP","PROJ"',
        '"HEADING","PROJ_ID"',
        '"UNIT",""',
        '"TYPE","ID"',
        '"DATA","P1"',
        "",
        '"GROUP","DICT"',
        '"HEADING","DICT_TYPE","DICT_GRP","DICT_HDNG","DICT_STAT","DICT_DTYP","DICT_DESC","DICT_UNIT","DICT_PGRP"',
        '"UNIT","","","","","","","",""',
        '"TYPE","X","X","X","X","X","X","X","X"',
        '"DATA","GROUP","XSMP","","","","Bespoke sample child","","SAMP"',
        '"DATA","HEADING","XSMP","LOCA_ID","KEY","ID","","",""',
        '"DATA","HEADING","XSMP","SAMP_TOP","KEY","2DP","","m",""',
        '"DATA","HEADING","XSMP","SAMP_REF","KEY","X","","",""',
        '"DATA","HEADING","XSMP","SAMP_TYPE","KEY","PA","","",""',
        '"DATA","HEADING","XSMP","SAMP_ID","KEY","ID","","",""',
        '"DATA","HEADING","XSMP","XSMP_REF","KEY","X","","",""',
        '"DATA","GROUP","XNKY","","","","Keyless sample child","","SAMP"',
        '"DATA","HEADING","XNKY","XNKY_TXT","OTHER","X","","",""',
        "",
        '"GROUP","LOCA"',
        '"HEADING","LOCA_ID"',
        '"UNIT",""',
        '"TYPE","ID"',
        '"DATA","BH01"',
        '"DATA","BH02"',
        "",
        '"GROUP","GEOL"',
        '"HEADING","LOCA_ID","GEOL_TOP","GEOL_BASE"',
        '"UNIT","","m","m"',
        '"TYPE","ID","2DP","2DP"',
        '"DATA","BH01","0.00","1.00"',
        '"DATA","BH02","0.00","2.00"',
        "",
        '"GROUP","SAMP"',
        '"HEADING","LOCA_ID","SAMP_TOP","SAMP_REF","SAMP_TYPE","SAMP_ID"',
        '"UNIT","","m","","",""',
        '"TYPE","ID","2DP","X","PA","ID"',
        '"DATA","BH01","1.00","1","U",""',
        '"DATA","BH01","2.00","2","U",""',
        '"DATA","BH02","1.00","1","U",""',
        "",
        '"GROUP","LLPL"',
        '"HEADING","LOCA_ID","SAMP_TOP","SAMP_REF","SAMP_TYPE","SAMP_ID","SPEC_REF","SPEC_DPTH","LLPL_LL"',
        '"UNIT","","m","","","","","m","%"',
        '"TYPE","ID","2DP","X","PA","ID","X","2DP","0DP"',
        '"DATA","BH01","1.00","1","U","","A","1.00","45"',
        '"DATA","BH01","1.00","1","U","","B","1.10","46"',
        '"DATA","BH01","2.00","2","U","","A","2.00","47"',
        '"DATA","BH02","1.00","1","U","","A","1.00","48"',
        "",
        '"GROUP","XSMP"',
        '"HEADING","LOCA_ID","SAMP_TOP","SAMP_REF","SAMP_TYPE","SAMP_ID","XSMP_REF"',
        '"UNIT","","m","","","",""',
        '"TYPE","ID","2DP","X","PA","ID","X"',
        '"DATA","BH01","1.00","1","U","","x1"',
        '"DATA","BH02","1.00","1","U","","x2"',
        "",
        '"GROUP","XNKY"',
        '"HEADING","XNKY_TXT"',
        '"UNIT",""',
        '"TYPE","X"',
        '"DATA","no identity"',
        "",
    ]
)


def _samp_id(h: lat.Ags4File, loca: str, top: float) -> str:
    samp = h.table("SAMP", keys=True)
    row = samp.filter((samp["LOCA_ID"] == loca) & (samp["SAMP_TOP"] == top))
    return row["_id"].item()


# --- the default by: {group}_ID -------------------------------------------


def test_at_on_a_heading_no_group_carries_raises():
    # GEOL_ID is in no group of this file: filtering on it would keep everything.
    with pytest.raises(ValueError, match="GEOL_ID"):
        lat.read(text=AGS).at("GEOL", ["anything"])


def test_at_on_a_heading_unknown_to_the_dictionary_raises():
    with pytest.raises(ValueError, match="NOPE_ID"):
        lat.read(text=AGS).at("NOPE", ["x"])


def test_chained_at_on_an_absent_heading_raises():
    with pytest.raises(ValueError, match="GEOL_ID"):
        lat.read(text=AGS).at("LOCA", ["BH01"]).at("GEOL", ["anything"])


def test_at_on_a_heading_some_group_carries_still_filters():
    # SAMP_ID is present (blank), so the filter stands — and matches nothing.
    q = lat.read(text=AGS).at("SAMP", ["S1"])
    assert q["SAMP"].height == 0
    assert q["LLPL"].height == 0


def test_at_a_none_value_warns_toward_by_id():
    # A blank ID reads as null, which IN never matches: the rows exist, the
    # filter cannot reach them.
    with pytest.warns(UserWarning, match='by="_id"'):
        lat.read(text=AGS).at("SAMP", [None])


def test_at_a_none_value_warns_from_a_chain_too():
    with pytest.warns(UserWarning, match='by="_id"'):
        lat.read(text=AGS).at("LOCA", ["BH01"]).at("SAMP", [None])


def test_at_without_a_none_value_does_not_warn(recwarn):
    lat.read(text=AGS).at("SAMP", ["S1"])
    lat.read(text=AGS).at("SAMP", [None], by="_id")
    assert not [w for w in recwarn if issubclass(w.category, UserWarning)]


# --- by="_id": the group's own rows, then its direct children --------------


def test_by_id_keeps_the_rows_and_their_direct_children():
    h = lat.read(text=AGS)
    q = h.at("SAMP", [_samp_id(h, "BH01", 1.0)], by="_id")
    samp = q["SAMP"]
    assert samp["LOCA_ID"].to_list() == ["BH01"]
    assert samp["SAMP_TOP"].to_list() == [1.0]
    assert q["LLPL"]["SPEC_REF"].to_list() == ["A", "B"]


def test_by_id_reaches_a_dict_declared_child():
    h = lat.read(text=AGS)
    q = h.at("SAMP", [_samp_id(h, "BH01", 1.0)], by="_id")
    assert q["XSMP"]["XSMP_REF"].to_list() == ["x1"]


def test_by_id_groups_are_the_group_and_its_keyed_direct_children():
    # XNKY is a direct child but carries no ids, so no row of it can be "at" one.
    h = lat.read(text=AGS)
    q = h.at("SAMP", [_samp_id(h, "BH01", 1.0)], by="_id")
    assert q.groups == ["SAMP", "LLPL", "XSMP"]
    assert set(q.frames()) == {"SAMP", "LLPL", "XSMP"}


def test_by_id_a_keyless_child_asked_for_by_name_has_no_rows():
    h = lat.read(text=AGS)
    q = h.at("SAMP", [_samp_id(h, "BH01", 1.0)], by="_id")
    assert q["XNKY"].height == 0


def test_by_id_raises_on_a_group_no_filter_reaches():
    # LOCA is SAMP's parent and GEOL its sibling: handing either back whole
    # would mix every borehole into "the rows at this sample".
    h = lat.read(text=AGS)
    q = h.at("SAMP", [_samp_id(h, "BH01", 1.0)], by="_id")
    for code in ("LOCA", "GEOL"):
        with pytest.raises(ValueError, match=code):
            _ = q[code]


def test_by_id_does_not_reach_grandchildren_through_the_parent():
    # LOCA's direct children are SAMP and GEOL; LLPL is a grandchild.
    h = lat.read(text=AGS)
    loca = h.table("LOCA", keys=True)
    bh01 = loca.filter(loca["LOCA_ID"] == "BH01")["_id"].item()
    q = h.at("LOCA", [bh01], by="_id")
    assert q.groups == ["LOCA", "GEOL", "SAMP"]
    assert q["SAMP"].height == 2
    with pytest.raises(ValueError, match="LLPL"):
        _ = q["LLPL"]


def test_by_id_with_a_heading_filter_reaches_what_either_reaches():
    h = lat.read(text=AGS)
    q = h.at("LOCA", ["BH01"]).at("SAMP", [_samp_id(h, "BH01", 1.0)], by="_id")
    assert q["GEOL"].height == 1  # reached by the LOCA_ID filter
    with pytest.raises(ValueError, match="PROJ"):
        _ = q["PROJ"]  # reached by neither


def test_heading_filters_alone_still_pass_unreached_groups_through():
    assert lat.read(text=AGS).at("LOCA", ["BH01"])["PROJ"].height == 1


def test_by_id_chains_with_a_heading_filter():
    h = lat.read(text=AGS)
    ids = [_samp_id(h, "BH01", 1.0), _samp_id(h, "BH02", 1.0)]
    q = h.at("SAMP", ids, by="_id").at("LOCA", ["BH02"])
    assert q["LLPL"]["LLPL_LL"].to_list() == [48]
    assert q["LOCA"]["LOCA_ID"].to_list() == ["BH02"]


def test_by_id_empty_values_selects_nothing():
    q = lat.read(text=AGS).at("SAMP", [], by="_id")
    assert q["SAMP"].height == 0
    assert q["LLPL"].height == 0


def test_by_id_honours_the_frame_key_default():
    h = lat.read(text=AGS)
    sid = _samp_id(h, "BH01", 1.0)
    assert "_id" not in h.at("SAMP", [sid], by="_id")["SAMP"].columns
    keyed = lat.read(text=AGS, keys=True).at("SAMP", [sid], by="_id")["LLPL"]
    assert set(keyed["_parent_id"].to_list()) == {sid}


def test_by_id_on_a_group_absent_with_its_children_raises():
    with pytest.raises(ValueError, match="NOPE"):
        lat.read(text=AGS).at("NOPE", ["x"], by="_id")


def test_by_id_rejects_an_unknown_by():
    with pytest.raises(ValueError, match="by="):
        lat.read(text=AGS).at("SAMP", ["x"], by="SAMP_REF")


def test_by_id_does_not_narrow_a_single_result_query():
    # A .query() base has no group to tell `_id` from `_parent_id` by.
    h = lat.read(text=AGS)
    q = h.query("SELECT * FROM LLPL").at("SAMP", [_samp_id(h, "BH01", 1.0)], by="_id")
    with pytest.raises(TypeError, match='by="_id"'):
        q.frame()


def test_by_id_without_a_base_names_the_missing_base():
    h = lat.read(text=AGS)
    q = h.at("SAMP", [_samp_id(h, "BH01", 1.0)], by="_id")
    with pytest.raises(TypeError, match="no base"):
        q.frame()
