"""`AGS4_to_dataframe(only_groups=…)` narrows the *result*, never the *raises*.

The narrowing used to happen entirely in Python: the native parse built and
crossed an Arrow table for every group in the file, and `AGS4_to_dataframe`
then kept the ones you asked for. #99 pushed the narrowing down into
`parse_compat_arrow`, so a group you did not ask for is never built at all
(~11 ms and ~25 MB of peak RSS on a 123-group file).

That makes a *behavioural* claim worth pinning, because it would be easy and
wrong to narrow more than the tables. python-ags4's three hard raises —
duplicate GROUP, ragged DATA row, duplicate heading under
`rename_duplicate_headers=False` — read the whole file, and they must keep
firing on offences in groups the caller filtered out. A narrowing that also
narrowed the raises would turn a rejected file into an accepted one: silent
data loss dressed up as a speed-up.

`only_groups` is upstream's own parameter (`python_ags4.AGS4.AGS4_to_dataframe`),
and the parity runner exercises it, but nothing in this repo's own suite did
until now — so the pushdown would have been guarded only by an external
checkout. Hence the differential tests below plus standalone invariants that
still mean something when the oracle is absent.
"""

from __future__ import annotations

import pytest
from laterite import compat as AGS4

try:
    from python_ags4 import AGS4 as up_AGS4

    _HAS_ORACLE = True
except Exception:  # pragma: no cover - oracle is a declared dev dependency
    _HAS_ORACLE = False

oracle = pytest.mark.skipif(
    not _HAS_ORACLE, reason="python-ags4 parity oracle not installed"
)


def _ags(*lines: str) -> str:
    return "\r\n".join(lines) + "\r\n"


_THREE_GROUPS = _ags(
    '"GROUP","PROJ"',
    '"HEADING","PROJ_ID"',
    '"UNIT",""',
    '"TYPE","ID"',
    '"DATA","P1"',
    "",
    '"GROUP","LOCA"',
    '"HEADING","LOCA_ID","LOCA_TYPE"',
    '"UNIT","",""',
    '"TYPE","ID","PA"',
    '"DATA","BH01","CP"',
    '"DATA","BH02","RC"',
    "",
    '"GROUP","SAMP"',
    '"HEADING","LOCA_ID","SAMP_TOP","SAMP_REF","SAMP_TYPE","SAMP_ID"',
    '"UNIT","","m","","",""',
    '"TYPE","ID","2DP","ID","PA","ID"',
    '"DATA","BH01","1.00","S1","D","BH01_1.00_S1_D"',
)


def _write(tmp_path, text: str, name: str = "site.ags"):
    p = tmp_path / name
    p.write_text(text, encoding="utf-8", newline="")
    return p


# --- the narrowing itself ---------------------------------------------------


@pytest.mark.parametrize(
    "only",
    [None, [], ["PROJ"], ["LOCA"], ["PROJ", "SAMP"], ["SAMP", "PROJ"]],
)
def test_narrowed_frames_equal_the_full_read(tmp_path, only):
    """The prize is that unselected tables are never built. The contract is that
    the selected ones are bit-for-bit what a full read produces — so narrowing
    can never be a way to get a *different* frame, only a cheaper one.

    `[]` is in the list deliberately: it is falsy, so the caller reads it as
    "all", and the pushdown has to agree rather than building nothing."""
    path = _write(tmp_path, _THREE_GROUPS)
    full, full_head = AGS4.AGS4_to_dataframe(path)
    got, got_head = AGS4.AGS4_to_dataframe(path, only_groups=only)

    # mirrors the caller's own `only_groups if only_groups else <all>` — `[]` is
    # falsy on both sides, which is the point of including it above
    expected = only if only else list(full)
    assert list(got) == expected
    # headings are reported for EVERY group either way — only the frames narrow
    assert got_head == full_head
    for code in got:
        assert got[code].equals(full[code]), f"{code} differs from the full read"


