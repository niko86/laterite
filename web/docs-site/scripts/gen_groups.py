"""Generate one reference page per AGS4 group from the shipped registry (#201).

Runs at `mkdocs build` (mkdocs-gen-files). Group + heading + KEY-chain data come
from `laterite.registry` — the same projection the shipped wheel exposes, so the
catalogue inherits the JSON->registry faithfulness gate, and gets the real KEY
tuples (which are NOT just the dictionary's per-heading `status==KEY`: e.g.
PROJ_ID / DICT_TYPE / ERES_CODE are keys the raw status field omits).

Edition PROVENANCE (added / removed in 4.x) and the FAMILY taxonomy + the type
glossary links come from `catalogue_data` (which reads the single-source union
dictionary directly — the registry serves only the latest-edition union and
drops per-edition membership). See that module for the data contract.

WHY a subprocess: mkdocs-gen-files executes this file via `runpy.run_path`, and
in an editable `uv sync` install (CI) importing the compiled
`laterite._laterite_native` from that runpy context fails with a circular-import
error. A clean child `python -c` import (the same path mkdocstrings uses
successfully) sidesteps it. The wheel is server-side (built by
`uv sync --group docs`), so the no-Pyodide-wheel constraint is respected.
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

import mkdocs_gen_files

# `catalogue_data` sits beside this script; runpy.run_path does not put the
# script's dir on sys.path for a plain file, so add it explicitly.
sys.path.insert(0, str(Path(__file__).resolve().parent))
import catalogue_data as cd

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
        capture_output=True,
        text=True,
        check=True,
    ).stdout
)

_DICT = cd.load_dict()
_ALL_EDS = cd.editions()


def _type_cell(type_code: str) -> str:
    """A type code linked to its glossary entry (folds 2DP->#ndp, etc.)."""
    key = cd.glossary_key(type_code)
    code = f"`{type_code}`"
    return f"[{code}](../types.md#{key})" if key else code


def _heading_rows(code: str, g: dict) -> str:
    """Heading table; gains an 'Editions' column only when some heading's edition
    span differs from its group's (a heading added/removed mid-life)."""
    jgroup = _DICT["groups"].get(code, {})
    group_eds = jgroup.get("eds", _ALL_EDS)
    jh_by_name = {h["name"]: h for h in jgroup.get("headings", [])}

    provs = {}
    for h in g["headings"]:
        jh = jh_by_name.get(h["name"])
        provs[h["name"]] = (
            cd.heading_provenance(group_eds, jh, _ALL_EDS) if jh else {"differs": False}
        )
    show_eds = any(p.get("differs") for p in provs.values())

    head = "| Heading | Status | Type | Unit | Description |"
    sep = "|---|:--|:--|:--|---|"
    if show_eds:
        head = "| Heading | Status | Type | Unit | Editions | Description |"
        sep = "|---|:--|:--|:--|:--|---|"
    rows = [head, sep]
    for h in g["headings"]:
        mark = " **(key)**" if h["is_key"] else ""
        cells = [
            f"`{h['name']}`{mark}",
            h["status"],
            _type_cell(h["type"]),
            h["unit"] or "",
        ]
        if show_eds:
            p = provs[h["name"]]
            if p.get("added_in"):
                cells.append(f"since {p['added_in']}")
            elif p.get("removed_in"):
                cells.append(f"removed {p['removed_in']}")
            else:
                cells.append("")
        cells.append(h["description"])
        rows.append("| " + " | ".join(cells) + " |")
    return "\n".join(rows)


nav = mkdocs_gen_files.Nav()
# (code, contents, family, parent, n_headings, edition_label)
index_rows: list[tuple[str, str, str, str, int, str]] = []

for code in sorted(GROUPS):
    g = GROUPS[code]
    fam = cd.family(code)
    prov = cd.group_provenance(code)
    edlabel = cd.edition_label(prov)
    index_rows.append(
        (code, g["contents"], fam, g["parent"] or "—", len(g["headings"]), edlabel)
    )

    crumb = " → ".join(f"[{c}]({c}.md)" for c in reversed(g["ancestors"]))
    own = g["key_headings"]
    inh = g["inherited"]

    out = [f"# {code} — {g['contents']}", ""]
    out.append(f"**Path:** {crumb}  ·  **Family:** {fam}")
    out.append("")
    out.append(f"**Editions:** {edlabel}")
    out.append("")
    if prov["added_in"]:
        out.append(f'!!! info "New in AGS {prov["added_in"]}"')
        out.append(f"    This group was introduced in AGS edition {prov['added_in']}.")
        out.append("")
    elif prov["removed_in"]:
        out.append(f'!!! warning "Removed in AGS {prov["removed_in"]}"')
        out.append(
            f"    This group was defined up to AGS {prov['span'][1]} and "
            f"removed in {prov['removed_in']}."
        )
        out.append("")
    out.append('!!! note "Key chain"')
    own_s = "`, `".join(own) if own else "(none)"
    if inh:
        out.append(
            f"    Identified by `{own_s}`. Inherits "
            f"`{'`, `'.join(inh)}` from `{g['parent']}`."
        )
    else:
        out.append(
            f"    Identified by `{own_s}`. Root-level under the project "
            f"— inherits no parent key."
        )
    out.append("")
    out.append(_heading_rows(code, g))
    out.append("")

    kids = g["children"]
    if kids:
        out.append(f"## Child groups ({len(kids)})")
        out.append("")
        out.append('<div class="grid cards" markdown>')
        out.append("")
        out.extend(f"- [`{k['code']}`]({k['code']}.md) — {k['contents']}" for k in kids)
        out.append("")
        out.append("</div>")
        out.append("")

    with mkdocs_gen_files.open(f"reference/groups/{code}.md", "w") as fd:
        fd.write("\n".join(out))

# --- landing: family cards (filter the table) + ONE paginated master table ---
# Pagination + filter + the family-card wiring are in docs/javascripts/catalogue.js
# (vanilla JS). Each card carries `data-family` so a click filters the table; the
# href degrades to a jump to the table if JS is off. Families render in the
# curated order from catalogue_data (not alphabetical).
land = [
    "# Group catalogue",
    "",
    f"All **{len(GROUPS)} AGS4 groups** — every group is one searchable, "
    "deep-linkable page. Pick a family to filter, type in the box, or page "
    "through the table (20 at a time). The **Editions** column shows the AGS "
    "edition span each group covers — so a group added in 4.2, or removed in "
    "4.2, stands out. The left sidebar lists them by family too.",
    "",
    '<div class="grid cards" markdown>',
    "",
]
for fam, desc in cd.FAMILIES:
    n = sum(1 for r in index_rows if r[2] == fam)
    if not n:
        continue
    label = f"**{fam}** — {n} groups"
    land.append(
        f'- [{label}](#group-table){{ data-family="{fam}" }}'
        + (f"<br><small>{desc}</small>" if desc else "")
    )
# The spotlight's caption (#401). Both facts are READ, not typed: the count is
# the registry's, the edition span is the union dictionary's. A caption is prose,
# and a measured number written into prose acquires no reader — nothing would
# fail when the dictionary gained a group or an edition, and it would be wrong on
# the next build.
_ED_SPAN = f"{_ALL_EDS[0]}–{_ALL_EDS[-1]}" if len(_ALL_EDS) > 1 else _ALL_EDS[0]
_EDITION_LABEL = "editions" if len(_ALL_EDS) > 1 else "edition"
land += [
    "",
    "</div>",
    "",
    '<div class="group-table" id="group-table" markdown>',
    "",
    f'<p class="group-caption">{len(GROUPS)} groups · '
    f"dictionary {_EDITION_LABEL} {_ED_SPAN}</p>",
    "",
    "| Code | Group | Family | Parent | Headings | Editions |",
    "|---|---|---|:--|--:|:--|",
]
for code, contents, fam, parent, n, edlabel in index_rows:
    land.append(
        f"| [`{code}`]({code}.md) | {contents} | {fam} | {parent} | {n} | {edlabel} |"
    )
land += ["", "</div>"]
with mkdocs_gen_files.open("reference/groups/index.md", "w") as fd:
    fd.write("\n".join(land))

# Sidebar nav follows the curated FAMILIES order (then code within a family), so
# it matches the landing cards rather than ordering families by first-seen code.
for fam, _desc in cd.FAMILIES:
    for code in sorted(c for c in GROUPS if cd.family(c) == fam):
        nav[(fam, code)] = f"{code}.md"

with mkdocs_gen_files.open("reference/groups/SUMMARY.md", "w") as fd:
    fd.write("- [Overview](index.md)\n")
    fd.writelines(nav.build_literate_nav())
