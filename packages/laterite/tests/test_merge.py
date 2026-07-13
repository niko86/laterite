"""`laterite.merge` — reconcile N AGS4 deliveries of one project into one file
(the Python face of the shared `laterite-ags4-merge` leaf). These assert the
real merged bytes + the warnings/revisions audit, not just "it runs"."""

import builtins

import laterite
import pytest

# Two deliveries of one project. LOCA is keyed on LOCA_ID.
#   file A: BH1 (NATE 100.00, GL 10.00), BH2 (NATE 200.00, GL 20.00)
#   file B: BH1 (NATE 100.00, GL 11.50 — a real GL revision, identical NATE),
#           BH3 (NATE 300.00, GL 30.00 — a new row)
# B also RE-TYPES LOCA_NATE 2DP -> X: the TYPE conflict strict mode rejects.
_A = "\r\n".join(
    [
        '"GROUP","PROJ"',
        '"HEADING","PROJ_ID","PROJ_NAME"',
        '"UNIT","",""',
        '"TYPE","ID","X"',
        '"DATA","P1","Demo"',
        '"GROUP","LOCA"',
        '"HEADING","LOCA_ID","LOCA_NATE","LOCA_GL"',
        '"UNIT","","m","m"',
        '"TYPE","ID","2DP","2DP"',
        '"DATA","BH1","100.00","10.00"',
        '"DATA","BH2","200.00","20.00"',
        "",
    ]
)
_B = "\r\n".join(
    [
        '"GROUP","PROJ"',
        '"HEADING","PROJ_ID","PROJ_NAME"',
        '"UNIT","",""',
        '"TYPE","ID","X"',
        '"DATA","P1","Demo"',
        '"GROUP","LOCA"',
        '"HEADING","LOCA_ID","LOCA_NATE","LOCA_GL"',
        '"UNIT","","m","m"',
        '"TYPE","ID","X","2DP"',
        '"DATA","BH1","100.00","11.50"',
        '"DATA","BH3","300.00","30.00"',
        "",
    ]
)


def test_merge_result_shape_and_bytes_reparse():
    """The lenient merge returns a real MergeResult whose bytes re-parse as AGS4."""
    res = laterite.merge(_A, _B, lenient=True)
    assert isinstance(res, laterite.MergeResult)
    assert isinstance(res.bytes, builtins.bytes)
    assert res.bytes.startswith(b'"GROUP"')
    assert isinstance(res.warnings, list)
    assert isinstance(res.revisions, list)
    # The merged bytes are valid AGS4 (emit re-validates), so read() accepts them.
    doc = laterite.read(res.bytes)
    assert doc is not None
    assert res.text == res.bytes.decode("utf-8")


def test_merge_union_keeps_every_borehole():
    """Union, not intersection: BH2 (A-only) and BH3 (B-only) both survive."""
    text = laterite.merge(_A, _B, lenient=True).text
    for bh in ("BH1", "BH2", "BH3"):
        assert f'"{bh}"' in text, f"{bh} dropped from the union: {text!r}"


def test_merge_recency_last_argument_wins():
    """A later file wins a KEY conflict — BH1's GL becomes B's 11.50, not A's 10.00."""
    loca = _loca_rows(laterite.merge(_A, _B, lenient=True).text)
    bh1 = next(r for r in loca if r[0] == "BH1")
    assert bh1[2] == "11.50"


def test_merge_reports_the_real_revision_only():
    """revisions names BH1's GL change; the type-widened but unchanged NATE is NOT
    a revision (identical raw value across a 2DP->X widen)."""
    res = laterite.merge(_A, _B, lenient=True)
    revs = [r for r in res.revisions if r["group"] == "LOCA"]
    assert len(revs) == 1
    r = revs[0]
    assert r["key"] == ["BH1"]
    assert r["changed"] == ["LOCA_GL"]
    assert "LOCA_NATE" not in r["changed"]
    assert r["winner_file"] == 1  # the later argument


def test_merge_strict_type_conflict_raises():
    """Strict (default): LOCA_NATE typed 2DP vs X is an unreconcilable conflict."""
    with pytest.raises(laterite.MergeConflictError) as ei:
        laterite.merge(_A, _B)
    assert ei.value.exit_code == 6
    assert "LOCA_NATE" in str(ei.value)


def test_merge_synthesises_a_merge_tran(tmp_path):
    """tran_issue + tran_date stamp a synthesised merge-TRAN; save() writes bytes."""
    res = laterite.merge(
        _A,
        _B,
        lenient=True,
        tran_issue="9",
        tran_date="2024-05-01",
        tran_producer="Merger",
    )
    assert '"GROUP","TRAN"' in res.text
    assert '"9"' in res.text
    out = res.save(tmp_path / "merged.ags")
    assert out.read_bytes() == res.bytes


def test_merge_needs_two_sources():
    with pytest.raises(ValueError):
        laterite.merge(_A)


def _loca_rows(text: str) -> list[list[str]]:
    """The LOCA DATA rows of a merged AGS4 document as unquoted field lists."""
    rows, in_loca = [], False
    for line in text.splitlines():
        fields = [f.strip('"') for f in line.split(",")]
        if fields[0] == "GROUP":
            in_loca = len(fields) > 1 and fields[1] == "LOCA"
        elif in_loca and fields[0] == "DATA":
            rows.append(fields[1:])
    return rows
