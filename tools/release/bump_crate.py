"""Bump ONE published engine crate — the per-crate half of what bump-version.sh was.

#781 (decided 2026-08-29) retired the engine lockstep: each published crate
carries its own version, moves when it changes, and publishes individually.
This is the tool that moves one. It rewrites the PAIR that must agree —

  * the crate's own `version` in `rust-packages/<crate>/Cargo.toml`, and
  * the `[workspace.dependencies]` entry siblings declare it at (the publish
    floor; cargo strips `path` at publish, so this is what a published sibling
    actually pins) —

then regenerates `Cargo.lock` and runs the faithfulness gate, so the two
spellings cannot drift apart by hand-editing one of them.
`tests/test_version_faithful.py::test_engine_dependency_versions_match_engine`
asserts the same equality per crate; this tool is why it should never fire.

The facade (`laterite`) is bumpable here too: it has no
`[workspace.dependencies]` entry, so only its manifest moves — that asymmetry
is expected, not a partial failure.

No commit, no tag, no publish — deliberately less than bump-version.sh does.
A per-crate bump is an input to a PR (or to the nightly cut machinery #781
step 4 designs), not a release act on its own. The publisher skips crates whose
version matches the registry, so a content change WITHOUT a bump silently keeps
the registry stale — that trap is recorded on #781, and running this is how the
next publish stops skipping the crate.

Usage:
    uv run --no-project python tools/release/bump_crate.py <crate> <major|minor|patch>
    uv run --no-project python tools/release/bump_crate.py <crate> --new-version X.Y.Z
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
WORKSPACE = ROOT / "rust-packages" / "Cargo.toml"

_VERSION_LINE = re.compile(r'^version = "([0-9]+)\.([0-9]+)\.([0-9]+)"', re.MULTILINE)


def die(msg: str) -> None:
    print(f"bump_crate: {msg}", file=sys.stderr)
    raise SystemExit(1)


def publishable_crates() -> list[str]:
    """Crates with their OWN version and no `publish = false` — the bumpable set.

    Derived from the manifests rather than listed, so a crate joining the
    publish set is covered the day it lands. The aux crates (`version.workspace
    = true`) and the product surfaces (`publish = false`) both fall out.
    """
    out = []
    for manifest in sorted(ROOT.glob("rust-packages/*/Cargo.toml")):
        text = manifest.read_text()
        if "publish = false" in text:
            continue
        if _VERSION_LINE.search(text):
            out.append(manifest.parent.name)
    return out


def bump(crate: str, part_or_version: str) -> None:
    manifest = ROOT / "rust-packages" / crate / "Cargo.toml"
    crates = publishable_crates()
    if crate not in crates:
        die(
            f"'{crate}' is not a publishable crate with its own version — one of: {', '.join(crates)}"
        )

    text = manifest.read_text()
    m = _VERSION_LINE.search(text)
    assert m  # publishable_crates() proved it
    old = ".".join(m.groups())
    major, minor, patch = (int(g) for g in m.groups())

    if part_or_version == "major":
        new = f"{major + 1}.0.0"
    elif part_or_version == "minor":
        new = f"{major}.{minor + 1}.0"
    elif part_or_version == "patch":
        new = f"{major}.{minor}.{patch + 1}"
    elif re.fullmatch(r"[0-9]+\.[0-9]+\.[0-9]+", part_or_version):
        new = part_or_version
    else:
        die(f"expected major|minor|patch or an X.Y.Z version, got {part_or_version!r}")
    if new == old:
        die(f"{crate} is already {old} — nothing to do")

    manifest.write_text(text.replace(f'version = "{old}"', f'version = "{new}"', 1))
    print(f"bump_crate: {crate} {old} -> {new}")

    ws = WORKSPACE.read_text()
    entry = re.compile(
        rf'^({re.escape(crate)} = \{{[^}}]*version = "){re.escape(old)}(")',
        re.MULTILINE,
    )
    ws_new, n = entry.subn(rf"\g<1>{new}\g<2>", ws)
    if n == 1:
        WORKSPACE.write_text(ws_new)
        print(f"bump_crate:   [workspace.dependencies] floor {old} -> {new}")
    elif crate == "laterite":
        print("bump_crate:   (facade has no [workspace.dependencies] entry — expected)")
    else:
        die(
            f"found {n} [workspace.dependencies] entries for {crate} at {old} — "
            "expected exactly 1; the manifest edit is applied, fix the entry by hand"
        )

    print("bump_crate: regenerating Cargo.lock…")
    subprocess.run(
        ["cargo", "update", "--workspace", "--quiet"],
        cwd=ROOT / "rust-packages",
        check=True,
    )
    print("bump_crate: verifying version faithfulness…")
    subprocess.run(
        ["uv", "run", "--no-sync", "pytest", "tests/test_version_faithful.py", "-q"],
        cwd=ROOT,
        check=True,
    )
    print(
        f"bump_crate: done — {crate} {new} stamped, not committed. The next\n"
        "  publish_crates.py run publishes it (siblings that did not bump are\n"
        "  skipped by version identity, which after THIS bump is the correct skip)."
    )


def main() -> None:
    if len(sys.argv) == 4 and sys.argv[2] == "--new-version":
        bump(sys.argv[1], sys.argv[3])
    elif len(sys.argv) == 3:
        bump(sys.argv[1], sys.argv[2])
    else:
        die(
            "usage: bump_crate.py <crate> <major|minor|patch>  or  <crate> --new-version X.Y.Z"
        )


if __name__ == "__main__":
    main()
