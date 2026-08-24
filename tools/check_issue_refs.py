#!/usr/bin/env python3
"""Assert that a bare `#N` in this tree means an issue or PR in THIS repo.

Bare `#N` is what a reader assumes is ours and what GitHub autolinks. Numbers
that mean somewhere else — the dev satellite, an upstream project — must carry
their repo: `laterite-dev#512`, `microsoft/mimalloc#1327`. The convention was
already in use for upstream projects (`apache/arrow#50549`, `pyo3#678`); what it
lacked was anything holding it.

Why that matters more than a dead link. A satellite number written bare while
this repo's highest was 457 was *visibly* broken — nothing resolved. As the
numbering climbed it stopped being dead and started resolving, to a different,
unrelated, entirely plausible page, autolinked, with nothing failing anywhere.
#458 measured it after the collision had begun: `laterite-dev#475`, the
reference-leaf extraction, is cited across two dozen files, and this repo's own
issue at that number is a DuckDB docs gate. A reader following it lands
somewhere real and wrong, which is the worst failure direction available.

Note for anyone writing ABOUT this: there is no per-occurrence exemption, so
prose here names foreign numbers qualified or not at all. A frozen set is a set
of numbers and cannot express "this one time, that number means ours" — and the
alternative, a file/line allowlist, would be a second thing to keep true. Say
"a satellite number" and the sentence survives.

**Scope, stated rather than implied.** This gate is a FROZEN set
(`foreign-issue-refs.json`) — it holds the tree against re-introducing a number
already known to be foreign, and it is blind to a NEW one. Deciding whether an
unrecognised `#N` is ours needs the API. So the run prints how many bare refs it
looked at and had no opinion on: a gate that drops input says what it dropped
(CLAUDE.md, Conventions), and "0 problems" over a set this narrow would read as
"every reference checks out" when it means nothing of the sort.

**The watermark, and why it is not the rule #458 rejected.** #458 considered
treating numbers ABOVE a threshold as foreign and rejected it: this repo's
numbering climbs past any threshold, so that gate would go quiet exactly as it
matured. This is the inverse and it fails the safe way — a number AT OR BELOW
what this repo has already issued is OURS, so it leaves the frozen set. The set
therefore shrinks as the numbering climbs, which is the direction that matches
reality, and a wrong watermark costs a false positive on a correct reference
rather than silence on a wrong one.

That rule was written into the JSON's own comment from the start and nothing
executed it, so it was discharged by hand: three edits to the same file in one
sitting (#498), each one releasing a number this repo had just been allocated.
The watermark is derived instead, from the squash-merge convention that puts the
PR number at the end of every merged subject on the default branch — an offline
authority, which this gate needs because it runs in the buildless `repo-gates`
job with no API access. `claimed_through` in the JSON is the manual override for
the one case git cannot see: a freshly-filed ISSUE, whose number no commit
carries until the next PR merges past it.

Usage:
    uv run --no-project python tools/check_issue_refs.py
    uv run --no-project python tools/check_issue_refs.py --list

Exit 0 when every foreign number in the tree is qualified, 1 when one is bare.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DATA = ROOT / "foreign-issue-refs.json"

# `#` + digits, NOT preceded by a word char, `/`, `-`, `&` or another `#`. The
# lookbehind is the whole job: it is what makes `laterite-dev#512`,
# `arrow-rs#10439`, `&#8212;` and `https://…#123` invisible here, so an already
# qualified reference cannot be re-flagged.
BARE = re.compile(r"(?<![\w/&#-])#(\d{2,5})\b")

# Binary and vendored formats; a `#N` inside them is not prose. `.lock` files
# carry checksums that can look like anything.
SKIP_SUFFIX = frozenset(
    {".lock", ".svg", ".png", ".jpg", ".jpeg", ".ico", ".woff", ".woff2", ".ttf"}
)


#: The PR number the squash-merge convention leaves at the end of a merged
#: subject: `fix(web): … (#457) (#496)` -> 496. Anchored to the END on purpose —
#: the inner `(#457)` is the ISSUE the PR closed, and taking it would read a
#: number this repo may not have issued yet as one it has.
MERGED_PR = re.compile(r"\(#(\d+)\)\s*$")

#: Where to read merged history from, best first. On a `pull_request` checkout
#: HEAD is the merge ref and its first parent is the base branch, so `origin/main`
#: is both available and the most precise; the fallbacks are for a local clone
#: that has no remote-tracking ref.
HISTORY_REFS = ("origin/main", "main", "HEAD")


def _git(*args: str) -> str | None:
    """Run git in this tree; None if git or the repository is unavailable."""
    try:
        proc = subprocess.run(
            ["git", "-C", str(ROOT), *args], capture_output=True, text=True, check=True
        )
    except (OSError, subprocess.CalledProcessError):
        return None
    return proc.stdout


def is_shallow() -> bool:
    return (_git("rev-parse", "--is-shallow-repository") or "").strip() == "true"


def watermark_from_git() -> tuple[int, str] | None:
    """The highest PR number merged into the default branch, and the ref it came
    from. None when no ref could be read at all."""
    for ref in HISTORY_REFS:
        subjects = _git("log", "--first-parent", "--format=%s", ref)
        if subjects is None:
            continue
        numbers = [
            int(m.group(1))
            for line in subjects.splitlines()
            if (m := MERGED_PR.search(line))
        ]
        if numbers:
            return max(numbers), ref
    return None


def load_expected(watermark: int) -> tuple[dict[int, str], list[int]]:
    """(number -> the prefix it must carry, the numbers released as ours).

    A number at or below the watermark has been issued by THIS repo, so a bare
    `#N` is a correct reference to it whatever the satellite also calls that
    number. Releasing it here is what stops the set failing correct references —
    and it is the only thing that has to happen, because both repos genuinely
    have an issue at that number.
    """
    doc = json.loads(DATA.read_text(encoding="utf-8"))
    frozen = dict.fromkeys(doc["satellite"]["numbers"], doc["satellite"]["repo"])
    frozen.update({int(n): repo for n, repo in doc["foreign"].items()})
    released = sorted(n for n in frozen if n <= watermark)
    for n in released:
        del frozen[n]
    return frozen, released


def resolve_watermark() -> tuple[int, str]:
    """(watermark, how it was derived) — the greater of git and the override."""
    declared = json.loads(DATA.read_text(encoding="utf-8")).get("claimed_through")
    found = watermark_from_git()
    if found is None:
        if declared is None:
            return 0, "no git history and no `claimed_through` — nothing released"
        return declared, "`claimed_through` (no git history to read)"
    number, ref = found
    if declared is not None and declared > number:
        return declared, f"`claimed_through` (above {ref}'s highest merged PR)"
    return number, f"highest merged PR on {ref}"


def tracked_files() -> list[str]:
    proc = subprocess.run(
        ["git", "-C", str(ROOT), "ls-files"],
        capture_output=True,
        text=True,
        check=True,
    )
    return proc.stdout.splitlines()


def main() -> int:
    # A shallow clone reads a watermark that is wrong-LOW, which quietly restores
    # the bug this replaced: correct references to recent numbers start failing,
    # and the message points at an innocent `#N` instead of at the checkout. Say
    # what is actually wrong. (changelog.yml carries `fetch-depth: 0` and the
    # same reasoning for the same class of failure.)
    if is_shallow():
        print(
            "check_issue_refs: this is a SHALLOW clone, so the merged-PR history "
            "the watermark\nis derived from is not here. The watermark would read "
            "low and this gate would\nreject correct references to recent "
            "numbers. Use `fetch-depth: 0`.",
            file=sys.stderr,
        )
        return 1

    watermark, how = resolve_watermark()
    expected, released = load_expected(watermark)

    if "--list" in sys.argv:
        for number, repo in sorted(expected.items()):
            print(f"  #{number:<5} -> {repo}#{number}")
        print(f"\n{len(expected)} number(s) reserved to other repositories")
        if released:
            print(
                f"{len(released)} released as ours at or below #{watermark} "
                f"({how}): {', '.join(f'#{n}' for n in released)}"
            )
        return 0

    violations: list[tuple[str, int, int]] = []
    unknown: set[int] = set()
    scanned = 0

    for rel in tracked_files():
        path = ROOT / rel
        if path.suffix in SKIP_SUFFIX or not path.is_file():
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except (UnicodeDecodeError, OSError):
            continue
        for lineno, line in enumerate(text.splitlines(), 1):
            for match in BARE.finditer(line):
                number = int(match.group(1))
                scanned += 1
                if number in expected:
                    violations.append((rel, lineno, number))
                else:
                    unknown.add(number)

    # The scope statement, printed pass or fail — see the module docstring. The
    # watermark rides in it for the same reason the unjudged count does: it is
    # the other thing that decides what this run did NOT look at, and one that
    # drifts on its own between runs.
    print(
        f"check_issue_refs: watermark #{watermark} ({how}); "
        f"{len(released)} number(s) released as this repo's own"
    )
    # The satellite half of the set empties on a schedule — every number in it
    # is one this repo's numbering eventually climbs past — and an empty half
    # is indistinguishable from a working one in the counts above. Say it, so
    # nobody reads a green tick as "the satellite refs are still guarded".
    satellite = json.loads(DATA.read_text(encoding="utf-8"))["satellite"]
    if not any(n in expected for n in satellite["numbers"]):
        print(
            "check_issue_refs: no satellite number is still held — the "
            "numbering has climbed past every one cited so far, so nothing "
            f"here is guarding a `{satellite['repo']}` reference right now"
        )
    print(
        f"check_issue_refs: {scanned} bare `#N` scanned; "
        f"{len(unknown)} distinct number(s) not in the frozen set, so unjudged "
        f"(this gate cannot tell a new foreign ref from one of ours)"
    )

    if not violations:
        print(
            f"check_issue_refs: OK — none of the {len(expected)} known-foreign "
            f"number(s) appears bare"
        )
        return 0

    print(
        f"\ncheck_issue_refs: {len(violations)} bare reference(s) to another "
        f"repository:\n"
    )
    for rel, lineno, number in violations:
        print(f"  {rel}:{lineno}  #{number} -> {expected[number]}#{number}")
    print(
        "\nA bare `#N` means an issue in THIS repo. These numbers belong "
        "elsewhere, so\nGitHub will autolink them to whatever this repo's #N "
        "turns out to be — a real,\nplausible, wrong page. Qualify them with "
        "the repo shown above."
    )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
