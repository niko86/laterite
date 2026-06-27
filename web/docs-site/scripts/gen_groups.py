"""Generate one reference page per AGS4 group from the shipped registry (#201).

Runs at `mkdocs build` (mkdocs-gen-files). The group + heading + KEY-chain data
comes from `laterite.registry` — the same projection the shipped wheel exposes,
so the catalogue inherits the JSON->registry faithfulness gate, and gets the
real KEY tuples (which are NOT just the dictionary's per-heading `status==KEY`:
e.g. PROJ_ID / DICT_TYPE / ERES_CODE are keys the raw status field omits).

WHY a subprocess: mkdocs-gen-files executes this file via `runpy.run_path`, and
in an editable `uv sync` install (CI) importing the compiled
`laterite._laterite_native` from that runpy context fails with a circular-import
error. A clean child `python -c` import (the same path mkdocstrings uses
successfully) sidesteps it. The wheel is server-side (built by
`uv sync --group docs`), so the no-Pyodide-wheel constraint is respected.

PROTOTYPE NOTE: the `code -> family` bucketing below is a *draft heuristic* — it
is the human-meaningful nav axis (the PROJ tree is too lopsided: 164/174 hang
off PROJ, LOCA has 50 children). The final mapping is owner-curated (#201 Q2).
"""

from __future__ import annotations

import json
import subprocess
import sys

import mkdocs_gen_files

# Dump the whole registry from a clean child interpreter (see module docstring).
_DUMP = r"""
import json
import laterite.registry as R
out = {}
for code in R.GROUPS:
    g = R.GROUPS[code]
    out[code] = {
        "contents": g.contents,
        "parent": g.parent,
        "headings": [
            {"name": h.name, "status": h.status, "type": h.type,
             "unit": h.unit, "description": h.description, "is_key": h.is_key}
            for h in g.headings
        ],
        "key_headings": [h.name for h in g.key_headings],
        "inherited": sorted(R.inherited_key_names(code)),
        "ancestors": R.ancestor_chain(code),
        "children": [{"code": k.code, "contents": k.contents}
                     for k in R.child_groups(code)],
    }
print(json.dumps(out))
"""

GROUPS: dict = json.loads(
    subprocess.run(
        [sys.executable, "-c", _DUMP],
        capture_output=True, text=True, check=True,
    ).stdout
)

# Metadata / housekeeping groups, mapped by code (no useful keyword).
_META = {"PROJ", "TRAN", "ABBR", "DICT", "FILE", "TYPE", "UNIT", "STND"}


def family(code: str, contents: str) -> str:
    """Draft heuristic bucketing (owner-curated later — #201 Q2)."""
    c = contents.lower()
    if code in _META:
        return "Project & housekeeping"
    if code == "LOCA" or "location" in c or "traverse" in c:
        return "Location & field"
    if "sampl" in c:
        return "Sampling"
    if any(k in c for k in (
        "drill", "flush", "casing", "chisel", "backfill", "core", "coring",
        "hole ", "water strike", "progress by", "advancement", "observation",
        "depth related", "diameter by depth", "orientation", "inclination",
        "construction", "custody", "schedule",
    )):
        return "Field, drilling & logging"
    if any(k in c for k in (
        "geolog", "stratum", "strata", "rock ", "weather", "discontinu",
        "fracture", "litholog",
    )):
        return "Geology & logging"
    if any(k in c for k in (
        "monitor", "installation", "piezomet", "groundwater", "standpipe",
        "gas", "instrument", "headspace", "ionisation", "contaminant",
    )):
        return "Monitoring & instrumentation"
    if any(k in c for k in (
        "penetration", "cone", "pressuremet", "vane", "plate", "dilatometer",
        "soakaway", "pumping", "geohydraulic", "point load", "schmidt",
        "scleroscope", "in situ", "in-situ", "dissipation", "field geo",
    )):
        return "In-situ testing"
    if any(k in c for k in (
        "laborator", "triaxial", "consolidat", "classif", "chemical",
        "compaction", "shrink", "swell", "cbr", "california bearing",
        "particle", "plasticity", "density", "moisture", "strength", "oedomet",
        "shear box", "resonant", "liquid and plastic", "suction", "mcv",
        "frost", "lime", "ten per cent", "aggregate", "abrasion", "crushing",
        "slake", "durability", "polished", "elongation", "flakiness", "impact",
        "soundness", "absorption", "chalk", "permeab",
    )):
        return "Laboratory testing"
    if "test" in c:
        return "Other testing"
    return "Other"


