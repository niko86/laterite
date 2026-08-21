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

A high-water-mark rule was considered and rejected in #458: this repo's numbering
climbs past any threshold, so it would go quiet exactly as it matured.

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


def load_expected() -> dict[int, str]:
    """number -> the prefix it must carry."""
    doc = json.loads(DATA.read_text(encoding="utf-8"))
    out = dict.fromkeys(doc["satellite"]["numbers"], doc["satellite"]["repo"])
    out.update({int(n): repo for n, repo in doc["foreign"].items()})
    return out


def tracked_files() -> list[str]:
    proc = subprocess.run(
        ["git", "-C", str(ROOT), "ls-files"],
        capture_output=True,
        text=True,
        check=True,
    )
    return proc.stdout.splitlines()


def main() -> int:
    expected = load_expected()

    if "--list" in sys.argv:
        for number, repo in sorted(expected.items()):
            print(f"  #{number:<5} -> {repo}#{number}")
        print(f"\n{len(expected)} number(s) reserved to other repositories")
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

    # The scope statement, printed pass or fail — see the module docstring.
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
