"""Batch A: per-rule fix selection (`fix(only=…, exclude=…)`) + the discoverable
`fixable_rules()` registry + the `FixableRule` literal.

These assert the OUTPUT, not just that a call ran: that a fix actually *clears*
the rule it targets (fix → re-validate → finding gone), that excluding a rule
*leaves* its finding, and that the typed/discoverable surfaces stay in lock-step
with the engine's fixable set. (#294 — the output-assertion backbone the
test-depth audit flagged as missing for everything but Rule 2a.)
"""

from __future__ import annotations

import typing

import laterite as L
import pytest

# One crafted source per safe-fixable rule, each triggering that rule.
CASES = {
    "1": dict(  # leading UTF-8 BOM -> Rule 1 (StripBom, safe)
        data=b"\xef\xbb\xbf"
        b'"GROUP","PROJ"\r\n"HEADING","PROJ_ID"\r\n"UNIT",""\r\n"TYPE","ID"\r\n"DATA","P1"\r\n'
    ),
    "2a": dict(  # bare-LF line endings -> Rule 2a (NormalizeCrlf, safe)
        text='"GROUP","PROJ"\n"HEADING","PROJ_ID"\n"UNIT",""\n"TYPE","ID"\n"DATA","P1"\n'
    ),
    "4": dict(  # DATA row shorter than HEADING -> Rule 4 (PadShortRow, safe)
        text='"GROUP","PROJ"\r\n"HEADING","PROJ_ID","PROJ_NAME"\r\n"UNIT","",""\r\n'
        '"TYPE","ID","X"\r\n"DATA","P1"\r\n'
    ),
    "8": dict(  # 2DP value at 1 dp -> Rule 8 (ReformatNumeric, safe)
        text='"GROUP","LOCA"\r\n"HEADING","LOCA_ID","LOCA_GL"\r\n"UNIT","","m"\r\n'
        '"TYPE","ID","2DP"\r\n"DATA","BH1","1.0"\r\n'
    ),
}


def _rules(report) -> set[str]:
    return set(report.by_rule().keys())


# --- discoverability + the typed surface stays in lock-step with the engine ---


def test_fixable_rules_matches_the_literal():
    """`FixableRule` (hand-written literal) must equal the engine's fixable set,
    so the type can't drift from what `fix` can actually repair."""
    from_engine = {r["rule"] for r in L.fixable_rules()}
    assert set(typing.get_args(L.FixableRule)) == from_engine


def test_fixable_rules_is_the_fixable_subset_of_list_rules():
    fixable = L.fixable_rules()
    assert fixable, "expected some fixable rules"
    assert all(r["fixable"] for r in fixable)
    listed = {r["rule"] for r in L.list_rules() if r["fixable"]}
    assert {r["rule"] for r in fixable} == listed


@pytest.mark.parametrize("kw", ["only", "exclude"])
def test_unknown_rule_label_raises(kw):
    with pytest.raises(ValueError, match="not fixable"):
        L.fix(text="x", **{kw: ["99"]})


# --- the output backbone: a fix actually clears the rule it targets ----------


@pytest.mark.parametrize("short", list(CASES))
def test_fix_clears_the_targeted_rule(short):
    label = f"AGS Format Rule {short}"
    before = L.read(**CASES[short]).validate()
    assert label in _rules(before.report), f"fixture should trigger {label}"
    fixed = L.fix(**CASES[short])  # safe set
    after = L.read(data=fixed.bytes).validate()
    assert label not in _rules(after.report), f"{label} must be gone after fix"


@pytest.mark.parametrize("short", list(CASES))
def test_exclude_leaves_the_targeted_rule_unfixed(short):
    label = f"AGS Format Rule {short}"
    fixed = L.fix(exclude=[short], **CASES[short])
    after = L.read(data=fixed.bytes).validate()
    assert label in _rules(after.report), f"{label} must remain when excluded"


def test_only_applies_just_the_named_rule():
    """A file with TWO fixable defects (Rule 2a bare-LF + Rule 8 precision):
    `only=['8']` fixes the precision and leaves the line endings."""
    src = dict(
        text='"GROUP","LOCA"\n"HEADING","LOCA_ID","LOCA_GL"\n"UNIT","","m"\n'
        '"TYPE","ID","2DP"\n"DATA","BH1","1.0"\n'
    )
    both = _rules(L.read(**src).validate().report)
    assert {"AGS Format Rule 2a", "AGS Format Rule 8"} <= both

    fixed = L.fix(only=["8"], **src)
    assert [a["kind"] for a in fixed.applied] == ["reformat_numeric"]
    after = _rules(L.read(data=fixed.bytes).validate().report)
    assert "AGS Format Rule 8" not in after  # the named rule was fixed
    assert "AGS Format Rule 2a" in after  # the unnamed rule was left alone


def test_chained_fix_honours_only_exclude():
    """The behavioural knobs are on the chained method too (free/chained parity)."""
    src = '"GROUP","PROJ"\n"HEADING","PROJ_ID"\n"UNIT",""\n"TYPE","ID"\n"DATA","P1"\n'
    kept = L.read(text=src).fix(exclude=["2a"])
    assert kept.fix_report.applied == []  # 2a excluded -> nothing applied
    done = L.read(text=src).fix(only=["2a"])
    assert [a["kind"] for a in done.fix_report.applied] == ["normalize_crlf"]


def test_risky_available_signals_withheld_risky_fixes():
    """A duplicate heading -> Rule 7 rename (a risky-only fix). The default safe
    fix withholds it and *reports* it as available; risky=True applies it and the
    signal clears — so a headless caller learns the opt-in tier exists."""
    dup = (
        '"GROUP","PROJ"\r\n"HEADING","PROJ_ID","PROJ_ID"\r\n"UNIT","",""\r\n'
        '"TYPE","ID","ID"\r\n"DATA","P1","P1"\r\n'
    )
    safe = L.fix(text=dup)
    assert safe.risky_available == 1  # the rename was withheld
    assert safe.applied == []  # nothing safe to apply here
    assert "1 more with risky=True" in repr(safe)

    risky = L.fix(text=dup, risky=True)
    assert risky.risky_available == 0
    assert [a["kind"] for a in risky.applied] == ["rename_duplicate_heading"]

    # a rule removed by `exclude` is no longer "available" (selection applies first)
    assert L.fix(text=dup, exclude=["7"]).risky_available == 0
