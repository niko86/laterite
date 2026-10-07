"""#1024 — ABBR code rewrites in ``fix``, and the Rule 16 near-miss FYI.

``fix`` takes ``merge``'s ``recode=`` / ``on_code_case=`` for one file: an
explicit instruction, so it applies without ``risky=True``, is a rule ``"16"``
repair for ``only=`` / ``exclude=``, and leaves a set whose rewrite would give
two rows one KEY as written, listed in ``FixResult.skipped``. Without the new
arguments nothing changes. ``validate(fyi=True)`` names the close codes for an
undefined PA value, and suggests only — nothing is rewritten.
"""

from __future__ import annotations

import json
from typing import Any

import laterite
import polars as pl
import pytest

STAMP = laterite.TranStamp(
    issue="1", date="2026-01-01", producer="A", recipient="B", status="Draft"
)
FYI16 = "FYI (Related to Rule 16)"


def _build(
    trig_cond: tuple[str, str] = ("Undisturbed", "UNDISTURBED"),
    samp_type: tuple[str, str] = ("U", "D"),
    samp_top: tuple[float, float] = (1.0, 2.0),
) -> bytes:
    """The #1009 delivery: SAMP, TRIG and LNMC for two specimens of BH1."""
    k = {
        "LOCA_ID": ["BH1", "BH1"],
        "SAMP_TOP": list(samp_top),
        "SAMP_REF": ["1", "1"],
        "SAMP_TYPE": list(samp_type),
        "SAMP_ID": ["S1", "S1"],
    }
    frames = {
        "PROJ": pl.DataFrame({"PROJ_ID": ["P1"]}),
        "LOCA": pl.DataFrame({"LOCA_ID": ["BH1"]}),
        "SAMP": pl.DataFrame(k),
        "TRIG": pl.DataFrame(
            {
                **k,
                "SPEC_REF": ["1", "1"],
                "SPEC_DPTH": list(samp_top),
                "TRIG_TYPE": ["UU", "UU"],
                "TRIG_COND": list(trig_cond),
            }
        ),
    }
    return laterite.build_ags4(
        frames, dict_version="4.1.1", synthesise_metadata=True, tran=STAMP
    ).bytes


def _column(data: bytes, group: str, heading: str) -> list[str]:
    return laterite.read(data)[group][heading].to_list()


def _abbr(data: bytes, heading: str) -> list[str]:
    abbr = laterite.read(data)["ABBR"]
    return abbr.filter(pl.col("ABBR_HDNG") == heading)["ABBR_CODE"].to_list()


def _recodes(res: laterite.FixResult) -> list[dict]:
    return [a for a in res.applied if a["kind"] == "recode_abbreviation"]


def test_without_the_new_arguments_fix_changes_no_code():
    data = _build()
    res = laterite.fix(data)
    assert _recodes(res) == []
    assert res.skipped == []
    assert _abbr(res.bytes, "TRIG_COND") == ["UNDISTURBED", "Undisturbed"]
    # `keep` and an empty recode are the defaults spelled out.
    assert laterite.fix(data, on_code_case="keep", recode={}).bytes == res.bytes


def test_standard_rewrites_to_the_standard_code_as_one_safe_rule_16_fix():
    res = laterite.fix(_build(), on_code_case="standard")
    (applied,) = _recodes(res)
    assert applied["risk"] == "safe"
    assert applied["rule"] == "AGS Format Rule 16"
    assert _column(res.bytes, "TRIG", "TRIG_COND") == ["UNDISTURBED", "UNDISTURBED"]
    assert _abbr(res.bytes, "TRIG_COND") == ["UNDISTURBED"]
    assert res.skipped == []
    # What remains is what a file written with one spelling would leave.
    plain = laterite.fix(_build(trig_cond=("UNDISTURBED", "UNDISTURBED")))
    assert res.findings == plain.findings


def test_recode_matches_standard_and_can_name_a_non_standard_target():
    data = _build()
    named = laterite.fix(data, recode={"TRIG_COND": {"Undisturbed": "UNDISTURBED"}})
    assert named.bytes == laterite.fix(data, on_code_case="standard").bytes
    mixed = laterite.fix(data, recode={"TRIG_COND": {"UNDISTURBED": "Undisturbed"}})
    assert _column(mixed.bytes, "TRIG", "TRIG_COND") == ["Undisturbed"] * 2
    assert _abbr(mixed.bytes, "TRIG_COND") == ["Undisturbed"]


def test_only_16_selects_the_rewrite_and_a_contradiction_is_refused():
    data = _build()
    picked = laterite.fix(data, on_code_case="standard", only=["16"])
    assert len(_recodes(picked)) == 1
    with pytest.raises(ValueError, match='rule "16"'):
        laterite.fix(data, on_code_case="standard", only=["4"])
    with pytest.raises(ValueError, match='rule "16"'):
        laterite.fix(data, recode={"TRIG_COND": {"Undisturbed": "X"}}, exclude=["16"])
    # Without a rewrite asked for, `only` / `exclude` mean what they always did.
    laterite.fix(data, exclude=["16"])


