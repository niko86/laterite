"""Refuse a NEW spelled-out crate count in prose — the #950 convention gate.

The class (week-36 curation, #950 instance 1): `RELEASING.md` said "all eleven
crates" while `PUBLISH_SET` held thirteen. A count spliced into prose acquires
no reader when the set changes, so nothing fails when it drifts — the same
mechanism as CLAUDE.md's measured-value rule. The repair de-numbered the
sentence; this gate's job is stopping the next one at the PR instead of at a
curation round months later.

It is a RATCHET, not a sweep: only occurrences on lines ADDED since `--base`
fail. Existing prose is counted and reported, never judged — a convention gate
that reddens on grandfathered text would be flipped off, not obeyed. Without
`--base` (or with an empty diff) nothing is judged and the run reports totals
only; unlike check_changelog, an empty diff here is a legitimate pass, because
the contract is "no NEW counts", not "this change was examined".

Two escape hatches, both deliberate:
  * a line carrying `<!-- historical -->` is exempt — a series that already
    happened ("eight crates at 0.9.0") cannot drift, per the measured-value
    rule's own carve-out (the marker is lint.py's A11 convention);
  * `ags-wiki/concepts/crate-map.md` is exempt — its count is MEASURED against
    the workspace manifest by lint.py's C6b, which is stronger than refusal.

Wholly-generated files are exempt (their generators are the gate), and
`<!-- BEGIN GENERATED -->` regions inside hand-written pages are stripped for
the same reason. Code fences and inline code are stripped: a count in captured
tool output is the tool's, not the author's.

Scope report (a gate that drops input says what it dropped): every run prints
how many occurrences it saw, how many sat on unchanged lines (not judged), and
how many were exempted, pass or fail.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

#: Whole files this gate never judges: generated renders (their generator is
#: the drift gate) and the one page whose count is measured rather than banned.
EXEMPT_FILES = {
    "CHANGELOG.md",
    "OBSERVATIONS.md",
    "ags-wiki/index.md",
    "ags-wiki/concepts/crate-map.md",
    "web/docs-site/docs/reference/divergences.md",
}

_UNITS = ["one", "two", "three", "four", "five", "six", "seven", "eight", "nine"]
_TEENS = [
    "ten",
    "eleven",
    "twelve",
    "thirteen",
    "fourteen",
    "fifteen",
    "sixteen",
    "seventeen",
    "eighteen",
    "nineteen",
]
_TENS = {"twenty": 20, "thirty": 30, "forty": 40}
NUMWORDS: dict[str, int] = {}
NUMWORDS.update({w: i + 1 for i, w in enumerate(_UNITS)})
NUMWORDS.update({w: i + 10 for i, w in enumerate(_TEENS)})
for _w, _n in _TENS.items():
    NUMWORDS[_w] = _n
    NUMWORDS.update({f"{_w}-{u}": _n + i + 1 for i, u in enumerate(_UNITS)})

#: A number token (word or numeral), at most one qualifying word, then
#: "crates". One qualifier catches "eleven ENGINE crates" / "13 publishable
#: crates" without reaching into unrelated sentences.
_PATTERN = re.compile(
    r"\b(" + "|".join(sorted(NUMWORDS, key=len, reverse=True)) + r"|\d+)"
    r"(?:\s+[a-z][a-z/-]*)?\s+crates\b"
)

HISTORICAL_MARK = "<!-- historical -->"
_GENERATED_REGION = re.compile(
    r"<!-- BEGIN GENERATED.*?<!-- END GENERATED[^>]*-->", re.DOTALL
)


def _blank_preserving_lines(text: str, pattern: re.Pattern[str]) -> str:
    """Replace every match with spaces so line numbers survive the strip."""
    return pattern.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), text)


def scan(text: str) -> list[tuple[int, str]]:
    """(line, matched phrase) for each spelled-out crate count in prose.

    Strips fenced code, inline code and generated regions first; a raw line
    carrying the `<!-- historical -->` marker is exempt even when the marker
    sits outside the matched phrase.
    """
    raw_lines = text.splitlines()
    stripped = _blank_preserving_lines(text, _GENERATED_REGION)
    stripped = _blank_preserving_lines(
        stripped, re.compile(r"^(?:```|~~~).*?^(?:```|~~~)\s*$", re.DOTALL | re.M)
    )
    stripped = _blank_preserving_lines(stripped, re.compile(r"`[^`\n]*`"))
    out: list[tuple[int, str]] = []
    for m in _PATTERN.finditer(stripped):
        line = stripped[: m.start()].count("\n") + 1
        if HISTORICAL_MARK in raw_lines[line - 1]:
            continue
        out.append((line, m.group(0)))
    return out


def judge(
    occurrences: dict[str, list[tuple[int, str]]], added: dict[str, set[int]]
) -> list[str]:
    """The findings: occurrences sitting on lines the diff added."""
    out = []
    for rel in sorted(occurrences):
        for line, phrase in occurrences[rel]:
            if line in added.get(rel, set()):
                out.append(
                    f"{rel}:{line}: NEW spelled-out crate count '{phrase}' — a "
                    f"count in prose drifts (#950); name the set instead of its "
                    f"size, or mark the line {HISTORICAL_MARK}"
                )
    return out


def added_lines(base: str) -> dict[str, set[int]]:
    """path -> line numbers added since `base` (merge-base semantics)."""
    diff = subprocess.run(
        ["git", "diff", "--unified=0", f"{base}...HEAD", "--", "*.md"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    out: dict[str, set[int]] = {}
    current = ""
    for ln in diff.splitlines():
        if ln.startswith("+++ b/"):
            current = ln[6:]
        elif ln.startswith("@@") and current:
            m = re.search(r"\+(\d+)(?:,(\d+))?", ln)
            if m:
                start, count = int(m.group(1)), int(m.group(2) or "1")
                out.setdefault(current, set()).update(range(start, start + count))
    return out


def tracked_markdown() -> list[str]:
    files = subprocess.run(
        ["git", "ls-files", "*.md"], cwd=ROOT, capture_output=True, text=True
    ).stdout.splitlines()
    return [f for f in files if f not in EXEMPT_FILES]


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument(
        "--base",
        default="",
        help="judge occurrences on lines added since this ref; "
        "omitted or unresolvable = report-only",
    )
    ap.add_argument(
        "--list", action="store_true", help="print every occurrence, judged or not"
    )
    args = ap.parse_args()

    occurrences: dict[str, list[tuple[int, str]]] = {}
    for rel in tracked_markdown():
        path = ROOT / rel
        if not path.is_file():
            continue
        hits = scan(path.read_text(encoding="utf-8"))
        if hits:
            occurrences[rel] = hits

    total = sum(len(v) for v in occurrences.values())
    added: dict[str, set[int]] = {}
    judged = "no base given — nothing judged"
    if args.base:
        try:
            added = added_lines(args.base)
            judged = f"lines added since {args.base}"
        except subprocess.CalledProcessError:
            judged = f"base {args.base!r} unresolvable — nothing judged"

    findings = judge(occurrences, added)
    print(
        f"check_spelled_counts: {total} spelled-count occurrence(s) in "
        f"{len(occurrences)} tracked .md file(s); {len(EXEMPT_FILES)} file(s) "
        f"exempt by list; judged: {judged}; {total - len(findings)} on "
        f"unchanged/exempt lines NOT judged (the ratchet's blind spot, "
        f"reported so it stays visible)"
    )
    if args.list:
        for rel in sorted(occurrences):
            for line, phrase in occurrences[rel]:
                print(f"  ({rel}:{line}: '{phrase}')")
    for f in findings:
        print(f"  - {f}")
    if findings:
        print("check_spelled_counts: FAIL")
        return 1
    print("check_spelled_counts: OK — no new spelled-out crate count")
    return 0


if __name__ == "__main__":
    sys.exit(main())
