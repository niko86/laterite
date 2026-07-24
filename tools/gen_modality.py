"""Render modality.json -> the ags-wiki modality-register page (the find-only
deliverable of the modality audit).

`modality.json` (repo root) is the SINGLE source of truth: every laterite
capability × surface × spelling, each I/O form tri-stated present|absent|divergent,
each absence/divergence verdicted gap|by-design with a reason + priority. This
script renders it to a human-readable per-capability matrix in
`ags-wiki/concepts/modality-register.md`.

There is deliberately NO faithfulness gate (unlike OBSERVATIONS.md): the page is
100%-derived tabular data with no migrated prose, so a `render == committed .md`
check would guard nothing that could meaningfully drift. The register's REALITY
is instead guarded by the standing gate `test_modality_parity.py`, which reflects
the live Python/Node/browser surfaces and fails when they diverge from the JSON.
The page is a generated artifact — regenerate it, don't hand-edit it.

The **sibling baseline** (which forms a capability is offered in *somewhere*) is
COMPUTED here, never stored in the JSON — a stored baseline is the
multi-source-of-truth class #181 exists to kill.

Usage:
    python tools/gen_modality.py            # render the wiki page
    python tools/gen_modality.py --summary  # print the P1/P2/P3 backlog census to stdout
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
JSON = REPO / "modality.json"
PAGE = REPO / "ags-wiki" / "concepts" / "modality-register.md"

# present / absent / divergent → cell glyph; priority → finding badge.
_GLYPH = {"present": "✓", "absent": "—", "divergent": "≈"}
_BADGE = {"P1": "🔴 P1", "P2": "🟠 P2", "P3": "🟡 P3", "by-design": "⚪ by-design"}

# The pages this register links out to (bidirectional citizens; keeps it off the
# orphan list and discoverable from the architecture spine).
_RELATED = [
    "crate-map",
    "parity-model",
    "pyo3-boundary",
    "tech-stack-wasm",
    "docs-site",
    # The sibling axis: this register asks whether a capability exists in a given
    # I/O *form*; the census asks whether it exists on that surface AT ALL.
    "surface-census",
    # A third, orthogonal axis this register deliberately excludes (see
    # excluded_axes): does a form that IS present agree on the ANSWER across
    # surfaces. cert-trust-v2 records the bug that gap let through and the
    # gate that now covers it.
    "cert-trust-v2",
    "start-here",
]

_FRONTMATTER = """\
---
type: concept
title: modality register
status: drafted
tags: [concept, architecture, api-parity]
ags_editions: []
repo_refs:
  register: "repo:modality.json"
  generator: "repo:tools/gen_modality.py"
  gate: "repo:packages/laterite/tests/test_modality_parity.py"
related: [{related}]
sources: []
---
"""


def _ordered(forms: list[str], present: dict[str, str]) -> list[str]:
    """Forms that appear in a cell, in the declared header order."""
    return [f for f in forms if f in present]


def _columns(cells: list[dict], direction: str, header: list[str]) -> list[str]:
    """The union of forms used across a capability's cells for one direction,
    in declared order — the column set for that direction's grid."""
    used: set[str] = set()
    for cell in cells:
        used |= set(cell.get(direction, {}))
    return [f for f in header if f in used]


def _baseline(cells: list[dict], direction: str) -> set[str]:
    """COMPUTED sibling baseline: the forms this capability is offered in
    *anywhere* (present or divergent on ≥1 surface) — the richest reachable set."""
    base: set[str] = set()
    for cell in cells:
        for form, state in cell.get(direction, {}).items():
            if state in ("present", "divergent"):
                base.add(form)
    return base


def _grid(cells: list[dict], direction: str, cols: list[str]) -> list[str]:
    if not cols:
        return []
    label = "input" if direction == "input" else "output"
    head = f"| surface (spelling) | {' | '.join(cols)} |"
    sep = "|" + "---|" * (len(cols) + 1)
    rows = [f"**{label.title()}**", "", head, sep]
    for cell in cells:
        forms = cell.get(direction, {})
        glyphs = " | ".join(_GLYPH.get(forms.get(f, ""), " ") for f in cols)
        rows.append(f"| {cell['surface']} ({cell.get('spelling', '?')}) | {glyphs} |")
    rows.append("")
    return rows


def _findings(cells: list[dict]) -> list[str]:
    out: list[str] = []
    for cell in cells:
        for g in cell.get("gaps", []):
            badge = _BADGE[g["priority"]]
            rel = f" `{g['related']}`" if g.get("related") else ""
            out.append(
                f"- {badge} · **{cell['surface']}** {g['direction']}.{g['form']}{rel} — {g['reason']}"
            )
    return out


