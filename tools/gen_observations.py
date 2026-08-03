#!/usr/bin/env python3
"""Generate `OBSERVATIONS.md` from `observations.json` — the O-N divergence
catalogue's single source of truth.

SSOT = `observations.json` (repo root). `OBSERVATIONS.md` is its rendered prose
view and is never hand-edited, the same shape as `gen_reference_groups` /
`gen_crate_graph` / `gen_changelog`.

WHY THIS EXISTS AT ALL. `CLAUDE.md` has instructed every contributor to "edit
observations.json … then regenerate with `uv run --no-sync python
tools/gen_observations.py`" for a long time, and three other generators cite the
pair as the pattern they mirror. Neither file existed. The catalogue was the one
large document in the repo described everywhere as generated and gated while
actually being hand-maintained with no gate at all — the exemplar was a phantom.

WHAT THE GATE BUYS. The `## Upstream-reportable` table is a rendering of the
records: one row per observation flagged `upstream`, carrying its kind and title.
Hand-maintained, it drifts from the records it summarises — an O-N can be
retitled, or reclassified BUG→SPEC, and the table keeps the old text. Rendered,
it cannot. That is the same defect class as the docs-site output blocks (#164),
fixed the same way.

WHAT THE MODEL DELIBERATELY DOES NOT DO. `CLAUDE.md` calls the record shape a
"5-field house style" (observed / spec / assessment / upstream-reportable / our
decision). Measured against the real catalogue, **47 of 50 records do not match
it**: labels vary (`python-ags4`, `Observed`, `Us`, `Evidence`, `Reality`), order
varies, and several carry state a fixed schema would destroy — `RESOLVED
(post-V8)`, `Us (before #422)` / `Us (after #422)`.

So a record's `body` is stored as verbatim Markdown rather than decomposed into
five keys. Normalising would have meant rewriting the majority of a CLEAN-ROOM
document to fit a shape it never had, losing meaning, in a change reviewable only
by hand. The structured part is what the rendering actually needs — `id`, `kind`,
`title`, `upstream` — and `--lint` reports records whose field labels depart from
the house style WITHOUT rewriting them, so the convention is enforced going
forward instead of retrofitted destructively.

THE WIKI'S SECOND COPY. Each O-N also has a page under `ags-wiki/observations/`
carrying the same classification in its frontmatter — `obs_tag` (the kind) and
`upstream_reportable`. Nothing had ever compared the two, and by the time this was
measured they had drifted: 12 pages read `upstream_reportable: false` while the
catalogue listed them as upstream-reportable, and O-46/O-47 had no page at all.
The prose on all 12 already said "[SPEC] — …"/"[BUG] — …", so only the machine-
readable flag was wrong — exactly the silent-disagreement failure a rendered table
prevents on the repo side, reappearing one directory over. `--check-wiki` closes
it: the SSOT is the catalogue, and a page may not disagree with it.

Modes:
  gen_observations.py              regenerate OBSERVATIONS.md from observations.json
  gen_observations.py --check      exit 1 if OBSERVATIONS.md is stale (CI drift gate)
  gen_observations.py --check-wiki exit 1 if an ags-wiki O-N page disagrees with
                                   the SSOT (missing/extra page, wrong obs_tag,
                                   wrong upstream_reportable, wrong anchor)
  gen_observations.py --lint       report records that depart from the house style
  gen_observations.py --ingest     one-off: parse a hand-written OBSERVATIONS.md
                                   into observations.json. Refuses to overwrite an
                                   existing SSOT.

Run: `uv run --no-sync python tools/gen_observations.py` (stdlib only).
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Iterator

ROOT = Path(__file__).resolve().parent.parent
JSON_PATH = ROOT / "observations.json"
MD_PATH = ROOT / "OBSERVATIONS.md"
WIKI_DIR = ROOT / "ags-wiki" / "observations"

#: The vault's by-tag / upstream-reportable synthesis. Its two lists are pure
#: derivations of the catalogue, so they are rendered into a marker-delimited
#: region (`gen_wiki_tables.py`'s convention) rather than hand-maintained — they
#: had frozen at "all 39 O-N entries" while the catalogue reached 50.
COVERAGE_MAP = ROOT / "ags-wiki" / "insights" / "observations-coverage-map.md"
COVERAGE_MARKER = "observations-coverage"

#: The house style CLAUDE.md documents. Used by --lint as a REPORT, never as a
#: schema — see the module docstring for why the catalogue is not normalised to it.
HOUSE_STYLE = ["Observed", "Spec", "Assessment", "Upstream-reportable", "Our decision"]

#: Labels that are established synonyms of a house-style field. `python-ags4` is
#: by far the most common form of "Observed" (34 of 50), because the observation
#: is nearly always about what python-ags4 does.
SYNONYMS = {
    "python-ags4": "Observed",
    "Us": "Observed",
    "Evidence": "Observed",
    "Context": "Observed",
    "Reality": "Observed",
    "Decision": "Our decision",
    "Plan": "Our decision",
}

RECORD_RE = re.compile(r"^### (O-\d+) \[([A-Z]+)\] (.*)$", re.M)
SECTION_RE = re.compile(r"^## (.*)$", re.M)

#: A record body's own verdict line, e.g. `- **Upstream-reportable**: **[SPEC]** — …`.
#: The bracket tag is the catalogue's own convention and tracks the boolean almost
#: perfectly: SPEC/BUG/VARIANCE (and a bare "Yes") are in the upstream table,
#: NO/NOTE/qualified prose are not. --lint reports where the two disagree.
UPSTREAM_LINE_RE = re.compile(
    r"^- \*\*Upstream[- ]reportable\*\*:?\s*\**\[?([A-Za-z]+)\]?", re.M | re.I
)
UPSTREAM_TAGS = {"SPEC", "BUG", "VARIANCE", "YES"}


# --------------------------------------------------------------------------- render


def render(data: dict) -> str:
    """observations.json -> the exact bytes of OBSERVATIONS.md."""
    out: list[str] = [f"# {data['title']}\n", data["preamble"]]

    up = data["upstream_section"]
    out.append(f"## {up['heading']}\n")
    out.append(up["intro"])
    out.append("| O-N | Kind | The case |\n|---|---|---|\n")
    out.extend(
        f"| {rec['id']} | {rec['kind']} | {rec['title']} |\n"
        for rec in _all_records(data)
        if rec.get("upstream")
    )
    out.append("\n")

    for sec in data["sections"]:
        out.append(f"## {sec['heading']}\n")
        out.append(sec["intro"])
        for rec in sec["observations"]:
            out.append(f"### {rec['id']} [{rec['kind']}] {rec['title']}\n")
            out.append(rec["body"])

    foot = data["footer"]
    out.append(f"## {foot['heading']}\n")
    out.append(foot["body"])
    return "".join(out)


def _all_records(data: dict) -> Iterator[dict]:
    for sec in data["sections"]:
        yield from sec["observations"]


# --------------------------------------------------------------------------- ingest


def ingest(md: str) -> dict:
    """Parse a hand-written OBSERVATIONS.md into the SSOT shape.

    One-off. Its correctness is proven by `render(ingest(md)) == md`, which the
    caller asserts — a lossy parse cannot pass that.
    """
    title_m = re.match(r"^# (.*)\n", md)
    assert title_m, "no H1 title"
    title = title_m.group(1)
    rest = md[title_m.end() :]

    heads = list(SECTION_RE.finditer(rest))
    assert heads, "no ## sections"
    preamble = rest[: heads[0].start()]

    blocks = []
    for i, h in enumerate(heads):
        end = heads[i + 1].start() if i + 1 < len(heads) else len(rest)
        blocks.append((h.group(1), rest[h.end() + 1 : end]))

    upstream_heading, upstream_body = blocks[0]
    # The table is regenerated from the records; keep only the prose above it.
    tbl = upstream_body.index("| O-N |")
    upstream_intro = upstream_body[:tbl]
    upstream_ids = set(re.findall(r"^\| (O-\d+) \|", upstream_body[tbl:], re.M))

    footer_heading, footer_body = blocks[-1]

    sections = []
    for heading, body in blocks[1:-1]:
        recs = list(RECORD_RE.finditer(body))
        intro = body[: recs[0].start()] if recs else body
        observations = []
        for j, m in enumerate(recs):
            rend = recs[j + 1].start() if j + 1 < len(recs) else len(body)
            observations.append(
                {
                    "id": m.group(1),
                    "kind": m.group(2),
                    "title": m.group(3),
                    "upstream": m.group(1) in upstream_ids,
                    "body": body[m.end() + 1 : rend],
                }
            )
        sections.append(
            {"heading": heading, "intro": intro, "observations": observations}
        )

    return {
        "title": title,
        "preamble": preamble,
        "upstream_section": {"heading": upstream_heading, "intro": upstream_intro},
        "sections": sections,
        "footer": {"heading": footer_heading, "body": footer_body},
    }


# ------------------------------------------------------------------- coverage map


def render_coverage(data: dict) -> str:
    """The coverage map's generated region: every O-N by tag, then the upstream set.

    Wikilinks use the vault's ZERO-PADDED page names (`[[O-01]]`), which is why
    this cannot simply echo the catalogue's ids.
    """
    by_tag: dict[str, list[str]] = {}
    upstream: list[str] = []
    for rec in _all_records(data):
        link = f"[[O-{int(rec['id'][2:]):02d}]]"
        by_tag.setdefault(rec["kind"], []).append(link)
        if rec.get("upstream"):
            upstream.append(link)

    # VARIANCE/SPEC/BUG first — the AGS-DFWG-relevant tags — then NOTE, then any
    # tag a later record introduces, so a new kind can't vanish from the page.
    order = ["VARIANCE", "SPEC", "BUG", "NOTE"]
    out = ["", "## By tag"]
    out.extend(
        f"- **{tag}** ({len(by_tag[tag])}): " + ", ".join(by_tag[tag])
        for tag in order + sorted(set(by_tag) - set(order))
        if tag in by_tag
    )
    out += [
        "",
        f"## Upstream-reportable ({len(upstream)})",
        ", ".join(upstream),
        "",
    ]
    return "\n".join(out)


def _splice(md: str, marker: str, block: str) -> str:
    """Replace the lines between the `<marker>` BEGIN/END comments."""
    begin, end = f"<!-- BEGIN GENERATED: {marker}", f"<!-- END GENERATED: {marker}"
    lines = md.split("\n")
    bi = next((i for i, ln in enumerate(lines) if ln.startswith(begin)), None)
    ei = next((i for i, ln in enumerate(lines) if ln.startswith(end)), None)
    if bi is None or ei is None:
        sys.exit(f"gen_observations: {marker!r} markers not found in {COVERAGE_MAP}")
    return "\n".join(lines[: bi + 1] + block.split("\n") + lines[ei:])


# ----------------------------------------------------------------------- check-wiki


def _frontmatter(txt: str) -> dict[str, str]:
    """The page's `---` block as flat `key: value` strings.

    Deliberately not a YAML parser (stdlib-only, same constraint as the wiki's own
    `lint.py`). The four keys this gate reads are all scalars written one per line,
    and `repo_refs.anchor` is picked up by its own indented `anchor:` key.
    """
    if not txt.startswith("---"):
        return {}
    end = txt.find("\n---", 3)
    block = txt[3:end] if end != -1 else txt[3:]
    out: dict[str, str] = {}
    for line in block.splitlines():
        m = re.match(r"^\s*([A-Za-z_]+):\s*(.*)$", line)
        if m:
            out[m.group(1)] = m.group(2).strip().strip('"')
    return out


def check_wiki(data: dict) -> list[str]:
    """Cross-check `ags-wiki/observations/O-NN.md` against the catalogue.

    The catalogue is the SSOT; a page may not disagree with it. Four ways it can:
    the page is missing (or names an O-N the catalogue doesn't have), its
    `observation_id` contradicts its own filename, its `obs_tag` contradicts the
    record's kind, or its `upstream_reportable` contradicts the record's flag —
    the last being the one that actually drifted, on 12 pages.

    Zero-padding is the wiki's filename convention (`O-01.md`) and the catalogue's
    ids are unpadded (`O-1`), so the two are mapped by integer, never by string.
    """
    problems: list[str] = []
    recs = {int(r["id"][2:]): r for r in _all_records(data)}
    pages = {
        int(p.stem[2:]): p for p in WIKI_DIR.glob("O-*.md") if p.stem[2:].isdigit()
    }

    problems.extend(
        f"O-{n}: in {JSON_PATH.name} but has no wiki page "
        f"({WIKI_DIR.relative_to(ROOT)}/O-{n:02d}.md) — copy templates/_template-observation.md"
        for n in sorted(set(recs) - set(pages))
    )
    problems.extend(
        f"{pages[n].name}: no O-{n} record in {JSON_PATH.name} — the page outlives its observation"
        for n in sorted(set(pages) - set(recs))
    )

    for n in sorted(set(recs) & set(pages)):
        rec, page = recs[n], pages[n]
        fm = _frontmatter(page.read_text(encoding="utf-8"))
        checks = [
            ("observation_id", fm.get("observation_id"), f"O-{n:02d}"),
            ("obs_tag", fm.get("obs_tag"), rec["kind"]),
            (
                "upstream_reportable",
                fm.get("upstream_reportable"),
                str(bool(rec["upstream"])).lower(),
            ),
            ("repo_refs.anchor", fm.get("anchor"), f"repo:OBSERVATIONS.md#o-{n}"),
        ]
        problems.extend(
            f"{page.name}: {key} is {got!r}, {JSON_PATH.name} says {want!r}"
            for key, got, want in checks
            if got != want
        )
    return problems


# --------------------------------------------------------------------------- lint


def lint(data: dict) -> list[str]:
    """Report records whose field labels depart from the house style.

    A report, not a gate: the catalogue predates the convention and normalising it
    would be a destructive rewrite (module docstring). New records should conform,
    and this is what shows whether they do.
    """
    problems = []
    seen: dict[str, str] = {}
    for rec in _all_records(data):
        if rec["id"] in seen:
            problems.append(f"{rec['id']}: duplicate id (also in {seen[rec['id']]})")
        seen[rec["id"]] = rec["id"]
        labels = re.findall(r"^- \*\*(.+?)\*\*", rec["body"], re.M)
        canon = [SYNONYMS.get(x, x) for x in labels]
        missing = [f for f in HOUSE_STYLE if f not in canon]
        if missing:
            problems.append(f"{rec['id']}: missing house-style field(s): {missing}")

        # The body's own bracket tag vs the boolean that renders the table. A
        # report and not a gate: the tag is a convention, and only the maintainer
        # can say whether a [VARIANCE] belongs in the actionable register.
        m = UPSTREAM_LINE_RE.search(rec["body"])
        tag = m.group(1).upper() if m else None
        if tag and (tag in UPSTREAM_TAGS) != bool(rec["upstream"]):
            problems.append(
                f"{rec['id']}: body says Upstream-reportable [{tag}] "
                f"but upstream={rec['upstream']} (the table follows the boolean)"
            )

    nums = sorted(int(r["id"][2:]) for r in _all_records(data))
    gaps = [n for n in range(1, nums[-1] + 1) if n not in nums]
    if gaps:
        problems.append(f"gaps in the O-N sequence: {gaps}")
    return problems


# --------------------------------------------------------------------------- main


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--check", action="store_true", help="fail if OBSERVATIONS.md is stale"
    )
    ap.add_argument(
        "--check-wiki",
        action="store_true",
        help="fail if an ags-wiki O-N page disagrees with the SSOT",
    )
    ap.add_argument("--lint", action="store_true", help="report house-style departures")
    ap.add_argument(
        "--ingest",
        action="store_true",
        help="one-off: build the SSOT from the Markdown",
    )
    args = ap.parse_args()

    if args.ingest:
        if JSON_PATH.exists():
            sys.exit(
                f"gen_observations: {JSON_PATH.name} already exists — refusing to overwrite the SSOT"
            )
        md = MD_PATH.read_text()
        data = ingest(md)
        if render(data) != md:
            sys.exit(
                "gen_observations: INGEST IS LOSSY — render(ingest(md)) != md; not writing"
            )
        JSON_PATH.write_text(json.dumps(data, indent=2, ensure_ascii=False) + "\n")
        n = sum(len(s["observations"]) for s in data["sections"])
        print(
            f"gen_observations: ingested {n} observations -> {JSON_PATH.name} (round-trip verified)"
        )
        return

    data = json.loads(JSON_PATH.read_text())

    if args.check_wiki:
        problems = check_wiki(data)
        for p in problems:
            print(f"  {p}")
        if problems:
            sys.exit(
                f"gen_observations: {len(problems)} wiki page(s) disagree with "
                f"{JSON_PATH.name}, which is the source of truth for the O-N catalogue."
            )
        n = sum(len(s["observations"]) for s in data["sections"])
        print(f"gen_observations: {n} wiki O-N page(s) agree with {JSON_PATH.name}")
        return

    if args.lint:
        problems = lint(data)
        for p in problems:
            print(f"  {p}")
        print(
            f"gen_observations: {len(problems)} record(s) depart from the house style"
        )
        return

    out = render(data)
    cov = _splice(
        COVERAGE_MAP.read_text(encoding="utf-8"),
        COVERAGE_MARKER,
        render_coverage(data),
    )
    if args.check:
        stale = [
            p.relative_to(ROOT)
            for p, want in ((MD_PATH, out), (COVERAGE_MAP, cov))
            if p.read_text(encoding="utf-8") != want
        ]
        if stale:
            sys.exit(
                f"gen_observations: STALE — {', '.join(map(str, stale))} "
                "generated from observations.json.\n"
                "Run: uv run --no-sync python tools/gen_observations.py"
            )
        print("gen_observations: OBSERVATIONS.md + the coverage map are up to date")
        return

    MD_PATH.write_text(out)
    COVERAGE_MAP.write_text(cov, encoding="utf-8")
    print(f"gen_observations: wrote {MD_PATH.name} + {COVERAGE_MAP.name}")


if __name__ == "__main__":
    main()
