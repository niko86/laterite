"""Which wiki pages describe the files I am about to change?

    uv run --no-project python ags-wiki/.bootstrap/librarian.py --paths <files…>

The vault has 424 pages and one navigational aid, `index.md`, which is an
inventory keyed on filename stem. That answers "is there a page called X". It
cannot answer the question anyone editing code actually has, because the page
covering `laterite-ags4-reference/src/dict.rs` is called `edition-resolution` and
no stem lookup will ever suggest it.

The answer is already on disk: 374 of 424 pages carry `repo:` citations. This
inverts them. It reads nothing but the vault, needs no index, and is a **lookup,
not a gate** — it always exits 0, nothing checks that you ran it, and no
enforcement anywhere depends on it. The gates that keep the wiki true are
elsewhere (`gen_crate_graph.py --check`, `lint.py`, the faithfulness tests); this
exists to save a search.

Three things it deliberately does that a naive inversion would not:

**It separates exact citations from directory sweeps.** Only ~18-23% of tracked
non-wiki files are cited exactly. Most apparent coverage is a page citing an
ancestor directory — `repo:web/` reaches 323 files from two pages that describe
none of them. Those print marked, because a tool that quietly counted them as
coverage would answer "yes, that's documented" for most of the tree.

**It ranks, then caps.** `ags_dictionary.json` is cited by 184 pages, 177 of them
the generated `groups/` tier. Unranked, the pages that actually explain the
dictionary are buried under AAVT.md and its siblings. Exact beats ancestor, then
explanatory classes beat the reference tiers, and the cap says what it dropped —
a silent truncation reads as "that is everything".

**It prints the title, not the stem.** 79 of 138 hand-written pages carry a
`title:` that differs from their filename, which is exactly the gap that makes a
stem index unusable for this.
"""

from __future__ import annotations

import argparse
import re
import sys
from typing import TYPE_CHECKING, NamedTuple

from refs import (
    REPO_REF,
    TRAILING_PUNCT,
    WIKI,
    frontmatter_block,
    ref_covers,
    strip_ref_suffix,
)

if TYPE_CHECKING:
    from pathlib import Path

#: Page classes, most-explanatory first. A `tool` page describes the artifact you
#: are editing; a `group` page describes one AGS group and is generated from the
#: dictionary. Anything unlisted sorts between the two — unknown is not a reason
#: to bury a page, only a reason not to promote it.
CLASS_ORDER = [
    "tool",
    "decision",
    "concept",
    "insight",
    "strategy",
    "comparison",
    "rule",
    "observation",
    "edition",
    "source",
    "type",
    "group",
]
_UNKNOWN_RANK = CLASS_ORDER.index("comparison")

#: Files that are not pages: the schema, the log, the generated index.
#: Templates need no entry — AGS-WIKI.md §2 gives every meta page a leading `_`,
#: which `_pages` already skips. A second `templates/` check was written here
#: first and mutation testing found it could not fail: nothing in that directory
#: lacks the prefix, so the rule was doing the work twice.
SKIP_STEMS = {"index", "log", "AGS-WIKI"}


class Hit(NamedTuple):
    """One page that cites the queried path.

    `rel` is the page's path within the vault, carried on the hit rather than
    recomputed from a module-level root — so a caller can point the whole lookup
    at a synthetic vault and the output still reads correctly.
    """

    rel: str
    title: str
    kind: str
    how: str

    def sort_key(self) -> tuple[int, int, str]:
        # Exact and glob citations first: the page named this file. Then by
        # class, then stable by path.
        #
        # There is no separate hand-written-before-generated dimension. One was
        # written and mutation testing showed it could never change an ordering:
        # the only generated tier is `groups/`, every page in it is `type: group`,
        # and `group` is already last in CLASS_ORDER. Two dimensions expressing
        # one fact is the drift this whole programme exists to remove.
        return (
            0 if self.how in ("exact", "glob") else 1,
            _rank(self.kind),
            self.rel,
        )


def _norm(path: str) -> str:
    """Repo-root-relative, as the citations are written.

    `removeprefix`, never `lstrip("./")` — a character-set strip turns
    `.github/workflows/ci.yml` into `github/workflows/ci.yml`, which then matches
    nothing and reports the file as uncited. It did exactly that on the first run.
    """
    path = path.strip()
    while path.startswith("./"):
        path = path[2:]
    return path


