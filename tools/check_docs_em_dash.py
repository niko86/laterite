#!/usr/bin/env python3
"""No em dash reaches a docs-site reader (#588, the docs half of #587).

The em dash is out of reader-facing copy across this project's surfaces. The
landing page holds that with a browser test that walks the rendered DOM. This is
the docs-site equivalent, and it has two halves because the site has two kinds of
page and neither half can see the other's.

**`--built <site-dir>` is the gate**, and it runs in the `docs` job on what
`mkdocs build --strict` just produced, because that is the only artefact the
policy is actually about. A source scan cannot be it: three of this site's page
families have no `.md` at all. `reference/groups/` is 174 pages emitted by
`scripts/gen_groups.py`, `reference/types/` and `reference/cli/` are emitted by
their own `gen-files` scripts, and the prose in all of them lives in f-strings a
`docs/**.md` walk never opens. That is not hypothetical: the source-only draft of
this gate reported clean while several hundred sat in the built site, which is
exactly the cost #588 predicted when it asked for the built scan.

The default (no argument) walks the SOURCE instead: `docs/**.md` plus the
`mkdocs.yml` keys that render on every page. It runs in `repo-gates`, needs no
build, answers in under a second, and points at the file and line to edit rather
than at a rendered artefact nobody hand-edits.

The two halves are COMPLEMENTARY, and neither subsumes the other. The built half
misses what `BUILT_SKIP` excludes by path — and `reference/api.md` and
`reference/modules.md` are not wholly generated: each opens with a hand-written
intro and carries hand-written blurbs between its `:::` directives, so the source
half is the only gate those paragraphs have. The source half misses the three
generated families entirely. Two gates over one policy is the shape this repo
keeps getting burned by, so the division is written down rather than inferred,
and it is a real division rather than a fast copy of one scan by another.

## What it does not look at, and why that is written down

Every gate here carries a class of input it silently does not read, and an
unstated scope is a green tick over exactly the thing that later breaks
(CLAUDE.md, Conventions). So both halves report their exclusions on every run,
pass or fail, rather than only when unhappy:

* **Fenced code blocks and inline code spans** (source), **`<pre>` and `<code>`**
  (built). A fence reproduces what a tool actually printed, and an `.out` capture
  that says `—` says it because the tool did. Rewriting one would make the page a
  claim about output nobody can reproduce. Counted and reported, never edited.
* **HTML comments**, which mkdocs does not render. The `doc-code: skip — why`
  markers live in them and spell their reason after an em dash, and a policy
  about what a reader sees has nothing to say about text no reader sees.
* **Three built page families, by decision, listed in `BUILT_SKIP`** with the
  reason on each and the count printed every run. They are the pages whose prose
  belongs to something other than this site: the wheel's docstrings, and a guide
  that ships inside two binaries. Neither is a docs-site edit.
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
    uv run --no-project python tools/check_docs_em_dash.py --built web/docs-dist
"""

from __future__ import annotations

import argparse
import html
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

