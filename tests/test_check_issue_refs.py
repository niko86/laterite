"""A bare `#N` must mean an issue in THIS repo, and the gate must say so.

#458's finding was not a dead link — it was a link about to come alive wrong. A
satellite number written bare while this repo's highest was 457 resolved to
nothing, which is at least obviously broken. As the numbering climbed, each one
started resolving to a different, unrelated, plausible page, autolinked, with
nothing failing anywhere. It arrives on a schedule and it is silent.

Two things are pinned here, and the second matters as much as the first:

* the frozen set does its job — a known-foreign number written bare is caught,
  and the same number already qualified is not;
* the gate REPORTS what it could not judge. The set is narrow by construction
  (it cannot tell a new foreign ref from one of ours without the API), so a bare
  "OK" would read as "every reference checks out". That is CLAUDE.md's rule — a
  gate that drops input says what it dropped — and a report nobody asserts is
  the same silence one level up.

In-process on purpose: this runs in the buildless `repo-gates` job beside the
gate itself, so it must not shell out to a second interpreter.
"""

from __future__ import annotations

import importlib.util
import json
import re
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]
DATA = REPO / "foreign-issue-refs.json"


def _load():
    """Import `tools/check_issue_refs.py` — `tools/` is not a package."""
    spec = importlib.util.spec_from_file_location(
        "check_issue_refs", REPO / "tools" / "check_issue_refs.py"
    )
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules["check_issue_refs"] = mod
    spec.loader.exec_module(mod)
    return mod


gate = _load()


def test_the_tree_is_clean(capsys: pytest.CaptureFixture[str]) -> None:
    """The sweep's own assertion: no known-foreign number is bare in the tree."""
    assert gate.main() == 0, capsys.readouterr().out


def test_it_reports_what_it_could_not_judge(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """The scope statement, on a PASSING run — the direction that goes quiet."""
    assert gate.main() == 0
    out = capsys.readouterr().out
    match = re.search(r"(\d+) distinct number\(s\) not in the frozen set", out)
    assert match, f"no scope line in:\n{out}"
    assert int(match.group(1)) > 0, (
        "the gate implies it judged every bare ref in the tree; it cannot — the "
        "frozen set is a few dozen numbers and the tree cites hundreds"
    )
    assert "unjudged" in out


def test_it_counts_what_it_scanned(capsys: pytest.CaptureFixture[str]) -> None:
    gate.main()
    match = re.search(r"(\d+) bare `#N` scanned", capsys.readouterr().out)
    assert match, "the gate does not say how much it looked at"
    assert int(match.group(1)) > 0


@pytest.mark.parametrize("number", [475, 1327])
def test_a_bare_foreign_number_is_matched_and_a_qualified_one_is_not(
    number: int,
) -> None:
    """Falsifiability, driven at the matcher rather than by dirtying the tree.

    The second half is the one that would fail loudest: if the lookbehind broke,
    the gate would flag every reference this sweep just fixed.
    """
    expected = gate.load_expected()
    assert number in expected, f"#{number} is not in the frozen set"

    assert gate.BARE.findall(f"the thing landed in #{number}, see there") == [
        str(number)
    ]
    assert gate.BARE.findall(f"landed in {expected[number]}#{number}") == []


def test_the_eslint_hex_example_is_not_in_the_set() -> None:
    """`#1024` in web/eslint.config.js is an example inside a comment about
    telling issue refs apart from hex colours — #458 says do not "fix" it. It
    stays out of the frozen set rather than being carved out per-file."""
    doc = json.loads(DATA.read_text(encoding="utf-8"))
    numbers = set(doc["satellite"]["numbers"]) | {int(n) for n in doc["foreign"]}
    assert 1024 not in numbers
    assert "#1024" in (REPO / "web" / "eslint.config.js").read_text(encoding="utf-8")


def test_every_reserved_number_has_a_repo() -> None:
    doc = json.loads(DATA.read_text(encoding="utf-8"))
    assert doc["satellite"]["repo"]
    assert doc["satellite"]["numbers"]
    for number, repo in doc["foreign"].items():
        assert "/" in repo, f"#{number} needs an owner/repo, got {repo!r}"
