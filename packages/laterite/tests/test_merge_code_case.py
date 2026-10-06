"""#1010 — ABBR codes that differ only by letter case in ``merge``.

Pinned on the issue's own repro: one delivery writes ``TRIG_COND`` as
``"Undisturbed"``, the next as ``"UNDISTURBED"``. Merge always warns
(``abbr_code_case``); it rewrites only when asked, by ``recode=`` or by
``on_code_case="standard"`` where exactly one standard code matches.
"""

from __future__ import annotations

import json
import typing
from typing import Any

import laterite
import polars as pl
import pytest
from laterite import _laterite_native as _native

STAMP = laterite.TranStamp(
    issue="1", date="2026-01-01", producer="A", recipient="B", status="Draft"
)
MERGE_STAMP = laterite.TranStamp(
    issue="2", date="2026-01-02", producer="A", recipient="B", status="Draft"
)


def _build(
    trig_cond: str = "UNDISTURBED", samp_type: tuple[str, str] = ("U", "D")
) -> bytes:
    """The issue's ``build``: SAMP, TRIG and LNMC for two specimens of BH1."""
    k = {
        "LOCA_ID": ["BH1", "BH1"],
        "SAMP_TOP": [1.0, 2.0],
        "SAMP_REF": ["1", "2"],
        "SAMP_TYPE": list(samp_type),
        "SAMP_ID": ["S1", "S2"],
    }
    spec = {"SPEC_REF": ["1", "1"], "SPEC_DPTH": [1.0, 2.0]}
    frames = {
        "PROJ": pl.DataFrame({"PROJ_ID": ["P1"]}),
        "LOCA": pl.DataFrame({"LOCA_ID": ["BH1"]}),
        "SAMP": pl.DataFrame(k),
        "TRIG": pl.DataFrame(
            {**k, **spec, "TRIG_TYPE": ["UU", "UU"], "TRIG_COND": [trig_cond] * 2}
        ),
        "LNMC": pl.DataFrame({**k, **spec, "LNMC_MC": [11.3, 14.1]}),
    }
    return laterite.build_ags4(
        frames, dict_version="4.1.1", synthesise_metadata=True, tran=STAMP
    ).bytes


def _merge(*sources: bytes, **kw: Any) -> laterite.MergeResult:
    if not sources:
        sources = (_build("Undisturbed"), _build("UNDISTURBED"))
    return laterite.merge(*sources, dict_version="4.1.1", tran=MERGE_STAMP, **kw)


def _cond(data: bytes) -> list[str]:
    return laterite.read(data)["TRIG"]["TRIG_COND"].to_list()


def _abbr(data: bytes, heading: str) -> list[str]:
    abbr = laterite.read(data)["ABBR"]
    return abbr.filter(pl.col("ABBR_HDNG") == heading)["ABBR_CODE"].to_list()


def _case(res: laterite.MergeResult) -> list[dict]:
    return [w for w in res.warnings if w["kind"] == "abbr_code_case"]


def test_the_default_warns_and_otherwise_changes_nothing():
    res = _merge()
    (w,) = _case(res)
    assert w["heading"] == "TRIG_COND"
    assert w["case"]["spellings"] == [
        {"code": "Undisturbed", "inputs": [0]},
        {"code": "UNDISTURBED", "inputs": [1]},
    ]
    assert w["case"]["suggested"] == "UNDISTURBED"
    assert w["case"]["resolved"] is None
    # Today's output: both spellings, and the case change is a revision.
    assert _abbr(res.bytes, "TRIG_COND") == ["Undisturbed", "UNDISTURBED"]
    assert [r["changed"] for r in res.revisions if r["group"] == "TRIG"] == [
        ["TRIG_COND"],
        ["TRIG_COND"],
    ]
    assert _merge(on_code_case="keep").bytes == res.bytes


def test_standard_settles_on_the_standard_spelling_and_validates():
    res = _merge(on_code_case="standard")
    assert _abbr(res.bytes, "TRIG_COND") == ["UNDISTURBED"]
    assert _cond(res.bytes) == ["UNDISTURBED", "UNDISTURBED"]
    assert not [r for r in res.revisions if r["group"] == "TRIG"]
    (w,) = _case(res)
    assert w["case"]["resolved"] == "UNDISTURBED"
    assert laterite.validate(res.bytes).is_valid


