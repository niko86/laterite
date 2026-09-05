"""The launcher human-content gate's extractors (tools/xcheck/check_cli_content.py).

The gate compares FACTS parsed out of each launcher's human output (#542); these
tests pin the extractors over canned output copied from real runs, so an
extractor that quietly stops seeing a fact — the gate's own blind-spot mode —
goes red here rather than green-by-agreeing-on-None. The canned PRE-fix npx
outputs are kept as the A/B record: they must extract to fact dicts that
DISAGREE with the binary's, because that disagreement is the defect the gate
exists to catch.
"""

from _tools import load_tool

_mod = load_tool("check_cli_content")
diff_facts_binary = _mod.diff_facts_binary
diff_facts_npx = _mod.diff_facts_npx
fix_facts_binary = _mod.fix_facts_binary
fix_facts_npx = _mod.fix_facts_npx
validate_facts_binary = _mod.validate_facts_binary
validate_facts_npx = _mod.validate_facts_npx

# --- validate ----------------------------------------------------------------

BINARY_CLEAN = "f.ags: clean (0 findings) — dictionary 4.2 (exact)\n"
BINARY_FINDINGS = "f.ags: 5 finding(s) — dictionary 4.1.1 (fallback)\n┌table┐\n"
NPX_CLEAN = "f.ags — 4.2 (exact)\n  clean — no findings\n"
NPX_FINDINGS = "f.ags — 4.1.1 (fallback)\n  5 finding(s)\n    8 (line 5, LOCA): bad\n"
#: The binary's output BEFORE #542 — no dictionary stated. Extracts to None
#: facts, which must not equal the npx extraction: that split was the finding.
BINARY_CLEAN_PRE_542 = "f.ags: clean (0 findings)\n"


def test_validate_binary_extracts_edition_resolution_count():
    assert validate_facts_binary(BINARY_CLEAN) == {
        "edition": "4.2",
        "resolution": "exact",
        "count": 0,
    }
    assert validate_facts_binary(BINARY_FINDINGS) == {
        "edition": "4.1.1",
        "resolution": "fallback",
        "count": 5,
    }


def test_validate_npx_extracts_the_same_facts():
    assert validate_facts_npx(NPX_CLEAN) == validate_facts_binary(BINARY_CLEAN)
    assert validate_facts_npx(NPX_FINDINGS) == validate_facts_binary(BINARY_FINDINGS)


def test_validate_pre_fix_output_disagrees_not_agrees():
    """An output missing the fact must extract to a DIFFERENT dict, never be
    absorbed — None-vs-None agreement would be the gate lying green."""
    facts = validate_facts_binary(BINARY_CLEAN_PRE_542)
    assert facts["edition"] is None
    assert facts != validate_facts_npx(NPX_CLEAN)


# --- diff --------------------------------------------------------------------

BINARY_DIFF = (
    "a.ags → b.ags\n"
    "  PROJ   +0 -0 ~0\n"
    "  LOCA   +1 -1 ~1\n"
    "  groups added:   SAMP\n"
    "  total: +1 added · −1 removed · ~1 changed\n"
)
BINARY_DIFF_REVERSED = (
    "b.ags → a.ags\n"
    "  PROJ   +0 -0 ~0\n"
    "  LOCA   +1 -1 ~1\n"
    "  groups removed: SAMP\n"
    "  total: +1 added · −1 removed · ~1 changed\n"
)
NPX_DIFF = (
    "a.ags → b.ags\n"
    "PROJ: +0 -0 ~0\n"
    "LOCA: +1 -1 ~1\n"
    "groups added: SAMP\n"
    "total: +1 -1 ~1\n"
)
#: npx BEFORE #542: changed groups only — no header, no heading-only PROJ, no
#: group add/remove lines, no totals.
NPX_DIFF_PRE_542 = "LOCA: +1 -1 ~1\n"


def test_diff_binary_extracts_all_fact_classes():
    facts = diff_facts_binary(BINARY_DIFF)
    assert facts == {
        "header": ["a.ags", "b.ags"],
        "groups": {"PROJ": [0, 0, 0], "LOCA": [1, 1, 1]},
        "groups_added": ["SAMP"],
        "groups_removed": [],
        "total": [1, 1, 1],
    }
    assert diff_facts_binary(BINARY_DIFF_REVERSED)["groups_removed"] == ["SAMP"]


def test_diff_npx_extracts_the_same_facts():
    assert diff_facts_npx(NPX_DIFF) == diff_facts_binary(BINARY_DIFF)


def test_diff_npx_total_line_is_not_a_group():
    """Regression: `total: +1 -1 ~1` shares the group-line shape, and a
    generic-first match chain filed it under a group named "total" on the
    gate's first run — the specific matches must win."""
    facts = diff_facts_npx(NPX_DIFF)
    assert "total" not in facts["groups"]
    assert facts["total"] == [1, 1, 1]


def test_diff_pre_fix_output_disagrees_not_agrees():
    facts = diff_facts_npx(NPX_DIFF_PRE_542)
    assert facts["header"] is None and facts["total"] is None
    assert facts != diff_facts_binary(BINARY_DIFF)


# --- fix ---------------------------------------------------------------------

BINARY_FIX = (
    "applied 1 fix(es) [reformat_numeric] → d.fixed.ags\n"
    "d.fixed.ags: 4 finding(s) remain (not mechanically fixable)\n"
)
NPX_FIX = (
    "<FixResult 106 bytes, 1 fix(es) applied, 4 residual finding(s)>"
    " [reformat_numeric] → d.fixed.ags\n"
)
#: npx BEFORE #542: the result line went to stderr, so stdout was EMPTY — the
#: no-facts arm of the gate is what catches that world, not a value split.
NPX_FIX_PRE_542_STDOUT = ""
#: npx after the stream fix but before the kinds joined the line — the gap the
#: content gate's first fix run caught live.
NPX_FIX_NO_KINDS = (
    "<FixResult 106 bytes, 1 fix(es) applied, 4 residual finding(s)> → d.fixed.ags\n"
)


def test_fix_extractors_agree_on_the_same_facts():
    facts = fix_facts_binary(BINARY_FIX)
    assert facts == {
        "applied": 1,
        "kinds": ["reformat_numeric"],
        "dest": "d.fixed.ags",
        "residual": 4,
    }
    assert fix_facts_npx(NPX_FIX) == facts


def test_fix_pre_fix_outputs_extract_to_missing_facts():
    empty = fix_facts_npx(NPX_FIX_PRE_542_STDOUT)
    assert all(v is None for v in empty.values())
    no_kinds = fix_facts_npx(NPX_FIX_NO_KINDS)
    assert no_kinds["applied"] == 1 and no_kinds["kinds"] is None
