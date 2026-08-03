#!/usr/bin/env python3
"""Fail when a doc references a file that isn't there.

Cheap, stdlib-only, and it exists because a session-by-session read cannot keep
up. A one-off audit on 2026-08-03 found four dead references in seconds, two of
them on PUBLISHED crates.io pages where a version's README is frozen forever.
The audit should not have been a session; it should have been this.

TWO RESOLUTION RULES, and the difference is the whole point.

  * A doc in the repo is read from a checkout, so a reference resolves if it
    exists relative to the doc, or to the repo root.

  * A PUBLISHED crate's README is read on crates.io, where the only files that
    exist are the ones inside that crate's package. `[x](../../OBSERVATIONS.md)`
    and `[x](OBSERVATIONS.md)` both resolve fine in a checkout and both 404 for
    the person deciding whether to `cargo add` your crate.

That second rule is what the one-off audit initially missed: a first pass that
fell back to the repo root reported the validator's README as clean, while the
live 0.9.0 page carried three dead `OBSERVATIONS.md` links and two dead
`README-cli` links. Published READMEs are therefore checked STRICTLY — no
repo-root fallback — and links must be absolute URLs to escape the package.

Not checked here, deliberately: URL liveness (network, flaky, and the docs job's
mkdocs strict build already link-gates the site), and anchors.

  check_doc_refs.py            report dead references
  check_doc_refs.py --check    same, but exit 1 if any (the CI gate)

Run: `uv run --no-project python tools/check_doc_refs.py --check` (stdlib only).
"""

from __future__ import annotations

import argparse
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

#: Docs read from a checkout: resolve doc-relative, then repo-relative.
REPO_DOCS = [
    "README.md",
    "CLAUDE.md",
    "CONTRIBUTING.md",
    "RELEASING.md",
    "COMPAT.md",
    "tools/README.md",
]

#: A backticked path-shaped token, and a Markdown link target.
PATH_RE = re.compile(
    r"`([A-Za-z0-9_./-]+\.(?:py|sh|rs|toml|json|ts|mjs|yml|yaml|md|ps1))`"
)
LINK_RE = re.compile(r"\[[^\]]*\]\(([^)#][^)]*)\)")

#: Prefixes that are never a local file.
EXTERNAL = ("http://", "https://", "mailto:", "#", "<")


def _targets(text: str) -> set[str]:
    out = {m for m in PATH_RE.findall(text) if "/" in m}
    out |= set(LINK_RE.findall(text))
    return {t.split("#")[0].strip() for t in out if t and not t.startswith(EXTERNAL)}


def _published_crates() -> list[Path]:
    """Crates without `publish = false` — the ones whose README becomes a public,
    permanent page the moment a version goes up."""
    out = []
    for man in sorted((ROOT / "rust-packages").glob("*/Cargo.toml")):
        pkg = tomllib.load(man.open("rb")).get("package", {})
        if pkg.get("publish") is not False:
            out.append(man.parent)
    return out


def scan() -> list[str]:
    problems: list[str] = []

    for rel in REPO_DOCS:
        doc = ROOT / rel
        if not doc.exists():
            continue
        for t in sorted(_targets(doc.read_text(errors="replace"))):
            if (doc.parent / t).exists() or (ROOT / t).exists():
                continue
            problems.append(f"{rel}: `{t}` does not exist")

    for crate in _published_crates():
        rd = crate / "README.md"
        if not rd.exists():
            problems.append(
                f"{crate.name}: PUBLISHED with no README — its crates.io page will be bare"
            )
            continue
        for t in sorted(_targets(rd.read_text(errors="replace"))):
            # Strict: only what ships inside the package exists on crates.io.
            if (crate / t).exists():
                continue
            problems.append(
                f"{crate.name}/README.md: `{t}` is not in the published package — "
                f"dead on crates.io (use an absolute github.com URL)"
            )

    return problems


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--check", action="store_true", help="exit 1 if any reference is dead"
    )
    args = ap.parse_args()

    problems = scan()
    for p in problems:
        print(f"  {p}")
    if not problems:
        print("check_doc_refs: every referenced path exists")
        return
    print(f"check_doc_refs: {len(problems)} dead reference(s)")
    if args.check:
        sys.exit(1)


if __name__ == "__main__":
    main()
