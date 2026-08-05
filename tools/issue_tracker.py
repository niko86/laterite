#!/usr/bin/env python3
"""Drive one tracking issue from one set of strings, for any reporting channel.

Two channels use it today: the nightly's failing jobs, and the weekly `ext:`
citation-drift check's confirmed-missing refs. They are the same state machine
over a different vocabulary, so `plan()` holds the transitions and a `Tracker`
holds every string. The file was `nightly_tracker.py` while there was one
channel; the name was the last thing here still saying so.

The nightly half is what the shape was learned from.

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

`plan_items()` is pure and holds every decision; `main()` only executes what it
returns. That split is what makes the behaviour testable without a GitHub API —
see tests/test_issue_tracker.py.

The second channel is what made the generalisation worth doing rather than
theoretical: `wiki-ext-drift.yml` drove its issue from inline shell that
create-or-commented and had all three defects listed above — it never closed,
never rewrote the body, and commented on every run. It now calls this.

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
from typing import TYPE_CHECKING, Any, NamedTuple

if TYPE_CHECKING:
    from collections.abc import Callable


class Tracker(NamedTuple):
    """One channel's vocabulary. Every string a channel needs is here, so adding
    a second is a config rather than a fork of the state machine.

    `body` takes (items, run_url, nights); `changed` takes (gained, lost,
    run_url); `closed` takes (run_url).
    """

    title: str
    marker: str
    body: Callable[[list[str], str, int], str]
    changed: Callable[[list[str], list[str], str], str]
    closed: Callable[[str], str]


TITLE = "Nightly CI failing"

#: Results that mean "this job did not pass". `skipped` is NOT one: a nightly
#: job can legitimately skip, and treating that as failure would make the
#: tracker cry wolf. Anything unrecognised counts as failing — an unknown result
#: must never read as "fine".
_PASSING = frozenset({"success", "skipped"})


#: Machine-readable state, parked in an HTML comment at the end of the body.
#: The body is prose for a human; this is what the next run reads back, so the
#: two can never disagree about what was failing last night. Keyed by the
#: channel's marker so two trackers cannot read each other's state.
def _marker_re(marker: str) -> re.Pattern[str]:
    return re.compile(
        rf"<!--\s*{re.escape(marker)}:\s*items=(?P<items>[^\s]*)"
        r"\s+nights=(?P<nights>\d+)\s*-->"
    )


def state_marker(marker: str, items: list[str], nights: int) -> str:
    return f"<!-- {marker}: items={','.join(items)} nights={nights} -->"


def failing_jobs(needs: dict[str, Any]) -> list[str]:
    """The jobs that did not pass, sorted so the set compares stably."""
    return sorted(
        name
        for name, data in needs.items()
        if (data or {}).get("result") not in _PASSING
    )


def read_state(body: str | None, marker: str) -> tuple[list[str], int]:
    """Recover (items, consecutive runs) from a tracker body.

    A body without the marker — hand-written, or from before this tool — reads as
    "no known state", so the next run treats its contents as a change and
    comments. Erring toward one extra comment beats silently swallowing a
    transition.
    """
    if not body:
        return [], 0
    m = _marker_re(marker).search(body)
    if not m:
        return [], 0
    items = [j for j in m.group("items").split(",") if j]
    return sorted(items), int(m.group("nights"))


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
            state_marker(NIGHTLY_MARKER, failing, nights),
        ]
    )


NIGHTLY_MARKER = "nightly-tracker"


def nightly(passing: list[str]) -> Tracker:
    """The nightly channel. `passing` is captured because only this channel has a
    meaningful complement to report — the ext: check's would be every citation
    in the vault."""
    return Tracker(
        title=TITLE,
        marker=NIGHTLY_MARKER,
        body=lambda items, run_url, nights: render_body(
            items, passing, run_url, nights
        ),
        changed=lambda gained, lost, run_url: _changed_line(
            gained, lost, run_url, "failing"
        ),
        closed=lambda run_url: (
            f"Recovered — every nightly job passed in {run_url}. Closing."
        ),
    )


EXT_DRIFT_MARKER = "ext-drift-tracker"


def _ext_body(items: list[str], run_url: str, nights: int) -> str:
    weeks = "first run to see it" if nights <= 1 else f"**seen {nights} runs running**"
    return "\n".join(
        [
            f"The weekly `ext:` check found {len(items)} confirmed-missing "
            f"citation(s) — {weeks}.",
            "",
            *(f"- `{i}`" for i in items),
            "",
            f"**Latest run:** {run_url}",
            "",
            "Confirmed missing means a real 404 from the GitHub API. A timeout, "
            "rate-limit or auth hiccup is reported as `unknown` and never reaches "
            "this issue — network flakiness must not read as a citation going "
            "stale.",
            "",
            "This body is rewritten by each run, so it always shows the CURRENT "
            "set; comments below mark only the runs where it changed. The issue "
            "closes itself when every citation resolves again.",
            "",
            state_marker(EXT_DRIFT_MARKER, items, nights),
        ]
    )


EXT_DRIFT = Tracker(
    title="wiki: ext: citation drift detected",
    marker=EXT_DRIFT_MARKER,
    body=_ext_body,
    changed=lambda gained, lost, run_url: _changed_line(
        gained, lost, run_url, "missing"
    ),
    closed=lambda run_url: (
        f"Every `ext:` citation resolves again as of {run_url}. Closing."
    ),
)


def _changed_line(gained: list[str], lost: list[str], run_url: str, word: str) -> str:
    parts = []
    if gained:
        parts.append(f"now also {word}: {', '.join(gained)}")
    if lost:
        parts.append(f"no longer {word}: {', '.join(lost)}")
    return f"What's {word} changed — {'; '.join(parts)}. {run_url}"


def plan_items(
    items: list[str],
    issue: dict[str, Any] | None,
    run_url: str,
    spec: Tracker,
) -> dict[str, Any]:
    """Decide what to do. Pure — every branch of the tracker's behaviour is here.

    Returns an action dict: {"action": one of none|create|update|close, ...}.
    `update` may carry a `comment`; `close` always does, so the closing note says
    which run cleared it rather than leaving a bare state change.
    """
    items = sorted(items)

    if not items:
        if issue is None:
            return {"action": "none", "why": f"nothing to report for {spec.title!r}"}
        return {
            "action": "close",
            "number": issue["number"],
            "comment": spec.closed(run_url),
        }

    prev, prev_nights = read_state(issue.get("body") if issue else None, spec.marker)
    nights = prev_nights + 1

    if issue is None:
        return {
            "action": "create",
            "title": spec.title,
            "body": spec.body(items, run_url, nights),
        }

    action: dict[str, Any] = {
        "action": "update",
        "number": issue["number"],
        "body": spec.body(items, run_url, nights),
    }
    if items != prev:
        action["comment"] = spec.changed(
            sorted(set(items) - set(prev)), sorted(set(prev) - set(items)), run_url
        )
    return action


def plan(
    needs: dict[str, Any],
    issue: dict[str, Any] | None,
    run_url: str,
) -> dict[str, Any]:
    """The nightly channel's entry point — unchanged signature on purpose.

    The transitions moved to `plan_items`; this still owns turning job results
    into a set of names, which is the only part specific to the nightly.
    """
    failing = failing_jobs(needs)
    return plan_items(
        failing, issue, run_url, nightly(sorted(set(needs) - set(failing)))
    )


# --- execution ---------------------------------------------------------------


def _gh(*args: str, check: bool = True) -> str:
    return subprocess.run(
        ["gh", *args], capture_output=True, text=True, check=check
    ).stdout.strip()


def _find_issue(repo: str, title: str) -> dict[str, Any] | None:
    """One search, then exact-title equality locally.

    The search narrows; it does not decide. `in:title` is a fuzzy match, so a
    differently-titled issue mentioning the same words would otherwise be
    mistaken for the tracker and rewritten.
    """
    raw = _gh(
        "issue",
        "list",
        "--repo",
        repo,
        "--state",
        "open",
        "--search",
        f'in:title "{title}"',
        "--json",
        "number,title,body",
    )
    for item in json.loads(raw or "[]"):
        if item.get("title") == title:
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


def _nightly_action(run_url: str, repo: str) -> dict[str, Any] | None:
    needs = json.loads(os.environ.get("NEEDS", "{}"))
    # `notify` depends on every other job, so it appears in nothing; but guard
    # anyway — a tracker that reports on itself would be nonsense.
    needs.pop("notify", None)
    if not needs:
        print("no job results supplied — refusing to act", file=sys.stderr)
        return None
    return plan(needs, _find_issue(repo, TITLE), run_url)


def _ext_drift_action(run_url: str, repo: str) -> dict[str, Any]:
    """Items come from `ITEMS`, one per line.

    An EMPTY `ITEMS` is a real value here, not a missing one: it is what closes
    the issue. That is the opposite of the nightly, where no job results means
    the workflow is broken and acting on the silence would be worse than
    refusing — hence the two entry points rather than one shared guard.
    """
    items = [ln.strip() for ln in os.environ.get("ITEMS", "").splitlines()]
    return plan_items(
        [i for i in items if i], _find_issue(repo, EXT_DRIFT.title), run_url, EXT_DRIFT
    )


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--tracker",
        choices=("nightly", "ext-drift"),
        default="nightly",
        help="which channel to drive (default: nightly)",
    )
    ap.add_argument("--dry-run", action="store_true", help="print the plan, do nothing")
    args = ap.parse_args()

    repo = os.environ["GITHUB_REPOSITORY"]
    run_url = os.environ.get("RUN_URL", "")

    # A dry run still LOOKS the tracker up: the interesting half of the plan is
    # which transition fires, and that depends entirely on what is already open.
    # Printing a plan computed against `None` would rehearse a case that isn't
    # the one about to run.
    if args.tracker == "nightly":
        action = _nightly_action(run_url, repo)
        if action is None:
            return 1
    else:
        action = _ext_drift_action(run_url, repo)

    if args.dry_run:
        print(json.dumps(action, indent=2))
        return 0

    _execute(action, repo)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