def test_recode_matches_standard_and_can_name_a_non_standard_target():
    named = _merge(recode={"TRIG_COND": {"Undisturbed": "UNDISTURBED"}})
    assert named.bytes == _merge(on_code_case="standard").bytes
    mixed = _merge(recode={"TRIG_COND": {"UNDISTURBED": "Undisturbed"}})
    assert _cond(mixed.bytes) == ["Undisturbed", "Undisturbed"]
    assert _abbr(mixed.bytes, "TRIG_COND") == ["Undisturbed"]
    # With standard as well, the recode wins for the set it names.
    both = _merge(
        on_code_case="standard", recode={"TRIG_COND": {"UNDISTURBED": "Undisturbed"}}
    )
    assert _cond(both.bytes) == ["Undisturbed", "Undisturbed"]


def test_a_concatenated_value_is_rewritten_part_by_part():
    # The non-standard spelling comes last, so it would win unrewritten.
    res = _merge(
        _build("UNDISTURBED+B"), _build("Undisturbed+B"), on_code_case="standard"
    )
    assert _cond(res.bytes) == ["UNDISTURBED+B", "UNDISTURBED+B"]


def test_a_rewrite_merging_two_rows_raises_and_keep_does_not():
    a, b = _build(samp_type=("U", "D")), _build(samp_type=("u", "D"))
    with pytest.raises(laterite.MergeConflictError, match="SAMP"):
        _merge(a, b, on_code_case="standard")
    with pytest.raises(laterite.MergeConflictError, match="key collision"):
        _merge(a, b, recode={"SAMP_TYPE": {"u": "U"}})
    samp = laterite.read(_merge(a, b).bytes)["SAMP"]["SAMP_TYPE"].to_list()
    assert "U" in samp and "u" in samp


def test_a_bad_mode_or_recode_is_refused_like_a_bad_type_clash():
    a = _build()
    with pytest.raises(laterite.BadDictError) as clash:
        laterite.merge(a, a, on_type_clash="yolo")  # type: ignore[arg-type]
    with pytest.raises(type(clash.value), match="keep, standard"):
        laterite.merge(a, a, on_code_case="first")  # type: ignore[arg-type]
    with pytest.raises(laterite.BadDictError, match="SAMP_REF"):
        _merge(recode={"SAMP_REF": {"1": "2"}})
    with pytest.raises(laterite.BadDictError, match="Disturbed"):
        _merge(recode={"TRIG_COND": {"Disturbed": "D"}})


def test_lat_merge_on_code_case_and_recode(capsys: Any, tmp_path: Any) -> None:
    from laterite import _cli

    a, b = tmp_path / "a.ags", tmp_path / "b.ags"
    a.write_bytes(_build("Undisturbed"))
    b.write_bytes(_build("UNDISTURBED"))
    out = tmp_path / "out.ags"
    base = ["merge", str(a), str(b), "--out", str(out), "--tran-issue", "2"]
    base += ["--tran-date", "2026-01-02", "--tran-producer", "A"]
    base += ["--tran-recipient", "B", "--tran-status", "Draft", "--json"]

    assert _cli.main(base) == 0
    report = json.loads(capsys.readouterr().out)
    assert [w["kind"] for w in report["warnings"]] == ["abbr_code_case"]
    assert _cond(out.read_bytes()) == ["UNDISTURBED", "UNDISTURBED"]  # input 1 won

    assert _cli.main([*base, "--on-code-case", "standard"]) == 0
    capsys.readouterr()
    standard = out.read_bytes()
    recode = tmp_path / "recode.json"
    recode.write_text(json.dumps({"TRIG_COND": {"Undisturbed": "UNDISTURBED"}}))
    assert _cli.main([*base, "--recode", str(recode)]) == 0
    capsys.readouterr()
    assert out.read_bytes() == standard

    assert _cli.main([*base, "--on-code-case", "first"]) == 5
    assert "--on-code-case" in capsys.readouterr().err
    recode.write_text(json.dumps({"SAMP_REF": {"1": "2"}}))
    assert _cli.main([*base, "--recode", str(recode)]) == 5
    assert "SAMP_REF" in capsys.readouterr().err
    recode.write_text("[1, 2]")
    assert _cli.main([*base, "--recode", str(recode)]) == 5
    capsys.readouterr()


def test_the_literal_and_the_cli_choices_match_the_engine():
    """`CodeCaseMode` cannot derive itself, so it is pinned to `CodeCaseMode::ALL`."""
    from laterite import _cli

    authority = list(_native.registry_code_case_modes())
    assert authority == ["keep", "standard"]
    assert list(typing.get_args(laterite.CodeCaseMode)) == authority
    assert list(_cli._CODE_CASE_CHOICES) == authority
