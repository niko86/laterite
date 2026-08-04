"""Generate the consolidated multi-edition AGS dictionary (heading-local layout).

`ags_dictionary.json` is a FAITHFUL projection of the five official AGS standard
dictionaries (`rust-packages/laterite-ags4-validator/data/Standard_dictionary_v4_*.ags`)
— the authoritative source. It exists so every consumer reads ONE fast JSON instead of
parsing five `.ags` files at runtime, and so the model can't silently drift from the
spec (a CI gate, `tests/test_dictionary_faithful.py`, re-runs `build()` and asserts the
committed file matches — the same drift guard the `.pyi` stub has).

Layout = HEADING-LOCAL: each heading is flat (`name,status,type,unit,description`) with
the default value taken from the latest edition (4.2), plus two optional fields ONLY when
needed — `by_ed` (per-edition overrides of just the fields that differ) and `eds`
(edition membership when the heading isn't in every edition the group is). Group-level
`parent_by_ed`/`desc_by_ed`/`order_by_ed` cover the rare group-meta/order variations.
Lookup is one step + overlay; there is no base-edition asymmetry. NO deviations from
spec — pure faithful.

The ABBR pick-list VALUES (the `(ABBR_HDNG, ABBR_CODE) -> ABBR_DESC` table the
validator uses for Rule 16) and the per-edition `TRAN_AGS` string (Rule 14) are
carried too — union'd the same heading-local way (latest-edition default +
`by_ed` + `eds`) under top-level `abbreviations` + `tran_ags`. These are the two
things the validator's Dictionary needs that aren't in the group/heading schema,
so carrying them lets the validator read this one JSON instead of re-parsing the
.ags. (UNIT/TYPE pick-list VALUES are deliberately NOT carried: the validator
doesn't use them — it checks each heading's own declared unit/type.)

Usage:  python tools/gen_dictionary.py            # write the JSON
        from tools.gen_dictionary import build    # the CI gate calls this
"""

from __future__ import annotations

import csv
import json
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Iterable

REPO = Path(__file__).resolve().parents[1]
SRC = REPO / "rust-packages/laterite-ags4-validator/data"
OUT = REPO / "rust-packages/laterite-ags4-reference/data/ags_dictionary.json"
EDITIONS = ["4.0.3", "4.0.4", "4.1", "4.1.1", "4.2"]
DEFAULT_EDITION = "4.2"  # the latest edition (the union's flat-field default)
# The edition auto-selection falls back to when a file's TRAN_AGS is absent /
# unparsable. Deliberately python-ags4's LATEST_DICT_VERSION (NOT the newest
# bundled edition) so dogfood parity reflects real defects, not fallback
# artefacts. Carried in the JSON so every surface (validator, wasm, web) reads
# ONE fallback instead of hand-copying it. Cross-ref the validator's dict::FALLBACK.
FALLBACK_EDITION = "4.1.1"
FIELDS = ("status", "type", "unit", "description")
_RANK = {e: i for i, e in enumerate(EDITIONS)}


def _parse(path: Path) -> dict:
    """One edition -> {group: {parent, description, headings:[{name,...}]}}, order preserved."""
    groups: dict = {}
    with path.open(newline="", encoding="latin-1") as fh:
        for r in csv.reader(fh):
            if len(r) < 4 or r[0] != "DATA":
                continue
            if r[1] == "GROUP":
                g = r[2]
                parent = r[9] if len(r) > 9 else ""
                grp = groups.setdefault(
                    g, {"parent": None, "description": None, "headings": []}
                )
                grp["parent"] = None if parent in ("", "-") else parent
                grp["description"] = r[6] or None
            elif r[1] == "HEADING" and len(r) >= 8:
                g = r[2]
                grp = groups.setdefault(
                    g, {"parent": None, "description": None, "headings": []}
                )
                grp["headings"].append(
                    {
                        "name": r[3],
                        "status": r[4],
                        "type": r[5],
                        "unit": r[7] or None,
                        "description": r[6],
                    }
                )
    return groups


def _latest(eds: Iterable[str]) -> str:
    """The newest edition in `eds`, by `_RANK` — never by string order, which
    would put "4.1.1" before "4.2"."""
    return max(eds, key=lambda e: _RANK[e])


def _fname(ed: str) -> str:
    return f"Standard_dictionary_v{ed.replace('.', '_')}.ags"