def render(doc: dict) -> str:
    header_in = doc["forms"]["input"]
    header_out = doc["forms"]["output"]
    parts: list[str] = [
        _FRONTMATTER.format(related=", ".join(_RELATED)),
        "# modality register",
        "",
    ]
    # Say so, on the page. The byte-faithfulness gate (test_modality_faithful) already
    # catches a hand-edit, but only AFTER someone has made one — and the page carried
    # no sign it was generated, so an editor had no way to know. It has now been
    # hand-edited at least once. The prose lives in modality.json's `preamble`.
    parts += [
        "> **Generated** by `tools/gen_modality.py` from `modality.json` — do not hand-edit.",
        "> Change the SSOT (prose lives in its `preamble`) and re-run the generator.",
        "",
    ]
    parts += ["## Definition", "", doc["preamble"], ""]

    # Backlog census up top — the actionable half of a find-only audit.
    census: dict[str, list[str]] = {"P1": [], "P2": [], "P3": []}
    by_design = 0
    for cap in doc["capabilities"]:
        for cell in cap["cells"]:
            for g in cell.get("gaps", []):
                if g["priority"] == "by-design":
                    by_design += 1
                else:
                    census[g["priority"]].append(
                        f"{cap['capability']}/{cell['surface']} ({g['direction']}.{g['form']})"
                    )
    parts += ["## Findings backlog (find-only — fixes are follow-ups)", ""]
    for pri in ("P1", "P2", "P3"):
        items = census[pri]
        parts.append(
            f"- **{_BADGE[pri]}** ({len(items)}): "
            + ("; ".join(items) if items else "—")
        )
    parts.append(
        f"- **{_BADGE['by-design']}** ({by_design}): intentional absences, rationale in each cell below."
    )
    parts += ["", "## Excluded axes", ""]
    for ax in doc["excluded_axes"]:
        parts.append(f"- **{ax['axis']}** — {ax['reason']}")
    parts.append("")

    parts += ["## The register", ""]
    for cap in doc["capabilities"]:
        cells = cap["cells"]
        parts += [f"### {cap['capability']} — {cap['summary']}", ""]
        base_in = sorted(_baseline(cells, "input"))
        base_out = sorted(_baseline(cells, "output"))
        rich = []
        if base_in:
            rich.append("in: " + ", ".join(base_in))
        if base_out:
            rich.append("out: " + ", ".join(base_out))
        parts += [f"*Offered anywhere — {' · '.join(rich)}*", ""]
        parts += _grid(cells, "input", _columns(cells, "input", header_in))
        parts += _grid(cells, "output", _columns(cells, "output", header_out))
        findings = _findings(cells)
        if findings:
            parts += ["_Findings:_", *findings, ""]
        notes = [
            f"- _{cell['surface']}_: {cell['note']}"
            for cell in cells
            if cell.get("note")
        ]
        if notes:
            parts += ["_Notes:_", *notes, ""]

    parts += [
        "## Legend",
        "",
        "Grid: ✓ present · — absent · ≈ divergent (present but shape differs from siblings). "
        "Findings: 🔴 P1 · 🟠 P2 · 🟡 P3 · ⚪ by-design. Generated from [[crate-map|the crate map]]'s "
        "surfaces via `repo:tools/gen_modality.py`; the SSOT is `repo:modality.json` and the standing "
        "gate is `repo:packages/laterite/tests/test_modality_parity.py`.",
        "",
    ]
    return "\n".join(parts).rstrip() + "\n"


def summary(doc: dict) -> str:
    lines: list[str] = ["Modality register — backlog census"]
    census: dict[str, list[str]] = {"P1": [], "P2": [], "P3": [], "by-design": []}
    for cap in doc["capabilities"]:
        for cell in cap["cells"]:
            for g in cell.get("gaps", []):
                census[g["priority"]].append(
                    f"{cap['capability']}/{cell['surface']}: {g['direction']}.{g['form']} ({g.get('related', '-')})"
                )
    for pri in ("P1", "P2", "P3", "by-design"):
        lines.append(f"\n{pri} ({len(census[pri])}):")
        lines += [f"  - {x}" for x in census[pri]]
    return "\n".join(lines)


def main() -> None:
    doc = json.loads(JSON.read_text(encoding="utf-8"))
    if "--summary" in sys.argv:
        print(summary(doc))
        return
    PAGE.write_text(render(doc), encoding="utf-8")
    n_gaps = sum(
        len(c.get("gaps", [])) for cap in doc["capabilities"] for c in cap["cells"]
    )
    print(
        f"rendered {len(doc['capabilities'])} capabilities / {n_gaps} findings -> {PAGE.relative_to(REPO)}"
    )


if __name__ == "__main__":
    main()
