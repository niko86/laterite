#!/usr/bin/env python3
"""Hold every platform list in the docs to the release matrix that builds it.

linux-arm64-gnu joined the Node release at 0.11.0. Four documents went on
saying there were three native packages, one of them the npm README, which is
the page a user installing the package reads. Nothing read those lists, so
nothing failed: the week-44 curation found three, and a grep for the fix found
the fourth. The build matrices in `release.yml` are what actually ships, so
they are the authority and every list is held to them.

What counts as a list is a paragraph naming two or more platforms in the same
vocabulary. One mention is a reference to that platform, not a claim about the
set, and is counted and skipped. Two vocabularies are recognised:

  * napi platform names (`linux-x64-gnu`, `@laterite/native-darwin-arm64`,
    `native-{a,b}` brace lists). Only the Node packages use them, so a list
    must equal `napi.targets` mapped to napi's names.
  * Rust target triples. Three release jobs build per triple, so a list must
    equal one of `build-wheels`, `build-binaries` or `build-node`.

A count of native packages ("three native packages") is held to the size of
`napi.targets` too, since that is the phrasing that went stale. "One native
package per target" is a rule, not a count, and is not judged.

Not recognised, and said so on every run: plain-English platform names
("macOS arm64", "Windows x64"). No doc lists them today, and telling a list of
them from a benchmark's machine description is not a pattern a scanner can
hold.

Skipped: fenced code (captured output is the tool's, not the author's),
`CHANGELOG.md` and `ags-wiki/log.md` (history), and any paragraph carrying
`<!-- historical -->`, which is lint.py's A11 marker for a series that
already happened.

Usage:
    uv run --no-project python tools/check_platform_lists.py

Exit 0 when every list agrees, 1 when one does not.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RELEASE = ROOT / ".github" / "workflows" / "release.yml"
NODE_PKG = ROOT / "rust-packages" / "laterite-node" / "package.json"
JOBS = ("build-wheels", "build-binaries", "build-node")
SKIP_FILES = {"CHANGELOG.md", "ags-wiki/log.md"}

#: napi-rs's own name for each Rust triple: `@laterite/native-<name>` is the
#: package it publishes. A target added to `napi.targets` that is missing here
#: fails the gate by name rather than going unchecked.
NAPI_NAMES = {
    "x86_64-unknown-linux-gnu": "linux-x64-gnu",
    "aarch64-unknown-linux-gnu": "linux-arm64-gnu",
    "x86_64-unknown-linux-musl": "linux-x64-musl",
    "aarch64-unknown-linux-musl": "linux-arm64-musl",
    "x86_64-apple-darwin": "darwin-x64",
    "aarch64-apple-darwin": "darwin-arm64",
    "x86_64-pc-windows-msvc": "win32-x64-msvc",
    "aarch64-pc-windows-msvc": "win32-arm64-msvc",
}

_NAPI = re.compile(
    r"(?<![A-Za-z0-9_])((?:linux|darwin|win32)-(?:x64|arm64)(?:-(?:gnu|musl|msvc))?)"
    r"(?![A-Za-z0-9_-])"
)
_BRACES = re.compile(r"native-\{([^}]*)\}")
_TRIPLE = re.compile(
    r"(?<![A-Za-z0-9_])((?:x86_64|aarch64)-(?:unknown-linux-(?:gnu|musl)"
    r"|apple-darwin|pc-windows-msvc))(?![A-Za-z0-9_-])"
)
_NUMWORDS = {
    w: i + 1
    for i, w in enumerate(
        ["one", "two", "three", "four", "five", "six", "seven", "eight", "nine"]
    )
}
_COUNT = re.compile(
    r"\b(\d+|" + "|".join(_NUMWORDS) + r")\s+(?:[\w`@/*-]+\s+){0,2}?"
    r"`?(?:@laterite/)?native(?:-\*)?`?\s+(?:packages?|pins?|addons?|binaries)\b"
    r"(?!\s+per\b)",
    re.I,
)
_FENCE = re.compile(r"^(```|~~~).*?^\1[^\n]*$", re.M | re.S)
_HISTORICAL = re.compile(r"<!--\s*historical\s*-->")


def matrix_targets(workflow: str) -> dict[str, set[str]]:
    """Each job's `target:` values, read from its own block of the file."""
    out: dict[str, set[str]] = {}
    for job in JOBS:
        m = re.search(
            rf"^  {re.escape(job)}:\n(.*?)(?=^  \S|\Z)", workflow, re.M | re.S
        )
        if not m:
            raise SystemExit(f"[platform-lists] FAIL: no `{job}` job in {RELEASE}")
        out[job] = set(
            re.findall(r"^\s+(?:- )?target:\s*([A-Za-z0-9_-]+)\s*$", m.group(1), re.M)
        )
        if not out[job]:
            raise SystemExit(f"[platform-lists] FAIL: `{job}` lists no targets")
    return out


