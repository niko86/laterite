#!/usr/bin/env python3
"""Create the crates.io Trusted Publishing configs `publish-crates.yml` needs.

Trusted Publishing is configured **per crate**, across the whole publish set.
Doing that through the crates.io UI is one visit to Settings → Trusted
Publishing per crate, each retyping the same four values — the shape that goes
wrong deep into the list. crates.io exposes the same thing as an API, so this
does it once.

It is not only a first-time step. **A new publishable crate needs its own config
or the publish fails at the registry**, after the earlier waves have already gone
out and cannot be withdrawn. So the crate list here is not a list: it is
`publish_crates.py`'s own `PUBLISH_SET`, imported, for the same reason that
script derives its waves from the manifests rather than hard-coding them — a
second statement of the set is a second thing to keep true.

One ordering constraint, learned live (`laterite-ags4-excel`, 2026-09-04): the
config API is keyed on the crate EXISTING — for a name with no release, both
the listing and the create answer 404. So a config cannot predate a crate's
first publish, and that first publish cannot ride the OIDC workflow at all: it
goes out with a publish token (`tools/publish_crates.py --execute`), and the
config is created right after, which is what lets every later publish of the
crate run unattended.

Exit 0 when every publishable crate has a config, 1 when one is missing — or
cannot exist yet because its crate has no release — and 2 on an API error.

The token this needs is NOT the publish credential. `publish-crates.yml` stores
nothing; it mints a short-lived token over OIDC. This one is a `trusted-publishing`
scoped token used once, to write the configs, and revoked afterwards:

    crates.io → Account Settings → API Tokens → New Token
      scope: trusted-publishing, shortest expiry offered

Usage:
    export CRATES_IO_TOKEN=...
    uv run --no-project python tools/release/trusted_publishing.py            # what is missing
    uv run --no-project python tools/release/trusted_publishing.py --create   # create it
"""

from __future__ import annotations

import json
import os
import sys
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))

API = "https://crates.io/api/v1/trusted_publishing/github_configs"
UA = "laterite-release-tooling (https://github.com/niko86/laterite)"

REPOSITORY_OWNER = "niko86"
REPOSITORY_NAME = "laterite"

# crates.io pins BOTH of these per config, so they are part of the publish
# contract rather than incidental. Renaming the workflow or the environment
# breaks every crate's config at once, at publish time, on the append-only
# registry — the assertion below is what turns that into a local failure instead.
WORKFLOW_FILENAME = "publish-crates.yml"
ENVIRONMENT = "crates"

# The crate name goes over the wire as `crate`, NOT as the Rust field name.
# crates.io's `NewGitHubConfig` spells it `pub krate: String` because `crate` is
# a reserved word in Rust, and carries `#[serde(rename = "crate")]` to put the
# real name back on the JSON. Reading the handler and not the rename cost a
# 422 — "missing field `crate`" — on the first create, which is at least the
# side of the failure that stops rather than the side that half-succeeds.
CRATE_KEY = "crate"


def crates() -> list[str]:
    """The publishable set, from the script that already owns it."""
    import publish_crates

    return sorted(set(publish_crates.PUBLISH_SET) - publish_crates.DEFERRED)


def call(method: str, url: str, token: str, body: dict | None = None) -> dict:
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(url, data=data, method=method)
    req.add_header("Authorization", token)
    req.add_header("User-Agent", UA)
    if data:
        req.add_header("Content-Type", "application/json")
    with urllib.request.urlopen(req, timeout=30) as resp:
        return json.loads(resp.read() or b"{}")


def existing(token: str) -> tuple[dict[str, list[dict]], list[str]]:
    """crate -> its configs, plus the crates the registry does not know.

    A 404 here is a fact about the crate, not an API failure: the config
    endpoints are keyed on the crate existing, so a crate ahead of its first
    publish 404s on the listing — and would 404 on the create too. Confirmed
    live on `laterite-ags4-excel` (2026-09-04); the docstring above carries the
    publish ordering that follows. Every OTHER error still aborts: a 403 is a
    credential fault (a publish-scoped token got exactly that, live), and
    carrying on would print a wall of wrong rows over a bad token.
    """
    out: dict[str, list[dict]] = {}
    unpublished: list[str] = []
    for krate in crates():
        try:
            got = call("GET", f"{API}?crate={krate}", token)
        except urllib.error.HTTPError as exc:
            if exc.code == 404:
                unpublished.append(krate)
                continue
            print(f"  {krate}: HTTP {exc.code} listing configs", file=sys.stderr)
            raise
        out[krate] = got.get("github_configs", [])
    return out, unpublished


def body_for(krate: str) -> dict:
    """The create payload, factored out so a test can pin its wire names."""
    return {
        "github_config": {
            CRATE_KEY: krate,
            "repository_owner": REPOSITORY_OWNER,
            "repository_name": REPOSITORY_NAME,
            "workflow_filename": WORKFLOW_FILENAME,
            "environment": ENVIRONMENT,
        }
    }


def matches(cfg: dict) -> bool:
    return (
        cfg.get("repository_owner") == REPOSITORY_OWNER
        and cfg.get("repository_name") == REPOSITORY_NAME
        and cfg.get("workflow_filename") == WORKFLOW_FILENAME
        and (cfg.get("environment") or None) == ENVIRONMENT
    )


def main() -> int:
    workflow = ROOT / ".github" / "workflows" / WORKFLOW_FILENAME
    if not workflow.is_file():
        print(
            f"{WORKFLOW_FILENAME} does not exist. Every crates.io config names it,"
            " so creating configs for a workflow that isn't there would publish"
            " nothing and fail at the registry.",
            file=sys.stderr,
        )
        return 2

    token = os.environ.get("CRATES_IO_TOKEN", "").strip()
    if not token:
        print(
            "CRATES_IO_TOKEN is not set — see this file's docstring.", file=sys.stderr
        )
        return 2

    create = "--create" in sys.argv
    try:
        have, unpublished = existing(token)
    except urllib.error.HTTPError:
        return 2

    missing = [k for k, cfgs in have.items() if not any(matches(c) for c in cfgs)]
    for krate in crates():
        if krate in unpublished:
            mark = "NO CRATE"
        elif krate in missing:
            mark = "MISSING"
        else:
            mark = "ok"
        print(f"  {mark:<8} {krate}")
    if unpublished:
        print(
            f"\n{len(unpublished)} crate(s) have no release on crates.io, so no "
            "config can exist for them yet: publish first "
            "(tools/publish_crates.py --execute, with a publish token), then "
            "re-run this."
        )

    if not missing and not unpublished:
        print(
            f"\nall {len(have)} publishable crate(s) trust "
            f"{REPOSITORY_OWNER}/{REPOSITORY_NAME} · {WORKFLOW_FILENAME} · "
            f"environment {ENVIRONMENT}"
        )
        return 0

    if not create:
        if missing:
            print(f"\n{len(missing)} crate(s) have no config. Re-run with --create.")
        return 1

    for krate in missing:
        try:
            call("POST", API, token, body_for(krate))
        except urllib.error.HTTPError as exc:
            detail = exc.read().decode(errors="replace")[:300]
            print(f"  FAILED  {krate}: HTTP {exc.code} {detail}", file=sys.stderr)
            return 2
        print(f"  created {krate}")

    if missing:
        print(
            f"\ncreated {len(missing)} config(s) — revoke the token; that part is done."
        )
    # A crate the registry does not know is still a config OWED, just one that
    # cannot be paid yet — exiting 0 over it would read as "all armed".
    return 1 if unpublished else 0


if __name__ == "__main__":
    raise SystemExit(main())
