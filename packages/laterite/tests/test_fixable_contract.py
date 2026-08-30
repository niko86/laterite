"""#294 Backbone — the **fixable-rule contract**.

Every rule the engine advertises as fixable (`fixable_rules()` / the
`rules_meta.json` SSOT) must be *reachable* and actually *repair* the defect it
claims: inject a violation → `fix()` → re-validate → the finding is gone. This is
the gate that would have caught #294 #1/#5 directly — a rule marked fixable whose
fix is unreachable, a no-op, or never proven end-to-end.

It is **driven by `fixable_rules()`**, not a hand-list: `CASES` must cover the
engine's fixable set exactly (`test_every_fixable_rule_has_a_case`), so adding a
new fixable rule to the engine without a proven round-trip here fails CI. That
coupling is the whole point — the contract can't silently fall behind the engine.

One documented partial: **Rule 7** (duplicate-heading rename) is a *risky*
best-effort fix. It resolves the duplicate, but the synthesized `<name>_1` heading
is not in the standard dictionary, so heading *order* can no longer be checked —
leaving a residual Rule 7 finding of a **different** kind. So its contract is "the
duplicate is resolved + the rename was applied", not "the label is gone"
(`test_rule_7_rename_resolves_the_duplicate`).
"""

from __future__ import annotations

import laterite as L
import pytest

# One crafted source per fixable rule, each triggering exactly that rule's fix.
# The 6/11a/11b triggers mirror the engine's own fix unit tests (fixes.rs) so the
# fixtures track what the engine actually repairs.
CASES: dict[str, dict] = {
    # leading UTF-8 BOM -> Rule 1 (StripBom, safe)
    "1": {
        "data": b"\xef\xbb\xbf"
        b'"GROUP","PROJ"\r\n"HEADING","PROJ_ID"\r\n"UNIT",""\r\n"TYPE","ID"\r\n"DATA","P1"\r\n'
    },
    # bare-LF line endings -> Rule 2a (NormalizeCrlf, safe)
    "2a": {
        "text": '"GROUP","PROJ"\n"HEADING","PROJ_ID"\n"UNIT",""\n"TYPE","ID"\n"DATA","P1"\n'
    },
    # DATA row shorter than HEADING -> Rule 4 (PadShortRow, safe)
    "4": {
        "text": '"GROUP","PROJ"\r\n"HEADING","PROJ_ID","PROJ_NAME"\r\n"UNIT","",""\r\n'
        '"TYPE","ID","X"\r\n"DATA","P1"\r\n'
    },
    # unquoted DATA row that binds without excess fields -> Rule 5
    # (QuoteUnquotedRow, safe — the conditional fix, #778; an OVERFLOWING row
    # stays declined, proven by test_rule_5_overflow_stays_declined below)
    "5": {
        "text": '"GROUP","PROJ"\r\n"HEADING","PROJ_ID","PROJ_NAME"\r\n"UNIT","",""\r\n'
        '"TYPE","ID","X"\r\nDATA,P1,  padded  \r\n'
    },
    # embedded CR inside a DATA value -> Rule 6 (StripEmbeddedCr, safe)
    "6": {
        "text": '"GROUP","PROJ"\r\n"HEADING","PROJ_ID"\r\n"UNIT",""\r\n"TYPE","ID"\r\n'
        '"DATA","a\rb"\r\n'
    },
    # duplicated heading -> Rule 7 (RenameDuplicateHeading, RISKY — see the partial note)
    "7": {
        "text": '"GROUP","LOCA"\r\n"HEADING","LOCA_ID","LOCA_ID"\r\n"UNIT","",""\r\n'
        '"TYPE","ID","ID"\r\n"DATA","BH1","BH1"\r\n'
    },
    # 2DP value at 1 dp -> Rule 8 (ReformatNumeric, safe)
    "8": {
        "text": '"GROUP","LOCA"\r\n"HEADING","LOCA_ID","LOCA_GL"\r\n"UNIT","","m"\r\n'
        '"TYPE","ID","2DP"\r\n"DATA","BH1","1.0"\r\n'
    },
    # TRAN with empty TRAN_DLIM -> Rule 11a (InsertTranDlim, safe)
    "11a": {
        "text": '"GROUP","TRAN"\r\n"HEADING","TRAN_DLIM","TRAN_RCON"\r\n"UNIT","",""\r\n'
        '"TYPE","X","X"\r\n"DATA","","+"\r\n'
    },
    # TRAN with empty TRAN_RCON -> Rule 11b (InsertTranRcon, safe)
    "11b": {
        "text": '"GROUP","TRAN"\r\n"HEADING","TRAN_DLIM","TRAN_RCON"\r\n"UNIT","",""\r\n'
        '"TYPE","X","X"\r\n"DATA","|",""\r\n'
    },
}