def _heading_rows(g: dict) -> str:
    rows = ["| Heading | Status | Type | Unit | Description |",
            "|---|:--|:--|:--|---|"]
    for h in g["headings"]:
        mark = " **(key)**" if h["is_key"] else ""
        rows.append(
            f"| `{h['name']}`{mark} | {h['status']} | `{h['type']}` | "
            f"{h['unit'] or ''} | {h['description']} |"
        )
    return "\n".join(rows)


nav = mkdocs_gen_files.Nav()
index_rows: list[tuple[str, str, str, str, int]] = []

for code in sorted(GROUPS):
    g = GROUPS[code]
    fam = family(code, g["contents"])
    index_rows.append((code, g["contents"], fam, g["parent"] or "—",
                       len(g["headings"])))
    nav[(fam, code)] = f"{code}.md"

    crumb = " → ".join(f"[{c}]({c}.md)" for c in reversed(g["ancestors"]))
    own = g["key_headings"]
    inh = g["inherited"]

    out = [f"# {code} — {g['contents']}", ""]
    out.append(f"**Path:** {crumb}  ·  **Family:** {fam}")
    out.append("")
    out.append('!!! note "Key chain"')
    own_s = "`, `".join(own) if own else "(none)"
    if inh:
        out.append(f"    Identified by `{own_s}`. Inherits "
                   f"`{'`, `'.join(inh)}` from `{g['parent']}`.")
    else:
        out.append(f"    Identified by `{own_s}`. Root-level under the project "
                   f"— inherits no parent key.")
    out.append("")
    out.append(_heading_rows(g))
    out.append("")

    kids = g["children"]
    if kids:
        out.append(f"## Child groups ({len(kids)})")
        out.append("")
        out.append('<div class="grid cards" markdown>')
        out.append("")
        for k in kids:
            out.append(f"- [`{k['code']}`]({k['code']}.md) — {k['contents']}")
        out.append("")
        out.append("</div>")
        out.append("")

    with mkdocs_gen_files.open(f"reference/groups/{code}.md", "w") as fd:
        fd.write("\n".join(out))

# --- landing: family cards (filter the table) + ONE paginated master table ---
# Pagination + filter + the family-card wiring are in docs/javascripts/catalogue.js
# (vanilla JS). Each card carries `data-family` so a click filters the table; the
# href degrades to a jump to the table if JS is off.
families = sorted({r[2] for r in index_rows})
land = ["# Group catalogue", "",
        f"All **{len(GROUPS)} AGS4 groups** — every group is one searchable, "
        "deep-linkable page. Pick a family to filter, type in the box, or page "
        "through the table (20 at a time). The left sidebar lists them by family "
        "too.", "",
        '<div class="grid cards" markdown>', ""]
for fam in families:
    n = sum(1 for r in index_rows if r[2] == fam)
    land.append(f'- [**{fam}** — {n} groups](#group-table){{ data-family="{fam}" }}')
land += ["", "</div>", "",
         '<div class="group-table" id="group-table" markdown>', "",
         "| Code | Group | Family | Parent | Headings |",
         "|---|---|---|:--|--:|"]
for code, contents, fam, parent, n in index_rows:
    land.append(f"| [`{code}`]({code}.md) | {contents} | {fam} | "
                f"{parent} | {n} |")
land += ["", "</div>"]
with mkdocs_gen_files.open("reference/groups/index.md", "w") as fd:
    fd.write("\n".join(land))

with mkdocs_gen_files.open("reference/groups/SUMMARY.md", "w") as fd:
    fd.write("- [Overview](index.md)\n")
    fd.writelines(nav.build_literate_nav())