def _group_data(path: Path, group: str) -> list[dict]:
    """DATA rows of one GROUP's *instance*, keyed by its HEADING columns — the
    group-aware read the validator's build.rs does. The ABBR pick-list + the
    TRAN_AGS value live in those groups' data rows, not the DICT schema rows
    `_parse` reads, so they need this separate pass."""
    cur: str | None = None
    cols: list[str] = []
    rows: list[dict] = []
    with path.open(newline="", encoding="latin-1") as fh:
        for r in csv.reader(fh):
            if not r:
                continue
            if r[0] == "GROUP":
                cur = r[1] if len(r) > 1 else None
            elif r[0] == "HEADING" and cur == group:
                cols = r[1:]
            elif r[0] == "DATA" and cur == group:
                # strict=False: AGS rows can carry trailing cells beyond the
                # HEADING; we read columns by name, so zip to the shorter.
                rows.append(dict(zip(cols, r[1:], strict=False)))
    return rows


def _parse_abbr(path: Path) -> dict:
    """(ABBR_HDNG, ABBR_CODE) -> ABBR_DESC for one edition; last definition wins
    (matches the validator's build.rs)."""
    return {
        (r["ABBR_HDNG"], r["ABBR_CODE"]): r["ABBR_DESC"]
        for r in _group_data(path, "ABBR")
    }


def _parse_tran_ags(path: Path) -> str:
    """The edition's declared TRAN_AGS (Rule 14) — the first TRAN data row."""
    rows = _group_data(path, "TRAN")
    return rows[0]["TRAN_AGS"] if rows else ""


def _build_abbreviations(abbr_parsed: dict) -> list:
    """Union the per-edition ABBR maps the same heading-local way as headings:
    latest-edition description as the flat default, `eds` when the code isn't in
    every edition, `by_ed` for the (few) descriptions that differ by edition."""
    out = []
    for hdng, code in sorted({k for m in abbr_parsed.values() for k in m}):
        present = [e for e in EDITIONS if (hdng, code) in abbr_parsed[e]]
        desc = abbr_parsed[_latest(present)][(hdng, code)]
        rec = {"heading": hdng, "code": code, "description": desc}
        if present != EDITIONS:
            rec["eds"] = present
        by = {
            e: abbr_parsed[e][(hdng, code)]
            for e in present
            if abbr_parsed[e][(hdng, code)] != desc
        }
        if by:
            rec["by_ed"] = by
        out.append(rec)
    return out


def _reconstruct_abbr(doc: dict, ed: str) -> dict:
    """Rebuild one edition's (ABBR_HDNG, ABBR_CODE) -> ABBR_DESC from the doc."""
    out = {}
    for rec in doc["abbreviations"]:
        if ed not in rec.get("eds", EDITIONS):
            continue
        out[(rec["heading"], rec["code"])] = rec.get("by_ed", {}).get(
            ed, rec["description"]
        )
    return out