def test_a_rewrite_that_would_merge_two_rows_is_skipped_and_reported():
    # Two SAMP rows whose KEYs differ only in SAMP_TYPE's case.
    data = _build(
        trig_cond=("Undisturbed", "UNDISTURBED"),
        samp_type=("U", "u"),
        samp_top=(1.0, 1.0),
    )
    res = laterite.fix(data, on_code_case="standard")
    assert res.skipped == [
        {
            "heading": "SAMP_TYPE",
            "codes": ["u"],
            "target": "U",
            "group": "SAMP",
            "key": ["BH1", "1.00", "1", "U", "S1"],
        }
    ]
    # The set is left as written; the rest of the file is still rewritten.
    assert _column(res.bytes, "SAMP", "SAMP_TYPE") == ["U", "u"]
    assert _column(res.bytes, "TRIG", "TRIG_COND") == ["UNDISTURBED", "UNDISTURBED"]
    assert "skipped" in repr(res)
    # A skip is not a residual finding: findings stay errors and warnings.
    assert all("skipped" not in json.dumps(f) for f in res.findings)


def test_a_bad_mode_or_recode_is_refused():
    data = _build()
    with pytest.raises(laterite.BadDictError, match="keep, standard"):
        laterite.fix(data, on_code_case="first")  # type: ignore[arg-type]
    with pytest.raises(laterite.BadDictError, match="SAMP_REF"):
        laterite.fix(data, recode={"SAMP_REF": {"1": "2"}})
    with pytest.raises(laterite.BadDictError, match="Disturbed"):
        laterite.fix(data, recode={"TRIG_COND": {"Disturbed": "D"}})


def test_the_handle_takes_the_same_arguments():
    repaired = laterite.read(_build()).fix(on_code_case="standard")
    assert repaired.fix_report is not None
    assert len(_recodes(repaired.fix_report)) == 1


def test_lat_fix_on_code_case_recode_and_skipped(capsys: Any, tmp_path: Any) -> None:
    from laterite import _cli

    src = tmp_path / "a.ags"
    src.write_bytes(_build())
    out = tmp_path / "out.ags"
    base = ["fix", str(src), "--fix-out", str(out), "--json"]

    _cli.main(base)
    report = json.loads(capsys.readouterr().out)
    assert "skipped" not in report
    assert _abbr(out.read_bytes(), "TRIG_COND") == ["UNDISTURBED", "Undisturbed"]

    _cli.main([*base, "--on-code-case", "standard"])
    capsys.readouterr()
    standard = out.read_bytes()
    assert _abbr(standard, "TRIG_COND") == ["UNDISTURBED"]
    recode = tmp_path / "recode.json"
    recode.write_text(json.dumps({"TRIG_COND": {"Undisturbed": "UNDISTURBED"}}))
    _cli.main([*base, "--recode", str(recode)])
    capsys.readouterr()
    assert out.read_bytes() == standard

    src.write_bytes(_build(samp_type=("U", "u"), samp_top=(1.0, 1.0)))
    _cli.main([*base, "--on-code-case", "standard"])
    assert json.loads(capsys.readouterr().out)["skipped"][0]["codes"] == ["u"]
    _cli.main(["fix", str(src), "--fix-out", str(out), "--on-code-case", "standard"])
    assert 'left "u" under SAMP_TYPE as written' in capsys.readouterr().out

    assert _cli.main([*base, "--on-code-case", "first"]) == 5
    assert "--on-code-case" in capsys.readouterr().err
    recode.write_text(json.dumps({"SAMP_REF": {"1": "2"}}))
    assert _cli.main([*base, "--recode", str(recode)]) == 5
    assert "SAMP_REF" in capsys.readouterr().err


def _misspelt() -> bytes:
    """One TRIG_COND cell one edit away from the code ABBR declares."""
    data = _build(trig_cond=("UNDISTURBED", "UNDISTURBED"))
    return data.replace(b'"UU","UNDISTURBED"\r\n\r\n', b'"UU","UNDISTURBD"\r\n\r\n')


def test_an_undefined_code_gets_a_did_you_mean_fyi_and_nothing_is_rewritten():
    bad = _misspelt()
    rep = laterite.validate(bad, fyi=True)
    fyis = [d["desc"] for d in rep.findings.to_dicts() if d["rule"] == FYI16]
    assert fyis == [
        '"UNDISTURBD" under TRIG_COND is not defined; '
        'did you mean "UNDISTURBED" (declared in ABBR)?'
    ]
    # Opt-in like every FYI, and fix never acts on a suggestion.
    assert [
        d for d in laterite.validate(bad).findings.to_dicts() if d["rule"] == FYI16
    ] == []
    fixed = laterite.fix(bad, on_code_case="standard")
    assert "UNDISTURBD" in _column(fixed.bytes, "TRIG", "TRIG_COND")
