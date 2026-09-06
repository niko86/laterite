"""The spelled-counts convention gate (#950): a NEW crate count in prose fails.

The gate is a ratchet — `scan` finds every occurrence, `judge` fails only
those on lines the diff added — so both halves are pinned separately, plus the
skips: like the em-dash gate, every class of input this scanner ignores is a
place the convention can be broken with a green tick over it, so the skips are
tested rather than just written down.

Stdlib only, so it runs in the buildless subset beside the other tools tests.
"""

from __future__ import annotations

import pytest
from _tools import load_tool


@pytest.fixture(scope="module")
def gate():
    return load_tool("check_spelled_counts")


def test_prose_count_is_found(gate):
    """The positive control — without it every skip test below could pass on a
    scanner that reports nothing."""
    hits = gate.scan("The workspace holds eleven crates today.\n")
    assert [(line, phrase) for line, phrase in hits] == [(1, "eleven crates")]


def test_numeral_and_qualified_counts_are_found(gate):
    hits = gate.scan("all 13 crates\nthe ten engine crates\n")
    assert [phrase for _, phrase in hits] == ["13 crates", "ten engine crates"]


def test_wrapped_count_reports_the_number_token_line(gate):
    hits = gate.scan("first line\nthe publish set's eleven\ncrates moved\n")
    assert [line for line, _ in hits] == [2]


def test_no_number_no_finding(gate):
    """'the QA crates' / 'these crates' state no count — agreement is not
    obligation, exactly as lint.py's C6b comment puts it."""
    assert gate.scan("the QA crates and these crates are fine\n") == []


def test_code_fence_is_skipped(gate):
    """A count in captured tool output is the tool's, not the author's."""
    assert gate.scan("```\n13 crates published\n```\n") == []


def test_inline_code_is_skipped(gate):
    assert gate.scan("run `publish 13 crates` to see\n") == []


def test_generated_region_is_skipped(gate):
    text = (
        "<!-- BEGIN GENERATED: crate-card -->\nall thirteen crates\n"
        "<!-- END GENERATED: crate-card -->\n"
    )
    assert gate.scan(text) == []


def test_historical_marker_exempts_the_line(gate):
    """A series that already happened cannot drift — the measured-value rule's
    own carve-out, spelled with lint.py's A11 marker."""
    text = "eight crates shipped at 0.9.0 <!-- historical -->\n"
    assert gate.scan(text) == []


def test_judge_fails_only_added_lines(gate):
    """The ratchet itself: same occurrence, judged only when its line is new."""
    occurrences = {"docs/a.md": [(3, "eleven crates")]}
    assert gate.judge(occurrences, {"docs/a.md": {3}}) != []
    assert gate.judge(occurrences, {"docs/a.md": {4}}) == []
    assert gate.judge(occurrences, {}) == []


def test_finding_names_file_line_and_the_way_out(gate):
    (finding,) = gate.judge(
        {"RELEASING.md": [(12, "eleven crates")]}, {"RELEASING.md": {12}}
    )
    assert finding.startswith("RELEASING.md:12:")
    assert "historical" in finding


def test_crate_map_is_exempt_by_list(gate):
    """crate-map's count is MEASURED by lint.py C6b — refusal would be a
    second, weaker gate on the same fact."""
    assert "ags-wiki/concepts/crate-map.md" in gate.EXEMPT_FILES
