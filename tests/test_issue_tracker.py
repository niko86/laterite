"""Every branch of the tracking-issue state machine, for both channels.

The tracker is the thing that tells you the nightly broke, so its own failures
are silent by construction — a tracker that stops opening issues looks exactly
like a run of good nights. `plan()` is pure precisely so this file can hold it
to account without a GitHub API.

The cases that matter are the transitions, not the steady states: green with a
tracker open (the bug that left #245 sitting after its cause was fixed), and a
changed failure set (which must speak up rather than quietly rewrite the body).

The `ext:` drift channel is the same machine over a different vocabulary. It is
exercised separately at the bottom rather than trusted to inherit: the point of
generalising was to give `wiki-ext-drift.yml` the closing and body-rewriting it
never had, and a shared `plan_items` proves nothing about whether the second
channel is actually wired to it.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]


def _load_tracker():
    """Import `tools/issue_tracker.py` as a module — `tools/` is not a package.
    Same shape as test_changelog_advisor's loader, so there is one way to do this."""
    spec = importlib.util.spec_from_file_location(
        "issue_tracker", REPO / "tools" / "issue_tracker.py"
    )
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules["issue_tracker"] = mod
    spec.loader.exec_module(mod)
    return mod


nt = _load_tracker()
TITLE = nt.TITLE
failing_jobs, plan, plan_items, read_state, render_body = (
    nt.failing_jobs,
    nt.plan,
    nt.plan_items,
    nt.read_state,
    nt.render_body,
)
MARKER = nt.NIGHTLY_MARKER

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


def test_a_docs_leg_failing_alone_opens_an_issue_that_names_it() -> None:
    """#493's fourth defect, from the tracker's side.

    Nothing here was ever wrong: `plan` reports whatever it is handed. It was
    handed seven of the nightly's twelve jobs, so a night where only a
    `docs-vs-released-*` leg failed produced no issue at all — a green tracker
    over a red run, indistinguishable from a good night. The dependency set is
    the other half and `tests/test_nightly_wiring.py` holds it; this is the half
    that says what happens once the leg does arrive."""
    action = plan(
        needs(**{"docs-vs-released-duckdb": "failure", "coverage": "success"}),
        None,
        RUN,
    )
    assert action["action"] == "create"
    assert "**Failing:** docs-vs-released-duckdb" in action["body"]


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
    assert read_state(render_body(["a", "b"], ["c"], RUN, 4), MARKER) == (
        ["a", "b"],
        4,
    )


def test_a_body_without_a_marker_reads_as_unknown() -> None:
    """Hand-edited or pre-tool bodies must not be mistaken for 'nothing failing',
    which would suppress the comment on a genuine change."""
    assert read_state("someone rewrote this by hand", MARKER) == ([], 0)
    assert read_state(None, MARKER) == ([], 0)


def test_unknown_previous_state_still_comments() -> None:
    action = plan(needs(deny="failure"), tracker("hand-written"), RUN)
    assert "now also failing: deny" in action["comment"]


def test_marker_survives_an_empty_failing_set_in_a_stale_body() -> None:
    """`items=` with nothing after it must parse as [], not ['']."""
    assert read_state("<!-- nightly-tracker: items= nights=2 -->", MARKER) == ([], 2)


def test_a_channel_cannot_read_another_channels_state() -> None:
    """Two trackers, one repo. Reading the wrong marker would have the ext: check
    inherit the nightly's night count and suppress its own change comment."""
    body = render_body(["deny"], [], RUN, 3)
    assert read_state(body, MARKER) == (["deny"], 3)
    assert read_state(body, nt.EXT_DRIFT_MARKER) == ([], 0)


# --- the ext: drift channel ----------------------------------------------------
#
# `wiki-ext-drift.yml` drove its issue from inline shell that create-or-commented:
# it never closed, never rewrote the body, and commented on every run. These are
# the four transitions it did not have, plus the one that must not be swallowed.

EXT = nt.EXT_DRIFT


def ext_tracker(body: str, number: int = 301) -> dict[str, object]:
    return {"number": number, "title": EXT.title, "body": body}


def test_ext_drift_opens_naming_what_is_missing() -> None:
    action = plan_items(["laterite/x:docs/a.md", "other/y"], None, RUN, EXT)
    assert action["action"] == "create"
    assert action["title"] == "wiki: ext: citation drift detected"
    assert "`laterite/x:docs/a.md`" in action["body"]
    assert "`other/y`" in action["body"]


def test_ext_drift_rewrites_the_body_without_commenting_when_unchanged() -> None:
    """The defect the inline shell had: a weekly comment saying the same thing."""
    body = EXT.body(["other/y"], RUN, 1)
    action = plan_items(["other/y"], ext_tracker(body), RUN, EXT)
    assert action["action"] == "update"
    assert "comment" not in action
    assert "seen 2 runs running" in action["body"]


def test_ext_drift_comments_only_when_the_set_changes() -> None:
    body = EXT.body(["other/y"], RUN, 2)
    action = plan_items(["other/y", "third/z"], ext_tracker(body), RUN, EXT)
    assert "now also missing: third/z" in action["comment"]
    assert "no longer missing" not in action["comment"]


def test_ext_drift_closes_itself_when_every_citation_resolves() -> None:
    """The half the inline shell never had — nothing could ever close that issue."""
    body = EXT.body(["other/y"], RUN, 4)
    action = plan_items([], ext_tracker(body), RUN, EXT)
    assert action["action"] == "close"
    assert action["number"] == 301
    assert RUN in action["comment"], "the closing note must name the clearing run"


def test_ext_drift_with_nothing_missing_and_no_issue_stays_quiet() -> None:
    assert plan_items([], None, RUN, EXT)["action"] == "none"


def test_a_lost_marker_costs_one_comment_never_a_swallowed_transition() -> None:
    """Someone edits the body and deletes the marker. The next run must read that
    as "no known state" and comment, not as "nothing was missing" and go silent."""
    action = plan_items(["other/y"], ext_tracker("hand-edited, marker gone"), RUN, EXT)
    assert action["action"] == "update"
    assert "now also missing: other/y" in action["comment"]
