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
  Exits 5 with no output when any stamp is under-bumped, any crate went
  unmeasured (#1147), or an owed stamp has no publish commit to measure from
  (#1155): the publisher uploads every stamp the registry lacks, so one bad
  stamp refuses the whole set. A first publish has nothing to measure from and
  does not refuse.
* `--render`  — the human report + the cut view, for the step summary.
* `--cascade` — after the owed bumps are applied, bump every engine crate whose
  published pins the moved floors left behind (a patch, or a minor when its
  API names the moved dep), round by round until the coherence check is clean
  (#1043). Prints the PR body's cascade section, plus any product release the
  stranded facade owes — never bumped here (#1073); exits 4, naming the crates
  still owed, when the loop stops making progress. It reads the tree as it
  stands, so it takes no snapshot file.
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

#: a stamp falls short of its required part, or a crate could not be
#: measured — nothing is dispatched (#1147).
EXIT_PUBLISH_REFUSED = 5


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
        elif c.tier == "product" and c.deps_behind:
            # The facade's stranded pins (#1073): no cut act pays them, so the
            # tracker is where the owed product release stays visible.
            out.append(f"{c.crate}:product-release-owed")
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
    """One bump the coherence cascade applied, and the moved floor behind it."""

    crate: str
    round: int
    because: tuple[str, ...]
    part: str = "patch"


def cascade_part(snapshot: str, because: tuple[str, ...] | list[str]) -> str:
    """'minor' when the stranded crate's API names a dep whose floor moved (#1073).

    A stranded pin is by construction an incompatible move, so this is
    `rs.public_dep_moves`' rule applied at cascade time: a patch here would
    leave the next night's cut owing a minor on top of it. Each reason
    starts with the moved dep's name (`rs.deps_behind` writes them).
    """
    deps = [reason.split(" ", 1)[0] for reason in because]
    return "minor" if any(rs.crate_path(d) in snapshot for d in deps) else "patch"


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
    bump: Callable[[str, str], None],
    limit: int,
    part: Callable[[str, tuple[str, ...]], str] = lambda crate, because: "patch",
) -> list[CascadeBump]:
    """Bump every crate the coherence check names until it names none (#1043).

    `part` picks each bump's part from the crate and its reasons — a patch,
    unless the crate's API names the moved dep (#1073).

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
            because = tuple(reasons[crate])
            chosen = part(crate, because)
            bump(crate, chosen)
            bumped.add(crate)
            applied.append(CascadeBump(crate, round_no, because, chosen))
    raise AssertionError("unreachable: the loop returns or raises")


def render_cascade(
    applied: list[CascadeBump], product: list[tuple[str, str]] | None = None
) -> str:
    """The cut PR body's cascade section: kept apart from the owed bumps.

    `product` is the facade's stranded pins (#1073), which the cascade never
    bumps — they are listed as the product release they owe instead.
    """
    lines: list[str] = []
    if applied:
        lines.append(
            "coherence cascade (#1043) — bumps, each because a moved floor left "
            "its published pins behind:"
        )
        lines.extend(
            f"  {b.crate} {b.part}   (round {b.round}: {'; '.join(b.because)})"
            for b in applied
        )
    if product:
        if lines:
            lines.append("")
        lines.append(
            "product release owed (#1073) — the facade's published pins are left "
            "behind; bump_crate.py is not its remedy, tools/release/cut-release.sh is:"
        )
        lines.extend(f"  {crate}: {reason}" for crate, reason in product)
    return "\n".join(lines)


def _bump(crate: str, part: str) -> None:
    # bump_crate.py, not a hand edit: it moves every spelling of the version
    # and regenerates the lock, which is what the PR then carries.
    subprocess.run(
        [
            sys.executable,
            str(Path(__file__).with_name("bump_crate.py")),
            crate,
            part,
        ],
        check=True,
    )


def main() -> int:
    if sys.argv[1:] == ["--cascade"]:
        # Absolute reading (no base): standing debt is the cut's to fix too.
        # The registry does not move mid-run, so each crate is asked once.
        fetch = functools.cache(rs.fetch_index)
        # The last reading's facade debt: the cascade never bumps it, but the
        # PR body names the product release it owes (#1073).
        product: list[tuple[str, str]] = []

        def stranded() -> list[tuple[str, str]]:
            reading = rs.coherence_reading(fetch, None)
            product[:] = reading.product
            # Every round, clean or not: a registry that answered for nobody
            # would otherwise read as "nothing stranded" and open the PR.
            print(
                f"cascade: {reading.asked} published crate(s) asked, "
                f"{reading.unreachable} unreachable, {reading.skipped} skipped "
                "(no recognised release tier)"
                + (" — concluding nothing for those" if reading.unreachable else ""),
                file=sys.stderr,
            )
            return reading.introduced

        def part(crate: str, because: tuple[str, ...]) -> str:
            return cascade_part(rs.snapshot_text(crate), because)

        try:
            applied = cascade(stranded, _bump, cascade_limit(), part)
        except CascadeStuck as stuck:
            print(f"cascade stuck: {stuck}", file=sys.stderr)
            return EXIT_CASCADE_STUCK
        text = render_cascade(applied, product)
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
    owed = publish_owed(status)
    reading = rs.stamp_reading(status)
    # Every run, so a night that dispatches says what it held to the rule.
    print(rs.render_stamps(reading), file=sys.stderr)
    # The whole set, not the bad stamp: `publish_crates.py` uploads every
    # stamp the registry lacks, so dispatching for the good ones would carry
    # the bad one with them — #1128's emit 0.20.5 went up beside validator.
    # An unbased stamp is skipped by the gate, not cleared by it (#1155): the
    # publisher would upload it unjudged.
    if owed and (
        reading.under or reading.unreachable or reading.unmeasured or reading.unbased
    ):
        print(
            "publish refused (#1147): nothing dispatched for "
            + ", ".join(owed)
            + " — the publisher uploads every stamp the registry lacks, "
            "the ones above included",
            file=sys.stderr,
        )
        return EXIT_PUBLISH_REFUSED
    for crate in owed:
        print(crate)
    return 0


if __name__ == "__main__":
    sys.exit(main())
