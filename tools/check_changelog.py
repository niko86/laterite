#!/usr/bin/env python3
"""Require a changelog entry from any PR that changes a shipped surface.

The gate this adds is one the 0.9.0 cut proved was missing. PRs #178-#182 all
merged with `[Unreleased]` **empty** — four of them carrying a `!` breaking
change. Nothing noticed, because nothing looked: the only thing in the repo that
reads `unreleased` is `bump-version.sh`, and it reads it at *release* time.

So the failure surfaced as "the release refuses to cut", days after the last of
the five merged, with the notes for two breaking changes still unwritten and the
authors' context long gone. Had `gen_changelog.py --release` not refused on an
empty section, 0.9.0 would have shipped a wasm API break and a TRAN behaviour
change with no note for either.

The rule: **if you changed something a consumer can observe, say so.** Not a
count, not a heuristic on the commit subject — the presence of a new entry,
compared against the base branch.

Four ways to pass, and the third and fourth are the interesting ones:

  no shipped surface touched   -> pass. Tests, CI, docs, tooling and the wiki
                                  change nothing a consumer installs.
  the changelog gained an entry-> pass. The normal path. "The changelog" means
                                  `unreleased` OR the head release block, which
                                  is still open until its tag is cut — see
                                  `declarable`.
  the PR ROLLS a release       -> pass. `bump-version.sh` moves `unreleased`
                                  INTO a version block and leaves it empty, so
                                  the release PR itself must not be required to
                                  refill what it just drained.
  the `no-changelog` label     -> pass, LOUDLY. An internal refactor genuinely
                                  may have nothing to declare. The label makes
                                  that a decision somebody took and attached to
                                  the PR, rather than a silence nobody noticed —
                                  which is the whole difference between this and
                                  what happened to #178-#182.

**An empty comparison must never read as a pass.** If the base ref is unreachable
or the diff comes back empty, that is an error, not a green run — the same trap
`check_parity.py` records eating a CI job, where a broken clone collected 0 tests
and printed a clean result. A gate that cannot see the diff has not checked
anything, and must say so.

Usage:
    tools/check_changelog.py --base origin/main
    tools/check_changelog.py --base origin/main --allow-empty   # the label opt-out
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
JSON_PATH = ROOT / "changelog.json"
CATEGORIES = ("added", "changed", "deprecated", "removed", "fixed", "security")

# What a consumer can observe. Deliberately the SHIPPED artefacts only: the
# engine + binding sources, the wheel's Python surface, the Node package's TS,
# the browser app's source, and the two manifests that decide what a
# `pip install` actually pulls (the dep-shape split is a user-visible contract —
# see CLAUDE.md).
SHIPPED_PREFIXES = (
    "rust-packages/",
    "packages/laterite/python/",
    "web/src/",
)
SHIPPED_FILES = (
    "packages/laterite/pyproject.toml",
    "pyproject.toml",
)

# Paths that are inside a shipped prefix but change nothing a consumer installs.
# Tests are the big one: a test-only PR (#177 was exactly this) has nothing to
# declare, and demanding an entry would train people to write noise.
EXEMPT_PARTS = ("/tests/", "/test/", "/benches/", "/bench/", "/examples/")
EXEMPT_SUFFIXES = (".test.ts", ".test.tsx", ".bench.ts", ".md")


def _git(*args: str) -> str:
    out = subprocess.run(
        ["git", *args], cwd=ROOT, capture_output=True, text=True, check=False
    )
    if out.returncode != 0:
        die(f"git {' '.join(args)} failed:\n{out.stderr.strip()}")
    return out.stdout


def die(msg: str) -> None:
    print(f"check_changelog: {msg}", file=sys.stderr)
    sys.exit(2)


def shipped(path: str) -> bool:
    """Does a change to `path` alter something a consumer receives?"""
    if any(f"/{path}".find(p) != -1 for p in EXEMPT_PARTS):
        return False
    if path.endswith(EXEMPT_SUFFIXES):
        return False
    return path.startswith(SHIPPED_PREFIXES) or path in SHIPPED_FILES


def entries(block: dict) -> list[str]:
    """Every entry's text, flattened. Identity, not count: an entry swapped for
    another of the same length must not read as 'unchanged'."""
    out = []
    for cat in CATEGORIES:
        out.extend(f"{cat}:{e.get('text', '')}" for e in block.get(cat) or [])
    return out


def declarable(data: dict) -> list[str]:
    """Where an entry may legitimately be written *right now*.

    Normally that is `unreleased`. But a release block that is rolled and not yet
    TAGGED is still open: 0.9.0 sat in that state for hours while a further PR
    landed inside it, and those notes correctly went into `[0.9.0]` rather than
    opening an `[Unreleased]` for a version that had not shipped. Counting only
    `unreleased` would have failed that PR for doing the right thing.

    Safe because the version is compared separately: if `releases[0]` differs
    between base and head the PR is a release ROLL and returns earlier, so this
    only ever compares the same block on both sides.
    """
    head_rel = (data.get("releases") or [{}])[0]
    return entries(data.get("unreleased", {})) + entries(head_rel)


def at_base(base: str) -> dict:
    raw = _git("show", f"{base}:changelog.json")
    if not raw.strip():
        die(f"changelog.json is empty at {base} — cannot compare")
    return json.loads(raw)


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--base", required=True, help="base ref, e.g. origin/main")
    ap.add_argument(
        "--allow-empty",
        action="store_true",
        help="the `no-changelog` label: pass with no entry, and say what was skipped",
    )
    args = ap.parse_args()

    # Resolve the base first. An unreachable ref is an ERROR — silently treating
    # it as "no changes" would make this gate pass on every PR forever.
    rev = _git("rev-parse", "--verify", f"{args.base}^{{commit}}").strip()
    merge_base = _git("merge-base", rev, "HEAD").strip()
    if not merge_base:
        die(f"no merge-base between {args.base} and HEAD")

    changed = [
        p for p in _git("diff", "--name-only", merge_base, "HEAD").splitlines() if p
    ]
    if not changed:
        die(
            f"no files differ between {merge_base[:12]} and HEAD.\n"
            "  A gate that sees no diff has checked nothing — refusing to report success.\n"
            "  (In CI this usually means a shallow clone: give actions/checkout fetch-depth: 0.)"
        )

    touched = sorted(p for p in changed if shipped(p))
    if not touched:
        print(
            f"check_changelog: OK — {len(changed)} file(s) changed, none of them a "
            "shipped surface (tests / CI / docs / tooling)."
        )
        return

    head = json.loads(JSON_PATH.read_text())
    base = at_base(rev)

    # A release roll drains `unreleased` into a version block. The release PR
    # must not then be asked to refill it.
    head_rel = (head.get("releases") or [{}])[0].get("version")
    base_rel = (base.get("releases") or [{}])[0].get("version")
    if head_rel != base_rel:
        print(
            f"check_changelog: OK — this PR rolls [{base_rel}] -> [{head_rel}]; "
            "`unreleased` is drained by design."
        )
        return

    was = declarable(base)
    added = [e for e in declarable(head) if e not in was]
    if added:
        print(f"check_changelog: OK — {len(added)} new changelog entry/entries:")
        for e in added:
            cat, _, text = e.partition(":")
            print(f"    {cat:<10} {text[:90]}{'…' if len(text) > 90 else ''}")
        return

    if args.allow_empty:
        print(
            "check_changelog: SKIPPED by the `no-changelog` label — "
            f"{len(touched)} shipped file(s) changed with no entry:"
        )
        for p in touched[:10]:
            print(f"    {p}")
        if len(touched) > 10:
            print(f"    … and {len(touched) - 10} more")
        return

    print(
        f"check_changelog: {len(touched)} shipped file(s) changed and the changelog "
        "gained nothing.\n",
        file=sys.stderr,
    )
    for p in touched[:15]:
        print(f"    {p}", file=sys.stderr)
    if len(touched) > 15:
        print(f"    … and {len(touched) - 15} more", file=sys.stderr)
    print(
        "\n  Add an entry to changelog.json's `unreleased`, then regenerate:\n"
        "      uv run --no-project python tools/gen_changelog.py\n"
        "  Mark a breaking change with a **Breaking:** in its text — "
        "`--advise` counts those to pick the next bump.\n\n"
        "  If this genuinely changes nothing a consumer can observe, put the\n"
        "  `no-changelog` label on the PR. That records the decision ON the PR instead\n"
        "  of leaving a silence nobody notices — which is exactly how #178-#182 reached\n"
        "  a release with two breaking changes unwritten.\n\n"
        "  Expect to need the label for a Rust test-only PR: unit tests live INSIDE\n"
        "  `src/*.rs`, so a pure test sweep is indistinguishable here from a behaviour\n"
        "  change to the same file. #177 (a mutation-sweep round touching only\n"
        "  `#[cfg(test)]` blocks) is the shape. Narrowing this by parsing diff hunks\n"
        "  for cfg(test) regions was considered and rejected — a gate that guesses\n"
        "  wrong in the LENIENT direction is worse than one that asks.",
        file=sys.stderr,
    )
    sys.exit(1)


if __name__ == "__main__":
    main()
