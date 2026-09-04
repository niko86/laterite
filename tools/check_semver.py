#!/usr/bin/env python3
"""Run `cargo semver-checks` over the publish set, against the crates.io registry.

The snapshot in `tools/release/public-api/` says WHAT changed in a crate's public
API. This says whether the change is **breaking** — whether a consumer's code
stops compiling — which is the question a version number is supposed to answer.

## Why the registry and not `main`

A version number is a promise to whoever ran `cargo add`. So the baseline has to
be **what they got**, which is the registry. `main` was the baseline until #782
for a stated reason — the crates went to crates.io for the first time on
2026-08-01, and for most of this repo's history there was nothing to compare
against — and this file carried its own switch-over condition: *"at the point
where every publishable crate has a release the registry baseline should replace
this."* Every crate then in the set had one, so it did — and a crate that joins
the set ahead of its first release (excel, at dec-facade-parity phase 5) is the
ABSENT case below, excluded and named, not a reason to reopen the switch.

The git baseline had a defect that is easy to miss, because it looks like
strictness. Both sides of the comparison carry the SAME version, so
`cargo semver-checks` reports `0.11.0 -> 0.11.0 (no change; assume minor)` and
fails every break — including one the tree's version already accounts for. That
is what made each breaking PR need its own bump (#730 → 0.10.0, #741 → 0.11.0,
#776 → 0.12.0), none of which published: the registry served 0.9.0 throughout.

## What the registry baseline does NOT do, and this is the important half

Once the tree is ahead of the registry, semver **already permits** the break, so
`cargo semver-checks` skips every lint and says so — `Checking X v0.9.0 ->
v0.12.0 (major change)`, `0 checks: 0 pass, 253 skip`. That is the correct
answer, not a hole: you declared the bump, you are allowed to break. But it means
this gate ENFORCES NOTHING for the rest of a cycle once the first bump lands, and
a gate that reports green while looking at nothing is exactly the shape this repo
refuses to ship silently.

So every run prints, per crate, whether it was **enforcing** (tree == published,
every lint live) or **ahead** (the version pair already permits the change). The
useful window is right after a publish, when the two are level — which is also
the only moment a missing bump could actually reach a consumer.

The number of lints that ran is NOT restated here. `cargo semver-checks` prints
it per crate on every run and that is the instrument; a second copy computed from
the version pair would be a reading, and would drift the first time the tool's
inference changes.

## A crate absent from the registry is not a failure

This is the reason this is a script and not one `cargo semver-checks` command. A
crate that has never published has no baseline, and asking for one aborts the
whole run rather than skipping that crate — taking every other crate's results
with it. That happened on the PR that added `laterite`, against the git baseline,
for the same structural reason.

So crates absent from the registry are excluded from the invocation and
**reported by name**. A gate that quietly checked all but one crate would look
exactly like one that checked the whole set.

Usage:
    python tools/check_semver.py                       # against the registry
    python tools/check_semver.py --baseline-rev <rev>  # against a git revision

`--baseline-rev` answers a DIFFERENT question — "what did this branch change
relative to that revision?" — which is occasionally worth asking by hand. It is
not what the gate runs, because a revision is not what any consumer installed.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
import tomllib
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
CRATES = REPO / "rust-packages"

sys.path.insert(0, str(Path(__file__).resolve().parent))
from check_package_contents import PUBLISH_SET  # noqa: E402


def die(msg: str) -> None:
    print(f"check_semver: {msg}", file=sys.stderr)
    raise SystemExit(2)


def tree_version(crate: str) -> str:
    """The version this tree would publish `crate` at, resolving inheritance."""
    man = tomllib.loads((CRATES / crate / "Cargo.toml").read_text())
    v = man.get("package", {}).get("version")
    if isinstance(v, str):
        return v
    ws = tomllib.loads((CRATES / "Cargo.toml").read_text())
    return ws["workspace"]["package"]["version"]


# `cargo info` writes `version:` in bold green when colour is on, and CI sets
# CARGO_TERM_COLOR=always at the job level — so the field label arrives wrapped in
# escapes and a plain `startswith` never matches it. Stripped rather than
# suppressed: an inherited TERM or a future default could turn colour back on, and
# a parser that only works when someone remembered to disable it is the same bug
# waiting.
_ANSI = re.compile(r"\x1b\[[0-9;]*m")

# cargo's own wording for a crate the registry does not have. Matching it is what
# separates "never published" from "could not ask" — see `published_version`.
_NOT_IN_REGISTRY = re.compile(r"could not find `[^`]+` in registry")


def published_version(crate: str) -> str | None:
    """The version crates.io serves for `crate`, or None if it has never published.

    Asked of cargo rather than of `cargo semver-checks`, so the answer arrives
    before the run instead of as a fatal error part-way through it — the same
    reason the git baseline asked git.

    Run from the REPO ROOT, which is deliberately outside the Cargo workspace:
    inside `rust-packages/` these crate names also name workspace members, and a
    lookup that could resolve to the member would report the tree's own number as
    though the registry served it.

    **None means "the registry does not have this crate", and nothing else.** Any
    other failure — a network fault, a subcommand that is not there, output this
    cannot parse — raises. The first cut returned None for all of them, and the
    difference is not cosmetic: `None` routes a crate into the ABSENT bucket,
    which is a claim that there is no prior API to break. Under CI's forced colour
    every one of the eleven took that path and the run reported eleven crates as
    never published, all of which are on crates.io. Only the "nothing checkable"
    guard turned that into a failure rather than a green vacuous pass.
    """
    proc = subprocess.run(
        ["cargo", "info", crate],
        capture_output=True,
        text=True,
        cwd=REPO,
    )
    stderr = _ANSI.sub("", proc.stderr)
    if proc.returncode != 0:
        if _NOT_IN_REGISTRY.search(stderr):
            return None
        die(f"cannot ask the registry about {crate}:\n{stderr.strip()}")
    found = version_in(proc.stdout)
    if found is None:
        die(
            f"`cargo info {crate}` succeeded but printed no version line — refusing "
            f"to guess whether it is published:\n{proc.stdout.strip()}"
        )
    return found


def version_in(stdout: str) -> str | None:
    """The `version:` field out of `cargo info` output, colour or no colour."""
    for line in _ANSI.sub("", stdout).splitlines():
        if line.startswith("version:"):
            return line.split(":", 1)[1].strip()
    return None


def classify(
    published: dict[str, str | None],
) -> tuple[list[str], list[str], list[str]]:
    """Split the publish set into (enforcing, ahead, absent).

    Extracted from the run so the partition can be tested without a network: the
    property worth holding is that every crate lands in exactly ONE bucket, which
    is what makes "reported what it skipped" true rather than aspirational.
    """
    enforcing: list[str] = []
    ahead: list[str] = []
    absent: list[str] = []
    for crate, live in published.items():
        if live is None:
            absent.append(crate)
        elif live == tree_version(crate):
            enforcing.append(crate)
        else:
            ahead.append(crate)
    return enforcing, ahead, absent


def in_git_baseline(crate: str, baseline: str) -> bool:
    """Did `crate` exist at `baseline`? Only for the `--baseline-rev` escape hatch."""
    proc = subprocess.run(
        ["git", "cat-file", "-e", f"{baseline}:rust-packages/{crate}/Cargo.toml"],
        capture_output=True,
        cwd=REPO,
    )
    return proc.returncode == 0


def run(argv: list[str]) -> int:
    # Flush first: cargo inherits this stdout, and a buffered report would land
    # AFTER the output it is meant to frame — which reads as no report at all.
    sys.stdout.flush()
    result = subprocess.run(["cargo", "semver-checks", *argv], cwd=CRATES)
    return 1 if result.returncode != 0 else 0


def against_git(baseline: str) -> int:
    proc = subprocess.run(
        ["git", "rev-parse", "--verify", baseline],
        capture_output=True,
        text=True,
        cwd=REPO,
    )
    if proc.returncode != 0:
        die(f"no such revision `{baseline}` — fetch it first")

    checkable = [c for c in PUBLISH_SET if in_git_baseline(c, baseline)]
    for crate in (c for c in PUBLISH_SET if c not in checkable):
        print(
            f"  skipped {crate} — did not exist at {baseline}, so there is no "
            "prior API to break"
        )
    if not checkable:
        # Every crate being new is possible exactly once, and is not a pass.
        die(
            f"no crate in the publish set exists at {baseline} — refusing to "
            "report a vacuous success"
        )

    print(
        f"check_semver: baseline is the git revision {baseline}, NOT the registry — "
        "this answers what the branch changed, not what a consumer would notice.\n"
    )
    return run(
        [
            "--baseline-rev",
            baseline,
            *[arg for crate in checkable for arg in ("-p", crate)],
        ]
    )


def against_registry() -> int:
    published = {c: published_version(c) for c in PUBLISH_SET}
    enforcing, ahead, absent = classify(published)
    checkable = enforcing + ahead

    print("check_semver: baseline is crates.io — what `cargo add` resolves.")
    for crate in enforcing:
        print(f"  ENFORCING  {crate} {published[crate]} — level with the registry")
    if ahead:
        print(
            "  ahead      the version pair already permits the change, so cargo skips\n"
            "             the lints and prints the count it skipped, per crate:"
        )
        for crate in ahead:
            print(
                f"               {crate:<26} {published[crate]} -> {tree_version(crate)}"
            )
    for crate in absent:
        print(f"  ABSENT     {crate} — never published, so there is no API to break")

    if not checkable:
        # Possible exactly once, on a repo that has published nothing.
        die(
            "no crate in the publish set is on crates.io — refusing to report a "
            "vacuous success"
        )
    print(
        f"\ncheck_semver: {len(enforcing)} of {len(PUBLISH_SET)} crate(s) enforcing; "
        f"{len(ahead)} ahead of the registry, {len(absent)} never published. A run "
        "with none enforcing gates nothing — that is the state after the cycle's "
        "first bump, not a defect.\n"
    )
    return run([arg for crate in checkable for arg in ("-p", crate)])


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--baseline-rev",
        metavar="REV",
        help=(
            "compare against a git revision instead of the registry — a different "
            "question, kept for hand use; not what the gate runs"
        ),
    )
    args = ap.parse_args()
    return against_git(args.baseline_rev) if args.baseline_rev else against_registry()


if __name__ == "__main__":
    raise SystemExit(main())