def _rank(kind: str) -> int:
    try:
        return CLASS_ORDER.index(kind)
    except ValueError:
        return _UNKNOWN_RANK


def _pages(wiki: Path) -> list[Path]:
    return sorted(
        p
        for p in wiki.rglob("*.md")
        if not p.name.startswith("_")
        and p.stem not in SKIP_STEMS
        and not any(part.startswith(".") for part in p.relative_to(wiki).parts)
    )


def _meta(text: str, page: Path) -> tuple[str, str]:
    """(title, type) — falling back to the stem and the containing directory.

    The directory fallback matters: a page missing `type:` still sorts with its
    neighbours instead of landing in the unknown bucket.
    """
    fm = frontmatter_block(text) or ""
    title = re.search(r"^title:\s*(.+?)\s*$", fm, re.M)
    kind = re.search(r"^type:\s*(.+?)\s*$", fm, re.M)
    parent = page.parent.name
    return (
        title.group(1).strip("\"'") if title else page.stem,
        kind.group(1).strip("\"'") if kind else parent.rstrip("s"),
    )


def catalogue(wiki: Path = WIKI) -> dict[str, tuple[str, str, set[str]]]:
    """vault-relative page -> (title, class, cited paths).

    Every `repo:` ref on the page, suffixes stripped.

    Frontmatter refs are included: `repo_refs.root` is often a page's only
    citation, and it is the most precise one on the page.
    """
    out: dict[str, tuple[str, str, set[str]]] = {}
    for page in _pages(wiki):
        text = page.read_text(encoding="utf-8")
        title, kind = _meta(text, page)
        cited = set()
        for m in REPO_REF.finditer(text):
            path = strip_ref_suffix(m.group(1)).rstrip(TRAILING_PUNCT)
            # A ref may carry a trailing selector after a space, e.g.
            # `repo:…/ags_dictionary.json groups[code=SAMP]`; the regex stops at
            # the space, so nothing to strip — but a bare `repo:` would leave "".
            if path:
                cited.add(path)
        out[page.relative_to(wiki).as_posix()] = (title, kind, cited)
    return out


def lookup(target: str, cat: dict[str, tuple[str, str, set[str]]]) -> list[Hit]:
    """Pages citing `target`, best first. `target` is repo-root-relative."""
    target = _norm(target)
    precision = {"exact": 0, "glob": 1, "ancestor": 2}
    hits: list[Hit] = []
    for rel, (title, kind, cited) in cat.items():
        best: str | None = None
        for ref in cited:
            how = ref_covers(ref, target)
            if how and (best is None or precision[how] < precision[best]):
                best = how
            if best == "exact":
                break
        if best:
            hits.append(Hit(rel, title, kind, best))
    return sorted(hits, key=Hit.sort_key)


def report(paths: list[str], limit: int, show_all: bool, wiki: Path = WIKI) -> str:
    cat = catalogue(wiki)
    lines: list[str] = []
    for raw in paths:
        target = _norm(raw)
        lines.append(target)
        hits = lookup(target, cat)
        if not hits:
            lines.append("  no page cites this path")
            lines.append("")
            continue
        shown = hits if show_all else hits[:limit]
        width = max(len(h.rel) for h in shown)
        for h in shown:
            note = (
                ""
                if h.how in ("exact", "glob")
                else "  (directory only — may not describe this file)"
            )
            lines.append(f"  {h.rel:<{width}}  {h.title}{note}")
        if len(hits) > len(shown):
            lines.append(f"  … +{len(hits) - len(shown)} more (--all)")
        lines.append("")
    return "\n".join(lines).rstrip() + "\n"


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(
        description="Which wiki pages cite these repo paths?",
        epilog="A lookup, not a gate — it always exits 0 and nothing checks "
        "that you ran it.",
    )
    ap.add_argument("--paths", nargs="+", required=True, metavar="PATH")
    ap.add_argument("--limit", type=int, default=5, help="pages per path (default 5)")
    ap.add_argument("--all", action="store_true", help="every hit, uncapped")
    args = ap.parse_args(argv)
    sys.stdout.write(report(args.paths, args.limit, args.all))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
