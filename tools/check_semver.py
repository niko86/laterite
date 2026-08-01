#!/usr/bin/env python3
"""Run `cargo semver-checks` over the publish set, against a git baseline.

The snapshot in `tools/release/public-api/` says WHAT changed in a crate's public
API. This says whether the change is **breaking** — whether a consumer's code
stops compiling — which is the question a version number is supposed to answer.

## Why a baseline revision rather than the registry

The stronger baseline is the last published version, and `cargo semver-checks`
prefers it. It is not usable here yet: the crates went to crates.io for the first
time on 2026-08-01, so for most of this repo's history there was nothing to
compare against, and a newly-added crate still has nothing. `main` is the
baseline that always exists. At the point where every publishable crate has a
release the registry baseline should replace this.

## A new crate has no baseline, and that is not a failure

This is the reason this is a script and not one `cargo semver-checks` command.
Passing `-p laterite` when `laterite` does not exist on `main` does not skip it —
the whole run aborts with `package 'laterite' not found`, taking the other ten
crates' results with it. That happened on the PR that added the crate.

So crates absent from the baseline are excluded from the invocation and
**reported by name**. A gate that quietly checked ten of eleven crates would look
exactly like one that checked all eleven.

Usage:
    python tools/check_semver.py                     # against origin/main
    python tools/check_semver.py --baseline <rev>    # against any revision
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
CRATES = REPO / "rust-packages"

sys.path.insert(0, str(Path(__file__).resolve().parent))
from check_package_contents import PUBLISH_SET  # noqa: E402


def die(msg: str) -> None:
    print(f"check_semver: {msg}", file=sys.stderr)
    raise SystemExit(2)


def in_baseline(crate: str, baseline: str) -> bool:
    """Did `crate` exist at `baseline`?

    Asked of git rather than of `cargo semver-checks`, so the answer arrives
    before the run instead of as a fatal error part-way through it.
    """
    proc = subprocess.run(
        ["git", "cat-file", "-e", f"{baseline}:rust-packages/{crate}/Cargo.toml"],
        capture_output=True,
        cwd=REPO,
    )
    return proc.returncode == 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--baseline",
        default="origin/main",
        help="revision to compare against (default: origin/main)",
    )
    args = ap.parse_args()

    proc = subprocess.run(
        ["git", "rev-parse", "--verify", args.baseline],
        capture_output=True,
        text=True,
        cwd=REPO,
    )
    if proc.returncode != 0:
        die(f"no such revision `{args.baseline}` — fetch it first")

    checkable = [c for c in PUBLISH_SET if in_baseline(c, args.baseline)]
    new = [c for c in PUBLISH_SET if c not in checkable]

    for crate in new:
        print(
            f"  skipped {crate} — did not exist at {args.baseline}, so there is no "
            "prior API to break"
        )
    if not checkable:
        # Every crate being new is possible exactly once, and is not a pass.
        die(
            f"no crate in the publish set exists at {args.baseline} — refusing to "
            "report a vacuous success"
        )

    print(f"checking {len(checkable)} crate(s) against {args.baseline}\n")
    result = subprocess.run(
        [
            "cargo",
            "semver-checks",
            "--baseline-rev",
            args.baseline,
            *[arg for crate in checkable for arg in ("-p", crate)],
        ],
        cwd=CRATES,
    )
    return 1 if result.returncode != 0 else 0


if __name__ == "__main__":
    raise SystemExit(main())
