#!/usr/bin/env python3
"""Diff `cargo package --list` against a checked-in manifest, per crate.

`cargo package` ships everything in a crate directory that is not excluded. No
crate here specified `include` until laterite#159, so the tarballs carried
`benches/`, every `tests/` file, and whatever else happened to be sitting in the
directory — `laterite-ags4-validator` alone would have shipped 41 test/bench
files and 2.1 MB of reference data no code reads.

Two reasons that is worth a gate rather than a one-time fix:

**crates.io is append-only.** `yank` marks a version unusable for new
resolution; it does not remove the tarball. A file that rides one publish is
public for good. This repo's discipline for that class of mistake is to grep the
diff before pushing — a habit that never sees a `.crate` tarball, because the
tarball is assembled at publish time from rules nobody re-reads.

**`include` is a whitelist, and whitelists rot silently in the safe direction
too.** A new `src/` module is picked up by `/src/**` automatically; a new
top-level directory is not. Both are invisible without something that compares
the actual list to an expected one.

So the manifest is the expected list and this is the comparison. It is
deliberately an EXACT set match, not "no unexpected files": a file vanishing from
a tarball is how you ship a crate that cannot build, and that failure is much
harder to read from the consumer's end than an extra file is.

The manifest records the WHOLE tarball, including the four files cargo writes
into every one regardless of `include` (`Cargo.toml`, `Cargo.toml.orig`,
`Cargo.lock`, `.cargo_vcs_info.json`). Filtering those out would make the
manifest a view of the tarball rather than the tarball, and the point is to see
everything that ships.

## What this does NOT check

`--list` asks cargo what it would put in the tarball. It does not build the
result. An `include` that drops a build-time input (a `build.rs`, a `data/` file
read by one) still produces a listable tarball that fails to compile for the
consumer.

`cargo package` without `--list` does run that verification build, and
`--verify-buildable` invokes it for every crate it can. How far that reaches is a
moving target, so the flag re-derives it each run rather than reading a second
hard-coded list:

- The in-workspace deps carry version requirements now, so cargo no longer refuses
  outright — that was the first ceiling and it is gone.
- The ceiling now is that a verification build resolves its dependencies from
  crates.io, where none of these are published. So the three dependency-free
  leaves build, and the other seven cannot until the wave beneath them exists on
  the registry. That is inherent to a first publish, not a defect.

Every skip prints its reason. A gate that quietly verified almost nothing would be
indistinguishable from one that verified everything.

Usage:
    python tools/check_package_contents.py            # compare against the manifest
    python tools/check_package_contents.py --write     # regenerate it after an intended change
    python tools/check_package_contents.py --verify-buildable
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
WORKSPACE = REPO / "rust-packages" / "Cargo.toml"
MANIFEST = REPO / "tools" / "release" / "package-contents.json"

#: Every crate that may reach crates.io, gated whether or not it has published
#: yet — "we'll add the gate when we publish it" is how the ungated one goes out.
#:
#: Two tiers, from `ags-wiki/design/dec-rust-api-crates-io.md`:
#:
#: * `laterite` — the USER-FACING facade, on its own 0.1.x clock. The only one a
#:   stranger is expected to depend on directly.
#: * the ten `laterite-ags4-*` / `laterite-transport` ENGINE crates, lockstep on
#:   the workspace version, which reshape as the format work demands.
#:
#: Both tiers are gated identically. The facade exists so the engine can move
#: without breaking consumers, but the engine is published too, so its surface
#: is a promise as well — just a promise to fewer people.
PUBLISH_SET = [
    "laterite",
    "laterite-ags4-core",
    "laterite-ags4-diff",
    "laterite-ags4-emit",
    "laterite-ags4-merge",
    "laterite-ags4-parse",
    "laterite-ags4-reference",
    "laterite-ags4-trust",
    "laterite-ags4-types",
    "laterite-ags4-validator",
    "laterite-transport",
]


def die(msg: str) -> None:
    print(f"check_package_contents: {msg}", file=sys.stderr)
    raise SystemExit(2)


def package_list(crate: str) -> list[str]:
    """The files `cargo package` would put in `crate`'s tarball.

    `--allow-dirty` because this runs on a working tree that legitimately has
    uncommitted changes (a PR branch mid-edit, or a `--write` right after
    editing an `include`). It only affects cargo's refusal to package a dirty
    tree; it does not change WHICH files are listed.
    """
    proc = subprocess.run(
        [
            "cargo",
            "package",
            "--manifest-path",
            str(WORKSPACE),
            "--list",
            "--allow-dirty",
            "-p",
            crate,
        ],
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        die(f"`cargo package --list -p {crate}` failed:\n{proc.stderr.strip()}")
    files = sorted(ln.strip() for ln in proc.stdout.splitlines() if ln.strip())
    # Zero is a bad witness: an empty list would make every comparison below
    # pass vacuously, and the most likely cause (cargo run from the wrong
    # directory) produces exactly that.
    if not files:
        die(f"{crate}: cargo listed NO files — refusing to treat that as a result")
    return files


def collect() -> dict[str, list[str]]:
    return {crate: package_list(crate) for crate in PUBLISH_SET}


#: `cargo package` failures that mean "not publishable YET", not "broken".
#:
#: Both are states of publish prep, and each is a strictly better place to be than
#: the last:
#:
#: 1. No version requirement on an in-workspace dep — cargo refuses outright. This
#:    was every dependent crate until the `[workspace.dependencies]` entries landed.
#: 2. The dependency has a version but is not ON crates.io yet, so the verification
#:    build cannot resolve it. Inherent to a first publish and the reason the plan
#:    publishes in waves: the leaves go first, and each wave becomes verifiable only
#:    once the wave under it is on the registry.
#:
#: Matched narrowly on OUR crate names. A missing EXTERNAL crate is a real failure
#: and must stay one — that is a broken dependency, not an unpublished sibling.
NOT_YET_PUBLISHABLE = (
    (
        "must have a version requirement",
        "an in-workspace dep has no version requirement",
    ),
    (
        "no matching package named `laterite",
        "depends on a sibling not yet on crates.io",
    ),
)


def verify_buildable() -> int:
    """Run the real verification build wherever it is reachable today.

    A crate that cannot be packaged for one of the [`NOT_YET_PUBLISHABLE`] reasons
    is SKIPPED rather than failed — those are known states of publish prep, not
    defects in its `include`. Skips are printed WITH the reason, because a gate that
    silently skipped everything would look identical to one that passed.
    """
    failed = 0
    for crate in PUBLISH_SET:
        proc = subprocess.run(
            [
                "cargo",
                "package",
                "--manifest-path",
                str(WORKSPACE),
                "--allow-dirty",
                "-p",
                crate,
            ],
            capture_output=True,
            text=True,
        )
        reason = next(
            (why for needle, why in NOT_YET_PUBLISHABLE if needle in proc.stderr), None
        )
        if proc.returncode == 0:
            print(f"  built   {crate}")
        elif reason:
            print(f"  skipped {crate} ({reason})")
        else:
            print(f"  FAILED  {crate}\n{proc.stderr.strip()}", file=sys.stderr)
            failed += 1
    return failed


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--write", action="store_true", help="regenerate the manifest")
    ap.add_argument(
        "--verify-buildable",
        action="store_true",
        help="also build each packaged tarball where the dep versions allow it",
    )
    args = ap.parse_args()

    actual = collect()

    if args.write:
        MANIFEST.write_text(
            json.dumps(actual, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
        )
        total = sum(len(v) for v in actual.values())
        print(
            f"wrote {MANIFEST.relative_to(REPO)} — {len(actual)} crates, {total} files"
        )
        return 0

    if not MANIFEST.exists():
        die(f"{MANIFEST.relative_to(REPO)} is missing — run with --write to create it")

    expected = json.loads(MANIFEST.read_text(encoding="utf-8"))

    if set(expected) != set(PUBLISH_SET):
        die(
            "the manifest and PUBLISH_SET name different crates: "
            f"manifest-only={sorted(set(expected) - set(PUBLISH_SET))} "
            f"code-only={sorted(set(PUBLISH_SET) - set(expected))}"
        )

    problems = []
    for crate in PUBLISH_SET:
        want, got = set(expected[crate]), set(actual[crate])
        if added := sorted(got - want):
            problems.append(f"  {crate}: NEW in the tarball: {added}")
        if gone := sorted(want - got):
            problems.append(f"  {crate}: GONE from the tarball: {gone}")

    if problems:
        print("check_package_contents: the packaged file set changed.\n")
        print("\n".join(problems))
        print(
            "\nIf this is intended, run `python tools/check_package_contents.py --write`"
            "\nand commit the manifest with the change that caused it. Read the additions"
            "\nfirst: crates.io is append-only, so a file that ships once cannot be"
            "\nwithdrawn."
        )
        return 1

    total = sum(len(v) for v in actual.values())
    print(
        f"check_package_contents: OK — {len(PUBLISH_SET)} crates, {total} files, no drift"
    )

    if args.verify_buildable:
        print("\nverification builds:")
        if verify_buildable():
            return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
