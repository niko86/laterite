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


def _exemplars() -> list[int]:
    """One satellite number and one third-party number, taken FROM the set.

    These were written as literals — `[475, 1327]` — and 475 was claimed by this
    repo four months later, at which point the test asserted that a number the
    set had correctly released was still in it. The exemplar is now the LOWEST
    satellite number STILL HELD, which is the one nearest to being claimed next
    and therefore the most useful canary; the third-party numbers belong to
    other projects and cannot be claimed here at all, so the lowest of those is
    a stable pick.

    "Still held" has to be read through the watermark since #498, not off the
    JSON: the list holds every satellite number ever cited here, released ones
    included, so `min()` over the raw file is once again a number this repo owns
    — the same trap the literals fell into, one layer along.
    """
    doc = json.loads(DATA.read_text(encoding="utf-8"))
    held, _ = gate.load_expected(gate.resolve_watermark()[0])
    satellite = [n for n in doc["satellite"]["numbers"] if n in held]
    assert satellite, "every satellite number has been claimed — pick a new canary"
    return [min(satellite), min(int(n) for n in doc["foreign"])]


@pytest.mark.parametrize("number", _exemplars())
def test_a_bare_foreign_number_is_matched_and_a_qualified_one_is_not(
    number: int,
) -> None:
    """Falsifiability, driven at the matcher rather than by dirtying the tree.

    The second half is the one that would fail loudest: if the lookbehind broke,
    the gate would flag every reference this sweep just fixed.
    """
    expected, _ = gate.load_expected(gate.resolve_watermark()[0])
    assert number in expected, f"#{number} is not in the effective frozen set"

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


# --- the watermark (#498) ----------------------------------------------------
#
# The set used to be pruned by hand, and the rule that says when to prune was
# written in the file's own comment with nothing executing it. Three edits in one
# sitting is what that costs; each of the three released a number this repo had
# just been allocated, and until it was made the gate FAILED a correct reference
# for being a number that used to mean somewhere else.


@pytest.mark.parametrize(
    ("subject", "expected"),
    [
        # The convention: `(#issue) (#pr)`. The PR is last, and it is the only
        # one this repo has certainly issued — an issue number can be cited by a
        # commit long before the repo's own numbering reaches it.
        ("fix(web): an aggregated bar folds its tail in SQL (#457) (#496)", 496),
        ("ci: `code` lists the tools a gated job runs (#494) (#495)", 495),
        # A PR merged without an issue.
        ("Build the shared primitives in Solid, and vendor the icon set (#418)", 418),
        # Not a merged subject at all.
        ("wip: halfway through the thing", None),
        ("docs: mentions #499 in passing but does not end with it", None),
        # Trailing whitespace survives; a trailing period does not make it one.
        ("chore: bump deps (#480)  ", 480),
        ("chore: see (#480).", None),
    ],
)
def test_only_the_trailing_number_is_read_as_a_merged_pr(
    subject: str, expected: int | None
) -> None:
    match = gate.MERGED_PR.search(subject)
    assert (int(match.group(1)) if match else None) == expected


def test_a_number_at_or_below_the_watermark_is_released() -> None:
    """The whole mechanism, at the boundary. `#N` where N == the watermark is
    ours — the watermark IS a number this repo issued, not one below it."""
    doc = json.loads(DATA.read_text(encoding="utf-8"))
    n = min(doc["satellite"]["numbers"])

    frozen, released = gate.load_expected(watermark=n)
    assert n not in frozen and n in released

    frozen, released = gate.load_expected(watermark=n - 1)
    assert n in frozen and n not in released


def test_the_override_and_git_take_whichever_is_higher(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """`claimed_through` exists for the one thing git cannot see — a freshly-filed
    issue whose number no commit carries yet. It must not be able to LOWER the
    watermark, or a forgotten override would re-freeze numbers the repo owns."""
    data = tmp_path / "foreign-issue-refs.json"
    monkeypatch.setattr(gate, "DATA", data)

    def declared(value: int | None) -> None:
        doc = {"satellite": {"repo": "x", "numbers": [9]}, "foreign": {}}
        if value is not None:
            doc["claimed_through"] = value
        data.write_text(json.dumps(doc), encoding="utf-8")

    monkeypatch.setattr(gate, "watermark_from_git", lambda: (500, "origin/main"))

    declared(None)
    assert gate.resolve_watermark()[0] == 500
    declared(400)
    assert gate.resolve_watermark()[0] == 500, "an override must never lower it"
    declared(600)
    assert gate.resolve_watermark()[0] == 600

    # No git at all — the override is the only authority left, and the run says so.
    monkeypatch.setattr(gate, "watermark_from_git", lambda: None)
    number, how = gate.resolve_watermark()
    assert number == 600
    assert "claimed_through" in how


def test_no_git_and_no_override_releases_nothing(
    monkeypatch: pytest.MonkeyPatch, tmp_path: Path
) -> None:
    """Fail toward the old behaviour, not toward silence: with no way to know what
    this repo owns, every frozen number stays frozen. That can only cost a false
    positive, where guessing high would let a genuinely foreign number through."""
    data = tmp_path / "foreign-issue-refs.json"
    data.write_text(
        json.dumps({"satellite": {"repo": "x", "numbers": [9]}, "foreign": {}}),
        encoding="utf-8",
    )
    monkeypatch.setattr(gate, "DATA", data)
    monkeypatch.setattr(gate, "watermark_from_git", lambda: None)

    number, how = gate.resolve_watermark()
    assert number == 0
    assert "nothing released" in how
    assert gate.load_expected(number)[1] == []


def test_a_shallow_clone_refuses_instead_of_reading_a_low_watermark(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    """The failure that would otherwise land on an innocent `#N`.

    A shallow checkout has no merged-PR history, so the watermark reads low and
    the gate rejects correct references to recent numbers — the exact bug #498
    fixed, restored by a checkout setting. It must name the checkout instead."""
    monkeypatch.setattr(gate, "is_shallow", lambda: True)
    assert gate.main() == 1
    err = capsys.readouterr().err
    assert "SHALLOW" in err
    assert "fetch-depth: 0" in err


def test_the_run_reports_the_watermark_it_used(
    capsys: pytest.CaptureFixture[str],
) -> None:
    """Same rule as the unjudged count above it: the watermark decides what this
    run did NOT look at, and unlike the frozen set it moves on its own between
    runs. A watermark nobody can see is the same blind spot one layer down."""
    gate.main()
    out = capsys.readouterr().out
    assert "watermark #" in out
    assert "released as this repo's own" in out


def test_this_repo_has_actually_reached_the_numbers_it_released() -> None:
    """The claim the watermark makes, held against the real history rather than
    against itself: every released number is at or below a PR number that is
    genuinely merged here."""
    watermark, _ = gate.resolve_watermark()
    found = gate.watermark_from_git()
    assert found is not None, "no merged history — see the shallow-clone test"
    assert watermark >= found[0]
    _, released = gate.load_expected(watermark)
    assert all(n <= watermark for n in released)
