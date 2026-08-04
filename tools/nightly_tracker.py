#!/usr/bin/env python3
"""Drive the one `Nightly CI failing` tracking issue from a nightly run's results.

The nightly's `notify` step used to do three things badly:

  * it never CLOSED. A fixed nightly left the tracker open, and because the step
    comments when a tracker already exists, the next failure appended a quiet
    comment instead of opening an issue anyone would notice. #245 sat open after
    its cause was fixed for exactly this reason.
  * it commented every single night, identically. A week of the same failure was
    seven copies of the same sentence, and the issue never said what was failing
    NOW — only what had failed each time.
  * it listed the seven job names THREE times (`needs:`, the `if:` expression,
    and `env:`). Adding a nightly job and missing one copy silently drops it from
    the tracker — a gate that reports on less than it guards, which is #207's
    shape. The job list now comes from `toJSON(needs)`, so there is one copy.

So the tracker is a state machine over one issue: the BODY is current state
(rewritten each night), comments are reserved for transitions worth reading, and
green closes it.

`plan()` is pure and holds every decision; `main()` only executes what it
returns. That split is what makes the behaviour testable without a GitHub API —
see tests/test_nightly_tracker.py.

Stdlib only, on purpose: the notify job runs on a bare ubuntu-latest with no
Python or uv setup step, and adding one to post an issue comment would be a poor
trade.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from typing import Any

TITLE = "Nightly CI failing"

#: Results that mean "this job did not pass". `skipped` is NOT one: a nightly
#: job can legitimately skip, and treating that as failure would make the
#: tracker cry wolf. Anything unrecognised counts as failing — an unknown result
#: must never read as "fine".
_PASSING = frozenset({"success", "skipped"})

#: Machine-readable state, parked in an HTML comment at the end of the body.
#: The body is prose for a human; this is what the next run reads back, so the
#: two can never disagree about what was failing last night.
_MARKER = re.compile(
    r"<!--\s*nightly-tracker:\s*jobs=(?P<jobs>[^\s]*)\s+nights=(?P<nights>\d+)\s*-->"
)


def failing_jobs(needs: dict[str, Any]) -> list[str]:
    """The jobs that did not pass, sorted so the set compares stably."""
    return sorted(
        name
        for name, data in needs.items()
        if (data or {}).get("result") not in _PASSING
    )


def read_state(body: str | None) -> tuple[list[str], int]:
    """Recover (failing jobs, consecutive nights) from a tracker body.

    A body without the marker — hand-written, or from before this tool — reads as
    "no known state", so the next run treats its failure as a change and comments.
    Erring toward one extra comment beats silently swallowing a transition.
    """
    if not body:
        return [], 0
    m = _MARKER.search(body)
    if not m:
        return [], 0
    jobs = [j for j in m.group("jobs").split(",") if j]
    return sorted(jobs), int(m.group("nights"))


def render_body(
    failing: list[str], passing: list[str], run_url: str, nights: int
) -> str:
    nights_line = (
        "first failure tonight"
        if nights <= 1
        else f"**failing {nights} nights running**"
    )
    return "\n".join(
        [
            f"The nightly is failing — {nights_line}.",
            "",
            f"**Failing:** {', '.join(failing)}",
            f"**Passing:** {', '.join(passing) if passing else '(none)'}",
            f"**Latest run:** {run_url}",
            "",
            "This body is rewritten by each nightly run, so it always shows the "
            "CURRENT state; comments below mark only the nights where what's "
            "failing actually changed. The issue closes itself when the nightly "
            "goes green.",
            "",
            f"<!-- nightly-tracker: jobs={','.join(failing)} nights={nights} -->",
        ]
    )


def plan(
    needs: dict[str, Any],
    issue: dict[str, Any] | None,
    run_url: str,
) -> dict[str, Any]:
    """Decide what to do. Pure — every branch of the tracker's behaviour is here.

    Returns an action dict: {"action": one of none|create|update|close, ...}.
    `update` may carry a `comment`; `close` always does, so the closing note says
    which run cleared it rather than leaving a bare state change.
    """
    failing = failing_jobs(needs)
    passing = sorted(set(needs) - set(failing))

    if not failing:
        if issue is None:
            return {"action": "none", "why": "nightly green, no tracker open"}
        return {
            "action": "close",
            "number": issue["number"],
            "comment": f"Recovered — every nightly job passed in {run_url}. Closing.",
        }

    prev_jobs, prev_nights = read_state(issue.get("body") if issue else None)
    nights = prev_nights + 1

    if issue is None:
        return {
            "action": "create",
            "title": TITLE,
            "body": render_body(failing, passing, run_url, nights),
        }

    action: dict[str, Any] = {
        "action": "update",
        "number": issue["number"],
        "body": render_body(failing, passing, run_url, nights),
    }
    if failing != prev_jobs:
        gained = sorted(set(failing) - set(prev_jobs))
        fixed = sorted(set(prev_jobs) - set(failing))
        parts = []
        if gained:
            parts.append(f"now also failing: {', '.join(gained)}")
        if fixed:
            parts.append(f"no longer failing: {', '.join(fixed)}")
        action["comment"] = f"What's failing changed — {'; '.join(parts)}. {run_url}"
    return action


# --- execution ---------------------------------------------------------------


def _gh(*args: str, check: bool = True) -> str:
    return subprocess.run(
        ["gh", *args], capture_output=True, text=True, check=check
    ).stdout.strip()


def _find_issue(repo: str) -> dict[str, Any] | None:
    raw = _gh(
        "issue",
        "list",
        "--repo",
        repo,
        "--state",
        "open",
        "--search",
        f'in:title "{TITLE}"',
        "--json",
        "number,title,body",
    )
    for item in json.loads(raw or "[]"):
        if item.get("title") == TITLE:
            return item
    return None


def _execute(action: dict[str, Any], repo: str) -> None:
    kind = action["action"]
    if kind == "none":
        print(f"nothing to do — {action['why']}")
        return
    if kind == "create":
        _gh(
            "issue",
            "create",
            "--repo",
            repo,
            "--title",
            action["title"],
            "--body",
            action["body"],
        )
        print("opened the nightly tracker")
        return
    number = str(action["number"])
    if kind in ("update", "close"):
        if "body" in action:
            _gh("issue", "edit", number, "--repo", repo, "--body", action["body"])
        if "comment" in action:
            _gh("issue", "comment", number, "--repo", repo, "--body", action["comment"])
        if kind == "close":
            _gh("issue", "close", number, "--repo", repo)
        print(f"{kind}d #{number}")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--dry-run", action="store_true", help="print the plan, do nothing")
    args = ap.parse_args()

    needs = json.loads(os.environ.get("NEEDS", "{}"))
    # `notify` depends on every other job, so it appears in nothing; but guard
    # anyway — a tracker that reports on itself would be nonsense.
    needs.pop("notify", None)
    if not needs:
        print("no job results supplied — refusing to act", file=sys.stderr)
        return 1

    repo = os.environ["GITHUB_REPOSITORY"]
    run_url = os.environ.get("RUN_URL", "")

    # A dry run still LOOKS the tracker up: the interesting half of the plan is
    # which transition fires, and that depends entirely on what is already open.
    # Printing a plan computed against `None` would rehearse a case that isn't
    # the one about to run.
    action = plan(needs, _find_issue(repo), run_url)
    if args.dry_run:
        print(json.dumps(action, indent=2))
        return 0

    _execute(action, repo)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
