#!/usr/bin/env python3
"""The nightly cut's three decisions, read off one release-status snapshot (#806).

The nightly job runs `release_status.py --json` ONCE and every later step reads
that file through this — so the tracker, the publish dispatch and the cut PR
all act on the same registry sweep rather than three sweeps that can disagree
mid-run. Every function here is pure over the status dict; `main()` is the only
line that touches the world.

The modes map one-to-one onto the job's steps:

* `--items`   — tracker tokens for `issue_tracker.py --tracker release`, one per
  line. Tokens, not sentences: the tracker's state marker is space-hostile, and
  a stable token set is what keeps a night with no news from commenting. Exits
  3 when there is nothing to report AND at least one crate concluded nothing —
  closing the tracker on partial knowledge would be claiming an all-clear the
  registry never gave.
* `--bumps`   — `<crate> <part>` rows for the PR mode's bump loop.
* `--publish-owed` — the crates whose stamp should be on the registry and is
  not; any output means "cancel any stale queued run and dispatch a fresh one".
* `--render`  — the human report + the cut view, for the step summary.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))  # not a package
import release_status as rs

#: "nothing to report, but not nothing owed" — the caller must leave the
#: tracker untouched rather than let an empty item list close it.
EXIT_UNCONCLUDED = 3


def tokens(status: dict) -> list[str]:
    """One space-free token per owed act, stable across nights until acted on."""
    out = []
    for c in status["engine_crates"]:
        action = c["cut_action"]
        if action == "bump":
            out.append(f"{c['crate']}:bump-{c['part_required']}")
        elif action == "publish":
            out.append(f"{c['crate']}:publish-{c['version']}")
        elif action == "human":
            out.append(f"{c['crate']}:human")
    return out


def bumps(status: dict) -> list[tuple[str, str]]:
    return [
        (c["crate"], c["part_required"])
        for c in status["engine_crates"]
        if c["cut_action"] == "bump"
    ]


def publish_owed(status: dict) -> list[str]:
    return [c["crate"] for c in status["engine_crates"] if c["cut_action"] == "publish"]


def unconcluded(status: dict) -> list[str]:
    return [
        c["crate"] for c in status["engine_crates"] if c["cut_action"] == "unconcluded"
    ]


def main() -> int:
    if len(sys.argv) != 3 or sys.argv[1] not in (
        "--items",
        "--bumps",
        "--publish-owed",
        "--render",
    ):
        print(
            "usage: engine_cut.py --items|--bumps|--publish-owed|--render status.json",
            file=sys.stderr,
        )
        return 2
    mode, path = sys.argv[1], Path(sys.argv[2])
    status = json.loads(path.read_text())

    if mode == "--render":
        print(rs.render(status))
        print()
        print(rs.render_cut(status))
        return 0
    if mode == "--items":
        got = tokens(status)
        dark = unconcluded(status)
        if not got and dark:
            print(
                f"concluded nothing for {', '.join(dark)} and found nothing owed "
                "elsewhere — refusing to clear the tracker on partial knowledge",
                file=sys.stderr,
            )
            return EXIT_UNCONCLUDED
        for token in got:
            print(token)
        return 0
    if mode == "--bumps":
        for crate, part in bumps(status):
            print(f"{crate} {part}")
        return 0
    for crate in publish_owed(status):
        print(crate)
    return 0


if __name__ == "__main__":
    sys.exit(main())
