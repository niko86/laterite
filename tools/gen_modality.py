"""Render modality.json -> the ags-wiki modality-register page (the find-only
deliverable of the modality audit).

`modality.json` (repo root) is the SINGLE source of truth: every laterite
capability × surface × spelling, each I/O form tri-stated present|absent|divergent,
each absence/divergence verdicted gap|by-design with a reason + priority. This
script renders it to a human-readable per-capability matrix in
`ags-wiki/concepts/modality-register.md`.

Three distinct risks, three answers. The middle one went unguarded until
2026-08-04, because the first one's argument was read as covering it:

1. A regen CLOBBERING hand-written text. Not possible here — the page is
   100%-derived tabular data with no migrated prose, unlike OBSERVATIONS.md,
   which was ingested from Markdown and can lose text to a bad render. This is
   why the page shipped without a faithfulness gate, and the reasoning holds.
2. The page going STALE — a `modality.json` edit landing without a regen. (1)
   does not cover this: "a regen destroys nothing" is not "the committed page
   still matches". Nothing failed, and a stale page misinforms every human who
   reads it, including the facade-floor census. `--check` is that gate.
3. `modality.json` diverging from the surfaces it describes. Guarded by the
   standing `test_modality_parity.py`, which reflects the live
   Python/Node/browser/Rust surfaces and fails when they disagree with the JSON.

The page is a generated artifact — regenerate it, don't hand-edit it.

The **sibling baseline** (which forms a capability is offered in *somewhere*) is
COMPUTED here, never stored in the JSON — a stored baseline is the
multi-source-of-truth class #181 exists to kill. The **facade floor** (what the
Rust crate owes, being the surface that drives python and node) is computed the
same way and for the same reason — see `_floor`. It is a minimum, not a gate.

Usage:
    python tools/gen_modality.py            # render the wiki page
    python tools/gen_modality.py --summary  # P1/P2/P3 census + the facade floor
    python tools/gen_modality.py --check    # fail if the rendered page is stale
"""

from __future__ import annotations

import argparse
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


# The Rust facade's baseline, owner-set 2026-08-04. The engine DRIVES python and
# node, so the crate that exposes it directly should offer at least what the
# weaker of its two dependents offers — the INTERSECTION of the forms python and
# node both provide, per capability.
#
# It is a FLOOR, not a rule and not a gate. Nothing fails CI on it, and "the
# floor does not owe this" is never a reason to leave something out: if a door is
# cheap, open it. What the floor is for is the opposite failure — a capability
# quietly missing on the Rust surface for years because no artefact ever said it
# was missing. Read it as a minimum to clear, not a ceiling to stop at.
#
# Computed here, never stored, for the same reason as `_baseline`: a floor
# written into the JSON is a second source of truth that goes stale the moment a
# door opens on either sibling. Two consequences fall out of stating it
# mechanically rather than deciding each case by hand:
#
#   * a capability only ONE sibling has (read_typed, read-output-view: python
#     only) sets no floor — the intersection is empty, so the facade owes
#     nothing. It may still choose to offer it.
#   * a form only one sibling offers (file-like on python but not node) is not
#     owed either. The floor is the LEAST of the two, not the union — and
#     `read`'s file-like entry is recorded as a cheap thing to add anyway,
#     precisely so the floor is not mistaken for a cap.
_FLOOR_SURFACES = ("python", "node")
_FACADE = "rust"


def _floor(cells: list[dict], direction: str) -> set[str]:
    """The facade floor: forms offered by EVERY surface in `_FLOOR_SURFACES`."""
    sets: list[set[str]] = []
    for surface in _FLOOR_SURFACES:
        cell = next((c for c in cells if c["surface"] == surface), None)
        if cell is None:
            return set()  # a sibling lacks the capability entirely → no floor
        sets.append(
            {
                f
                for f, s in cell.get(direction, {}).items()
                if s in ("present", "divergent")
            }
        )
    return set.intersection(*sets) if sets else set()