#: Built pages whose prose is not this site's to rewrite, each with the reason.
#: Keys are path prefixes relative to the site dir. Their em dashes are COUNTED
#: and printed on every run rather than passed over in silence, because a
#: declared exclusion nobody can see is the blind spot with a green tick on it.
#:
#: All three are the same judgement: the excluded text belongs to something the
#: docs site only DISPLAYS — the wheel's docstrings, or a guide that ships inside
#: two binaries. #588 draws that line explicitly, and it is the line that keeps
#: this gate from turning a docs task into an API or a shipped-binary change.
#:
#: Note what a path prefix cannot express: two of these pages are a MIX, part
#: generated and part hand-written, and excluding the path excludes both halves
#: here. The hand-written halves are gated all the same, by the source scan
#: reading their `.md` — which is why these two halves are complementary rather
#: than one being a faster copy of the other.
BUILT_SKIP = {
    "reference/api/": (
        "MOSTLY mkdocstrings rendering the shipped wheel's own docstrings, which "
        "#588 puts out of scope because the fix would be an API-surface edit. "
        "The page's own hand-written intro and section prose are NOT excluded: "
        "they live in docs/reference/api.md, which the source half reads"
    ),
    "reference/modules/": (
        "MOSTLY mkdocstrings rendering the shipped wheel's own docstrings, which "
        "#588 puts out of scope because the fix would be an API-surface edit. "
        "The page's own hand-written intro and per-module blurbs are NOT "
        "excluded: they live in docs/reference/modules.md, which the source "
        "half reads"
    ),
    # NOT granted by #588, and saying so is the point. That issue enumerates its
    # carve-outs exhaustively (the stylesheets, the scripts, and the shipped
    # package's own docstrings) and never mentions the CLI guide; its in-scope
    # measurement counted `docs/**.md` only, which this page is not. So this
    # exclusion is a scope call made HERE, and it leaves an acceptance criterion
    # ("no U+2014 in rendered docs prose") unmet on one page rather than met.
    # #588's own rule for a case like this is that it is worth its own ticket,
    # which is #681: either the guide is rewritten and this entry goes, or the
    # policy is declared not to cover terminal output and this reason is
    # rewritten to say so. Until one of those, the count below is not a zero.
    "reference/cli/": (
        "EXCLUDED BY A CALL MADE HERE, NOT BY #588. The page is the shipped "
        "`lat --readme` guide: one authority (rust-packages/laterite-cli/"
        "README-cli.md) `include_str!`d into the binary, plus two mirrors held "
        "byte-identical to it by tools/gen_cli_readme.py, so rewriting it "
        "changes what a shipped program prints rather than what this site says. "
        "That is a shipped-content change, and #681 is where it gets decided; "
        "until then these are known-unfixed, not known-absent. The count is "
        "PROSE only, and smaller than it looks: the guide's `lat --help` blocks "
        "are skipped as code like anywhere else, and one offending heading is "
        "counted once per place the theme renders it"
    ),
}

#: What this half still cannot see, printed with the exclusions rather than left
#: for someone to discover. Neither is hypothetical: the first is a string this
#: repo owns, and the second is how `site_description` escaped the source scan.
BUILT_BLIND_SPOTS = (
    "reference/cli/ also carries the note scripts/gen_cli.py writes above the "
    "shipped guide, which IS ours; the exclusion is by path, so that note is "
    "unread by this half, and it has no `.md` for the other half to read",
    "attribute text other than the meta description — an `alt` or a `title` is "
    "read aloud or shown on hover, but tag-stripping drops both",
    "search/search_index.json, which feeds the search dropdown's snippets: it is "
    "not an HTML page, so this walk never opens it",
)

#: Everything between these tags is what a tool printed, not what an author
#: wrote — the built-site counterpart of the source scan's fences. Anchored with
#: a backreference so `<pre><code>…</code></pre>` is consumed once, by `pre`.
HTML_CODE_RE = re.compile(r"<(script|style|pre|code)\b[^>]*>.*?</\1\s*>", re.S | re.I)
HTML_TAG_RE = re.compile(r"<[^>]+>")
#: `site_description` reaches a reader as an ATTRIBUTE, never as a text node, so
#: stripping tags would drop the single string that renders on every page. The
#: tag is parsed into attributes rather than matched by one pattern, so this is
#: pinned to neither the order the theme writes them in nor whether it quotes
#: them — both are details of whoever wrote the template.
META_TAG_RE = re.compile(r"<meta\b[^>]*>", re.I)
ATTR_RE = re.compile(r"""\b([\w-]+)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))""")

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
    #: Which token opened the fence we are inside, or None. Tracked rather than
    #: toggled on any fence-shaped line, because ``` and ~~~ do not close each
    #: other: a `~~~` printed inside a ``` block is content, and treating it as
    #: a close would end the fence early and report the code after it as prose.
    fence: str | None = None
    # Comments are blanked across the WHOLE text first, so one spanning several
    # lines needs no second state machine beside the fence one. Newlines are
    # kept, so line numbers still point where they should. EVERY decision below
    # reads the masked copy: testing the raw line for a fence while testing the
    # masked one for prose let a fence marker inside a comment open a block that
    # was never open, and the real prose after it was then passed AND counted as
    # skipped, so the gate under-reported and lied about why.
    masked = HTML_COMMENT_RE.sub(lambda m: re.sub(r"[^\n]", " ", m.group(0)), text)
    for n, (line, clean) in enumerate(
        zip(text.split("\n"), masked.split("\n"), strict=True), start=1
    ):
        if m := FENCE_RE.match(clean):
            token = m.group(1)
            if fence is None:
                fence = token
            elif token == fence:
                fence = None
            skipped += line.count(EM_DASH)
            continue
        if fence is not None:
            skipped += line.count(EM_DASH)
            continue
        prose = INLINE_CODE_RE.sub("", clean)
        skipped += line.count(EM_DASH) - prose.count(EM_DASH)
        if EM_DASH in prose:
            hits.append((n, line.strip()))
    return hits, skipped


