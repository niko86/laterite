#!/usr/bin/env python3
"""Publish the crates to crates.io, in dependency waves, idempotently.

A first publish is not one command repeated per crate. The crates depend on
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
- Publish anything in `DEFERRED` — crates prepared for the registry but held back
  deliberately. A tool that quietly included one is how a held crate goes out.
  The set is empty as of 2026-08-04 (`laterite-ags4-diff` and `laterite-ags4-merge`
  were its only members and went out on 2026-08-05); the mechanism stays for the
  next one.

Nothing happens without `--execute`. The default run performs every check and
prints exactly what it would do.

## Two things crates.io will do to you

Both were hit during the 0.9.0 publish. Neither is a defect and neither leaves
partial state, but neither is discoverable except by publishing:

1. **A verified email address is required.** crates.io populates the address
   from GitHub but leaves it UNVERIFIED, and nothing says so until an upload is
   rejected `400 A verified email address is required to publish`. Every other
   step — login, token scopes, packaging, the verification build — succeeds
   first. Verify at <https://crates.io/settings/profile> before starting.
2. **New crates are rate-limited.** There is a burst allowance and then roughly
   one new crate per interval; a first publish of several trips it near the end.
   The 429 names the time to retry. This is why the script is idempotent: the
   fix is to wait and re-run, and re-running must not try to upload the ones
   that already went out.

## Publishing part of the plan

`--through-wave N` stops after wave N. Publishing one wave, looking at the live
pages, and only then committing the rest is worth the pause on a FIRST publish:
crate metadata is frozen per version, so a README that reads badly or a category
you would not have chosen can still be corrected on the crates that have not
gone out yet.

Usage:
    python tools/publish_crates.py                      # dry run — check everything, publish nothing
    python tools/publish_crates.py --execute            # publish every wave
    python tools/publish_crates.py --execute --through-wave 1   # publish wave 1, then stop
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

#: Crates prepared for the registry but deliberately not published yet — gated by
#: the packaging tools like every other publishable crate, and skipped here.
#:
#: Emptied 2026-08-04: `laterite-ags4-diff` and `laterite-ags4-merge` were the
#: only two entries, held back through the first publish so day one's surface was
#: as small as possible. That is spent — both went out at 0.9.0 on 2026-08-05,
#: taking the engine tier to ten published crates. The mechanism
#: stays because it will be wanted again — though the next prepared crate,
#: `laterite-ags4-excel` (dec-facade-parity phase 5), never needed it: since
#: #889 the PR merge is the human gate, so it joined `PUBLISH_SET` and rode the
#: unattended sweep instead of waiting here.
DEFERRED: set[str] = set()

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


def crate_version(crate: str) -> str:
    """`crate`'s published version, resolving workspace inheritance.

    Per-crate, NOT one number for the whole run — and since #781 that is the
    scheme, not an exception: every published crate carries its own version
    (the facade always did; the engine crates joined it when the lockstep was
    retired). The workspace-inheritance branch below survives for any future
    member that still inherits, and because deleting a working fallback buys
    nothing.

    One known trap, recorded on #781: the skip below compares VERSION identity,
    not manifest content. A crate whose content changed at an unchanged version
    is skipped silently — bump it first (tools/release/bump_crate.py).
    """
    pkg = manifest(crate)["package"]
    version = pkg.get("version")
    if isinstance(version, str):
        return version
    if isinstance(version, dict) and version.get("workspace") is True:
        return tomllib.loads(WORKSPACE.read_text(encoding="utf-8"))["workspace"][
            "package"
        ]["version"]
    die(f"{crate}: cannot determine its version from the manifest")
    raise AssertionError("unreachable")  # die() exits


def waves(crates: set[str]) -> list[list[str]]:
    """`crates` grouped so every crate follows everything inside `crates` it needs.

    Derived from the manifests, so it cannot disagree with them. Dependencies
    OUTSIDE the set are ignored: a crate held back is not a reason to hold back
    something that does not depend on it.
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


def preflight(round_one: set[str]) -> None:
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
    print(f"preflight OK — main @ {local[:8]}, tree clean")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--execute", action="store_true", help="actually publish (default: dry run)"
    )
    ap.add_argument(
        "--through-wave",
        type=int,
        metavar="N",
        help="stop after wave N (default: all waves)",
    )
    args = ap.parse_args()

    round_one = set(PUBLISH_SET) - DEFERRED
    plan = waves(round_one)
    versions = {c: crate_version(c) for c in round_one}

    if args.through_wave is not None and not 1 <= args.through_wave <= len(plan):
        die(f"--through-wave {args.through_wave}: there are {len(plan)} waves")
    last = args.through_wave or len(plan)

    held = (
        f", {len(DEFERRED)} held back: {', '.join(sorted(DEFERRED))}"
        if DEFERRED
        else ""
    )
    print(f"{len(round_one)} publishable crate(s){held}\n")
    for i, layer in enumerate(plan, 1):
        # Name what is being left out. A partial run that printed the same thing
        # as a full one would be indistinguishable from having published
        # everything, which is the wrong belief to hold about a registry you
        # cannot take anything back from.
        held = "   (NOT this run — --through-wave)" if i > last else ""
        named = ", ".join(f"{c} {versions[c]}" for c in layer)
        print(f"  wave {i}: {named}{held}")
    print()

    preflight(round_one)

    if not args.execute:
        print("\nDRY RUN — nothing published. Re-run with --execute.")
        return 0

    for i, layer in enumerate(plan[:last], 1):
        print(f"\n=== wave {i}: {', '.join(layer)} ===")
        fresh = []
        for crate in layer:
            version = versions[crate]
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
            pending = [c for c in pending if not on_registry(c, versions[c])]
            if pending:
                time.sleep(INDEX_POLL_S)
        if pending:
            die(
                f"{pending} uploaded but still not resolvable after {INDEX_TIMEOUT_S}s. "
                "They ARE published — do not re-upload. Wait, then re-run to continue."
            )
        print(f"  wave {i} resolvable")

    done = sum(len(layer) for layer in plan[:last])
    print(f"\npublished/verified {done} of {len(round_one)} crate(s)")
    if last < len(plan):
        remaining = [c for layer in plan[last:] for c in layer]
        print(
            f"STOPPED after wave {last} as asked — {len(remaining)} crate(s) NOT published: "
            f"{', '.join(remaining)}\nRe-run without --through-wave to continue; the waves "
            "already done will be skipped."
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
