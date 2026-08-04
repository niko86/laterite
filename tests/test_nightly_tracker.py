"""Every branch of the nightly tracker's state machine.

The tracker is the thing that tells you the nightly broke, so its own failures
are silent by construction — a tracker that stops opening issues looks exactly
like a run of good nights. `plan()` is pure precisely so this file can hold it
to account without a GitHub API.

The cases that matter are the transitions, not the steady states: green with a
tracker open (the bug that left #245 sitting after its cause was fixed), and a
changed failure set (which must speak up rather than quietly rewrite the body).
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]


def _load_tracker():
    """Import `tools/nightly_tracker.py` as a module — `tools/` is not a package.
    Same shape as test_changelog_advisor's loader, so there is one way to do this."""
    spec = importlib.util.spec_from_file_location(
        "nightly_tracker", REPO / "tools" / "nightly_tracker.py"
    )
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules["nightly_tracker"] = mod
    spec.loader.exec_module(mod)
    return mod


nt = _load_tracker()
TITLE = nt.TITLE
failing_jobs, plan, read_state, render_body = (
    nt.failing_jobs,
    nt.plan,
    nt.read_state,
    nt.render_body,
)

RUN = "https://github.com/niko86/laterite/actions/runs/1"


def needs(**results: str) -> dict[str, dict[str, str]]:
    return {name: {"result": r} for name, r in results.items()}


def tracker(body: str, number: int = 245) -> dict[str, object]:
    return {"number": number, "title": TITLE, "body": body}


# --- which results count as failure ------------------------------------------


@pytest.mark.parametrize(
    ("result", "is_failing"),
    [
        ("success", False),
        ("skipped", False),  # a legitimate skip must not cry wolf
        ("failure", True),
        ("cancelled", True),
        ("", True),  # an unknown result must never read as fine
        ("timed_out", True),
    ],
)
def test_result_classification(result: str, *, is_failing: bool) -> None:
    assert bool(failing_jobs(needs(deny=result))) is is_failing


def test_failing_jobs_is_sorted() -> None:
    """The set is compared against last night's; ordering must not fake a change."""
    got = failing_jobs(needs(wiki="failure", coverage="failure", deny="success"))
    assert got == ["coverage", "wiki"]


# --- green ---------------------------------------------------------------------


def test_green_with_no_tracker_does_nothing() -> None:
    assert plan(needs(deny="success"), None, RUN)["action"] == "none"


def test_green_with_open_tracker_closes_it() -> None:
    """The #245 bug: the old step only ran on failure, so nothing ever closed."""
    action = plan(needs(deny="success", coverage="success"), tracker("whatever"), RUN)
    assert action["action"] == "close"
    assert action["number"] == 245
    assert RUN in action["comment"]


# --- first failure -------------------------------------------------------------


def test_first_failure_creates_the_tracker() -> None:
    action = plan(needs(deny="failure", coverage="success"), None, RUN)
    assert action["action"] == "create"
    assert action["title"] == TITLE
    assert "**Failing:** deny" in action["body"]
    assert "**Passing:** coverage" in action["body"]
    assert "first failure tonight" in action["body"]


# --- repeat failure ------------------------------------------------------------


def test_same_failure_updates_body_without_commenting() -> None:
    """A week of one broken gate should be one issue, not seven identical comments."""
    body = render_body(["deny"], ["coverage"], RUN, 1)
    action = plan(needs(deny="failure", coverage="success"), tracker(body), RUN)
    assert action["action"] == "update"
    assert "comment" not in action
    assert "failing 2 nights running" in action["body"]


def test_night_count_accumulates() -> None:
    body = render_body(["deny"], [], RUN, 6)
    action = plan(needs(deny="failure"), tracker(body), RUN)
    assert "failing 7 nights running" in action["body"]


# --- the failure set changing --------------------------------------------------


def test_a_new_failure_joining_is_worth_a_comment() -> None:
    body = render_body(["deny"], ["coverage"], RUN, 3)
    action = plan(needs(deny="failure", coverage="failure"), tracker(body), RUN)
    assert action["action"] == "update"
    assert "now also failing: coverage" in action["comment"]


def test_a_partial_fix_is_worth_a_comment() -> None:
    """Two failing, one fixed — the issue stays open but must say so."""
    body = render_body(["coverage", "deny"], [], RUN, 2)
    action = plan(needs(deny="failure", coverage="success"), tracker(body), RUN)
    assert "no longer failing: coverage" in action["comment"]
    assert "**Failing:** deny" in action["body"]


def test_a_swap_reports_both_directions() -> None:
    body = render_body(["coverage"], ["deny"], RUN, 1)
    action = plan(needs(deny="failure", coverage="success"), tracker(body), RUN)
    assert "now also failing: deny" in action["comment"]
    assert "no longer failing: coverage" in action["comment"]


# --- the state marker ----------------------------------------------------------


def test_state_round_trips_through_the_body() -> None:
    assert read_state(render_body(["a", "b"], ["c"], RUN, 4)) == (["a", "b"], 4)


def test_a_body_without_a_marker_reads_as_unknown() -> None:
    """Hand-edited or pre-tool bodies must not be mistaken for 'nothing failing',
    which would suppress the comment on a genuine change."""
    assert read_state("someone rewrote this by hand") == ([], 0)
    assert read_state(None) == ([], 0)


def test_unknown_previous_state_still_comments() -> None:
    action = plan(needs(deny="failure"), tracker("hand-written"), RUN)
    assert "now also failing: deny" in action["comment"]


def test_marker_survives_an_empty_failing_set_in_a_stale_body() -> None:
    """`jobs=` with nothing after it must parse as [], not ['']."""
    assert read_state("<!-- nightly-tracker: jobs= nights=2 -->") == ([], 2)
