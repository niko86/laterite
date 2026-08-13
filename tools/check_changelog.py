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

Five ways to pass, and the third, fourth and fifth are the interesting ones:

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
  a BOT moved declared versions-> pass, LOUDLY, printing every bump. Dependabot
                                  can do none of the four above: it cannot write
                                  an entry and it cannot apply a label, so every
                                  weekly PR sat red until a human intervened.
                                  See `_BOT_ACTORS` for how narrow this is and
                                  what deliberately still asks.

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
import re
import subprocess
import sys
import tomllib
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

# --- manifests: which TABLE moved, not which file -----------------------------
#
# A path-only rule fires on a dev-tooling bump that changes nothing a consumer
# receives. `globals` 14->17 in laterite-node (#233) is an eslint devDependency;
# `ruff`/`ty`/`marimo`/`hypothesis` in the root pyproject (#236) are all in
# `[dependency-groups]`. Both failed this gate and both were waved through with
# `no-changelog` — and that label only works while applying it stays a deliberate
# act. Make it the routine answer to a weekly bot PR and the one bump that DOES
# change shipped behaviour gets waved through with the rest. #232 was that bump,
# in the same batch: a `@napi-rs/cli` bump whose regenerated loader gained error
# chaining and a new env var, both observable from the published package.
#
# So for these three manifest kinds the gate parses both sides and asks which
# section moved. The allowlists below are DEV-ONLY sections; everything else,
# including anything unrecognised, counts as shipped. That polarity is the point
# — an unknown table must not buy silence, and the module docstring's rule that a
# gate guessing LENIENTLY is worse than one that asks applies here too.
_DEV_ONLY_SECTIONS: dict[str, frozenset[str]] = {
    # `[project]` is the shipped contract (deps, optional-deps, requires-python,
    # scripts); `[tool.maturin]` decides how the wheel is built. Neither is here.
    "pyproject.toml": frozenset(
        {
            "dependency-groups",
            "tool.uv",
            "tool.ruff",
            "tool.pytest",
            "tool.coverage",
            "tool.ty",
            "tool.vulture",
            "tool.mypy",
            "tool.hypothesis",
        }
    ),
    # `[workspace.dependencies]` lives under `workspace`, so a real dep bump in
    # the workspace manifest stays shipped — only a crate's own dev-dependencies
    # are exempt.
    "Cargo.toml": frozenset({"dev-dependencies"}),
    # NOT `overrides` or `optionalDependencies`: both change the tree a consumer
    # installs. Only the dev tree is exempt.
    "package.json": frozenset({"devDependencies"}),
}

# Lockfiles split, and NOT the way "a lock just records the manifests" suggests.
# #237 is the counter-example that settled it: it changed `rust-packages/Cargo.lock`
# and NOTHING else — the manifest range already admitted the new versions, so
# dependabot moved only the lock — while bumping napi 3.11 -> 3.12, which is
# compiled into the published addon. Treating every lock as a follower would have
# let a real dependency change through with no entry and no label.
#
# The asymmetry is about who resolves. `Cargo.lock` decides exactly what gets
# COMPILED into the wheel, the node addon, `lat` and the wasm — this repo ships
# built artefacts, so the lock is part of the shipped contract even when no
# manifest moved. npm and uv consumers resolve their own tree from the published
# RANGES; those locks govern our installs, not theirs.
_SHIPPED_LOCKFILES = ("Cargo.lock",)
_FOLLOWING_LOCKFILES = ("package-lock.json", "uv.lock", "pnpm-lock.yaml")


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