# Rule 7's rename resolves the duplicate but introduces a non-dictionary heading,
# so the label survives as an order-check finding — a documented partial (below).
_PARTIAL = {"7"}
# The rules whose fix must fully clear their label on re-validation.
_CLEAN = sorted(set(CASES) - _PARTIAL)


def _rules(report) -> set[str]:
    return set(report.by_rule().keys())


def test_every_fixable_rule_has_a_case():
    """The coupling that makes this a gate: `CASES` covers the engine's fixable
    set EXACTLY. A new fixable rule with no proven round-trip here fails CI; a
    stale case for a no-longer-fixable rule does too."""
    engine = {r["rule"] for r in L.fixable_rules()}
    assert set(CASES) == engine, (
        f"fixable-case drift — missing {sorted(engine - set(CASES))}, "
        f"stale {sorted(set(CASES) - engine)} (add a round-trip CASE for each "
        f"fixable rule, or drop the stale one)"
    )


@pytest.mark.parametrize("short", list(CASES))
def test_fixture_triggers_its_rule(short):
    """Each fixture actually raises the rule it targets — so the round-trip below
    is proving a real repair, not passing vacuously."""
    label = f"AGS Format Rule {short}"
    before = L.read(**CASES[short]).validate()
    assert label in _rules(before.report), f"fixture for {label} should trigger it"


@pytest.mark.parametrize("short", list(CASES))
def test_fix_is_reachable(short):
    """`fix()` (safe + risky) applies at least one fix for the case — the rule is
    reachable from the headless surface, not merely advertised."""
    fixed = L.fix(risky=True, **CASES[short])
    assert fixed.applied, f"Rule {short}: fix() applied nothing"


@pytest.mark.parametrize("short", _CLEAN)
def test_fix_clears_the_rule_on_revalidate(short):
    """The round-trip: inject -> fix -> re-validate -> the rule's finding is gone."""
    label = f"AGS Format Rule {short}"
    fixed = L.fix(risky=True, **CASES[short])
    after = L.read(data=fixed.bytes).validate()
    assert label not in _rules(after.report), (
        f"{label} must be gone after fix — got {sorted(_rules(after.report))}"
    )


def test_rule_5_overflow_stays_declined():
    """The conditional half of Rule 5's entry (#778): a row that split into MORE
    fields than the group declares headings is the genuinely ambiguous case —
    nothing can say which side of the comma the heading wanted — and the fix
    engine must leave it alone, Rule 5 finding intact."""
    text = (
        '"GROUP","PROJ"\r\n"HEADING","PROJ_ID","PROJ_NAME"\r\n"UNIT","",""\r\n'
        '"TYPE","ID","X"\r\nDATA,P1,Acme, Bloggs and Co\r\n'
    )
    fixed = L.fix(risky=True, text=text)
    assert not any(a["kind"] == "quote_unquoted_row" for a in fixed.applied), (
        f"an overflowing row must not be re-quoted: {fixed.applied}"
    )
    # The fixer re-validates its own output — the survived findings are the
    # honest record that the row still needs a human.
    assert any(f["rule"] == "AGS Format Rule 5" for f in fixed.findings), (
        f"the finding must survive: {fixed.findings}"
    )


def test_rule_7_rename_resolves_the_duplicate():
    """Rule 7's documented partial: the risky rename clears the DUPLICATE (the two
    identical headings become distinct) and is applied — but because the `_1`
    heading is non-dictionary, heading *order* can no longer be checked, so a
    residual Rule 7 finding of a different kind remains. The contract is the
    resolved duplicate, not a clean label."""
    fixed = L.fix(risky=True, **CASES["7"])
    assert [a["kind"] for a in fixed.applied] == ["rename_duplicate_heading"]
    # The duplicate is gone: the HEADING row no longer repeats a name.
    headings = L.read(data=fixed.bytes).headings("LOCA")
    assert len(headings) == len(set(headings)), f"duplicate not resolved: {headings}"
    assert "LOCA_ID" in headings and "LOCA_ID_1" in headings
