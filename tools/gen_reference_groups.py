#!/usr/bin/env python
"""Generate the ags-wiki group reference tier from ags_dictionary.json (D6).

The `groups/*.md` reference tier is ~90% a mechanical projection of the AGS4
dictionary (parent/children, KEY/REQUIRED tuples, the heading table, the
position erDiagram) with a thin, fully-templatable veneer (Purpose, Variations
4.2-status banner, the standard concept-links). Before D6 it was hand-bootstrapped
and had drifted from the SSOT (every page's `ags_editions` was a uniform `[4.1]`
placeholder; root descriptions were `(scaffolded) CODE` stubs). This makes the
dictionary the single source: one group page per dict group, regenerated here,
gated by `--check` (committed == render()) in the `wiki-lint` job — advisory
per-PR, hard in `nightly.yml`. That is the whole gate. This line named a paired
pytest (2026-07-23 → 2026-08-05) that was never written, so a reader weighing
whether the 174 group pages could drift was counting a second guard that did not
exist.

Scope = every group in ags_dictionary.json (the AGS4 union, 174). "Current in
4.2" is expressed by the Variations banner (present / deprecated / removed), not
by dropping groups — a removed group keeps its page + its place in the parent's
children list (so no orphans, and observation pages that cite e.g. ERES keep
resolving). The 3 AGS-L draft groups (CONL/TREL/TRIL) are NOT in the dictionary
and stay hand-authored, so nothing here reads them: the render loop walks the
dictionary, not the directory. `--check` now NAMES them on every run, pass or
fail, which is the only place that exception is recorded — this line used to
point at an `AGS_L_EXCEPTIONS` constant "in the faithfulness test", and neither
the constant nor a test containing it has ever existed in either tree. That is
the same fault the paragraph above corrects four lines earlier, in the same
docstring, uncorrected. #757 is what surfaced it: those three pages are the only
ones whose `parent:` has ever been wrong, so "recorded in a pointer to nothing"
was doing real work as a reason not to look.

A few facts aren't in the dictionary and are carried here as small, spec-cited
constants: the deprecation/removal successors (spec Foreword / §3.6), the
high-volume row-count hint, and the root-group note. Everything else derives
from the dict entry {eds, parent, description, headings[{name,status,type,unit,
description}]}.

Run:  uv run --no-sync python tools/gen_reference_groups.py          # write
      uv run --no-sync python tools/gen_reference_groups.py --check  # CI gate
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
DICT = (
    REPO / "rust-packages" / "laterite-ags4-reference" / "data" / "ags_dictionary.json"
)
GROUPS_DIR = REPO / "ags-wiki" / "groups"
DICT_REF = "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json"

ALL_EDITIONS = ["4.0.3", "4.0.4", "4.1", "4.1.1", "4.2"]
CURRENT_EDITION = "4.2"
# the four navigation links every group page carries (uniform across the tier).
CONCEPT_LINKS = [
    "parent-child-graph",
    "key-tuple-pseudo-keys",
    "heading-status-vocabulary",
    "rule-10c-parent-child",
]

# --- facts NOT in the dictionary (spec/domain-sourced, cited) ---
# Deprecated in 4.2 (strike-through, spec §3.6) — present but discouraged.
DEPRECATED_IN_42 = {"SCPG", "SCPT"}
# Successor group(s) for a deprecated/removed group (spec Foreword / §3.6).
SUPERSEDED_BY = {
    "SCPG": "CPDx/CPTx",
    "SCPT": "CPDx/CPTx",
    "ERES": "ELRG",
    "IPRG": "FGHG",
    "IPRT": "FGHS",
}
# High row-count groups (a UI/perf hint, not a spec fact). New dict groups with
# no page today default to false — refine here if a large group is added.
HIGH_VOLUME = {
    "DETL",
    "DISC",
    "ERES",
    "DOBS",
    "GRAT",
    "ECTN",
    "MOND",
    "ISAT",
    "LBST",
    "PMTD",
    "SCDT",
    "SCPT",
    "WETH",
}

# Dict heading-status vocabulary: KEY, REQUIRED, KEY+REQUIRED (both — e.g.
# PROJ_ID), DEPRECATED, OTHER. A key is inherently non-null, so a KEY+REQUIRED
# heading is listed under key_headings only (not doubled into required_headings)
# and displays with the KEY marker — matching the bootstrap convention.
_STATUS_DISP = {
    "KEY": "**KEY**",
    "KEY+REQUIRED": "**KEY**",
    "REQUIRED": "*REQ*",
    "DEPRECATED": "`DEP`",
}


def _fmlist(items: list[str]) -> str:
    return "[" + ", ".join(items) + "]"


def _children(dic: dict, code: str) -> list[str]:
    return sorted(c for c, v in dic.items() if v.get("parent") == code)


def _keys(headings: list[dict]) -> list[str]:
    return [h["name"] for h in headings if h["status"] in ("KEY", "KEY+REQUIRED")]


def _required(headings: list[dict]) -> list[str]:
    return [h["name"] for h in headings if h["status"] == "REQUIRED"]


def _variations(code: str, eds: list[str]) -> str:
    if code in DEPRECATED_IN_42:
        return (
            f"> [!warning] **DEPRECATED in AGS 4.2** (strike-through in "
            f"`spec:AGS4-4.2-2025.pdf` §3.6) — to be removed in a future "
            f"edition; superseded by {SUPERSEDED_BY[code]}.\n\n"
            f"Still valid in 4.2 but discouraged; a producer/consumer "
            f"interoperability risk."
        )
    if CURRENT_EDITION not in eds:  # removed before 4.2
        succ = SUPERSEDED_BY.get(code, "a successor group")
        return (
            f"> [!warning] **REMOVED in AGS 4.2** (`spec:AGS4-4.2-2025.pdf` "
            f"Foreword). Present only in {eds[0]}–{eds[-1]} files; "
            f"superseded by {succ}.\n\n"
            f"Files using this group are valid ≤{eds[-1]}, invalid under "
            f"4.2 — a concrete edition-dependent validation divergence (Phase "
            f"D probe candidate)."
        )
    return (
        "No group-level change at 4.2 (present across the in-scope editions). "
        "Granular per-heading edition deltas live in the AGS online **Change "
        "Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` "
        "Foreword → ags.org.uk/.../change-log). Heading-level archaeology is "
        "deferred to a targeted Ingest if a rule/O-N interaction needs it (per "
        "[[ags4-rules-frozen-dictionary-evolves]])."
    )


def render_group(code: str, dic: dict) -> str:
    v = dic[code]
    parent = v.get("parent") or ""
    desc = v["description"]
    eds = v["eds"]
    headings = v["headings"]
    kids = _children(dic, code)
    keys = _keys(headings)
    reqs = _required(headings)
    is_root = not parent

    # --- frontmatter ---
    related = CONCEPT_LINKS + ([parent] if parent else []) + kids
    fm = [
        "---",
        "type: group",
        f"title: {code} — {desc}",
        "status: drafted",
        "tags: [group]",
        f"group_code: {code}",
        f"parent: {parent if parent else chr(34) + chr(34)}",
        f"is_high_volume: {'true' if code in HIGH_VOLUME else 'false'}",
        f"varies_between_editions: {'true' if set(eds) != set(ALL_EDITIONS) else 'false'}",
        f"key_headings: {_fmlist(keys)}",
        f"required_headings: {_fmlist(reqs)}",
        f"ags_editions: {_fmlist(eds)}",
        "repo_refs:",
        f'  dictionary: "{DICT_REF} groups[code={code}]"',
        f"related: {_fmlist(related)}",
        "sources: []",
        "---",
    ]

    # --- Purpose ---
    if is_root:
        purpose = (
            f"> [!quote] The **{code}** group — {desc}. It is a **root / "
            f"non-hierarchy** group (file submission & description — Rules "
            f"13–18 territory). See [[parent-child-graph]]."
        )
    else:
        purpose = (
            f"> [!quote] The **{code}** group — {desc}. It is a **child of "
            f"[[{parent}]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]]."
        )

    # --- Position / erDiagram ---
    erd = ["```mermaid", "erDiagram"]
    if parent:
        erd.append(f"  {parent} ||--o{{ {code} : has")
    erd.extend(f"  {code} ||--o{{ {k} : has" for k in kids)
    erd.append(f"  {code} {{")
    erd.extend(f"    KEY {kh}" for kh in keys)
    erd.append("  }")
    erd.append("```")
    parent_line = (
        f"- Parent: [[{parent}]]" if parent else "- Parent: _(root — no parent)_"
    )
    kids_line = (
        ("- Children: " + " ".join(f"[[{k}]]" for k in kids))
        if kids
        else "- Children: _none_"
    )

    # --- Headings table ---
    htbl = [
        f"> [!quote] Rendered from `{DICT_REF} groups[code={code}]` (the repo's "
        f"model authority — AGS edition {CURRENT_EDITION}). Suggested UNITs + "
        f"worked examples are in the cited spec PDF, not duplicated here.",
        "",
        f"{len(headings)} heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = "
        f"REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.",
        "",
        "| Heading | Status | Type | Description |",
        "|---|---|---|---|",
    ]
    for h in headings:
        st = _STATUS_DISP.get(h["status"], "OTHER")
        htbl.append(
            f"| `{h['name']}` | {st} | `{h['type']}` | {h.get('description') or ''} |"
        )

    # --- Relational notes ---
    keytuple = ", ".join(f"`{k}`" for k in keys) or "_(none)_"
    kids_inline = " ".join(f"[[{k}]]" for k in kids) if kids else "_none_"
    if is_root:
        relnotes = (
            f"KEY tuple: {keytuple}. Children ({len(kids)}): {kids_inline}. "
            f"Parent linkage is implicit/absent — Rule 10c is skipped for root "
            f"groups (see [[non-hierarchy-ten-vs-parentless-list]]). See "
            f"[[key-tuple-pseudo-keys]] · [[denormalised-child-rows]]."
        )
    else:
        relnotes = (
            f"KEY tuple: {keytuple}. Children ({len(kids)}): {kids_inline}. As "
            f"a child it **denormalises** its parent's KEY columns into every "
            f"row; [[rule-10c-parent-child]] re-resolves that repeated tuple "
            f"upward to [[{parent}]]. See [[key-tuple-pseudo-keys]] · "
            f"[[denormalised-child-rows]]."
        )

    # --- Related ---
    related_block = " · ".join(f"[[{x}]]" for x in related)

    body = [
        f"# {code} — {desc}",
        "",
        "## Purpose",
        purpose,
        "",
        "## Position in the model",
        "",
        *erd,
        "",
        parent_line,
        kids_line,
        "- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]",
        "",
        "## Headings",
        *htbl,
        "",
        "Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).",
        "",
        "## Relational notes",
        relnotes,
        "",
        "## Variations",
        _variations(code, eds),
        "",
        "## Related",
        related_block,
    ]
    return "\n".join(fm) + "\n" + "\n".join(body) + "\n"


def _dict() -> dict:
    return json.loads(DICT.read_text(encoding="utf-8"))["groups"]


def _ungenerated() -> list[str]:
    """Pages under `groups/` this generator does not produce, and so never reads.

    The render loop below walks the DICTIONARY, not the directory, so a page for
    a group the dictionary has never heard of is not stale, not fresh, and not
    mentioned — it is simply absent from the gate. Three such pages exist (the
    AGS-L drafts CONL/TREL/TRIL), and #757 found that they are exactly the pages
    whose `parent:` has ever been wrong: the field is correct everywhere a
    machine could check it and wrong everywhere it could not.

    So this is the blind spot with the defects in it, and CLAUDE.md's rule
    applies literally — a gate that drops input says what it dropped. Printed on
    every run, pass or fail, because a bare "OK: 174 pages" over a directory of
    177 is a green tick on three rows nothing looked at.
    """
    dic = _dict()
    return sorted(p.stem for p in GROUPS_DIR.glob("*.md") if p.stem not in dic)


def main(argv: list[str]) -> int:
    dic = _dict()
    check = "--check" in argv
    stale = []
    for code in sorted(dic):
        rendered = render_group(code, dic)
        path = GROUPS_DIR / f"{code}.md"
        if check:
            if not path.exists() or path.read_text(encoding="utf-8") != rendered:
                stale.append(code)
        else:
            path.write_text(rendered, encoding="utf-8")
    outside = _ungenerated()

    def report_scope() -> None:
        if outside:
            print(
                f"  not generated, so not checked ({len(outside)}): "
                f"{', '.join(outside)} — outside {DICT.name}, held by no gate"
            )

    if check:
        if stale:
            print(
                f"group reference tier STALE ({len(stale)}): {stale[:20]}"
                + (" …" if len(stale) > 20 else ""),
                file=sys.stderr,
            )
            report_scope()
            print(
                "run `uv run --no-sync python tools/gen_reference_groups.py`",
                file=sys.stderr,
            )
            return 1
        print(f"group reference tier OK: {len(dic)} pages match render()")
        report_scope()
        return 0
    print(f"wrote {len(dic)} group pages from {DICT.name}")
    report_scope()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