def test_a_group_outside_the_narrowing_has_no_table_built(tmp_path):
    """The mechanism, asserted at the native boundary rather than inferred from
    a timing. A skipped group still carries its headings and line anchors (the
    raises need them); what it must not carry is the Arrow table."""
    from laterite.compat import _compat_arrow

    path = _write(tmp_path, _THREE_GROUPS)
    p = _compat_arrow(path, "utf-8", ["LOCA"])
    groups = p["groups"]

    assert set(groups) == {"PROJ", "LOCA", "SAMP"}, "every group still crosses"
    assert "table" in groups["LOCA"]
    for skipped in ("PROJ", "SAMP"):
        assert "table" not in groups[skipped], f"{skipped} was built anyway"
        assert list(groups[skipped]["headings"]), "headings still cross"
        assert groups[skipped]["group_line"] > 0, "line anchors still cross"


def test_unnarrowed_read_still_builds_every_table(tmp_path):
    """`only_groups=None` is the default path every existing caller takes; it
    must be exactly what it was before the pushdown."""
    from laterite.compat import _compat_arrow

    p = _compat_arrow(_write(tmp_path, _THREE_GROUPS), "utf-8", None)
    assert all("table" in g for g in p["groups"].values())


# --- what narrowing must NOT narrow -----------------------------------------


def test_duplicate_group_still_raises_when_the_offender_is_filtered_out(tmp_path):
    """LOCA appears twice; the caller asks only for PROJ. The file is still
    invalid and python-ags4 still rejects it."""
    text = _THREE_GROUPS + _ags(
        "",
        '"GROUP","LOCA"',
        '"HEADING","LOCA_ID"',
        '"UNIT",""',
        '"TYPE","ID"',
        '"DATA","BH03"',
    )
    path = _write(tmp_path, text)
    with pytest.raises(AGS4.AGS4Error):
        AGS4.AGS4_to_dataframe(path, only_groups=["PROJ"])


def test_ragged_row_still_raises_when_the_offender_is_filtered_out(tmp_path):
    """A short DATA row in SAMP, and a caller who only wants PROJ."""
    text = _THREE_GROUPS + _ags('"DATA","BH01","2.00"')
    path = _write(tmp_path, text)
    with pytest.raises(AGS4.AGS4Error):
        AGS4.AGS4_to_dataframe(path, only_groups=["PROJ"])


def test_duplicate_heading_still_raises_when_the_offender_is_filtered_out(tmp_path):
    """`rename_duplicate_headers=False` + a dup heading in LOCA, asking for PROJ.
    This one is renamed/raised on the Python side over ALL groups, so it also
    pins that the pushdown did not tempt anyone into narrowing that loop."""
    text = _ags(
        '"GROUP","PROJ"',
        '"HEADING","PROJ_ID"',
        '"UNIT",""',
        '"TYPE","ID"',
        '"DATA","P1"',
        "",
        '"GROUP","LOCA"',
        '"HEADING","LOCA_ID","LOCA_ID"',
        '"UNIT","",""',
        '"TYPE","ID","ID"',
        '"DATA","BH01","BH01"',
    )
    path = _write(tmp_path, text)
    with pytest.raises(AGS4.AGS4Error):
        AGS4.AGS4_to_dataframe(
            path, only_groups=["PROJ"], rename_duplicate_headers=False
        )


def test_asking_for_a_group_the_file_does_not_have(tmp_path):
    """Unchanged by the pushdown, and pinned so it stays that way: the missing
    code raises rather than silently yielding an empty result."""
    path = _write(tmp_path, _THREE_GROUPS)
    with pytest.raises(KeyError):
        AGS4.AGS4_to_dataframe(path, only_groups=["NOPE"])


# --- differential against the oracle ----------------------------------------


@oracle
@pytest.mark.parametrize("only", [None, ["PROJ"], ["PROJ", "SAMP"]])
def test_narrowed_read_matches_python_ags4(tmp_path, only):
    path = _write(tmp_path, _THREE_GROUPS)
    ours, our_head = AGS4.AGS4_to_dataframe(path, only_groups=only)
    theirs, their_head = up_AGS4.AGS4_to_dataframe(str(path), only_groups=only)

    assert list(ours) == list(theirs)
    assert our_head == their_head
    for code in ours:
        assert list(ours[code].columns) == list(theirs[code].columns)
        assert ours[code].shape == theirs[code].shape