def _facade_debt(cells: list[dict]) -> tuple[set[str], set[str]]:
    """(in, out) forms the floor requires that the facade does not offer."""
    cell = next((c for c in cells if c["surface"] == _FACADE), None)
    have = {
        d: {
            f
            for f, s in (cell.get(d, {}) if cell else {}).items()
            if s in ("present", "divergent")
        }
        for d in ("input", "output")
    }
    return (
        _floor(cells, "input") - have["input"],
        _floor(cells, "output") - have["output"],
    )


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
        debt_in, debt_out = _facade_debt(cells)
        if debt_in or debt_out:
            owed = []
            if debt_in:
                owed.append("in: " + ", ".join(sorted(debt_in)))
            if debt_out:
                owed.append("out: " + ", ".join(sorted(debt_out)))
            parts += [
                f"*Below the facade floor — the Rust crate does not yet offer "
                f"{' · '.join(owed)}, which python and node both do. A minimum to "
                f"clear, not a gate*",
                "",
            ]
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
        "surfaces via `repo:tools/gen_modality.py`; the SSOT is `repo:modality.json`. Two standing "
        "gates: `repo:packages/laterite/tests/test_modality_parity.py` holds the SSOT against the "
        "live surfaces, and `gen_modality.py --check` holds this page against the SSOT.",
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

    # The facade floor, as a number you can watch fall. `test_version_faithful`
    # holds the Rust crate outside both version tiers "until it reaches feature
    # parity with the Python and Node surfaces" — this is that trigger made
    # checkable instead of aspirational.
    # Split the shortfall by what was DECIDED about it. The floor itself stays
    # purely mechanical (python n node) — a human override never moves it, or it
    # stops measuring anything. What the verdict does is separate "still to do"
    # from "deliberately not doing", so the number can actually reach zero.
    buckets: dict[str, list[str]] = {"planned": [], "by-design": [], "undecided": []}
    clear = 0
    for cap in doc["capabilities"]:
        d_in, d_out = _facade_debt(cap["cells"])
        if not (d_in or d_out):
            if _floor(cap["cells"], "input") or _floor(cap["cells"], "output"):
                clear += 1
            continue
        bits = []
        if d_in:
            bits.append("in: " + ", ".join(sorted(d_in)))
        if d_out:
            bits.append("out: " + ", ".join(sorted(d_out)))
        cell = next((c for c in cap["cells"] if c["surface"] == _FACADE), None)
        verdict = (cell or {}).get("facade_verdict", "undecided")
        buckets.setdefault(verdict, []).append(
            f"  - {cap['capability']}: {' · '.join(bits)}"
        )
    short = sum(len(v) for v in buckets.values())
    lines += [
        "",
        f"Facade floor (rust >= python n node, owner-set 2026-08-04; a minimum, "
        f"not a gate) — {clear} clear, {short} below:",
    ]
    for label, key in (
        ("to add", "planned"),
        ("deliberately not adding", "by-design"),
        ("UNDECIDED", "undecided"),
    ):
        rows = buckets.get(key) or []
        lines.append(f"\n  {label} ({len(rows)}):")
        lines += rows
    return "\n".join(lines)


def uncovered(doc: dict) -> list[str]:
    """(capability, surface) pairs with NO cell at all — the register's blind spot.

    Every guard over this register reads the cells that exist. `--check` holds the
    page against the JSON, and a missing cell renders faithfully as nothing, so
    both agree while both omit the same row; `test_register_well_formed` iterates
    `cap["cells"]`, so an absent one is never visited. Ten cells were missing when
    #771 found them by reading, not by any run going red.

    This is still the REPORT — it prints on every run, pass or fail. It is no
    longer the only thing watching: #779 recorded the last twelve verdicts and
    landed the assertion, which reads THIS function rather than restating the
    predicate, so the count printed here and the pair that fails CI can never
    disagree. See `test_every_capability_has_a_cell_for_every_surface`.
    """
    surfaces = [s["id"] for s in doc["surfaces"]]
    out: list[str] = []
    for cap in doc["capabilities"]:
        have = {c["surface"] for c in cap["cells"]}
        out.extend(f"{cap['capability']}/{sid}" for sid in surfaces if sid not in have)
    return out


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--summary", action="store_true", help="P1/P2/P3 census + the facade floor"
    )
    ap.add_argument(
        "--check", action="store_true", help=f"fail if {PAGE.name} is stale"
    )
    args = ap.parse_args()

    doc = json.loads(JSON.read_text(encoding="utf-8"))
    if args.summary:
        print(summary(doc))
        return

    out = render(doc)
    if args.check:
        if PAGE.read_text(encoding="utf-8") != out:
            sys.exit(
                f"gen_modality: {PAGE.relative_to(REPO)} is stale — {JSON.name} is "
                f"the source of truth. Regenerate with "
                f"`uv run --no-sync python tools/gen_modality.py`."
            )
        print(f"gen_modality: {PAGE.relative_to(REPO)} matches {JSON.name}")
        _report_uncovered(doc)
        return

    PAGE.write_text(out, encoding="utf-8")
    n_gaps = sum(
        len(c.get("gaps", [])) for cap in doc["capabilities"] for c in cap["cells"]
    )
    print(
        f"rendered {len(doc['capabilities'])} capabilities / {n_gaps} findings -> {PAGE.relative_to(REPO)}"
    )
    _report_uncovered(doc)


def _report_uncovered(doc: dict) -> None:
    """Say what this generator does not look at, on every run, pass or fail."""
    missing = uncovered(doc)
    total = len(doc["capabilities"]) * len(doc["surfaces"])
    if not missing:
        print(f"gen_modality: all {total} (capability, surface) pairs carry a cell")
        return
    print(
        f"gen_modality: {len(missing)} of {total} (capability, surface) pairs have NO "
        f"cell, so nothing here or in test_register_well_formed reads them: "
        + ", ".join(missing)
    )


if __name__ == "__main__":
    main()
