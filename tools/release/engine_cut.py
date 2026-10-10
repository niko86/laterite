#!/usr/bin/env python3
"""The nightly cut's three decisions, read off one release-status snapshot (#806).

The nightly job runs `release_status.py --json` ONCE and every later step reads
that file through this — so the tracker, the publish dispatch and the cut PR
all act on the same registry sweep rather than three sweeps that can disagree
mid-run. Every function here is pure over the typed report (`rs.Report`, #925);
`main()` is the only line that touches the world.

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
* `--cascade` — after the owed bumps are applied, patch every crate whose
  published pins the moved floors left behind, round by round until the
  coherence check is clean (#1043). Prints the PR body's cascade section;
  exits 4, naming the crates still owed, when the loop stops making progress.
  It reads the tree as it stands, so it takes no snapshot file.
"""

from __future__ import annotations

import functools
import json
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import TYPE_CHECKING

sys.path.insert(0, str(Path(__file__).resolve().parent))  # not a package
import release_status as rs

if TYPE_CHECKING:
    from collections.abc import Callable

#: "nothing to report, but not nothing owed" — the caller must leave the
#: tracker untouched rather than let an empty item list close it.
EXIT_UNCONCLUDED = 3

#: the cascade stopped making progress — the job must fail before the PR step.
EXIT_CASCADE_STUCK = 4


def tokens(status: rs.Report) -> list[str]:
    """One space-free token per owed act, stable across nights until acted on."""
    out = []
    for c in status.engine_crates:
        if c.cut_action == "bump":
            out.append(f"{c.crate}:bump-{c.part_required}")
        elif c.cut_action == "publish":
            out.append(f"{c.crate}:publish-{c.version}")
        elif c.cut_action == "human":
            out.append(f"{c.crate}:human")
    return out


def bumps(status: rs.Report) -> list[tuple[str, str]]:
    return [
        (c.crate, c.part_required)
        for c in status.engine_crates
        if c.cut_action == "bump"
    ]


def publish_owed(status: rs.Report) -> list[str]:
    return [c.crate for c in status.engine_crates if c.cut_action == "publish"]


def unconcluded(status: rs.Report) -> list[str]:
    return [c.crate for c in status.engine_crates if c.cut_action == "unconcluded"]


@dataclass(frozen=True)
class CascadeBump:
    """One patch the coherence cascade applied, and the moved floor behind it."""

    crate: str
    round: int
    because: tuple[str, ...]


class CascadeStuck(Exception):
    """The cascade cannot finish: it would loop, or ran out of rounds."""

    def __init__(self, demanded: list[str], why: str) -> None:
        self.demanded = demanded
        super().__init__(f"{why}; still demanded: {', '.join(demanded)}")


def cascade_limit() -> int:
    """One round per published engine crate.

    A round that makes progress bumps at least one crate no earlier round did,
    so a cascade over N crates that has not settled after N rounds never will.
    """
    return sum(1 for c in rs.engine_crates() if rs.release_tier(c) == "engine")


def cascade(
    stranded: Callable[[], list[tuple[str, str]]],
    bump: Callable[[str], None],
    limit: int,
) -> list[CascadeBump]:
    """Patch every crate the coherence check names until it names none (#1043).

    A patch moves the bumped crate's own floor, which can strand the crates
    pinning IT — #1031 walked two such rounds by hand, and the nightly rebuild
    threw the hand-applied bumps away each night. `stranded` is the check's
    own reading, so the check stays the one authority on what is owed.

    A crate demanded again after its bump did not clear it, and bumping it
    twice will not either: a round that adds no new crate, or a cascade still
    owed after `limit` rounds, raises rather than looping or opening a PR.
    """
    applied: list[CascadeBump] = []
    bumped: set[str] = set()
    for round_no in range(1, limit + 2):
        debt = stranded()
        if not debt:
            return applied
        reasons: dict[str, list[str]] = {}
        for crate, reason in debt:
            reasons.setdefault(crate, []).append(reason)
        demanded = sorted(reasons)
        if round_no > limit:
            raise CascadeStuck(demanded, f"round limit {limit} reached")
        new = [c for c in demanded if c not in bumped]
        if not new:
            raise CascadeStuck(demanded, f"round {round_no} added no new crate")
        for crate in new:
            bump(crate)
            bumped.add(crate)
            applied.append(CascadeBump(crate, round_no, tuple(reasons[crate])))
    raise AssertionError("unreachable: the loop returns or raises")


def render_cascade(applied: list[CascadeBump]) -> str:
    """The cut PR body's cascade section: kept apart from the owed bumps."""
    if not applied:
        return ""
    lines = [
        "coherence cascade (#1043) — patches, each because a moved floor left "
        "its published pins behind:"
    ]
    lines.extend(
        f"  {b.crate} patch   (round {b.round}: {'; '.join(b.because)})"
        for b in applied
    )
    return "\n".join(lines)


def _bump_patch(crate: str) -> None:
    # bump_crate.py, not a hand edit: it moves every spelling of the version
    # and regenerates the lock, which is what the PR then carries.
    subprocess.run(
        [
            sys.executable,
            str(Path(__file__).with_name("bump_crate.py")),
            crate,
            "patch",
        ],
        check=True,
    )


def main() -> int:
    if sys.argv[1:] == ["--cascade"]:
        # Absolute reading (no base): standing debt is the cut's to fix too.
        # The registry does not move mid-run, so each crate is asked once.
        fetch = functools.cache(rs.fetch_index)

        def stranded() -> list[tuple[str, str]]:
            reading = rs.coherence_reading(fetch, None)
            # Every round, clean or not: a registry that answered for nobody
            # would otherwise read as "nothing stranded" and open the PR.
            print(
                f"cascade: {reading.asked} published engine crate(s) asked, "
                f"{reading.unreachable} unreachable"
                + (" — concluding nothing for those" if reading.unreachable else ""),
                file=sys.stderr,
            )
            return reading.introduced

        try:
            applied = cascade(stranded, _bump_patch, cascade_limit())
        except CascadeStuck as stuck:
            print(f"cascade stuck: {stuck}", file=sys.stderr)
            return EXIT_CASCADE_STUCK
        text = render_cascade(applied)
        if text:
            print(text)
        return 0
    if len(sys.argv) != 3 or sys.argv[1] not in (
        "--items",
        "--bumps",
        "--publish-owed",
        "--render",
    ):
        print(
            "usage: engine_cut.py --items|--bumps|--publish-owed|--render status.json"
            " | --cascade",
            file=sys.stderr,
        )
        return 2
    mode, path = sys.argv[1], Path(sys.argv[2])
    # Rehydrate into the typed report the accessors and renderers take — the
    # wire file stays plain JSON (nightly.yml wrote it with `--json`), and a
    # key it does not carry fails HERE, loudly, not as a wrong empty answer
    # three steps later.
    status = rs.Report.from_json(json.loads(path.read_text()))

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