def scan_config(text: str) -> list[tuple[int, str, str]]:
    """Reader-facing `mkdocs.yml` values carrying an em dash: (line, key, value).

    Separate from `scan` and separately testable, because this half is the one
    the source-only gate was missing: `site_description` and `copyright` are one
    string each, in a file a `docs/**.md` walk has no reason to open, and they
    render on every built page.

    Read as text rather than as YAML so this stays stdlib-only and runs in the
    buildless job. A value that wraps across lines is followed until the
    indentation ends, which is how `site_description` is written.
    """
    out: list[tuple[int, str, str]] = []
    lines = text.split("\n")
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
        value = " ".join(" ".join(block).split())
        if EM_DASH in value:
            out.append((n + 1, key, value))
    return out


def _attrs(tag: str) -> dict[str, str]:
    """One HTML tag's attributes, lowercased names, quoted or not."""
    return {
        m.group(1).lower(): next(g for g in m.group(2, 3, 4) if g is not None)
        for m in ATTR_RE.finditer(tag)
    }


def scan_html(text: str) -> tuple[list[str], int]:
    """Reader-visible em dashes in one BUILT page, and how many were skipped.

    Excerpts come back rather than line numbers. A line number in generated HTML
    points at a file nobody edits and a column nobody can find; the sentence
    itself is what someone greps for to reach the f-string that produced it.

    Order matters here. Comments and code are removed while the markup is still
    intact, because that is the only point at which their boundaries exist —
    after tags are stripped, a fence is indistinguishable from a paragraph.
    """
    skipped = 0

    def _drop(m: re.Match[str]) -> str:
        nonlocal skipped
        # Unescaped before counting, or a `&mdash;` inside a fence is dropped
        # from the skip total and the reported scope understates itself — the
        # one number this gate exists to keep honest.
        matched = m.group(0)
        skipped += html.unescape(matched).count(EM_DASH)
        # Code leaves a mark and comments do not. The excerpt is the only
        # locator this half can give, and a sentence with its code spans
        # silently deleted ("Stack / — nothing runs") is one nobody can grep
        # back to the f-string that produced it.
        return " " if matched.startswith("<!--") else "`…`"

    body = HTML_CODE_RE.sub(_drop, HTML_COMMENT_RE.sub(_drop, text))
    # Pulled out before the tags go, because this one is an ATTRIBUTE: it is the
    # search snippet and the link preview, so it is read far more often than the
    # page, and stripping tags would silently drop it.
    meta = " ".join(
        html.unescape(attrs["content"])
        for tag in META_TAG_RE.findall(body)
        if (attrs := _attrs(tag)).get("name") == "description" and "content" in attrs
    )
    prose = f"{html.unescape(HTML_TAG_RE.sub(' ', body))} {meta}"

    excerpts: list[str] = []
    for m in re.finditer(EM_DASH, prose):
        window = prose[max(0, m.start() - 45) : m.start() + 45]
        excerpts.append(" ".join(window.split()))
    return excerpts, skipped


#: How many failures to print before summarising the rest. Shared by both halves
#: so one truncation policy cannot become two that disagree about what was hidden.
FAILURE_CAP = 40


def report_failures(failures: list[str], headline: str) -> int:
    """Print a capped failure list, and say how many the cap hid. Always 1."""
    print(f"\n{headline}", file=sys.stderr)
    for line in failures[:FAILURE_CAP]:
        print(line, file=sys.stderr)
    if len(failures) > FAILURE_CAP:
        print(f"  … and {len(failures) - FAILURE_CAP} more", file=sys.stderr)
    return 1


