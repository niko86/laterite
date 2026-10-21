"""The corpus-drift report (#1034): it names what moved, and always says what it saw.

`tools/check_corpus_drift.py` turns the two corpus-qa JSON outputs (the crawl
manifest and the `baseline --check` result) into the `corpus-drift` CI job's
verdict. Every case is driven through the pure `report()` with doctored inputs,
so each property the issue asked for is pinned:

  * the summary line prints on every run, pass, drift and error alike, because
    a gate that drops input must say what it dropped;
  * a drift names the file (by its fixture name, not only the sha) and each
    added / removed finding, then the re-freeze command;
  * the verdict cannot be argued out of the binary's own exit code: a clean
    JSON beside exit 1 (or a drift beside exit 0) is refused, not believed.

Stdlib only, so it runs in the buildless subset beside the other tools tests.
"""

from __future__ import annotations

from typing import Any

import pytest
from _tools import load_tool

ROOT = "rust-packages/laterite-ags4-corpus-qa/vendor/pyags4-tests"
BASE = "rust-packages/laterite-ags4-corpus-qa/baselines/pyags4-vendor.json"

CRAWL: dict[str, Any] = {
    "schema": 1,
    "root": ROOT,
    "selection": "all",
    "walk_skipped": 0,
    "files": [
        {"source": f"{ROOT}/a.ags", "dest": "harvested/x__a.ags", "sha256": "a" * 64},
        {"source": f"{ROOT}/b.ags", "dest": "harvested/y__b.ags", "sha256": "b" * 64},
        {
            "source": f"{ROOT}/b-copy.ags",
            "dest": "harvested/z__b-copy.ags",
            "sha256": "b" * 64,
        },
    ],
}

CLEAN: dict[str, Any] = {
    "committed_files": 2,
    "current_files": 2,
    "only_in_committed": [],
    "only_in_current": [],
    "changed": [],
}

DRIFT: dict[str, Any] = {
    "committed_files": 2,
    "current_files": 2,
    "only_in_committed": [],
    "only_in_current": [],
    "changed": [
        {
            "sha256": "b" * 64,
            "committed_outcome": "findings",
            "current_outcome": "findings",
            "added": [],
            "removed": [
                {
                    "rule": "AGS Format Rule 8",
                    "line": 76,
                    "group": "SAMP",
                    "field_index": 3,
                }
            ],
            "dict_change": "",
        }
    ],
}

ROOT_FILES = [f"{ROOT}/a.ags", f"{ROOT}/b.ags", f"{ROOT}/b-copy.ags", f"{ROOT}/README"]


@pytest.fixture(scope="module")
def gate():
    return load_tool("check_corpus_drift")


def _run(gate, drift, rc, crawl=CRAWL):
    return gate.report(
        crawl=crawl,
        drift=drift,
        check_exit=rc,
        root_files=ROOT_FILES,
        root=ROOT,
        baseline=BASE,
    )


def _summary(lines: list[str]) -> str:
    found = [x for x in lines if x.startswith("corpus-drift:")]
    assert len(found) == 1, lines
    return found[0]


def test_clean_passes_and_still_reports_its_scope(gate):
    lines, code = _run(gate, CLEAN, 0)
    assert code == 0
    s = _summary(lines)
    assert "3 files checked" in s
    assert "2 by content" in s
    assert "walk_skipped 0" in s
    # README sits under the root and the crawl did not take it — said, not hidden.
    assert "not selected 1" in s
    assert "only_in_committed 0" in s
    assert "only_in_current 0" in s
    assert "changed 0" in s


def test_drift_fails_and_names_the_file_and_the_finding(gate):
    lines, code = _run(gate, DRIFT, 1)
    assert code == 1
    text = "\n".join(lines)
    # Both fixtures carrying that content are named, not just the sha.
    assert "b.ags" in text
    assert "b-copy.ags" in text
    assert "- AGS Format Rule 8 SAMP L76 f3" in text
    assert "changed 1" in _summary(lines)


def test_drift_names_the_refreeze_command(gate):
    lines, _ = _run(gate, DRIFT, 1)
    text = "\n".join(lines)
    assert f"baseline --out {BASE}" in text
    assert f"crawl --root {ROOT}" in text


def test_added_findings_and_severity_are_shown(gate):
    drift = {
        **DRIFT,
        "changed": [
            {
                **DRIFT["changed"][0],
                "added": [
                    {
                        "rule": "FYI (Related to Rule 16)",
                        "line": 11,
                        "group": "ABBR",
                        "severity": "fyi",
                    }
                ],
                "removed": [],
            }
        ],
    }
    lines, code = _run(gate, drift, 1)
    assert code == 1
    assert "+ FYI (Related to Rule 16) ABBR L11 [fyi]" in "\n".join(lines)


def test_only_in_either_side_is_drift_and_is_listed(gate):
    drift = {
        **CLEAN,
        "only_in_committed": ["c" * 64],
        "only_in_current": ["a" * 64],
    }
    lines, code = _run(gate, drift, 1)
    assert code == 1
    text = "\n".join(lines)
    assert "c" * 12 in text
    assert "a.ags" in text
    s = _summary(lines)
    assert "only_in_committed 1" in s
    assert "only_in_current 1" in s


@pytest.mark.parametrize(("drift", "rc"), [(CLEAN, 1), (DRIFT, 0)])
def test_a_verdict_that_disagrees_with_the_exit_code_is_refused(gate, drift, rc):
    lines, code = _run(gate, drift, rc)
    assert code == 2
    assert "disagree" in "\n".join(lines)
    _summary(lines)


def test_an_errored_check_still_prints_the_summary(gate):
    lines, code = _run(gate, None, 3)
    assert code == 3
    s = _summary(lines)
    assert "3 files checked" in s
    assert "only_in_committed ?" in s


def test_walk_skips_and_copy_errors_are_counted(gate):
    crawl = {
        **CRAWL,
        "walk_skipped": 2,
        "files": [*CRAWL["files"][:2], {**CRAWL["files"][2], "copy_error": "denied"}],
    }
    lines, _ = _run(gate, CLEAN, 0, crawl=crawl)
    s = _summary(lines)
    assert "walk_skipped 2" in s
    assert "copy errors 1" in s
