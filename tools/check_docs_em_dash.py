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
* **The built site.** This reads `.md`, so a dash introduced by the theme, a
  template override or a plugin is outside it. Named because "no em dash
  reaches the reader" is the policy and this checks a proxy for it.
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

EM_DASH = "—"
FENCE_RE = re.compile(r"^\s*(```|~~~)")
INLINE_CODE_RE = re.compile(r"`[^`]*`")

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
    for n, line in enumerate(text.split("\n"), start=1):
        if FENCE_RE.match(line):
            in_fence = not in_fence
            skipped += line.count(EM_DASH)
            continue
        if in_fence:
            skipped += line.count(EM_DASH)
            continue
        prose = INLINE_CODE_RE.sub("", line)
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
        f"inside code fences or inline code, where the character reproduces "
        f"what a tool printed (--skipped lists them), and {non_md} in the "
        f"stylesheets and scripts, which are not reader-facing copy. The BUILT "
        f"site is not read: this checks the source a fix goes into."
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
