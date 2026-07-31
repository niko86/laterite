#!/usr/bin/env python3
"""Build each publish-set crate on the `rust-version` it declares.

`rust-version` is a promise to a toolchain we do not control. Cargo enforces it
in one direction only — it refuses to USE a dependency whose `rust-version`
exceeds the running compiler — and never checks that our own crates honour their
own. So the field sat at 1.85 across twenty-four crates, unverified, and the
first run of this script found a `let` chain (stable in 1.88) in
`laterite-ags4-emit`: the promise had been false for as long as that line
existed, and nothing anywhere could say so.

On crates.io the field is load-bearing in a way it is not in a workspace. A
consumer pinned to an older toolchain resolves to the newest version whose
`rust-version` fits; declare a floor lower than the code needs and they resolve
to us and fail to compile, with a diagnostic that points at our source and not
at the wrong promise that sent them there.

## The toolchain comes FROM the manifests

Deliberately not a constant here. Hard-coding "test on 1.85" beside a manifest
that says `rust-version = "1.85"` is two statements of one fact, and raising the
floor would leave this testing the old one — passing, while checking nothing
that matters. Crates are grouped by declared version and each group is built on
its own, so a crate that legitimately needs a higher floor stays honest without
dragging the rest up with it.

## Libraries only

No `--all-targets`. The promise covers what a consumer compiles: the library.
Our benches pull `criterion`, which itself requires 1.86 — including them would
fail the gate on a dev-dependency that is not part of the promise, and the
tempting fix (raise `rust-version` until it passes) would break the promise in
order to make the gate green.

Usage:
    python tools/check_msrv.py
"""

from __future__ import annotations

import subprocess
import sys
import tomllib
from collections import defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
CRATES = REPO / "rust-packages"
WORKSPACE = CRATES / "Cargo.toml"

# One definition of the publish set, next to the packaging gate. See
# tools/check_public_api.py for why this is imported rather than restated.
sys.path.insert(0, str(Path(__file__).resolve().parent))
from check_package_contents import PUBLISH_SET  # noqa: E402


def die(msg: str) -> None:
    print(f"check_msrv: {msg}", file=sys.stderr)
    raise SystemExit(2)


def declared_msrv(crate: str) -> str:
    """The `rust-version` in `crate`'s manifest.

    An absent field is a failure, not a default. Publishing without one tells
    cargo the crate builds on ANY toolchain, which is a broader promise than
    "1.85" and a harder one to keep.
    """
    manifest = CRATES / crate / "Cargo.toml"
    try:
        pkg = tomllib.loads(manifest.read_text(encoding="utf-8"))["package"]
    except (OSError, KeyError) as exc:
        die(f"{crate}: cannot read [package] from {manifest}: {exc}")
    version = pkg.get("rust-version")
    if not isinstance(version, str):
        die(
            f"{crate}: no `rust-version` in its manifest — a published crate without "
            "one claims to build on every toolchain there has ever been"
        )
    return version


def toolchain_installed(version: str) -> bool:
    proc = subprocess.run(
        ["rustup", "toolchain", "list"], capture_output=True, text=True
    )
    return any(ln.startswith(version) for ln in proc.stdout.splitlines())


def main() -> int:
    by_version: dict[str, list[str]] = defaultdict(list)
    for crate in PUBLISH_SET:
        by_version[declared_msrv(crate)].append(crate)

    failed = 0
    for version, crates in sorted(by_version.items()):
        if not toolchain_installed(version):
            die(
                f"toolchain {version} is not installed — "
                f"`rustup toolchain install {version} --profile minimal`"
            )
        print(f"checking {len(crates)} crate(s) on {version}: {', '.join(crates)}")
        proc = subprocess.run(
            [
                "cargo",
                f"+{version}",
                "check",
                "--locked",
                "--manifest-path",
                str(WORKSPACE),
                *[arg for crate in crates for arg in ("-p", crate)],
            ],
            capture_output=True,
            text=True,
        )
        if proc.returncode != 0:
            print(
                f"\ncheck_msrv: the publish set does NOT build on {version}, which its "
                f"manifests promise.\n{proc.stderr.strip()}\n",
                file=sys.stderr,
            )
            print(
                "Fix the code, not the promise: raising `rust-version` to whatever the "
                "\ncode happens to need today makes this gate green by widening what a "
                "\nconsumer's toolchain must be. If the newer feature is genuinely worth "
                "\nit, raise the floor deliberately and say so in the changelog.",
                file=sys.stderr,
            )
            failed += 1

    if failed:
        return 1
    total = sum(len(c) for c in by_version.values())
    floors = ", ".join(sorted(by_version))
    print(f"check_msrv: OK — {total} crates build on their declared floor ({floors})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
