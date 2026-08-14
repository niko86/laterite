#!/usr/bin/env python3
"""Generate CHANGELOG.md from changelog.json — the single source of truth.

This mirrors the repo's established SSOT→generated pattern (`observations.json`
→ `gen_observations.py` → `OBSERVATIONS.md`). **`changelog.json` is
authoritative; `CHANGELOG.md` is derived** — never hand-edit the Markdown. A CI
`--check` gate enforces that the committed Markdown matches a fresh render.

Two things a hand-maintained Markdown changelog couldn't give us, and the reason
for the switch:

  * **A leak gate by construction.** Every entry is scanned for forbidden tokens
    (the dormant AGS5 strand's names, plus anything in an optional private
    denylist) — so an AGS5/company mention can't reach the public file the way it
    did in the old hand-written CHANGELOG. A hit fails generation loudly.
  * **A structured roll.** The release step rolls `unreleased` into a dated
    version *in the JSON* (`--release`), then regenerates — no fragile
    text-substitution on the Markdown (which is why the file could silently drift
    or, as happened, be deleted by a mirror sync with no gate noticing).

An entry is `{"text": …, "prs": [...], "breaking": true}` — `breaking` optional,
defaulting to false. It is what `--advise` reads, and it must agree with the
`**Breaking:**` marker in the text (exit 4 if not); RELEASING.md documents the
convention for authors. It is also what the **Breaking changes** index at the top
of the rendered file is built from — see `_breaking_index`, gated by
`tests/test_changelog_breaking_index.py`.

Exit codes: 1 stale render · 2 leak gate · 3 empty release · 4 breaking-marker
disagreement.

Modes:
  gen_changelog.py                 regenerate CHANGELOG.md from changelog.json
  gen_changelog.py --check         exit 1 if CHANGELOG.md is stale (CI drift gate)
  gen_changelog.py --advise        read [Unreleased] and recommend the next bump
                                   (the fix backlog IS `unreleased.fixed`, so this
                                   flags when accumulated fixes warrant a patch cut)
  gen_changelog.py --release X [--date YYYY-MM-DD]
                                   roll `unreleased` into a new [X] release in
                                   changelog.json, then regenerate. Replaces the
                                   bump-my-version CHANGELOG roll (see RELEASING.md).
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import date as date_cls
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Iterator

ROOT = Path(__file__).resolve().parent.parent
JSON_PATH = ROOT / "changelog.json"
MD_PATH = ROOT / "CHANGELOG.md"
DENYLIST_PATH = ROOT / ".changelog-denylist"  # optional, gitignored, owner-maintained

# Keep a Changelog's category order — only non-empty ones render.
CATEGORIES = ["added", "changed", "deprecated", "removed", "fixed", "security"]

# The leak gate. The AGS5 strand is a dormant CONCEPT, never a shipped feature in
# public output (CLAUDE.md); these are the names that must not appear in the
# public changelog. Extend per-project via `.changelog-denylist` (one token or
# /regex/ per line) for company names etc. that can't be committed here.
FORBIDDEN = [
    r"\.ags5db\b",
    r"\.agsx\b",
    r"\[ags5\]",
    r"\blaterite-ags5\b",
    r"\bags5db\b",
    r"\blat-db\b",
    r"\bAGS5\b",
]


def _load_denylist() -> list[str]:
    pats = list(FORBIDDEN)
    if DENYLIST_PATH.exists():
        for raw in DENYLIST_PATH.read_text(encoding="utf-8").splitlines():
            line = raw.strip()
            if not line or line.startswith("#"):
                continue
            # `/regex/` is a raw pattern; a bare token is matched literally.
            pats.append(
                line[1:-1]
                if line.startswith("/") and line.endswith("/")
                else re.escape(line)
            )
    return pats


def _leak_check(data: dict) -> list[str]:
    """Return a list of 'location: token' violations; empty means clean."""
    pats = [re.compile(p, re.IGNORECASE) for p in _load_denylist()]
    hits: list[str] = []

    def scan(where: str, text: str) -> None:
        for pat in pats:
            m = pat.search(text or "")
            if m:
                hits.append(f"{where}: {m.group(0)!r}")

    def scan_block(where: str, block: dict) -> None:
        scan(f"{where} summary", block.get("summary", ""))
        for cat in CATEGORIES:
            for i, e in enumerate(block.get(cat, [])):
                scan(f"{where} {cat}[{i}]", e.get("text", ""))

    scan_block("unreleased", data.get("unreleased", {}))
    for rel in data.get("releases", []):
        scan_block(f"[{rel.get('version', '?')}]", rel)
    return hits


def _render_entry(entry: dict, repo: str) -> str:
    text = entry["text"].rstrip()
    prs = entry.get("prs") or []
    if prs:
        links = ", ".join(f"[#{n}](https://github.com/{repo}/pull/{n})" for n in prs)
        text = f"{text} ({links})"
    return f"- {text}"


def _render_block(block: dict, repo: str) -> list[str]:
    out: list[str] = []
    summary = (block.get("summary") or "").strip()
    if summary:
        out += ["", summary]
    for cat in CATEGORIES:
        entries = block.get(cat) or []
        if not entries:
            continue
        out += ["", f"### {cat.capitalize()}", ""]
        out += [_render_entry(e, repo) for e in entries]
    return out


def _heading(rel: dict | None) -> str:
    """A section's heading text — what follows `## `. `None` is [Unreleased].

    One function because the breaking index links INTO these sections: the
    anchor is derived from the same string the heading is printed from, so the
    two cannot be changed apart.
    """
    if rel is None:
        return "[Unreleased]"
    return f"[{rel['version']}] — {rel['date']}"


#: Everything GitHub's heading slugger drops — anything that is not a word
#: character, an ASCII hyphen or a space.
_NOT_IN_SLUG = re.compile(r"[^\w\- ]", re.UNICODE)


def _anchor(heading: str) -> str:
    """GitHub's fragment for a heading: lowercased, punctuation dropped, spaces
    hyphenated.

    Each space becomes its own hyphen — GitHub does not collapse runs — so
    `[0.10.0] — 2026-08-02` anchors at `#0100--2026-08-02`, a DOUBLE hyphen
    where the em dash was. That reads like a typo and is not: python-markdown's
    slugify *would* collapse it, which is why this is written out here rather
    than borrowed from whichever slugifier happens to be importable. `CHANGELOG.md`
    is read on GitHub — the docs site does not render it (RELEASING.md).
    """
    return _NOT_IN_SLUG.sub("", heading.strip().lower()).replace(" ", "-")


def render(data: dict) -> str:
    repo = data["repo"]
    kac = data.get("keepachangelog", "1.1.0")
    lines = [
        "# Changelog",
        "",
        "All notable changes to this project are documented in this file.",
        "",
        f"The format is based on [Keep a Changelog](https://keepachangelog.com/en/{kac}/),",
        "and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).",
        "",
        "<!-- GENERATED FROM changelog.json BY tools/gen_changelog.py — DO NOT EDIT BY HAND -->",
    ]
    lines += _breaking_index(data)
    lines += ["", f"## {_heading(None)}"]
    lines += _render_block(data.get("unreleased", {}), repo)

    releases = data.get("releases", [])
    for rel in releases:
        lines += ["", f"## {_heading(rel)}"]
        lines += _render_block(rel, repo)

    # Link references (compare for Unreleased, tag for each release).
    lines += [""]
    if releases:
        newest = releases[0]["version"]
        lines.append(
            f"[Unreleased]: https://github.com/{repo}/compare/v{newest}...HEAD"
        )
    else:
        lines.append(f"[Unreleased]: https://github.com/{repo}/commits/HEAD")
    for rel in releases:
        v = rel["version"]
        lines.append(f"[{v}]: https://github.com/{repo}/releases/tag/v{v}")
    return "\n".join(lines).rstrip() + "\n"


def _load() -> dict:
    return json.loads(JSON_PATH.read_text(encoding="utf-8"))


def _empty_block() -> dict:
    return {"summary": "", **{c: [] for c in CATEGORIES}}


def do_release(version: str, when: str | None) -> None:
    data = _load()
    unreleased = data.get("unreleased", {})
    if not any(unreleased.get(c) for c in CATEGORIES):
        print(
            f"gen_changelog: refusing to release {version} — [Unreleased] is empty",
            file=sys.stderr,
        )
        sys.exit(3)
    # Advice, not a veto — the owner can have a reason the categories don't
    # carry. But a silent disagreement is how a version gets mis-cut onto a
    # registry that cannot re-cut it, so say it out loud at the moment it counts.
    part, advised, why = advise(data)
    if version != advised:
        print(
            f"gen_changelog: NOTE — the advisor says {part.upper()} → {advised}, "
            f"not {version}.\n  Why: {why}",
            file=sys.stderr,
        )
    when = when or date_cls.today().isoformat()
    rolled = {"version": version, "date": when}
    rolled["summary"] = unreleased.get("summary", "")
    for c in CATEGORIES:
        rolled[c] = unreleased.get(c, [])
    data.setdefault("releases", []).insert(0, rolled)
    data["unreleased"] = _empty_block()
    JSON_PATH.write_text(
        json.dumps(data, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )
    print(f"gen_changelog: rolled [Unreleased] → [{version}] — {when}")
    _write(data)


def _write(data: dict) -> None:
    hits = _leak_check(data)
    if hits:
        print(
            "gen_changelog: LEAK-GATE — forbidden tokens in changelog entries:",
            file=sys.stderr,
        )
        for h in hits:
            print(f"  {h}", file=sys.stderr)
        sys.exit(2)
    MD_PATH.write_text(render(data), encoding="utf-8")
    print(f"gen_changelog: wrote {MD_PATH.relative_to(ROOT)}")


# Compatibility is a property of the change, so it is DECLARED (`"breaking":
# true` on the entry) rather than inferred from the prose. It used to be a
# `\bbreaking\b` search over the entry text, which cannot tell a marker from a
# sentence denying one: "a non-breaking change" matched (the hyphen is a word
# boundary), as did "this is not a breaking change" and "avoids breaking
# downstream consumers". Six years of entries happened to be true positives only
# because the house style was disciplined — and the cost of the first false one
# is a wrong version on an append-only registry, which can never be re-cut.
#
# The prose marker survives as a CROSS-CHECK, not as the signal (see
# `_breaking_check`): the flag and the rendered `**Breaking:` marker must agree,
# so neither can be updated without the other. Same shape as the wiki/observation
# gate — two representations held in agreement, not one guessed from the other.
_MARKER = re.compile(r"\*\*[Bb]reaking\b[^*]*\*\*")


def _entries(block: dict) -> Iterator[tuple[str, int, dict]]:
    """Yield `(category, index, entry)` over every entry in a block."""
    for cat in CATEGORIES:
        for i, e in enumerate(block.get(cat) or []):
            yield cat, i, e


def _breaking_count(block: dict) -> int:
    """How many entries DECLARE themselves breaking."""
    return sum(1 for _, _, e in _entries(block) if e.get("breaking") is True)


def _breaking_check(data: dict) -> list[str]:
    """Return `flag != marker` disagreements; empty means the two agree.

    Both directions are violations. A flag with no marker ships a breaking
    change whose entry never says so to the reader; a marker with no flag tells
    the reader while leaving the advisor to recommend a patch.
    """
    hits: list[str] = []

    def scan(where: str, block: dict) -> None:
        for cat, i, e in _entries(block):
            flag = e.get("breaking") is True
            marker = bool(_MARKER.search(e.get("text", "")))
            if flag and not marker:
                hits.append(
                    f'{where} {cat}[{i}]: "breaking": true but the text carries no '
                    "**Breaking:** marker"
                )
            elif marker and not flag:
                hits.append(
                    f"{where} {cat}[{i}]: text carries a **Breaking:** marker but "
                    '"breaking": true is missing'
                )

    scan("unreleased", data.get("unreleased", {}))
    for rel in data.get("releases", []):
        scan(f"[{rel.get('version', '?')}]", rel)
    return hits


# --- the breaking-changes index ---------------------------------------------
#
# "Pre-1.0 a minor may break you — read the changelog before upgrading" is a
# promise that only pays out if the changelog can be READ. This file is ~40KB of
# prose, and the question a consumer arrives with is not "what changed" but "does
# the step I am about to take break me" — a six-line answer buried in it.
#
# So the answer is rendered at the top, from the same declared flag `advise()`
# bumps on. Not a second hand-maintained list: an entry cannot be flagged
# breaking and left out of the index, because the index IS the flag, and the
# `--check` drift gate fails the moment the two could disagree.

#: The bold sentence the house style opens every entry with.
_HEADLINE = re.compile(r"\A\*\*(?P<head>.+?)\*\*", re.DOTALL)

#: A leading `Breaking:` on that sentence. Redundant under a heading that already
#: says so, and dropping it makes the column read as one list of changes rather
#: than four rows announcing themselves and three not.
_LEADING_BREAKING = re.compile(r"\A\*{0,2}[Bb]reaking\b:?\*{0,2}\s*")


def _first_sentence(text: str, limit: int = 120) -> str:
    """The first sentence of `text`, hard-truncated if it has none."""
    text = text.strip()
    m = re.search(r"(?<=[.!?])\s", text)
    s = (text[: m.start()] if m else text).strip()
    return s if len(s) <= limit else s[: limit - 1].rstrip() + "…"


def _headline(text: str) -> str:
    """An entry's one-line form, for the index.

    The house style opens each entry with a bold summary sentence — the wording
    its author already chose. Quoting that beats paraphrasing: a summary written
    twice is a summary that can end up disagreeing with the entry it points at.

    Two entries (0.7.0, 0.8.0) put their summary first and the bare
    `**Breaking:**` marker mid-paragraph, so stripping the prefix can empty the
    headline; those fall back to the first sentence after the marker.
    """
    text = " ".join(text.split())
    m = _HEADLINE.match(text)
    head = _LEADING_BREAKING.sub("", m.group("head") if m else text).strip()
    if not head:
        head = _first_sentence(text[m.end() :] if m else text)
    # A raw pipe would end the table cell it sits in.
    return head.replace("|", r"\|")


def _breaking_index(data: dict) -> list[str]:
    """The rendered index — one row per entry declaring itself breaking."""
    releases = data.get("releases", [])
    blocks: list[tuple[str, dict | None, dict]] = [
        ("Unreleased", None, data.get("unreleased", {})),
        *((r["version"], r, r) for r in releases),
    ]
    rows = [
        f"| [{label}](#{_anchor(_heading(rel))}) | {_headline(e['text'])} |"
        for label, rel, block in blocks
        for _, _, e in _entries(block)
        if e.get("breaking") is True
    ]

    # Which bump a break lands on is the era's business, not a constant — the
    # same fact `advise()` computes. Asserting "the MINOR" here would go false at
    # 1.0 while the advisor kept getting it right, on the one page the promise
    # sends people to.
    current = releases[0]["version"] if releases else "0.0.0"
    era = (
        "at `0.x` a breaking change takes the **MINOR**"
        if current.startswith("0.")
        else "past `1.0` a breaking change takes the **MAJOR**"
    )
    out = [
        "",
        "## Breaking changes",
        "",
        f"Every change listed here declares itself breaking — {era}, so these are "
        "the only version steps that can break you. Rendered from the same flag "
        "the release advisor reads, so a declared break cannot be missing from it.",
        "",
    ]
    if not rows:
        return [*out, "None — no release has declared a breaking change."]
    return [*out, "| version | change |", "|---|---|", *rows]


def _bump(base: str, part: str) -> str:
    """Increment a clean `X.Y.Z`."""
    major, minor, patch = (int(x) for x in base.split("."))
    if part == "major":
        return f"{major + 1}.0.0"
    if part == "minor":
        return f"{major}.{minor + 1}.0"
    return f"{major}.{minor}.{patch + 1}"


def advise(data: dict) -> tuple[str, str, str]:
    """Recommend the next bump from [Unreleased]. Returns `(part, version, why)`.

    The axis is COMPATIBILITY, not change size (RELEASING.md), and which bump a
    break lands on depends on where the project is:

        pre-1.0 (0.x)  features and fixes are a PATCH; a BREAKING change takes
                       the MINOR. `1.0.0` is saved for a deliberate "stable"
                       signal, so MAJOR is unreachable here.
        1.0 and after  standard SemVer: additive is a MINOR, breaking a MAJOR.

    Splitting this out of the printing makes it testable, and lets `--release`
    warn when the version being cut disagrees with it.
    """
    unreleased = data.get("unreleased", {})
    counts = {c: len(unreleased.get(c) or []) for c in CATEGORIES}
    breaking = _breaking_count(unreleased)
    current = data["releases"][0]["version"] if data.get("releases") else "0.0.0"
    pre_1_0 = current.startswith("0.")

    if breaking:
        part = "minor" if pre_1_0 else "major"
        era = "pre-1.0 (0.x) — a breaking change takes the MINOR"
        why = f"{breaking} breaking change(s) queued; {era}."
    elif pre_1_0:
        part = "patch"
        why = (
            "pre-1.0 (0.x) — features and fixes are a PATCH; only a breaking "
            "change takes the MINOR. Nothing queued declares itself breaking."
        )
    elif counts["added"]:
        part = "minor"
        why = "new features (added) with nothing breaking — a MINOR, post-1.0 SemVer."
    else:
        part = "patch"
        why = "only fixes and non-breaking changes are queued."
    return part, _bump(current, part), why


def do_advise(data: dict) -> None:
    """Read `unreleased` and recommend the next bump. The fix backlog is exactly
    `unreleased.fixed` — no second register to drift — so this is where an
    accumulation of fixes surfaces as 'time for a patch'."""
    unreleased = data.get("unreleased", {})
    counts = {c: len(unreleased.get(c) or []) for c in CATEGORIES}
    current = data["releases"][0]["version"] if data.get("releases") else "0.0.0"

    print(f"gen_changelog: release advisor (current shipped: {current})")
    populated = [(c, n) for c, n in counts.items() if n]
    if not populated:
        print("  [Unreleased] is empty — nothing to release.")
        return
    print("  [Unreleased] holds:")
    for c, n in populated:
        n_break = sum(
            1
            for cat, _, e in _entries(unreleased)
            if cat == c and e.get("breaking") is True
        )
        print(f"    {c:<10} {n}{f'  ({n_break} breaking)' if n_break else ''}")

    part, version, why = advise(data)
    print(f"\n  Recommended bump: {part.upper()} → {version}")
    print(f"    Why: {why}")
    ship = counts["fixed"] + counts["security"]
    if ship and part != "patch":
        print(f"    The {ship} queued fix(es)/security item(s) ship with it.")
    elif ship:
        print(
            f"    → {ship} fix(es)/security item(s) are ready — cut a patch to ship them."
        )


def main() -> None:
    ap = argparse.ArgumentParser(
        description="Generate CHANGELOG.md from changelog.json"
    )
    ap.add_argument(
        "--advise",
        action="store_true",
        help="recommend the next SemVer bump from [Unreleased] (fix-backlog signal)",
    )
    ap.add_argument(
        "--check", action="store_true", help="fail if CHANGELOG.md is stale (CI gate)"
    )
    ap.add_argument(
        "--release",
        metavar="VERSION",
        help="roll [Unreleased] into a dated release, then regenerate",
    )
    ap.add_argument(
        "--date", metavar="YYYY-MM-DD", help="release date (default: today)"
    )
    args = ap.parse_args()

    data = _load()
    hits = _leak_check(data)
    if hits:
        print(
            "gen_changelog: LEAK-GATE — forbidden tokens in changelog entries:",
            file=sys.stderr,
        )
        for h in hits:
            print(f"  {h}", file=sys.stderr)
        sys.exit(2)

    # Every mode enforces this, not just `--check`: an entry whose flag and prose
    # disagree is already wrong in the JSON, so the render, the roll and the
    # advice are all downstream of the same bad input.
    disagreements = _breaking_check(data)
    if disagreements:
        print(
            "gen_changelog: BREAKING-MARKER — the `breaking` flag and the "
            "**Breaking:** marker disagree:",
            file=sys.stderr,
        )
        for h in disagreements:
            print(f"  {h}", file=sys.stderr)
        sys.exit(4)

    if args.advise:
        do_advise(data)
        return

    if args.release:
        do_release(args.release, args.date)
        return

    rendered = render(data)
    if args.check:
        current = MD_PATH.read_text(encoding="utf-8") if MD_PATH.exists() else ""
        if current != rendered:
            print(
                "gen_changelog: CHANGELOG.md is STALE — regenerate with "
                "`uv run --no-project python tools/gen_changelog.py`",
                file=sys.stderr,
            )
            sys.exit(1)
        print("gen_changelog: CHANGELOG.md is up to date")
        return

    _write(data)


if __name__ == "__main__":
    main()
