"""`repo:` citation parsing and resolution — the one copy.

Extracted from `lint.py` so a second tool can resolve a citation without
importing the linter. `check_ext_drift.py` already documents why that was not
an option ("importing it would execute its whole top-level"), and its answer was
to re-derive the parsing locally — which is the multi-source-of-truth rot the
wiki machinery exists to catch, sitting inside the machinery. One copy, here.

This module is import-safe by construction: no top-level I/O, no argument
parsing, nothing but constants and pure functions over `REPO_ROOT`.

The citation grammar itself is AGS-WIKI.md §1. A ref is `repo:<path>` with an
optional suffix — `:NN` / `:NN-MM` for lines, `::symbol` for a symbol,
`#anchor` — and the path may be a glob or carry non-nested `{a,b}` alternation.
"""

from __future__ import annotations

import re
from pathlib import Path

WIKI = Path(__file__).resolve().parent.parent
REPO_ROOT = WIKI.parent

#: The `repo:` citation itself. The lookbehind stops it matching inside a longer
#: token (e.g. a URL fragment ending `...repo:`), and the character class stops
#: it swallowing the backticks, pipes and parens that wrap refs in prose.
REPO_REF = re.compile(r"(?<![A-Za-z0-9_-])repo:([^\s`()|]+)")

#: Punctuation a ref may pick up from the prose around it. Public because the
#: wikilink and CLAUDE.md scans strip the same set from their own captures.
TRAILING_PUNCT = ".,;:!?'\")"


def strip_ref_suffix(raw: str) -> str:
    """path/glob portion of a repo: ref — strips #anchor, ::symbol, :line."""
    val = raw
    if "#" in val:
        val = val.split("#", 1)[0]
    if "::" in val:
        val = val.split("::", 1)[0]
    # trailing :NNN or :NNN-NNN (single line or a line range)
    m = re.match(r"^(.*):(\d+)(-\d+)?$", val)
    if m:
        val = m.group(1)
    return val


def _expand_braces(pattern: str) -> list[str]:
    # non-nested {a,b,c} alternation only — the only shape seen in-vault
    # (e.g. `{python,node,cli,duckdb}`); good enough for a report-only pass.
    m = re.search(r"\{([^{}]+)\}", pattern)
    if not m:
        return [pattern]
    prefix, suffix = pattern[: m.start()], pattern[m.end() :]
    out = []
    for alt in m.group(1).split(","):
        out.extend(_expand_braces(prefix + alt + suffix))
    return out


def _is_glob(s: str) -> bool:
    return any(c in s for c in "*?[")


def path_exists(path_str: str) -> bool:
    if not path_str:
        return False
    for expanded in _expand_braces(path_str):
        if _is_glob(expanded):
            try:
                if any(REPO_ROOT.glob(expanded)):
                    return True
            except (NotImplementedError, ValueError):
                continue
        elif (REPO_ROOT / expanded).exists():
            return True
    return False


def resolve_ref(raw: str) -> tuple[bool, str]:
    """(exists, path_used) — tries the ref as-written, then progressively
    strips trailing punctuation. Needed because ~17% of refs in this vault
    are bare (non-backtick) mentions inside `repo_refs: {...: "repo:..."}`
    frontmatter, so the raw regex capture drags in a trailing `"` — a real
    ref, not a dead one, once the quote is off."""
    candidate = strip_ref_suffix(raw)
    seen = set()
    while candidate and candidate not in seen:
        seen.add(candidate)
        if path_exists(candidate):
            return True, candidate
        if candidate[-1] in TRAILING_PUNCT:
            candidate = candidate[:-1]
        else:
            break
    return False, candidate
