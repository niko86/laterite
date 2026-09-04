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

The engine baseline is the last **published** version, not the last stamp
(#806's prerequisite): a delta measured from a stamp that never published
reports a bump as owed when the standing number already covers it — a wrong
verdict a human shrugs at and a machine acts on. When the registry gives no
answer, the delta falls back to the crate's own last stamp and the cut
concludes nothing from it.

## The cut, and the signal no crate-local reading has

Since #806 this also derives the ACTION per engine crate — bump, publish,
needs-a-human, or nothing — which is what the nightly cut executes. Three
signals feed the owed part: the API delta (breaking and additive both take a
minor under 0.x; never a major — that is a one-way door to 1.0.0), code changed
under the crate with no API movement (patch), and the one #809 proved
invisible to everything crate-local: **a dependency's floor moved past the pins
the registry froze**. In-tree the laterite deps are `path` deps and always
unify, so the workspace compiles, every gate passes, and the published graph is
still incoherent — a consumer mixing two crates gets two copies of the moved
dep. The sparse-index rows carry each published version's requirement ranges,
so the answer is derived from data this report already fetches.

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
import io
import json
import re
import subprocess
import sys
import tarfile
import tomllib
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
#: Where the published tarballs live — the only place that records WHICH commit
#: a published version was built from (`.cargo_vcs_info.json`).
STATIC_CRATES = "https://static.crates.io/crates"
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

    One file there is NOT a crate: `laterite.all-features.txt`, the facade's
    second snapshot (dec-facade-parity decision 4). A crates.io name cannot
    carry a dot, so a dotted stem is that file and never a crate — filtered
    here rather than special-cased downstream, or the sweep would ask the
    registry about a crate that cannot exist.
    """
    return sorted(f.stem for f in SNAPSHOTS.glob("*.txt") if "." not in f.stem)


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


def highest_live(versions: list[dict] | None) -> str:
    """The newest NON-YANKED published version — the baseline the cut measures from.

    Distinct from the display column, which shows the highest version including
    yanked ones: a yanked version is a fact worth showing but not a baseline —
    nobody resolves against it.
    """
    if not versions:
        return ""
    live = [v.get("vers", "") for v in versions if not v.get("yanked")]
    return max(live, key=version_key) if live else ""


def workspace_floors(manifest_text: str | None = None) -> dict[str, str]:
    """crate -> the `[workspace.dependencies]` version floor siblings declare it at.

    This is what a published sibling actually pins (cargo strips `path` at
    publish), so it is the tree's side of the coherence question. `manifest_text`
    lets the PR gate read the floors of a git ref instead of the working tree.
    """
    src = manifest_text if manifest_text is not None else ENGINE_MANIFEST.read_text()
    deps = tomllib.loads(src).get("workspace", {}).get("dependencies", {})
    return {
        name: spec["version"]
        for name, spec in deps.items()
        if isinstance(spec, dict)
        and isinstance(spec.get("version"), str)
        and name.startswith("laterite")
    }


TIERS = ("engine", "product")


def release_tier(crate: str) -> str:
    """'engine' | 'product' | '?' from `[package.metadata.laterite].release_tier`.

    The cut acts on `engine` crates only; the facade is `product` until it
    reaches parity (dec-facade-parity) and then rides the product trains. An
    unlabelled crate reads as '?' and the cut refuses to act on it — the guard
    test in tests/test_engine_cut.py is what makes the omission loud rather
    than silently tiered.
    """
    try:
        data = tomllib.loads(crate_manifest(crate).read_text())
    except (OSError, tomllib.TOMLDecodeError):
        return "?"
    tier = (
        data.get("package", {})
        .get("metadata", {})
        .get("laterite", {})
        .get("release_tier")
    )
    return tier if tier in TIERS else "?"


def caret_admits(req: str, version: str) -> bool:
    """Does the requirement `req` admit `version`, under cargo's caret rules?

    On 0.x this is the whole ballgame: `^0.11.0` does NOT admit 0.12.0, which
    is how a floor moving under a published crate strands it. Only caret and
    `=` are decided; any other operator (this repo never publishes one) reads
    as admitting, because the false-alarm direction here is a bump demanded
    that isn't owed.
    """
    req = req.strip()
    if req.startswith("="):
        return req[1:].strip() == version
    if req.startswith("^"):
        req = req[1:].strip()
    if not req or not req[0].isdigit():
        return True
    base, v = version_key(req), version_key(version)
    if v < base:
        return False
    major, minor, patch = base[0], base[1], base[2]
    if major > 0:
        upper = (major + 1, 0, 0, 0)
    elif minor > 0:
        upper = (0, minor + 1, 0, 0)
    else:
        upper = (0, 0, patch + 1, 0)
    return v < upper


def deps_behind(
    versions: list[dict] | None, live: str, floors: dict[str, str]
) -> list[str]:
    """Which of the LIVE published version's laterite pins the tree's floors left.

    Each sparse-index row carries the full `deps` array of requirement ranges —
    the data #809's fault was written in, fetched all along and never read. A
    non-empty answer means: this crate will not be re-published (its version
    matches the registry), yet a consumer following the tree's floors can no
    longer unify it with its siblings.
    """
    row = next((v for v in versions or [] if v.get("vers") == live), None)
    if row is None:
        return []
    out = []
    for dep in row.get("deps", []):
        name = dep.get("package") or dep.get("name", "")
        floor = floors.get(name)
        if floor and not caret_admits(dep.get("req", ""), floor):
            out.append(f"{name} {dep.get('req')} left behind by floor {floor}")
    return sorted(out)


def published_sha(crate: str, version: str) -> str:
    """The commit the published tarball records in `.cargo_vcs_info.json`, or ''.

    The stamp commit is NOT the publish: content can land between the bump and
    the publish at the same number, and the registry copy carries it. emit
    0.13.0 proved the direction matters — measured from its stamp it showed
    +7 -4 owed, when every one of those lines was already ON the registry and
    a machine acting on that verdict would have spent 0.14.0 on nothing.
    """
    url = f"{STATIC_CRATES}/{crate}/{crate}-{version}.crate"
    req = urllib.request.Request(url, headers={"User-Agent": UA})
    try:
        with urllib.request.urlopen(req, timeout=REGISTRY_TIMEOUT_S) as resp:
            blob = resp.read()
        with tarfile.open(fileobj=io.BytesIO(blob), mode="r:gz") as tar:
            member = tar.extractfile(f"{crate}-{version}/.cargo_vcs_info.json")
            if member is None:
                return ""
            sha = json.loads(member.read()).get("git", {}).get("sha1", "")
            return sha if isinstance(sha, str) else ""
    except (
        urllib.error.URLError,
        TimeoutError,
        ValueError,
        tarfile.TarError,
        KeyError,
        OSError,
    ):
        return ""


def commit_exists(sha: str) -> bool:
    return bool(sha) and sh("git", "cat-file", "-t", sha) == "commit"


def stamp_of_version(manifest: Path, version: str) -> str:
    """The commit that last set `manifest`'s version line to `version`, or ''.

    `-G` finds both the bump TO this version and the later bump AWAY from it;
    the manifest's content AT each commit tells the two apart. This is what
    makes the published version — not the latest stamp — the measuring point.
    """
    rel = str(manifest.relative_to(ROOT))
    needle = f'version = "{version}"'
    out = sh(
        "git",
        "log",
        "--format=%H",
        "-G",
        f'^version = "{re.escape(version)}"',
        "--",
        rel,
    )
    for sha in out.splitlines():
        if needle in sh("git", "show", f"{sha}:{rel}"):
            return sha
    return ""


def code_changed_since(sha: str, crate: str) -> bool:
    """Any shipped-code file changed under the crate since `sha` — #806's second signal.

    Catches the change the snapshot cannot: behaviour moved but no `pub`
    signature did. What this deliberately does NOT see, so it is written down:
    `tests/`, `benches/`, `examples/` (a consumer never compiles them), the
    README (`check_released_crate_readmes.py` is the gate that forces a publish
    when it rewrites onto new API), and the manifest itself — a metadata or
    comment edit must not cut a release, at the price that a dep-only change
    with zero code movement goes unseen here and waits for a human or the
    coherence signal.
    """
    if not sha:
        return False
    prefix = f"rust-packages/{crate}/"
    out = sh("git", "diff", "--name-only", f"{sha}..HEAD", "--", prefix)
    skip = (prefix + "tests/", prefix + "benches/", prefix + "examples/")
    inert = (prefix + "Cargo.toml", prefix + "README.md")
    return any(
        ln and not ln.startswith(skip) and ln not in inert for ln in out.splitlines()
    )


def required_part(
    added: int, removed: int, code_changed: bool, deps_stale: bool
) -> str:
    """The 0.x part mapping (#806): breaking and additive both take a MINOR.

    Never 'major' — that is a one-way door to 1.0.0 and a human's to open. A
    stale pin takes a minor too: the moved types sit in the pinning crate's own
    public signatures, so the re-pin is breaking for its consumers.
    """
    if added or removed or deps_stale:
        return "minor"
    if code_changed:
        return "patch"
    return "none"


def covers(published: str, stamped: str, part: str) -> bool:
    """Does the stamped version already contain the owed `part` over `published`?

    This is the check that stops the machine spending versions: a stamp that
    already covers the part means the cut owes a PUBLISH, not another bump on
    an append-only registry.
    """
    if part == "none":
        return True
    p, s = version_key(published), version_key(stamped)
    if s <= p:
        return False
    if part == "patch":
        return True
    return s[:2] > p[:2]


def cut_action(
    state: str,
    tier: str,
    live: str,
    stamped: str,
    part: str,
    baseline_kind: str,
    deps_stale: bool,
) -> tuple[str, str]:
    """(action, why) for one crate: none | bump | publish | human | unconcluded.

    Every branch either acts on knowledge the registry confirmed or declines to
    act at all — 'unconcluded' is the machine's way of saying a human should
    look, which is different from 'nothing owed'. The append-only registry sets
    the bias: when the baseline is only a stamp (the published tarball's commit
    could not be placed in this history), an API or code delta is NOT acted on,
    because the delta may already be on the registry and a bump would spend a
    version on nothing. The stale-pin signal is registry-derived and survives
    that doubt.
    """
    if tier != "engine":
        return "none", f"tier is {tier!r} — outside the engine cut"
    if state in ("unknown", "skipped"):
        return "unconcluded", "the registry was not asked or did not answer"
    if state == "yanked":
        return "human", "its stamp is yanked — append-only, needs a fresh number"
    if state == "new":
        return "publish", "first publish"
    if live and version_key(live) > version_key(stamped):
        return "human", f"the registry ({live}) is ahead of the tree ({stamped})"
    if not baseline_kind:
        return "unconcluded", f"no baseline places the published {live} in this history"
    if part == "none":
        if state == "owed":
            return "publish", f"{stamped} is stamped; the registry has {live}"
        return "none", "level with the registry"
    if baseline_kind == "stamp" and not deps_stale:
        return "unconcluded", (
            f"moved since the {live} stamp, but the published content is not "
            "placeable in this history — the delta may already be on the registry"
        )
    if covers(live, stamped, part):
        # Only reachable when state == "owed": a covering stamp is by
        # construction ahead of the published version, so it cannot already
        # be on the registry.
        return "publish", f"the stamped {stamped} already covers the owed {part}"
    return "bump", ""


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
    floors = workspace_floors()
    crates = []
    for crate in engine_crates():
        manifest = crate_manifest(crate)
        tier = release_tier(crate)
        sha, stamp = last_stamp(manifest, "/^version/,+1")
        version = version_of(manifest, r'^version\s*=\s*"([^"]+)"')
        rows = fetch(crate) if fetch else None
        state, published = registry_state(version, rows) if fetch else ("skipped", "—")
        live = highest_live(rows)
        baseline_kind = baseline = ""
        if live:
            # Best: the exact commit the published tarball was built from.
            psha = published_sha(crate, live)
            if commit_exists(psha):
                baseline_kind, baseline = "publish", psha
            else:
                # Fallback: the commit that stamped the version line. Content
                # can have landed between that stamp and the publish, so the
                # cut treats a delta measured from here as unactable.
                st = stamp_of_version(manifest, live)
                if st:
                    baseline_kind, baseline = "stamp", st
        if baseline:
            since, delta_baseline = baseline, f"{baseline_kind} {live}"
            code = code_changed_since(baseline, crate)
        else:
            # No published version to measure from: fall back to the crate's
            # own last stamp for the report, and conclude nothing in the cut.
            since, delta_baseline = sha, "last stamp"
            code = False
        added, removed, removed_names = api_delta(since, crate)
        # A crate being republished anyway (owed/new) carries fresh floors with
        # it, so stale pins are only a fact about crates the registry already
        # has at their stamped version.
        stale = deps_behind(rows, live, floors) if state == "ok" else []
        part = required_part(added, removed, code, bool(stale))
        action, why = cut_action(
            state, tier, live, version, part, baseline_kind, bool(stale)
        )
        if action == "bump":
            reasons = []
            if added or removed:
                reasons.append(f"API +{added} -{removed} since {live}")
            if stale:
                reasons.append("published pins left behind: " + "; ".join(stale))
            if code and not (added or removed):
                reasons.append("code changed with no API movement")
            why = "; ".join(reasons)
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
                "tier": tier,
                "published_live": live,
                "delta_baseline": delta_baseline,
                "code_changed": code,
                "deps_behind": stale,
                "part_required": part,
                "cut_action": action,
                "cut_why": why,
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
        "engine crates (per-crate since #781; verdict = API delta since the last"
        " PUBLISHED version, or the last stamp when the registry has no answer):"
    ]
    for c in s["engine_crates"]:
        verdicts = [] if c["verdict"] == "none" else [c["verdict"].upper()]
        shout = REGISTRY_FLAG.get(c["registry_state"])
        if shout:
            verdicts.append(shout)
        if c["cut_action"] == "bump":
            verdicts.append(f"BUMP {c['part_required'].upper()} OWED")
        elif c["cut_action"] == "human":
            verdicts.append("NEEDS A HUMAN")
        if c["deps_behind"] and c["cut_action"] != "bump":
            # A crate the cut does not act on (the facade, an unlabelled crate)
            # can still be pinning versions the floors have left — a fact, not
            # an action, so it is said without being shouted as owed.
            verdicts.append("PINS BEHIND FLOORS")
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
            "  public API REMOVED since a crate's baseline — a consumer has to follow:"
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
    owed_crates = [c for c in s["engine_crates"] if c["cut_action"] == "bump"]
    humans = [c for c in s["engine_crates"] if c["cut_action"] == "human"]
    unpublished = [
        c for c in s["engine_crates"] if c["registry_state"] in ("owed", "new")
    ]
    unknown = sum(1 for c in s["engine_crates"] if c["registry_state"] == "unknown")
    parts = []
    if owed_crates:
        parts.append(
            "crate bumps owed: "
            + ", ".join(f"{c['crate']} ({c['part_required']})" for c in owed_crates)
        )
    if humans:
        parts.append("needs a human: " + ", ".join(c["crate"] for c in humans))
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


def render_cut(s: dict) -> str:
    """The actionable view (#806): what the nightly cut would DO, one line per act."""
    by = {
        a: [c for c in s["engine_crates"] if c["cut_action"] == a]
        for a in ("bump", "publish", "human", "unconcluded")
    }
    lines = ["engine cut (#806) — baseline: the last PUBLISHED version of each crate:"]
    lines.extend(
        f"  bump    {c['crate']} {c['part_required']}   ({c['cut_why']})"
        for c in by["bump"]
    )
    lines.extend(
        f"  publish {c['crate']} {c['version']}   ({c['cut_why']})"
        for c in by["publish"]
    )
    lines.extend(f"  HUMAN   {c['crate']}: {c['cut_why']}" for c in by["human"])
    lines.extend(f"  ?       {c['crate']}: {c['cut_why']}" for c in by["unconcluded"])
    if not (by["bump"] or by["publish"] or by["human"]):
        lines.append(
            "  nothing owed"
            + (
                f" — but {len(by['unconcluded'])} crate(s) concluded nothing, so this"
                " is not an all-clear"
                if by["unconcluded"]
                else ""
            )
        )
    if by["bump"]:
        lines.append("")
        lines.append("commands (what the nightly's PR mode runs):")
        lines.extend(
            "  uv run --no-project python tools/release/bump_crate.py "
            f"{c['crate']} {c['part_required']}"
            for c in by["bump"]
        )
    lines.append("")
    lines.append(f"  {registry_scope(s)}")
    return "\n".join(lines)


def check_coherence(
    fetch: Callable[[str], list[dict] | None], base_manifest_text: str | None
) -> int:
    """The PR gate for #809's class: a floor may not move past a published pin.

    Fails (1) only on debt this tree INTRODUCES relative to the base manifest —
    standing debt is the nightly cut's to fix, and a gate that reddens every PR
    over someone else's debt is a gate that gets skipped. With no base, all
    debt counts (the nightly's absolute reading). An unreachable registry
    concludes nothing and fails nothing, out loud — the scope line prints on
    every run, pass or fail.
    """
    floors = workspace_floors()
    base_floors = (
        workspace_floors(base_manifest_text) if base_manifest_text is not None else {}
    )
    introduced: list[str] = []
    standing = unreachable = asked = 0
    for crate in engine_crates():
        if release_tier(crate) != "engine":
            continue
        rows = fetch(crate)
        if rows is None:
            unreachable += 1
            continue
        asked += 1
        version = version_of(crate_manifest(crate), r'^version\s*=\s*"([^"]+)"')
        state, _ = registry_state(version, rows)
        if state != "ok":
            continue  # being republished anyway — its fresh floors ride along
        live = highest_live(rows)
        now = deps_behind(rows, live, floors)
        before = (
            set(deps_behind(rows, live, base_floors))
            if base_manifest_text is not None
            else set()
        )
        standing += len(set(now) & before)
        introduced += (f"{crate}: {d}" for d in now if d not in before)
    print(
        f"coherence: {asked} published engine crate(s) asked, {unreachable} unreachable"
        + (
            f", {standing} standing stale pin(s) left to the nightly cut"
            if standing
            else ""
        )
    )
    if unreachable:
        print(
            "  the unreachable crates concluded NOTHING — this pass does not cover them."
        )
    if introduced:
        print()
        print("this change moves a floor past pins the registry already froze. The")
        print("pinned crates will not republish (their versions match the registry),")
        print("so a consumer mixing them gets two copies of the moved crate (#809).")
        print("Bump them in this same PR:")
        for line in introduced:
            print(f"  {line}")
        return 1
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument(
        "--json", action="store_true", help="machine-readable, for a workflow step"
    )
    ap.add_argument(
        "--nag", action="store_true", help="one line, for a scheduled summary"
    )
    ap.add_argument(
        "--cut",
        action="store_true",
        help="the actionable view: what the nightly cut would do (#806)",
    )
    ap.add_argument(
        "--check-coherence",
        action="store_true",
        help="gate: fail if this tree's floors strand a published crate's pins (#809)",
    )
    ap.add_argument(
        "--base",
        default=None,
        help="with --check-coherence: only debt introduced relative to REF fails",
    )
    ap.add_argument(
        "--no-registry",
        action="store_true",
        help="skip the crates.io read (offline, or a deliberately fast local run)",
    )
    args = ap.parse_args()

    if args.check_coherence:
        if args.no_registry:
            print("coherence: --no-registry asked nothing, so nothing is concluded.")
            return 0
        base_text = None
        if args.base:
            proc = subprocess.run(
                ["git", "show", f"{args.base}:rust-packages/Cargo.toml"],
                cwd=ROOT,
                capture_output=True,
                text=True,
                check=False,
            )
            if proc.returncode != 0:
                # A base that cannot be read must not fail innocents on an
                # absolute reading — say so and conclude nothing instead.
                print(f"coherence: cannot read {args.base} — concluding nothing.")
                return 0
            base_text = proc.stdout
        return check_coherence(fetch_index, base_text)

    s = collect(fetch=None if args.no_registry else fetch_index)
    if args.json:
        print(json.dumps(s, indent=2))
    elif args.nag:
        print(render_nag(s))
    elif args.cut:
        print(render_cut(s))
    else:
        print(render(s))
    return 0


if __name__ == "__main__":
    sys.exit(main())