def _blob(ref: str, path: str) -> str | None:
    """One file's text at a ref, or None if it does not exist there.

    Deliberately NOT `_git`: a manifest absent on one side is a legitimate answer
    (the file was added or deleted), and `dev_only_change` turns that None into
    "shipped". Routing it through `_git` would abort the whole run instead.
    """
    out = subprocess.run(
        ["git", "show", f"{ref}:{path}"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    return out.stdout if out.returncode == 0 else None


def shipped(path: str) -> bool:
    """Does a change to `path` alter something a consumer receives?

    Path-level only. For manifests this is the FIRST of two questions — see
    `dev_only_change`, which then asks which table actually moved.
    """
    if any(f"/{path}".find(p) != -1 for p in EXEMPT_PARTS):
        return False
    if path.endswith(EXEMPT_SUFFIXES):
        return False
    return path.startswith(SHIPPED_PREFIXES) or path in SHIPPED_FILES


def lock_follows_manifest(path: str) -> bool:
    """A lock that declares nothing on its own — held back and only counted if
    something else in the same PR shipped. `Cargo.lock` is NOT one of these; see
    the constants above for why."""
    return path.rsplit("/", 1)[-1] in _FOLLOWING_LOCKFILES


def is_shipped_lockfile(path: str) -> bool:
    return path.rsplit("/", 1)[-1] in _SHIPPED_LOCKFILES


def manifest_kind(path: str) -> str | None:
    """Which allowlist applies, if any. Keyed on the file NAME, so every crate's
    Cargo.toml and every package.json in the tree is covered without listing them."""
    name = path.rsplit("/", 1)[-1]
    return name if name in _DEV_ONLY_SECTIONS else None


def sections(data: dict) -> dict[str, object]:
    """Flatten a manifest to comparable sections.

    One level, except `tool`, which flattens to `tool.<name>` — `[tool.ruff]` and
    `[tool.maturin]` are both under `tool` and must not share a verdict.
    """
    out: dict[str, object] = {}
    for key, value in data.items():
        if key == "tool" and isinstance(value, dict):
            out.update({f"tool.{sub}": v for sub, v in value.items()})
        else:
            out[key] = value
    return out


def changed_sections(before: dict, after: dict) -> set[str]:
    a, b = sections(before), sections(after)
    return {k for k in a.keys() | b.keys() if a.get(k) != b.get(k)}


def dev_only_change(kind: str, before: str | None, after: str | None) -> bool:
    """Did this manifest change touch ONLY dev-tooling sections?

    Fails closed everywhere it cannot be sure: a side that is missing (the file
    was added or deleted), text that will not parse, or a section outside the
    allowlist all return False, i.e. shipped. A manifest the gate cannot read is
    a manifest it has not checked.
    """
    if before is None or after is None:
        return False
    try:
        parse = (
            tomllib.loads if kind.endswith(".toml") else lambda s: json.loads(s or "{}")
        )
        moved = changed_sections(parse(before), parse(after))
    except (tomllib.TOMLDecodeError, json.JSONDecodeError, TypeError, ValueError):
        return False
    return bool(moved) and moved <= _DEV_ONLY_SECTIONS[kind]


# --- the fifth way to pass: a bot moving a version it already declared --------
#
# Dependabot can do none of the other four. It cannot write a changelog entry
# and it cannot apply `no-changelog`, so every weekly PR that moves a floor in
# `[project] dependencies` sat red until a human intervened. That is precisely
# the failure `check_upstream_pin.py` refuses to create: a check that fails for
# reasons its author cannot act on is one people learn to click past — and this
# gate's entire value is that a red run still means something.
#
# The waiver is narrow, and what it does NOT cover is the point:
#
#   * `Cargo.lock` is never waivable. It decides what gets COMPILED into the
#     wheel, the addon, `lat` and the wasm, so #237 (napi 3.11 -> 3.12, lock
#     only, no manifest) must keep asking. This falls out for free rather than
#     being special-cased: it is not a following lockfile and has no manifest
#     kind, so it disqualifies the PR by not being classifiable.
#   * A source file anywhere in the same PR disqualifies it, same mechanism.
#     #232 is the shape — a `@napi-rs/cli` bump whose regenerated loader gained
#     error chaining and a new env var, both observable from the published
#     package.
#   * Anything but a version move on a requirement BOTH sides declare — a new
#     dependency, a dropped one, a `requires-python` change, a renamed extra, a
#     reordered list — disqualifies it.
#
# What it cannot see, and no amount of manifest parsing could: whether the new
# version of a dependency changes behaviour a consumer notices. For a range in
# `pyproject.toml` or `package.json` the consumer resolves their own tree, so
# the floor move IS the declaration and the semantics inside it are upstream's.
# That is the argument for where this line sits, and it is also why the line
# stops at anything we compile.
_BOT_ACTORS = frozenset({"dependabot[bot]", "app/dependabot"})

# A name only counts as a name when a constraint (or an extras bracket) follows
# it. Cargo's `serde = "1.0"` and npm's `"eslint": "^9.39.5"` put the name in the
# KEY and the bare constraint in the value; PEP 508 carries both in one string.
_REQUIREMENT = re.compile(
    r"^\s*(?P<name>[A-Za-z][A-Za-z0-9._-]*)\s*(?P<rest>.*)$", re.S
)


def requirement_name(spec: str) -> str:
    """The requirement a specifier names, or `""` when the name lives in the key.

    `polars>=1.43.2` -> `polars`; `pandas<3` -> `pandas`; `laterite` -> `laterite`
    (a bare name still identifies itself, so swapping it is not a bump).
    `1.0`, `^9.39.5` and `>=1.43.2` -> `""`, because there is nothing to compare
    and the key path above them already established which package it is.
    """
    m = _REQUIREMENT.match(spec)
    return m.group("name") if m else ""


def leaves(
    node: object, path: tuple[str, ...] = ()
) -> list[tuple[tuple[str, ...], object]]:
    """Every scalar in a parsed manifest, keyed by its full path.

    List items are keyed by INDEX, so a reordered dependency list reads as a
    change rather than a match. That is the conservative direction: a reorder we
    cannot explain should ask, not shrug.
    """
    if isinstance(node, dict):
        return [x for k, v in node.items() for x in leaves(v, (*path, str(k)))]
    if isinstance(node, list):
        return [x for i, v in enumerate(node) for x in leaves(v, (*path, str(i)))]
    return [(path, node)]


def bumps_only(kind: str, before: str | None, after: str | None) -> list[str] | None:
    """The version moves in this manifest, or None if anything else moved.

    Fails closed exactly the way `dev_only_change` does — a missing side, text
    that will not parse, a key that appears or disappears, or a value that is not
    a string on both sides all return None. "I could not tell" must never read as
    "nothing shipped".
    """
    if before is None or after is None:
        return None
    try:
        parse = (
            tomllib.loads if kind.endswith(".toml") else lambda s: json.loads(s or "{}")
        )
        a = dict(leaves(parse(before)))
        b = dict(leaves(parse(after)))
    except (tomllib.TOMLDecodeError, json.JSONDecodeError, TypeError, ValueError):
        return None
    if a.keys() != b.keys():
        return None

    moved = []
    for key, old in a.items():
        new = b[key]
        if old == new:
            continue
        if not isinstance(old, str) or not isinstance(new, str):
            return None
        if requirement_name(old) != requirement_name(new):
            return None
        moved.append(f"{'.'.join(key)}: {old} -> {new}")
    return moved


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
    ap.add_argument(
        "--actor",
        default="",
        help=(
            "the PR author (github.event.pull_request.user.login). Only ever read "
            "to enable the bot waiver — a contributor cannot set it, because it "
            "comes from the event payload rather than from the branch."
        ),
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

    candidates = sorted(p for p in changed if shipped(p))

    # Second pass over the manifests: a dev-tooling bump changes a shipped FILE
    # without changing a shipped SURFACE. Locks are held back and decided last —
    # they follow their manifests rather than declaring anything themselves.
    touched, dev_only, locks = [], [], []
    for path in candidates:
        if lock_follows_manifest(path):
            locks.append(path)
            continue
        kind = manifest_kind(path)
        if kind and dev_only_change(kind, _blob(merge_base, path), _blob("HEAD", path)):
            dev_only.append(path)
            continue
        touched.append(path)
    if touched:
        touched += locks
    else:
        dev_only += locks

    # Say what was set aside, always. A gate that narrows silently is how you get
    # a gate nobody knows has stopped covering something.
    if dev_only:
        print(
            f"check_changelog: {len(dev_only)} file(s) changed only dev-tooling "
            "sections (or are lockfiles with no shipped manifest change):"
        )
        for p in dev_only:
            print(f"    {p}")

    if not touched:
        print(
            f"check_changelog: OK — {len(changed)} file(s) changed, none of them a "
            "shipped surface (tests / CI / docs / tooling / dev deps)."
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

    if args.actor in _BOT_ACTORS:
        waived: dict[str, list[str]] | None = {}
        for path in touched:
            if lock_follows_manifest(path):
                continue  # already settled above: it declares nothing on its own
            kind = manifest_kind(path)
            bumps = (
                bumps_only(kind, _blob(merge_base, path), _blob("HEAD", path))
                if kind
                else None
            )
            if bumps is None:
                waived = None
                break
            waived[path] = bumps
        if waived is not None:
            print(
                f"check_changelog: OK — {args.actor} moved declared versions only, "
                f"across {len(waived)} manifest(s):"
            )
            for path, bumps in waived.items():
                for line in bumps:
                    print(f"    {path}  {line}")
            print(
                "\n  No entry required: for a manifest RANGE the floor move is itself the\n"
                "  declaration, and the consumer's own resolver decides the rest. A source\n"
                "  file or a Cargo.lock in the same PR would have disqualified this — see\n"
                "  `_BOT_ACTORS` for why the line sits there."
            )
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
