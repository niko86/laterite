#!/usr/bin/env python
"""A10 — best-effort external-repo drift check for `ext:` citations.

Companion to lint.py's A9 (the `ext:<repo-id>:<path>` allowlist check).
A9 only verifies a repo-id is on the allowlist — it can't know whether the
allowlisted repo (or a specific cited path within it) still exists, since
that's a GitHub API call, not a local filesystem check, and this repo's
lint.py deliberately stays network-free so `--since` stays fast and doesn't
flake in CI. This script is the network-touching half, run on its own
schedule (weekly cron, see .github/workflows/wiki-ext-drift.yml) — never
imported by lint.py, never part of the PR-blocking path.

Report-only by design: exits 0 unconditionally. "Drift" here means a
CONFIRMED 404 from the GitHub API — a network hiccup, timeout, or rate
limit is `unknown`, not drift (never let API flakiness manufacture a false
positive; see the module's own `check_gh` docstring).

Doesn't import lint.py (importing it would execute its whole top-level
scan-and-sys.exit() body as a side effect) — the EXT_ALLOWLIST is instead
pulled out of lint.py's *source text* by regex, so this script can never
drift from lint.py's own allowlist without also changing lint.py itself.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

WIKI = Path(__file__).resolve().parent.parent
LINT_PY = WIKI / ".bootstrap" / "lint.py"
SKIP_DIRS = {".obsidian", ".bootstrap", "templates"}

EXT_REF = re.compile(r"(?<![A-Za-z0-9_-])ext:([^\s`()|]+)")
_TRAILING_PUNCT = ".,;:!?'\")"


def load_allowlist() -> set[str]:
    txt = LINT_PY.read_text(encoding="utf-8")
    m = re.search(r"EXT_ALLOWLIST = \{(.*?)\}", txt, re.S)
    if not m:
        print(
            "could not find EXT_ALLOWLIST in lint.py — nothing to check",
            file=sys.stderr,
        )
        return set()
    return set(re.findall(r'"([^"]+)"', m.group(1)))


def collect_ext_refs(allowlist: set[str]) -> set[tuple[str, str | None]]:
    """(repo_id, path_or_None) pairs actually cited in the vault, for
    allowlisted repo-ids only — mirrors lint.py's EXT_REF scan without
    duplicating its allowlist-miss reporting (that's A9's job, not this
    script's)."""
    found: set[tuple[str, str | None]] = set()
    for p in WIKI.rglob("*.md"):
        if set(p.parts) & SKIP_DIRS:
            continue
        # log.md is append-only HISTORY: an entry narrating a past drift (e.g.
        # "found ext:…/test-duckdb-ext.sh 404s") spells the very ref it reports
        # dead, so scanning it re-flags the drift FOREVER (laterite-dev#495) — the citation
        # isn't a live claim, it's the record of finding it gone. Exempt it, the
        # same treatment lint.py gives log.md's dead repo: refs. Live pages, where
        # a citation IS a standing claim, are still scanned.
        if p.name == "log.md":
            continue
        txt = p.read_text(encoding="utf-8")
        for m in EXT_REF.finditer(txt):
            raw = m.group(1)
            if "..." in raw or "…" in raw:
                continue  # grammar-doc placeholder, not a real citation
            clean = raw
            while clean and clean[-1] in _TRAILING_PUNCT:
                clean = clean[:-1]
            repo_id, path = (
                [*clean.split(":", 1), None][:2] if ":" in clean else (clean, None)
            )
            if repo_id in allowlist:
                found.add((repo_id, path))
    return found


def check_gh(api_path: str) -> str:
    """'ok' | 'missing' | 'unknown' — never raises. A confirmed HTTP 404 is
    the only outcome that counts as drift; anything else (timeout, rate
    limit, gh not authed, transient 5xx) is 'unknown' so a flaky runner
    can't manufacture a false "this got deleted" report."""
    try:
        r = subprocess.run(
            ["gh", "api", api_path], capture_output=True, text=True, timeout=20
        )
    except Exception:
        return "unknown"
    if r.returncode == 0:
        return "ok"
    if "HTTP 404" in r.stderr or "Not Found" in r.stderr:
        return "missing"
    return "unknown"


def main() -> int:
    allowlist = load_allowlist()
    print(f"EXT_ALLOWLIST ({len(allowlist)}): {', '.join(sorted(allowlist))}")
    refs = collect_ext_refs(allowlist)
    print(f"cited ext: refs in the vault: {len(refs)} unique (repo, path) pairs\n")

    missing: list[str] = []
    unknown: list[str] = []
    checked_repos: set[str] = set()

    for repo_id in sorted(allowlist):
        if "/" not in repo_id:
            print(
                f"  {repo_id}: SKIPPED (not a GitHub owner/repo — "
                f"e.g. the GitLab-hosted python-ags4 mirror; not "
                f"API-checkable here)"
            )
            continue
        status = check_gh(f"repos/{repo_id}")
        print(f"  {repo_id}: {status.upper()}")
        checked_repos.add(repo_id)
        if status == "missing":
            missing.append(repo_id)
        elif status == "unknown":
            unknown.append(repo_id)

    scoped_refs = sorted(
        (r for r in refs if r[0] in checked_repos), key=lambda r: (r[0], r[1] or "")
    )
    for repo_id, path in scoped_refs:
        if not path:
            continue
        status = check_gh(f"repos/{repo_id}/contents/{path}")
        label = f"{repo_id}:{path}"
        print(f"  {label}: {status.upper()}")
        if status == "missing":
            missing.append(label)
        elif status == "unknown":
            unknown.append(label)

    print(f"\nCONFIRMED MISSING (real drift): {len(missing)}")
    for m in missing:
        print(f"  - {m}")
    print(f"UNKNOWN (network/API issue — not counted as drift): {len(unknown)}")
    for u in unknown:
        print(f"  - {u}")

    # Machine-readable lines the workflow step greps for. Report-only: this
    # script's own exit code is always 0 regardless of what it found.
    #
    # One `DRIFT_REF=` per missing citation, not just the count: the tracking
    # issue is driven by the SET, so it can say what newly broke and what came
    # back rather than restating a number. A count alone can only ever produce
    # the "N confirmed-missing citation(s)" sentence the old inline shell
    # commented every single week.
    print()
    for m in missing:
        print(f"DRIFT_REF={m}")
    print(f"DRIFT_COUNT={len(missing)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
