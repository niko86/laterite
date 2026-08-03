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

Modes:
  gen_observations.py            regenerate OBSERVATIONS.md from observations.json
  gen_observations.py --check    exit 1 if OBSERVATIONS.md is stale (CI drift gate)
  gen_observations.py --lint     report records that depart from the house style
  gen_observations.py --ingest   one-off: parse a hand-written OBSERVATIONS.md
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

    if args.lint:
        problems = lint(data)
        for p in problems:
            print(f"  {p}")
        print(
            f"gen_observations: {len(problems)} record(s) depart from the house style"
        )
        return

    out = render(data)
    if args.check:
        if MD_PATH.read_text() != out:
            sys.exit(
                "gen_observations: OBSERVATIONS.md is STALE — it is generated from "
                "observations.json.\nRun: uv run --no-sync python tools/gen_observations.py"
            )
        print("gen_observations: OBSERVATIONS.md is up to date")
        return

    MD_PATH.write_text(out)
    print(f"gen_observations: wrote {MD_PATH.name}")


if __name__ == "__main__":
    main()