def check_built(site: Path, show_skipped: bool) -> int:
    """The gate proper: what `mkdocs build` actually produced (#588).

    This is the half that can see the 174 group pages, the type catalogue and
    the CLI reference, none of which have a `.md` for the source scan to open.
    """
    pages = [p for p in sorted(site.rglob("*.html")) if p.is_file()]
    if not pages:
        print(
            f"check_docs_em_dash: no HTML under {site} — the site did not build, "
            f"and a scan of nothing is not a pass",
            file=sys.stderr,
        )
        return 1

    failures: list[str] = []
    scanned = skipped_in_code = 0
    excluded: dict[str, int] = dict.fromkeys(BUILT_SKIP, 0)
    skipped_detail: list[str] = []

    for path in pages:
        rel = path.relative_to(site).as_posix()
        excerpts, skipped = scan_html(path.read_text(encoding="utf-8", errors="ignore"))
        if prefix := next((p for p in BUILT_SKIP if rel.startswith(p)), None):
            excluded[prefix] += len(excerpts)
            continue
        scanned += 1
        skipped_in_code += skipped
        if skipped and show_skipped:
            skipped_detail.append(f"{rel}: {skipped} inside code")
        failures.extend(f"  {rel}\n      … {excerpt} …" for excerpt in excerpts)

    print(
        f"check_docs_em_dash --built: read {scanned} built page(s) under "
        f"{site}, everything a reader sees INCLUDING the pages that have no "
        f"Markdown source. Skipped {skipped_in_code} occurrence(s) inside "
        f"<pre>, <code> and HTML comments, none of which is authored prose "
        f"(--skipped lists them). Not read, by decision:"
    )
    for prefix, reason in BUILT_SKIP.items():
        print(f"  {prefix} — {excluded[prefix]} occurrence(s): {reason}")
    for spot in BUILT_BLIND_SPOTS:
        print(f"  still unread: {spot}")
    if show_skipped:
        for line in skipped_detail:
            print(f"  {line}")

    if failures:
        return report_failures(
            failures,
            f"check_docs_em_dash --built: {len(failures)} em dash(es) reached a "
            f"reader. Fix the SOURCE, which for a page with no `.md` is the "
            f"generator that emits it (web/docs-site/scripts/), then rebuild:",
        )

    print("check_docs_em_dash --built: OK, no em dash reached a docs-site reader")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument(
        "--skipped",
        action="store_true",
        help="list every occurrence this gate deliberately does not read",
    )
    ap.add_argument(
        "--built",
        metavar="SITE_DIR",
        type=Path,
        help="scan a built site instead of the source — this is the gate (#588)",
    )
    args = ap.parse_args()

    if args.built:
        if not args.built.is_dir():
            print(
                f"check_docs_em_dash: {args.built} is not a directory — run "
                f"`mkdocs build` first",
                file=sys.stderr,
            )
            return 1
        return check_built(args.built, args.skipped)

    if not DOCS.is_dir():
        print(f"check_docs_em_dash: {DOCS} is not a directory", file=sys.stderr)
        return 1

    failures: list[str] = []
    pages = files_with_prose = skipped_in_code = 0
    skipped_detail: list[str] = []

    for n, key, value in scan_config(MKDOCS.read_text(encoding="utf-8")):
        where = (
            'the <meta name="description">, so it is the search snippet and the '
            "link preview"
            if key == "site_description"
            else "in the page footer"
        )
        failures.append(
            f"  {MKDOCS.name}:{n}  {key}: {value[:100]}"
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
        f"built page. NOT read: the built site, and therefore none of the pages "
        f"that exist only in it (reference/groups/, reference/types/ and "
        f"reference/cli/ are emitted by web/docs-site/scripts/ and have no `.md` "
        f"to open), nor anything the theme injects. `--built` is the half that "
        f"reads those; this one is the fast pre-check that names a line to edit."
    )
    if args.skipped:
        for line in skipped_detail:
            print(f"  {line}")

    if failures:
        return report_failures(
            failures,
            f"check_docs_em_dash: {len(failures)} em dash(es) in reader-facing "
            f"prose across {files_with_prose} page(s). Rewrite the sentence "
            f"rather than swapping the character — a comma, a colon, a full "
            f"stop or a pair of brackets each say something the dash was "
            f"standing in for:",
        )

    print("check_docs_em_dash: OK — no em dash in reader-facing docs prose")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
