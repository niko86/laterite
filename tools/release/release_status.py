"""What is unreleased, on both tiers — and what part the next bump should be.

## Why this exists

The two version tiers (`product` and `engine`, split 2026-08-01) each move only
when someone runs `bump-version.sh`. Nothing bumps on merge, which is correct —
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
  factual public surface per published crate. A `+pub` line is an addition, a
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
* **Nothing here reads crates.io, PyPI or npm.** "Unreleased" means "not
  stamped in this tree", which is not the same as "not on the registry" — a
  stamped version whose tag was never cut is invisible to this. That failure is
  real (0.8.1/0.8.2 exist in git and on no registry) and this does not catch it.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SNAPSHOTS = ROOT / "tools" / "release" / "public-api"
CHANGELOG = ROOT / "changelog.json"
ENGINE_MANIFEST = ROOT / "rust-packages" / "Cargo.toml"
PRODUCT_MANIFEST = ROOT / "pyproject.toml"

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


def engine_version() -> str:
    # The [workspace.package] one, not the [workspace.dependencies] pins that
    # happen to carry the same string.
    text = ENGINE_MANIFEST.read_text()
    head = text.index("[workspace.package]")
    return version_of_text(text[head:])


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


def api_delta(since: str) -> tuple[int, int, list[str]]:
    """Net public-API additions and removals in the snapshots since `since`.

    Net, not raw: a snapshot regeneration can rewrite a line in place, which
    shows as one `-pub` and one `+pub` for the same signature and is not a
    change to the surface at all.
    """
    if not since:
        return 0, 0, []
    diff = sh("git", "diff", f"{since}..HEAD", "--", str(SNAPSHOTS.relative_to(ROOT)))
    added = {ln[1:] for ln in diff.splitlines() if ln.startswith("+pub")}
    removed = {ln[1:] for ln in diff.splitlines() if ln.startswith("-pub")}
    net_add, net_rm = added - removed, removed - added
    return len(net_add), len(net_rm), sorted(net_rm)


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


def collect() -> dict:
    engine_sha, engine_stamp = last_stamp(
        ENGINE_MANIFEST, "/^\\[workspace.package\\]/,+2"
    )
    _, product_stamp = last_stamp(PRODUCT_MANIFEST, "/^version/,+1")
    added, removed, removed_names = api_delta(engine_sha)
    sections = changelog_sections()
    return {
        "engine": {
            "version": engine_version(),
            "last_stamp": engine_stamp,
            "api_added": added,
            "api_removed": removed,
            "api_removed_names": removed_names[:20],
            "verdict": verdict_from_api(added, removed),
        },
        "product": {
            "version": version_of(PRODUCT_MANIFEST, r'^version\s*=\s*"([^"]+)"'),
            "last_stamp": product_stamp,
            "verdict": verdict_from_sections(sections),
        },
        "changelog_unreleased": sections,
    }


def render(s: dict) -> str:
    e, p = s["engine"], s["product"]
    lines = [
        f"engine   {e['version']:<10} last stamped {e['last_stamp']}",
        f"         public API since: +{e['api_added']} -{e['api_removed']}"
        f"   ->  {e['verdict'].upper()}",
        f"product  {p['version']:<10} last stamped {p['last_stamp']}",
        "         changelog [unreleased]: "
        + (
            ", ".join(f"{k} {v}" for k, v in sorted(s["changelog_unreleased"].items()))
            or "empty"
        )
        + f"   ->  {p['verdict'].upper()}",
    ]
    if e["api_removed_names"]:
        lines.append("")
        lines.append(
            "  public API REMOVED since the last engine stamp — a consumer has to follow:"
        )
        lines += [f"    {n}" for n in e["api_removed_names"]]
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
        "  the engine verdict is from the API snapshots; the product verdict is from the"
    )
    lines.append(
        "  changelog sections. The product's own API surface is NOT measured — no committed"
    )
    lines.append(
        "  snapshot exists for the Python/Node surface — so treat that one as a suggestion."
    )
    lines.append(
        "  Neither reads a registry: 'unreleased' means 'not stamped here', which does not"
    )
    lines.append("  catch a stamped version whose tag was never cut.")
    return "\n".join(lines)


def render_nag(s: dict) -> str:
    e, p = s["engine"], s["product"]
    owed = [
        t
        for t, v in (("engine", e["verdict"]), ("product", p["verdict"]))
        if v != "none"
    ]
    if not owed:
        return "release-status: nothing owed — engine and product are both level with the tree."
    return (
        f"release-status: engine {e['version']} (+{e['api_added']} -{e['api_removed']} API) · "
        f"product {p['version']} · {' + '.join(owed)} release owed "
        f"(engine {e['verdict']}, product {p['verdict']})"
    )


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument(
        "--json", action="store_true", help="machine-readable, for a workflow step"
    )
    ap.add_argument(
        "--nag", action="store_true", help="one line, for a scheduled summary"
    )
    args = ap.parse_args()

    s = collect()
    if args.json:
        print(json.dumps(s, indent=2))
    elif args.nag:
        print(render_nag(s))
    else:
        print(render(s))
    return 0


if __name__ == "__main__":
    sys.exit(main())
