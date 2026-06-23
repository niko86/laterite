"""`laterite.diff` / `Ags4File.diff` — the KEY-aware, type-aware revision diff
(the Python face of the shared `laterite-ags4-diff` leaf)."""

import laterite

# LOCA keyed on LOCA_ID. Revision vs baseline:
#   BH01  LOCA_GL 10.00 -> 11.00   (a genuine change)
#   BH02  LOCA_GL 20.00 -> 20.0    (formatting-only at 2DP — must NOT be a diff)
#   BH03  removed · BH04 added
_BASE = "\r\n".join(
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
        '"DATA","BH01","10.00"',
        '"DATA","BH02","20.00"',
        '"DATA","BH03","30.00"',
        "",
    ]
)
_REV = "\r\n".join(
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
        '"DATA","BH01","11.00"',
        '"DATA","BH02","20.0"',
        '"DATA","BH04","40.00"',
        "",
    ]
)


def _loca(delta: dict) -> dict:
    return next(g for g in delta["groups"] if g["code"] == "LOCA")


def test_diff_counts_and_changed_cell():
    d = laterite.diff(_BASE, _REV)
    assert (d["total_changed"], d["total_added"], d["total_removed"]) == (1, 1, 1)
    loca = _loca(d)
    assert loca["keyed"] is True
    assert (loca["changed"], loca["added"], loca["removed"]) == (1, 1, 1)
    # the one genuine change: BH01's LOCA_GL 10.00 -> 11.00
    changed = [r for r in loca["rows"] if r["kind"] == "changed"]
    assert len(changed) == 1 and changed[0]["key"] == ["BH01"]
    cell = changed[0]["cells"][0]
    assert cell["heading"] == "LOCA_GL"
    assert cell["a"] == "10.00" and cell["b"] == "11.00"
    assert cell["type"] == "2DP"


def test_formatting_only_change_is_suppressed():
    # BH02 "20.00" -> "20.0" is the same 2DP value, so it must not be a change.
    loca = _loca(laterite.diff(_BASE, _REV))
    changed_keys = [r["key"] for r in loca["rows"] if r["kind"] == "changed"]
    assert ["BH02"] not in changed_keys


def test_added_and_removed_rows():
    loca = _loca(laterite.diff(_BASE, _REV))
    kinds = {tuple(r["key"]): r["kind"] for r in loca["rows"]}
    assert kinds.get(("BH04",)) == "added"
    assert kinds.get(("BH03",)) == "removed"


def test_ags4file_diff_method_and_bytes_input():
    # Method form (self = baseline) and raw-bytes input both agree.
    a = laterite.read(text=_BASE)
    assert a.diff(_REV)["total_changed"] == 1
    d = laterite.diff(_BASE.encode("utf-8"), _REV.encode("utf-8"))
    assert (d["total_changed"], d["total_added"], d["total_removed"]) == (1, 1, 1)


def test_cli_diff(tmp_path, capsys):
    """`lat-check <a> --diff <b>` via the Python `_cli` — the summary + `--json`."""
    import json

    from laterite import _cli

    a, b = tmp_path / "a.ags", tmp_path / "b.ags"
    a.write_text(_BASE)
    b.write_text(_REV)
    assert _cli.main([str(a), "--diff", str(b)]) == 0
    out = capsys.readouterr().out
    assert "LOCA" in out and "+1 -1 ~1" in out
    assert _cli.main([str(a), "--diff", str(b), "--json"]) == 0
    delta = json.loads(capsys.readouterr().out)
    assert (delta["total_changed"], delta["total_added"], delta["total_removed"]) == (
        1,
        1,
        1,
    )