def build() -> dict:
    """Parse the five official dicts -> the heading-local doc. Self-verifies that every
    edition reconstructs EXACTLY (groups, headings, values, order) before returning."""
    parsed = {ed: _parse(SRC / _fname(ed)) for ed in EDITIONS}
    abbr_parsed = {ed: _parse_abbr(SRC / _fname(ed)) for ed in EDITIONS}
    tran_parsed = {ed: _parse_tran_ags(SRC / _fname(ed)) for ed in EDITIONS}

    # gather, per group, its per-edition meta + per-heading per-edition defs
    acc: dict = {}
    for ed in EDITIONS:
        for g, gv in parsed[ed].items():
            gg = acc.setdefault(
                g, {"eds": [], "parent": {}, "desc": {}, "order": {}, "head": {}}
            )
            gg["eds"].append(ed)
            gg["parent"][ed] = gv["parent"]
            gg["desc"][ed] = gv["description"]
            gg["order"][ed] = [h["name"] for h in gv["headings"]]
            for h in gv["headings"]:
                gg["head"].setdefault(h["name"], {})[ed] = {f: h[f] for f in FIELDS}

    groups_out: dict = {}
    for g, gg in acc.items():
        g_eds = [e for e in EDITIONS if e in gg["eds"]]
        g_latest = _latest(g_eds)
        grp = {
            "eds": g_eds,
            "parent": gg["parent"][g_latest],
            "description": gg["desc"][g_latest],
            "headings": [],
        }
        p_by = {e: gg["parent"][e] for e in g_eds if gg["parent"][e] != grp["parent"]}
        d_by = {e: gg["desc"][e] for e in g_eds if gg["desc"][e] != grp["description"]}
        if p_by:
            grp["parent_by_ed"] = p_by
        if d_by:
            grp["desc_by_ed"] = d_by
        # heading order: latest-edition order, then any heading only in older editions appended
        order = list(gg["order"][g_latest])
        for e in reversed(g_eds):
            for n in gg["order"][e]:
                if n not in order:
                    order.append(n)
        for n in order:
            per_ed = gg["head"][n]
            h_eds = [e for e in EDITIONS if e in per_ed]
            default = per_ed[_latest(h_eds)]
            rec = {"name": n, **default}
            if h_eds != g_eds:
                rec["eds"] = h_eds
            by = {}
            for e in h_eds:
                diff = {f: per_ed[e][f] for f in FIELDS if per_ed[e][f] != default[f]}
                if diff:
                    by[e] = diff
            if by:
                rec["by_ed"] = by
            grp["headings"].append(rec)
        ord_by = {}
        for e in g_eds:
            derived = [n for n in order if e in gg["head"][n]]
            if derived != gg["order"][e]:
                ord_by[e] = gg["order"][e]
        if ord_by:
            grp["order_by_ed"] = ord_by
        groups_out[g] = grp

    doc = {
        "format_version": "1.0.0",
        "source": "faithful projection of official AGS Standard_dictionary_v4_*.ags",
        "layout": "heading-local",
        "default_edition": DEFAULT_EDITION,
        "fallback_edition": FALLBACK_EDITION,
        "editions": EDITIONS,
        "tran_ags": {ed: tran_parsed[ed] for ed in EDITIONS},
        "deviations": [],  # none — pure faithful to spec
        "groups": groups_out,
        "abbreviations": _build_abbreviations(abbr_parsed),
    }
    _self_verify(doc, parsed, abbr_parsed, tran_parsed)
    return doc


def reconstruct(doc: dict, ed: str) -> dict:
    """Rebuild one edition's full {group: {parent, description, headings}} from the doc."""
    out = {}
    for g, grp in doc["groups"].items():
        if ed not in grp["eds"]:
            continue
        hs = []
        for h in grp["headings"]:
            if ed not in h.get("eds", grp["eds"]):
                continue
            d = {f: h[f] for f in FIELDS}
            d.update(h.get("by_ed", {}).get(ed, {}))
            hs.append({"name": h["name"], **d})
        if ed in grp.get("order_by_ed", {}):
            by = {x["name"]: x for x in hs}
            hs = [by[n] for n in grp["order_by_ed"][ed] if n in by]
        out[g] = {
            "parent": grp.get("parent_by_ed", {}).get(ed, grp["parent"]),
            "description": grp.get("desc_by_ed", {}).get(ed, grp["description"]),
            "headings": hs,
        }
    return out


def _self_verify(doc: dict, parsed: dict, abbr_parsed: dict, tran_parsed: dict) -> None:
    for ed in EDITIONS:
        got = reconstruct(doc, ed)
        if got != parsed[ed]:
            bad = [
                g for g in set(got) | set(parsed[ed]) if got.get(g) != parsed[ed].get(g)
            ]
            raise AssertionError(
                f"reconstruct({ed}) != source; groups differ: {bad[:6]}"
            )
        if _reconstruct_abbr(doc, ed) != abbr_parsed[ed]:
            raise AssertionError(f"reconstruct ABBR({ed}) != source")
        if doc["tran_ags"][ed] != tran_parsed[ed]:
            raise AssertionError(f"tran_ags({ed}) != source")


def main() -> None:
    doc = build()
    OUT.write_text(json.dumps(doc, indent=2) + "\n")
    n_grp = len(doc["groups"])
    n_head = sum(len(g["headings"]) for g in doc["groups"].values())
    n_var = sum(
        1 for g in doc["groups"].values() for h in g["headings"] if "by_ed" in h
    )
    n_abbr = len(doc["abbreviations"])
    print(
        f"wrote {OUT.relative_to(REPO)}  ({n_grp} groups, {n_head} headings, "
        f"{n_var} with per-edition overrides, {n_abbr} abbreviations)  "
        f"editions={doc['editions']}"
    )


if __name__ == "__main__":
    main()
