"""The launcher human-content gate's extractors (tools/xcheck/check_cli_content.py).

The gate compares FACTS parsed out of each launcher's human output (#542); these
tests pin the extractors over canned output copied from real runs, so an
extractor that quietly stops seeing a fact — the gate's own blind-spot mode —
goes red here rather than green-by-agreeing-on-None. The canned PRE-fix npx
outputs are kept as the A/B record: they must extract to fact dicts that
DISAGREE with the binary's, because that disagreement is the defect the gate
exists to catch.
"""

import importlib.util
import sys
from pathlib import Path

_XCHECK_TOOLS = Path(__file__).resolve().parents[1] / "tools" / "xcheck"


def _load():
    # The house pattern for testing a tools/ script (test_check_changelog.py):
    # load by path, not by import name — the buildless repo-gates job installs
    # nothing, and a bare import would also trip the marker-faithfulness gate.
    sys.path.insert(0, str(_XCHECK_TOOLS))  # the gate imports its sibling emit_cli
    spec = importlib.util.spec_from_file_location(
        "check_cli_content", _XCHECK_TOOLS / "check_cli_content.py"
    )
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules["check_cli_content"] = mod
    spec.loader.exec_module(mod)
    return mod


_mod = _load()
diff_facts_binary = _mod.diff_facts_binary
diff_facts_npx = _mod.diff_facts_npx
validate_facts_binary = _mod.validate_facts_binary
validate_facts_npx = _mod.validate_facts_npx

# --- validate ----------------------------------------------------------------

BINARY_CLEAN = "f.ags: clean (0 findings) — dictionary 4.2 (exact)\n"
BINARY_FINDINGS = "f.ags: 5 finding(s) — dictionary 4.1.1 (fallback)\n┌table┐\n"
NPX_CLEAN = "f.ags — 4.2 (exact)\n  clean — no findings\n"
NPX_FINDINGS = "f.ags — 4.1.1 (fallback)\n  5 finding(s)\n    8 (line 5, LOCA): bad\n"
#: The binary's output BEFORE #542 — no dictionary stated. Extracts to None
#: facts, which must not equal the npx extraction: that split was the finding.
BINARY_CLEAN_PREFIX_542 = "f.ags: clean (0 findings)\n"


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


def test_validate_prefix_output_disagrees_not_agrees():
    """An output missing the fact must extract to a DIFFERENT dict, never be
    absorbed — None-vs-None agreement would be the gate lying green."""
    facts = validate_facts_binary(BINARY_CLEAN_PREFIX_542)
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
NPX_DIFF_PREFIX_542 = "LOCA: +1 -1 ~1\n"


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


def test_diff_prefix_output_disagrees_not_agrees():
    facts = diff_facts_npx(NPX_DIFF_PREFIX_542)
    assert facts["header"] is None and facts["total"] is None
    assert facts != diff_facts_binary(BINARY_DIFF)
