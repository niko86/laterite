"""What is unreleased, on both tiers — and what part the next bump should be.

## Why this exists

The product tier moves only when someone runs `bump-version.sh`; each published
engine crate moves only when someone runs `bump_crate.py` on it (per-crate since
#781). Nothing bumps on merge, which is correct —
a release is a decision, not a side effect. But nothing said a release was
*owed* either, and the two drifted: between engine 0.9.0 (#184) and this being
written, the product shipped three times (0.10.0, 0.10.1, 0.11.0) while the
engine stayed put and accumulated four figures of public API. crates.io
consumers — and `laterite-duckdb`, which pins the engine crates from the
registry — were that far behind what every `pip install laterite` user ran.

That is a *reporting* gap, not a scheme gap. This is the report.

## Where the numbers come from

The bump part is DERIVED, from the two records this repo already keeps and
already gates:

* `tools/release/public-api/*.txt` — committed `cargo-public-api` snapshots, the
  factual public surface per published crate — and, since #781, the census of
  the per-crate engine tier: one snapshot, one crate, one verdict. A `+pub` line is an addition, a
  `-pub` line is a removal. This is the only source for the additive axis:
  `cargo semver-checks` has **no `function_added` lint at all**, so an addition
  is invisible to it by construction (it also skips every `minor` lint when
  baseline and current versions match, which they always do between releases).
* `changelog.json` — every PR is forced to add an entry under a Keep-a-Changelog
  section by `tools/check_changelog.py`, so the sections are a complete record
  of what kind of change landed.

The two answer different questions and neither is authoritative alone. The
snapshots are facts about the Rust API and say nothing about the wheel's Python
surface; the changelog is reader-facing prose and cannot tell a CLI flag from a
`pub fn`. So both are printed, the engine verdict is taken from the snapshots,
and the product verdict is taken from the changelog — each from the source that
can actually see it.

## Scope — what this does NOT look at

* **The product's own API surface is not measured.** There is no committed
  snapshot for the Python or Node surface the way there is for the crates, so
  the product verdict rests on changelog sections alone and is a suggestion
  rather than a derivation. `modality.json` is the nearest thing and is a
  capability register, not an API diff.
* **`laterite-duckdb` is a separate repo and is not inspected.** It pins the
  engine crates from crates.io, so it is downstream of an engine publish; this
  reports that it will need attention, never that it has had it (#717).
* **crates.io is read; PyPI and npm are not.** Each engine crate's stamp is
  checked against the sparse index — what `cargo` itself resolves against — so
  "stamped here, never published" is a reported state rather than an invisible
  one. It was invisible, and not theoretically: `laterite-ags4-emit` 0.12.0 was
  stamped, written up as published, and absent from the registry, with nothing
  in the tree able to notice. The product tier keeps the old blind spot,
  because nothing here asks PyPI or npm — a stamped product version whose tag
  was never cut is still invisible (0.8.1/0.8.2 exist in git and on no
  registry).
* **A failed registry read is not a finding.** Unreachable travels to the
  render as its own state and is never folded into "unpublished": a nag that
  cries publish-owed because a runner lost DNS is a nag that gets switched off.
  How many crates went unasked is printed on every run.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import urllib.error
import urllib.request
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable

ROOT = Path(__file__).resolve().parents[2]
SNAPSHOTS = ROOT / "tools" / "release" / "public-api"
CHANGELOG = ROOT / "changelog.json"
ENGINE_MANIFEST = ROOT / "rust-packages" / "Cargo.toml"
PRODUCT_MANIFEST = ROOT / "pyproject.toml"

#: The sparse index rather than the JSON API: this is what `cargo` resolves
#: against, so "present here" is exactly "a consumer can depend on it", and it
#: is CDN-served rather than governed by the API's crawler policy.
REGISTRY_INDEX = "https://index.crates.io"
#: Shared with tools/release/trusted_publishing.py, so the release tooling
#: reaches crates.io under one recognisable identity.
UA = "laterite-release-tooling (https://github.com/niko86/laterite)"
REGISTRY_TIMEOUT_S = 10

#: Registry state -> what the verdict column shouts. `ok`, `unknown` and
#: `skipped` deliberately say nothing there: one needs no action and the other
#: two are absences of knowledge, which the footer reports as counts instead.
REGISTRY_FLAG = {
    "owed": "PUBLISH OWED",
    "new": "FIRST PUBLISH OWED",
    "yanked": "STAMP IS YANKED",
}

#: Changelog section -> the smallest bump that section justifies.
#:
#: `removed` is the only unambiguous major: a feature that is gone breaks whoever
#: used it. `changed` is NOT mapped to major even though it can be breaking —
#: Keep-a-Changelog's "changes in existing functionality" covers both a reworded
#: message and a signature that moved, and a verdict that shouts MAJOR on every
#: release because some entry landed under `changed` is a verdict nobody reads.
#: It is reported separately instead, as the section a human has to look at.
SECTION_BUMP = {
    "removed": "major",
    "changed": "minor",
    "deprecated": "minor",
    "added": "minor",
    "fixed": "patch",
    "security": "patch",
}

#: The section whose entries cannot be classified from their section alone.
AMBIGUOUS = "changed"
ORDER = ["major", "minor", "patch", "none"]


def sh(*args: str) -> str:
    """Run a git command from the repo root and return stripped stdout."""
    return subprocess.run(
        args, cwd=ROOT, capture_output=True, text=True, check=False
    ).stdout.strip()


def version_of(manifest: Path, pattern: str) -> str:
    m = re.search(pattern, manifest.read_text(), re.MULTILINE)
    return m.group(1) if m else "?"


def engine_crates() -> list[str]:
    """The published set, derived from the snapshots — one file per crate.

    The snapshot directory is already the census of what has a public API to
    answer for; a crate joining the publish set gains a snapshot in the same PR
    (check_public_api refuses otherwise), so there is no second list to forget.
    """
    return sorted(f.stem for f in SNAPSHOTS.glob("*.txt"))


def crate_manifest(crate: str) -> Path:
    return ROOT / "rust-packages" / crate / "Cargo.toml"


def version_of_text(text: str) -> str:
    m = re.search(r'^version\s*=\s*"([^"]+)"', text, re.MULTILINE)
    return m.group(1) if m else "?"


def last_stamp(manifest: Path, anchor: str) -> tuple[str, str]:
    """The last commit that changed `anchor`'s version line in `manifest`.

    Matched on the VERSION LINE rather than on the commit subject, because both
    tiers commit the same `release: X` subject — `bump-version.sh` line 190 uses
    one message for both while telling the engine caller it wrote
    `release: engine X`. A subject grep cannot tell the tiers apart; a diff of
    the line that actually moved can.
    """
    rel = manifest.relative_to(ROOT)
    out = sh(
        "git",
        "log",
        "--format=%H %h %ad %s",
        "--date=short",
        "-L",
        f"{anchor}:{rel}",
        "--no-patch",
    )
    first = out.splitlines()[0] if out else ""
    if not first:
        return "", "(no stamp found)"
    sha, rest = first.split(" ", 1)
    return sha, rest


def api_delta(since: str, crate: str) -> tuple[int, int, list[str]]:
    """Net public-API additions and removals in ONE crate's snapshot since `since`.

    Net, not raw: a snapshot regeneration can rewrite a line in place, which
    shows as one `-pub` and one `+pub` for the same signature and is not a
    change to the surface at all.

    Per crate since #781: the whole-directory diff this used to take collapsed
    eleven independently versioned surfaces into one verdict, which is exactly
    the reading lockstep imposed and per-crate versioning exists to retire.
    """
    if not since:
        return 0, 0, []
    snap = SNAPSHOTS / f"{crate}.txt"
    diff = sh("git", "diff", f"{since}..HEAD", "--", str(snap.relative_to(ROOT)))
    added = {ln[1:] for ln in diff.splitlines() if ln.startswith("+pub")}
    removed = {ln[1:] for ln in diff.splitlines() if ln.startswith("-pub")}
    net_add, net_rm = added - removed, removed - added
    return len(net_add), len(net_rm), sorted(net_rm)


def index_path(crate: str) -> str:
    """`crate`'s path in the sparse index, per cargo's name-length layout.

    Implemented in full rather than assuming the 4+ branch every crate here
    happens to occupy: a wrong path 404s, a 404 reads as "never published", and
    that is the false-alarm direction.
    """
    name = crate.lower()
    if len(name) <= 2:
        return f"{len(name)}/{name}"
    if len(name) == 3:
        return f"3/{name[0]}/{name}"
    return f"{name[:2]}/{name[2:4]}/{name}"


def fetch_index(crate: str) -> list[dict] | None:
    """Every version of `crate` the sparse index carries; `None` if unreachable.

    `None` is not an empty list, and that difference is the point.
    `publish_crates.py::on_registry` collapses every failure to "no" because it
    retries — both answers mean "wait" there. A report gets no second ask, so
    "could not reach the registry" has to survive as its own answer.
    """
    req = urllib.request.Request(
        f"{REGISTRY_INDEX}/{index_path(crate)}", headers={"User-Agent": UA}
    )
    try:
        with urllib.request.urlopen(req, timeout=REGISTRY_TIMEOUT_S) as resp:
            body = resp.read().decode()
    except urllib.error.HTTPError as exc:
        # 404 is a fact — the crate has never been published — not a failure.
        return [] if exc.code == 404 else None
    except (urllib.error.URLError, TimeoutError, ValueError):
        return None
    out: list[dict] = []
    for line in body.splitlines():
        if not line.strip():
            continue
        try:
            out.append(json.loads(line))
        except json.JSONDecodeError:
            return None  # a half-read index is not evidence of anything
    return out


def version_key(version: str) -> tuple[int, ...]:
    """Sort key for a plain `x.y.z`, with a prerelease sorting BELOW its release.

    That last part is load-bearing rather than pedantry: if `0.12.0-rc.1` is the
    newest thing on the index, the newest RELEASE is not 0.12.0, and a key that
    treated the two as equal would report a release the registry does not carry
    — inventing progress instead of understating it. A non-numeric part counts
    as 0 rather than raising, for the same reason: this runs in a report that
    must not fall over on a version shape nobody anticipated.
    """
    core, _, prerelease = version.partition("-")
    parts = [int(p) if p.isdigit() else 0 for p in core.split("+")[0].split(".")]
    parts += [0] * (3 - len(parts))  # a short core compares against a full one
    return (*parts, 0 if prerelease else 1)


def registry_state(stamped: str, versions: list[dict] | None) -> tuple[str, str]:
    """`(state, highest published)` for one crate's stamped version.

    Five states because five different things have to happen next: `ok`
    nothing; `owed` someone runs the publish; `new` the same, but it is a first
    upload; `yanked` needs a human, since crates.io is append-only and that
    number can never be published again; `unknown` means conclude nothing.
    """
    if versions is None:
        return "unknown", "?"
    if not versions:
        return "new", "—"
    highest = max((v.get("vers", "") for v in versions), key=version_key)
    match = next((v for v in versions if v.get("vers") == stamped), None)
    if match is None:
        return "owed", highest
    if match.get("yanked"):
        return "yanked", highest
    return "ok", highest


def registry_scope(status: dict) -> str:
    """What the registry read did not cover — printed on every run.

    The house rule is that a gate dropping input says what it dropped, and an
    unreachable crate is dropped input. This line is the only thing standing
    between "we could not ask" and a silent green.
    """
    states = [c["registry_state"] for c in status["engine_crates"]]
    total = len(states)
    skipped = sum(1 for st in states if st == "skipped")
    unknown = sum(1 for st in states if st == "unknown")
    if total and skipped == total:
        return "registry: NOT ASKED (--no-registry) — no publish state derived for any crate."
    return (
        f"registry: {total - skipped - unknown} of {total} crates answered, "
        f"{unknown} unreachable, {skipped} not asked — nothing concluded for either."
    )


def changelog_sections() -> dict[str, int]:
    if not CHANGELOG.exists():
        return {}
    unreleased = json.loads(CHANGELOG.read_text()).get("unreleased", {})
    return {k: len(v) for k, v in unreleased.items() if isinstance(v, list) and v}


def verdict_from_sections(sections: dict[str, int]) -> str:
    parts = [SECTION_BUMP[s] for s in sections if s in SECTION_BUMP]
    return min(parts, key=ORDER.index) if parts else "none"


def verdict_from_api(added: int, removed: int) -> str:
    if removed:
        return "major"
    if added:
        return "minor"
    return "none"


def collect(fetch: Callable[[str], list[dict] | None] | None = fetch_index) -> dict:
    """The whole report. `fetch=None` asks the registry nothing."""
    _, product_stamp = last_stamp(PRODUCT_MANIFEST, "/^version/,+1")
    sections = changelog_sections()
    crates = []
    for crate in engine_crates():
        manifest = crate_manifest(crate)
        sha, stamp = last_stamp(manifest, "/^version/,+1")
        added, removed, removed_names = api_delta(sha, crate)
        version = version_of(manifest, r'^version\s*=\s*"([^"]+)"')
        state, published = (
            registry_state(version, fetch(crate)) if fetch else ("skipped", "—")
        )
        crates.append(
            {
                "crate": crate,
                "version": version,
                "last_stamp": stamp,
                "registry_state": state,
                "registry_latest": published,
                "api_added": added,
                "api_removed": removed,
                "api_removed_names": removed_names[:20],
                "verdict": verdict_from_api(added, removed),
            }
        )
    return {
        "engine_crates": crates,
        "product": {
            "version": version_of(PRODUCT_MANIFEST, r'^version\s*=\s*"([^"]+)"'),
            "last_stamp": product_stamp,
            "verdict": verdict_from_sections(sections),
        },
        "changelog_unreleased": sections,
    }


def render(s: dict) -> str:
    p = s["product"]
    lines = [
        "engine crates (per-crate since #781; verdict = API delta since its own last stamp):"
    ]
    for c in s["engine_crates"]:
        verdicts = [] if c["verdict"] == "none" else [c["verdict"].upper()]
        shout = REGISTRY_FLAG.get(c["registry_state"])
        if shout:
            verdicts.append(shout)
        flag = f"   ->  {', '.join(verdicts)}" if verdicts else ""
        reg = f"crates.io {c['registry_latest']}"
        lines.append(
            (
                f"  {c['crate']:<26} {c['version']:<8} "
                f"+{c['api_added']} -{c['api_removed']}  {reg:<18}{flag}"
            ).rstrip()
        )
    lines += [
        f"product  {p['version']:<10} last stamped {p['last_stamp']}",
        "         changelog [unreleased]: "
        + (
            ", ".join(f"{k} {v}" for k, v in sorted(s["changelog_unreleased"].items()))
            or "empty"
        )
        + f"   ->  {p['verdict'].upper()}",
    ]
    removed = [
        (c["crate"], n) for c in s["engine_crates"] for n in c["api_removed_names"]
    ]
    if removed:
        lines.append("")
        lines.append(
            "  public API REMOVED since a crate's last stamp — a consumer has to follow:"
        )
        lines += [f"    {crate}: {n}" for crate, n in removed]
    n_changed = s["changelog_unreleased"].get(AMBIGUOUS, 0)
    if n_changed:
        lines.append("")
        plural = "y" if n_changed == 1 else "ies"
        lines.append(
            f"  {n_changed} entr{plural} under `{AMBIGUOUS}` — the one section that cannot be"
        )
        lines.append(
            "  classified from its section alone. Read them: a reworded message is a patch,"
        )
        lines.append("  a signature that moved is a major.")
    lines.append("")
    lines.append(
        "  each crate's verdict is from ITS API snapshot; the product verdict is from the"
    )
    lines.append(
        "  changelog sections. The product's own API surface is NOT measured — no committed"
    )
    lines.append(
        "  snapshot exists for the Python/Node surface — so treat that one as a suggestion."
    )
    if all(c["registry_state"] == "skipped" for c in s["engine_crates"]):
        lines.append(
            "  The crates.io column is EMPTY this run — nothing was asked, so no crate here"
        )
        lines.append("  is claimed to be published or unpublished.")
    else:
        lines.append(
            "  crates.io IS read — the column is the highest version its sparse index"
        )
        lines.append(
            "  carries, so a stamp that never reached the registry shows as PUBLISH OWED."
        )
    lines.append(
        "  PyPI and npm are NOT read, so the product tier keeps that blind spot: a stamped"
    )
    lines.append("  version whose tag was never cut is still invisible here.")
    lines.append(f"  {registry_scope(s)}")
    return "\n".join(lines)


def render_nag(s: dict) -> str:
    p = s["product"]
    owed_crates = [c for c in s["engine_crates"] if c["verdict"] != "none"]
    unpublished = [
        c for c in s["engine_crates"] if c["registry_state"] in ("owed", "new")
    ]
    unknown = sum(1 for c in s["engine_crates"] if c["registry_state"] == "unknown")
    parts = []
    if owed_crates:
        parts.append(
            "crate bumps owed: "
            + ", ".join(f"{c['crate']} ({c['verdict']})" for c in owed_crates)
        )
    if unpublished:
        parts.append(
            "stamped but not on crates.io: "
            + ", ".join(c["crate"] for c in unpublished)
        )
    if p["verdict"] != "none":
        parts.append(f"product {p['version']} release owed ({p['verdict']})")
    # Said out loud even when nothing else is owed: a bare "nothing owed" would
    # be claiming knowledge a failed registry read does not have.
    caveat = f" ({unknown} crate(s) unreachable on crates.io)" if unknown else ""
    if not parts:
        return (
            "release-status: nothing owed — every crate and the product are level "
            f"with their stamps{caveat}."
        )
    return "release-status: " + " · ".join(parts) + caveat


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument(
        "--json", action="store_true", help="machine-readable, for a workflow step"
    )
    ap.add_argument(
        "--nag", action="store_true", help="one line, for a scheduled summary"
    )
    ap.add_argument(
        "--no-registry",
        action="store_true",
        help="skip the crates.io read (offline, or a deliberately fast local run)",
    )
    args = ap.parse_args()

    s = collect(fetch=None if args.no_registry else fetch_index)
    if args.json:
        print(json.dumps(s, indent=2))
    elif args.nag:
        print(render_nag(s))
    else:
        print(render(s))
    return 0


if __name__ == "__main__":
    sys.exit(main())
