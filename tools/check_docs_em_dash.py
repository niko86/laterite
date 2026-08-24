#!/usr/bin/env python3
"""No em dash reaches a docs-site reader (#588, the docs half of #587).

The em dash is out of reader-facing copy across this project's surfaces. The
landing page holds that with a browser test that walks the rendered DOM; the
docs site is fifty-odd Markdown pages built by mkdocs, and walking the built
HTML would mean building the site to find a character. This walks the SOURCE
instead, which is also where the fix belongs.

## What it does not look at, and why that is written down

Every gate here carries a class of input it silently does not read, and an
unstated scope is a green tick over exactly the thing that later breaks
(CLAUDE.md, Conventions). So this reports its exclusions on every run, pass or
fail, rather than only when it is unhappy:

* **Fenced code blocks and inline code spans.** A fence reproduces what a tool
  actually printed, and an `.out` capture that says `—` says it because the
  tool did. Rewriting one would make the page a claim about output nobody can
  reproduce. Counted and reported, never edited.
* **HTML comments**, which mkdocs does not render. The `doc-code: skip — why`
  markers live in them and spell their reason after an em dash, and a policy
  about what a reader sees has nothing to say about text no reader sees.
* **Two whole page families that exist only after a build.** `reference/groups/`
  is one page per AGS4 group, rendered from `ags_dictionary.json`, and its prose
  is the STANDARD's own heading descriptions: rewriting those would not fix a
  house style, it would misquote the specification. `reference/api`, `types`,
  `modules` and `cli` are mkdocstrings rendering the shipped wheel's docstrings,
  which #588 puts out of scope explicitly. Neither has a `.md` this gate could
  read, and neither should be rewritten, so naming them is the whole of what can
  be done about them here.
* **Anything the theme itself injects.** A dash from a template override or a
  plugin is outside this. Named because "no em dash reaches the reader" is the
  policy and a source scan is a proxy for it.
* **CSS and JS under `docs/`.** Comments in the stylesheets and the two small
  scripts are not reader-facing copy, and #588 puts them out of scope
  explicitly. Counted so the number is visible rather than assumed to be zero.

## The generated pages

`reference/divergences.md` is rendered from `observations.json` and is not
hand-editable; the rest of the pages carry generated OUTPUT in marker slots
(`gen_doc_outputs.py`) inside fences this gate already skips. So a failure on a
generated page means the fix goes to the generator or its source data, and the
message says so rather than inviting a hand-edit the next regeneration undoes.

Usage:
    uv run --no-project python tools/check_docs_em_dash.py
    uv run --no-project python tools/check_docs_em_dash.py --skipped
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOCS = ROOT / "web" / "docs-site" / "docs"
MKDOCS = ROOT / "web" / "docs-site" / "mkdocs.yml"

#: The `mkdocs.yml` keys whose values a reader actually sees. `site_description`
#: is the `<meta name="description">`, so it is the search snippet and the link
#: preview; `copyright` renders in every page footer. Both are read here for a
#: reason worth writing down: they are ONE string each in a file this gate had
#: no reason to open, and they render on every built page, so between them they
#: were the single largest source of em dashes in the built site while a scan of
#: `docs/**.md` alone reported it clean. The rest of that file is YAML comments,
#: which no reader sees.
MKDOCS_READER_KEYS = ("site_name", "site_description", "copyright")

EM_DASH = "—"
FENCE_RE = re.compile(r"^\s*(```|~~~)")
INLINE_CODE_RE = re.compile(r"`[^`]*`")
#: HTML comments, which mkdocs does not render, so nothing in one is reader
#: copy. This carve-out is not tidiness: the `doc-code: skip — why` markers that
#: `gen_doc_outputs.py` reads spell their reason after an em dash, and without
#: it this gate demanded they be rewritten to satisfy a policy about text no
#: reader can see. Two authors hit it independently and both correctly refused
#: to touch the markers rather than work around the gate.
HTML_COMMENT_RE = re.compile(r"<!--.*?-->", re.S)

#: Pages no hand-edit can fix, mapped to what to edit instead. A gate that says
#: "line 12 has an em dash" about a rendered file sends someone to edit a file
#: the next `--check` overwrites.
GENERATED = {
    "reference/divergences.md": (
        "rendered by tools/gen_observations.py from observations.json — fix "
        "the record's `user_facing.summary`, or the generator's own prose, "
        "then regenerate"
    ),
}


def scan(text: str) -> tuple[list[tuple[int, str]], int]:
    """Em dashes in prose, and how many were skipped as code.

    Both halves come back because the skip count is the gate's own blind spot
    made visible: a page whose dashes are all inside fences passes, and a
    reader of the run should be able to see that is what happened.
    """
    hits: list[tuple[int, str]] = []
    skipped = 0
    in_fence = False
    # Comments are blanked across the WHOLE text first, so one spanning several
    # lines needs no second state machine beside the fence one. Newlines are
    # kept, so line numbers still point where they should.
    masked = HTML_COMMENT_RE.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), text)
    for n, (line, clean) in enumerate(
        zip(text.split("\n"), masked.split("\n"), strict=True), start=1
    ):
        if FENCE_RE.match(line):
            in_fence = not in_fence
            skipped += line.count(EM_DASH)
            continue
        if in_fence:
            skipped += line.count(EM_DASH)
            continue
        prose = INLINE_CODE_RE.sub("", clean)
        skipped += line.count(EM_DASH) - prose.count(EM_DASH)
        if EM_DASH in prose:
            hits.append((n, line.strip()))
    return hits, skipped


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--skipped",
        action="store_true",
        help="list every occurrence this gate deliberately does not read",
    )
    args = ap.parse_args()

    if not DOCS.is_dir():
        print(f"check_docs_em_dash: {DOCS} is not a directory", file=sys.stderr)
        return 1

    failures: list[str] = []
    pages = files_with_prose = skipped_in_code = 0
    skipped_detail: list[str] = []

    # Read as text, not as YAML: this gate is stdlib-only so it runs in the
    # buildless job, and the values wanted are single scalars a line scan reads
    # exactly. A value that wraps across lines is followed until the indentation
    # ends, which is how `site_description` is written today.
    lines = MKDOCS.read_text(encoding="utf-8").split("\n")
    for n, line in enumerate(lines):
        if line.startswith((" ", "\t", "#")) or ":" not in line:
            continue
        key = line.split(":", 1)[0].strip()
        if key not in MKDOCS_READER_KEYS:
            continue
        block = [line.split(":", 1)[1]]
        for cont in lines[n + 1 :]:
            if not cont.strip() or not cont.startswith((" ", "\t")):
                break
            block.append(cont)
        joined = " ".join(" ".join(block).split())
        if EM_DASH in joined:
            where = (
                'the <meta name="description">, so it is the search snippet '
                "and the link preview"
                if key == "site_description"
                else "in the page footer"
            )
            failures.append(
                f"  {MKDOCS.name}:{n + 1}  {key}: {joined[:100]}"
                f"\n      ^ renders on EVERY built page ({where})"
            )

    for path in sorted(DOCS.rglob("*.md")):
        pages += 1
        rel = path.relative_to(DOCS).as_posix()
        hits, skipped = scan(path.read_text(encoding="utf-8"))
        skipped_in_code += skipped
        if skipped and args.skipped:
            skipped_detail.append(f"{rel}: {skipped} inside code")
        if not hits:
            continue
        files_with_prose += 1
        note = GENERATED.get(rel)
        for n, line in hits:
            failures.append(
                f"  {rel}:{n}  {line[:110]}"
                + (f"\n      ^ {note}" if note and n == hits[0][0] else "")
            )

    # Out of scope by decision, counted so the number is a fact rather than an
    # assumption. #588: "Code blocks, quoted output captures … and the wiki".
    non_md = sum(
        p.read_text(encoding="utf-8", errors="ignore").count(EM_DASH)
        for p in sorted(DOCS.rglob("*"))
        if p.is_file() and p.suffix in {".css", ".js"}
    )

    print(
        f"check_docs_em_dash: scanned {pages} Markdown page(s) under "
        f"{DOCS.relative_to(ROOT)}; skipped {skipped_in_code} occurrence(s) "
        f"inside code fences, inline code or HTML comments, none of which a "
        f"reader sees (--skipped lists them), and {non_md} in the "
        f"stylesheets and scripts, which are not reader-facing copy. Also read: "
        f"{MKDOCS.name}'s {', '.join(MKDOCS_READER_KEYS)}, which render on every "
        f"built page. NOT read: the built site, nor the two page families that "
        f"only exist in it, being reference/groups/ (the AGS4 dictionary's own "
        f"descriptions, which this may not rewrite) and the mkdocstrings API "
        f"pages (the wheel's docstrings, out of scope per #588)."
    )
    if args.skipped:
        for line in skipped_detail:
            print(f"  {line}")

    if failures:
        print(
            f"\ncheck_docs_em_dash: {len(failures)} em dash(es) in reader-facing "
            f"prose across {files_with_prose} page(s). Rewrite the sentence "
            f"rather than swapping the character — a comma, a colon, a full "
            f"stop or a pair of brackets each say something the dash was "
            f"standing in for:",
            file=sys.stderr,
        )
        for line in failures[:40]:
            print(line, file=sys.stderr)
        if len(failures) > 40:
            print(f"  … and {len(failures) - 40} more", file=sys.stderr)
        return 1

    print("check_docs_em_dash: OK — no em dash in reader-facing docs prose")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