def paragraphs(text: str) -> list[tuple[int, str]]:
    """(first line, text) per blank-line-separated block, fences blanked."""
    text = _FENCE.sub(lambda m: "\n" * m.group(0).count("\n"), text)
    out, start, buf = [], 1, []
    for n, line in enumerate(text.split("\n"), 1):
        if line.strip():
            if not buf:
                start = n
            buf.append(line)
        elif buf:
            out.append((start, "\n".join(buf)))
            buf = []
    if buf:
        out.append((start, "\n".join(buf)))
    return out


def napi_names(para: str) -> set[str]:
    names = set(_NAPI.findall(para))
    for group in _BRACES.findall(para):
        names |= {g.strip() for g in group.split(",") if g.strip()}
    return names


def scan(
    rel: str, text: str, node: set[str], jobs: dict[str, set[str]], tally: dict
) -> list[str]:
    """Findings for one document; `tally` collects the scope report."""
    findings = []
    want_napi = {NAPI_NAMES[t] for t in node}
    for line, para in paragraphs(text):
        names, triples = napi_names(para), set(_TRIPLE.findall(para))
        counts = _COUNT.findall(para)
        if not (names or triples or counts):
            continue
        if _HISTORICAL.search(para):
            tally["historical"] += 1
            continue
        for found, vocab in ((names, "napi"), (triples, "triple")):
            if len(found) == 1:
                tally["single"] += 1
            elif found:
                tally["lists"] += 1
                if vocab == "napi" and found != want_napi:
                    findings.append(
                        f"{rel}:{line}: lists {sorted(found)}; napi.targets "
                        f"builds {sorted(want_napi)}"
                    )
                if vocab == "triple" and found not in jobs.values():
                    named = "; ".join(f"{j} {sorted(s)}" for j, s in jobs.items())
                    findings.append(
                        f"{rel}:{line}: lists {sorted(found)}, which is no "
                        f"release job's set ({named})"
                    )
        for word in counts:
            tally["counts"] += 1
            n = int(word) if word.isdigit() else _NUMWORDS[word.lower()]
            if n != len(node):
                findings.append(
                    f"{rel}:{line}: counts {word} native packages; napi.targets "
                    f"has {len(node)}"
                )
    return findings


def main() -> int:
    jobs = matrix_targets(RELEASE.read_text(encoding="utf-8"))
    node = set(json.loads(NODE_PKG.read_text(encoding="utf-8"))["napi"]["targets"])
    if missing := sorted(node - NAPI_NAMES.keys()):
        print(
            f"[platform-lists] FAIL: no napi name for {missing}; add it to "
            "NAPI_NAMES so lists naming it can be checked",
            file=sys.stderr,
        )
        return 1
    files = subprocess.run(
        ["git", "-C", str(ROOT), "ls-files", "*.md"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout.split()
    tally = {"lists": 0, "single": 0, "counts": 0, "historical": 0}
    findings = []
    skipped = 0
    for rel in files:
        if rel in SKIP_FILES:
            skipped += 1
            continue
        text = (ROOT / rel).read_text(encoding="utf-8", errors="replace")
        findings += scan(rel, text, node, jobs, tally)
    print(
        f"[platform-lists] {len(files) - skipped} docs scanned, {skipped} skipped "
        f"as history; {tally['lists']} platform lists and {tally['counts']} "
        f"package counts judged, {tally['single']} single mentions and "
        f"{tally['historical']} historical paragraphs not judged. Plain-English "
        f"platform names are not recognised."
    )
    for f in findings:
        print(f"[platform-lists] FAIL: {f}", file=sys.stderr)
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
