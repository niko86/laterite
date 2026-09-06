"""The parity-coverage-map gate (#950): the page's restated numbers stay level.

Every check is driven through the pure `check(doc, failures)` with doctored
pairs, so each failure class the week-35 curation found by hand is pinned to
go red — and the anchors are pinned to MISS loudly, because a reworded page a
gate silently stops reading is the same green-over-blind-spot this repo's
gate rules exist to prevent.

Stdlib only, so it runs in the buildless subset beside the other tools tests.
"""

from __future__ import annotations

import pytest
from _tools import load_tool

FAILURES = {
    "python_ags4_version": "1.2.0",
    "total_tests": 131,
    "known_failures": {
        "tests/test_ags4.py::test_version": {},
        "tests/test_check.py::test_rule_6_2": {},
    },
}

LEVEL_DOC = """# Parity coverage

**129 / 131 of python-ags4 1.2.0's own test suite passes through
`laterite.compat` (98%).** The remaining 2 are deliberate
non-closures, enumerated below.

Expected result: **129 passed, 2 failed**, anchored against python-ags4 **1.2.0**.

## The 2 deliberate non-closures

| python-ags4 test | category | reason |
|---|---|---|
| `test_version`  | identity | reports laterite's version |
| `test_rule_6_2` | O-47     | Rule 6 vs a torn row |

## Coverage by module
"""


@pytest.fixture(scope="module")
def gate():
    return load_tool("check_parity_coverage_map")


def test_level_pair_passes(gate):
    findings, notes = gate.check(LEVEL_DOC, FAILURES)
    assert findings == []
    assert any("NOT judged" in n for n in notes)


def test_the_real_pair_is_level(gate):
    """The positive control against the committed tree — the gate's whole
    reason to exist is that this pair drifted three ways in one curation."""
    import json

    findings, _ = gate.check(
        gate.DOC.read_text(encoding="utf-8"),
        json.loads(gate.FAILURES.read_text(encoding="utf-8")),
    )
    assert findings == []


def test_missing_row_is_a_finding(gate):
    """The week-35 shape: O-47 entered the JSON, the table stayed at 9 rows."""
    doc = LEVEL_DOC.replace(
        "| `test_rule_6_2` | O-47     | Rule 6 vs a torn row |\n", ""
    )
    findings, _ = gate.check(doc, FAILURES)
    assert any("missing `test_rule_6_2`" in f for f in findings)


def test_extra_row_is_a_finding(gate):
    """The other direction: a divergence closed, the record must move too."""
    doc = LEVEL_DOC.replace(
        "| `test_rule_6_2` | O-47     | Rule 6 vs a torn row |",
        "| `test_rule_6_2` | O-47     | Rule 6 vs a torn row |\n"
        "| `test_retired`  | old      | closed long ago |",
    )
    findings, _ = gate.check(doc, FAILURES)
    assert any("`test_retired` is not in" in f for f in findings)


def test_heading_count_disagreement_is_a_finding(gate):
    doc = LEVEL_DOC.replace("## The 2 deliberate", "## The 3 deliberate")
    findings, _ = gate.check(doc, FAILURES)
    assert any("heading says 3" in f for f in findings)


def test_headline_arithmetic_is_derived_from_the_json(gate):
    doc = LEVEL_DOC.replace("**129 / 131", "**130 / 131")
    findings, _ = gate.check(doc, FAILURES)
    assert any("headline says 130 / 131" in f for f in findings)


def test_stale_percentage_is_a_finding(gate):
    doc = LEVEL_DOC.replace("(98%)", "(92%)")
    findings, _ = gate.check(doc, FAILURES)
    assert any("percentage 92%" in f for f in findings)


def test_expected_result_line_is_derived(gate):
    doc = LEVEL_DOC.replace("**129 passed, 2 failed**", "**121 passed, 10 failed**")
    findings, _ = gate.check(doc, FAILURES)
    assert any("121 passed, 10 failed" in f for f in findings)


def test_reworded_heading_misses_loudly(gate):
    """A parse miss is a finding, never a silent pass."""
    doc = LEVEL_DOC.replace("## The 2 deliberate non-closures", "## Non-closures")
    findings, _ = gate.check(doc, FAILURES)
    assert any("anchor missing" in f for f in findings)


def test_version_pin_must_appear(gate):
    doc = LEVEL_DOC.replace("1.2.0", "an upstream release")
    findings, _ = gate.check(doc, FAILURES)
    assert any("python-ags4 1.2.0" in f for f in findings)
