#!/usr/bin/env python3
"""Publish the engine crates to crates.io, in dependency waves, idempotently.

A first publish is not one command repeated eight times. The crates depend on
each other, and a dependent cannot be published until the crate it depends on is
**resolvable from the registry** — not merely uploaded. crates.io accepts an
upload and then takes a short while to make it appear in the index, so a script
that publishes the whole list in a loop fails partway through with
`no matching package named 'laterite-ags4-parse' found` and leaves the release
half-done. Half-done is the bad state here: crates.io is append-only, so the
crates that did go out cannot be withdrawn while the rest are fixed.

So this waits for each wave to become resolvable before starting the next, and
it is **idempotent** — a crate already on the registry at this version is
skipped rather than retried. Re-running after a failure resumes; it does not
start over.

## The waves are computed, not written down

They are derived from the manifests each run. A hard-coded order would be a
second statement of the dependency graph, and the failure it produces (a crate
published before something it needs) is exactly the one this exists to prevent.

## What it refuses to do

- Publish from a dirty tree, or from anything other than `main`. What goes to
  crates.io is immutable; it must be a commit that exists and passed CI.
- Publish a crate whose manifest says `publish = false`. That flag is the safety
  catch; this reports it rather than working around it.
- Publish `laterite-ags4-diff` / `laterite-ags4-merge`. They are held for 0.2 by
  the decision recorded in `ags-wiki/design/dec-rust-api-crates-io.md`, and a
  tool that quietly included them would be how a deferred crate goes out.

Nothing happens without `--execute`. The default run performs every check and
prints exactly what it would do.

Usage:
    python tools/publish_crates.py              # dry run — check everything, publish nothing
    python tools/publish_crates.py --execute    # actually publish
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
import tomllib
import urllib.error
import urllib.request
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
CRATES = REPO / "rust-packages"
WORKSPACE = CRATES / "Cargo.toml"

sys.path.insert(0, str(Path(__file__).resolve().parent))
from check_package_contents import PUBLISH_SET  # noqa: E402

#: Held for 0.2 — see the design page. Gated by the packaging tools (all ten) but
#: NOT published in round one, which is eight.
DEFERRED = {"laterite-ags4-diff", "laterite-ags4-merge"}

#: How long to wait for an uploaded crate to become resolvable from the index.
#: Generous: the cost of waiting too long is a slow release, and the cost of not
#: waiting long enough is a release that stops halfway through and cannot be
#: rolled back.
INDEX_TIMEOUT_S = 600
INDEX_POLL_S = 10

UA = "laterite-publish-script (https://github.com/niko86/laterite)"


def die(msg: str) -> None:
    print(f"publish_crates: {msg}", file=sys.stderr)
    raise SystemExit(2)


def sh(*args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
    proc = subprocess.run(args, capture_output=True, text=True, cwd=REPO)
    if check and proc.returncode != 0:
        die(f"`{' '.join(args)}` failed:\n{proc.stderr.strip()}")
    return proc


def manifest(crate: str) -> dict:
    return tomllib.loads((CRATES / crate / "Cargo.toml").read_text(encoding="utf-8"))


def workspace_version() -> str:
    return tomllib.loads(WORKSPACE.read_text(encoding="utf-8"))["workspace"]["package"][
        "version"
    ]


def waves(crates: set[str]) -> list[list[str]]:
    """`crates` grouped so every crate follows everything inside `crates` it needs.

    Derived from the manifests, so it cannot disagree with them. Dependencies
    OUTSIDE the set are ignored: a crate held back for 0.2 is not a reason to
    hold back something that does not depend on it.
    """
    deps: dict[str, set[str]] = {}
    for c in crates:
        m = manifest(c)
        names: set[str] = set()
        for section in ("dependencies", "build-dependencies"):
            names |= {k for k in m.get(section, {}) if k in crates}
        deps[c] = names

    out, done = [], set()
    while len(done) < len(crates):
        layer = sorted(c for c in crates - done if deps[c] <= done)
        if not layer:
            die(f"dependency cycle among {sorted(crates - done)}")
        out.append(layer)
        done |= set(layer)
    return out


def on_registry(crate: str, version: str) -> bool:
    """Is `crate` at `version` resolvable from crates.io right now?

    404 means the crate does not exist yet — a normal state before its first
    publish, not an error. Any other failure is treated as "not yet", because
    the caller's response to both is the same: wait and ask again.
    """
    req = urllib.request.Request(
        f"https://crates.io/api/v1/crates/{crate}/{version}",
        headers={"User-Agent": UA},
    )
    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            body = json.loads(resp.read())
    except (urllib.error.URLError, TimeoutError, json.JSONDecodeError):
        return False
    # A yanked version still resolves for existing consumers but must not be
    # treated as "published successfully" by a release run.
    return body.get("version", {}).get("yanked") is False


def preflight(round_one: set[str], version: str) -> None:
    branch = sh("git", "rev-parse", "--abbrev-ref", "HEAD").stdout.strip()
    if branch != "main":
        die(f"on branch `{branch}` — publish from `main`, which is what CI tested")
    if sh("git", "status", "--porcelain").stdout.strip():
        die("the working tree is dirty — what goes to crates.io must be a real commit")

    local = sh("git", "rev-parse", "HEAD").stdout.strip()
    sh("git", "fetch", "--quiet", "origin", "main")
    remote = sh("git", "rev-parse", "origin/main").stdout.strip()
    if local != remote:
        die(
            f"HEAD ({local[:8]}) is not origin/main ({remote[:8]}) — push or pull first"
        )

    blocked = sorted(
        c for c in round_one if manifest(c)["package"].get("publish") is False
    )
    if blocked:
        die(
            "these crates say `publish = false`, which is the safety catch, not an "
            f"obstacle to route around: {blocked}"
        )
    print(f"preflight OK — main @ {local[:8]}, version {version}, tree clean")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--execute", action="store_true", help="actually publish (default: dry run)"
    )
    args = ap.parse_args()

    version = workspace_version()
    round_one = set(PUBLISH_SET) - DEFERRED
    plan = waves(round_one)

    print(
        f"round one: {len(round_one)} crates at {version} "
        f"({len(DEFERRED)} held for 0.2: {', '.join(sorted(DEFERRED))})\n"
    )
    for i, layer in enumerate(plan, 1):
        print(f"  wave {i}: {', '.join(layer)}")
    print()

    preflight(round_one, version)

    if not args.execute:
        print("\nDRY RUN — nothing published. Re-run with --execute.")
        return 0

    for i, layer in enumerate(plan, 1):
        print(f"\n=== wave {i}: {', '.join(layer)} ===")
        fresh = []
        for crate in layer:
            if on_registry(crate, version):
                print(f"  skip    {crate} {version} (already on crates.io)")
                continue
            print(f"  publish {crate} {version}")
            proc = subprocess.run(
                ["cargo", "publish", "--manifest-path", str(WORKSPACE), "-p", crate],
                text=True,
                cwd=REPO,
            )
            if proc.returncode != 0:
                die(
                    f"{crate} failed to publish. Crates already uploaded stay uploaded — "
                    "fix the cause and re-run; this script resumes rather than restarting."
                )
            fresh.append(crate)

        if not fresh:
            continue
        # The wait is between waves, not after each crate: crates in one wave are
        # independent by construction, so only the NEXT wave needs them resolvable.
        print(f"  waiting for {len(fresh)} crate(s) to appear in the index…")
        deadline = time.monotonic() + INDEX_TIMEOUT_S
        pending = list(fresh)
        while pending and time.monotonic() < deadline:
            pending = [c for c in pending if not on_registry(c, version)]
            if pending:
                time.sleep(INDEX_POLL_S)
        if pending:
            die(
                f"{pending} uploaded but still not resolvable after {INDEX_TIMEOUT_S}s. "
                "They ARE published — do not re-upload. Wait, then re-run to continue."
            )
        print(f"  wave {i} resolvable")

    print(f"\npublished {len(round_one)} crates at {version}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
