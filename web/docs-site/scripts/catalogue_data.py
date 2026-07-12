"""Shared, side-effect-free data layer for the AGS4 group catalogue (#201).

Both the build-time generators (`gen_groups.py`, `gen_types.py`, run by
mkdocs-gen-files) and the content-drift gate (`tests/test_groups_catalogue_faithful.py`)
import this module, so the catalogue can never quietly diverge from the
dictionary or the glossary from the type codes it documents.

Pure stdlib (json only) — deliberately does NOT import `laterite`, so the gate
can load it without the compiled wheel, and the edition/provenance facts come
straight from the single-source union dictionary rather than the registry (which
intentionally drops per-edition membership and exposes only the latest union).

Two things live here that the registry can't give us:

1. **Edition provenance.** `ags_dictionary.json` carries an `"eds"` array on every
   group (and, where it differs, on a heading) plus a top-level ordered
   `"editions"` list. From those we derive "added in 4.x" / "removed in 4.x" — the
   union the registry serves is latest-edition-flattened and has no such field.
2. **The AGS data-type glossary.** The type codes (`ID`, `2DP`, `DT`, …) are
   defined in the standard dictionary's own `TYPE` group; the canonical mapping +
   validation rules live in `laterite-types`. `TYPE_GLOSSARY` is the synthesis of
   both, anchored so heading tables can deep-link each code.
"""

from __future__ import annotations

import json
from functools import lru_cache
from pathlib import Path

# repo_root/rust-packages/... — this file is web/docs-site/scripts/catalogue_data.py
_DICT_PATH = (
    Path(__file__).resolve().parents[3]
    / "rust-packages/laterite-ags4-reference/data/ags_dictionary.json"
)


@lru_cache(maxsize=1)
def load_dict() -> dict:
    """The single-source union dictionary (cached)."""
    return json.loads(_DICT_PATH.read_text(encoding="utf-8"))


def editions() -> list[str]:
    """Canonical edition order, e.g. ['4.0.3', '4.0.4', '4.1', '4.1.1', '4.2']."""
    return list(load_dict()["editions"])


# ---------------------------------------------------------------------------
# Edition provenance (group + heading "added / removed in 4.x")
# ---------------------------------------------------------------------------

def _span_provenance(eds: list[str], all_eds: list[str]) -> dict:
    """Reduce an `eds` membership list to a presentable provenance record.

    `added_in` is set only when the entry's first edition is *not* the format's
    first edition; `removed_in` is the edition immediately AFTER the entry's last,
    set only when the entry drops out before the latest edition. `span` is the
    `(first, last)` the entry actually covers.
    """
    first, last = all_eds[0], all_eds[-1]
    added = eds[0] if eds[0] != first else None
    removed = None
    if eds[-1] != last:
        removed = all_eds[all_eds.index(eds[-1]) + 1]
    return {
        "eds": list(eds),
        "span": (eds[0], eds[-1]),
        "added_in": added,
        "removed_in": removed,
        "all_editions": added is None and removed is None,
    }


def group_provenance(code: str) -> dict:
    """Edition provenance for one group, derived from its `eds` array."""
    doc = load_dict()
    return _span_provenance(doc["groups"][code]["eds"], doc["editions"])


def heading_eds(group_eds: list[str], heading: dict) -> list[str]:
    """A heading's editions — its own `eds` if present, else the group's."""
    return heading.get("eds", group_eds)


def heading_provenance(group_eds: list[str], heading: dict, all_eds: list[str]) -> dict:
    """Heading provenance RELATIVE to its group.

    A heading is only "added in 4.x" / "removed in 4.x" when it joins later than,
    or drops out before, its parent group — i.e. only the *delta* against the
    group is interesting (otherwise a group added in 4.2 would noisily mark every
    one of its headings "added in 4.2"). Returns `differs=False` when the heading
    simply tracks its group.
    """
    h_eds = heading_eds(group_eds, heading)
    g_first, g_last = group_eds[0], group_eds[-1]
    added = h_eds[0] if h_eds[0] != g_first else None
    removed = None
    if h_eds[-1] != g_last:
        removed = all_eds[all_eds.index(h_eds[-1]) + 1]
    return {
        "span": (h_eds[0], h_eds[-1]),
        "added_in": added,
        "removed_in": removed,
        "differs": added is not None or removed is not None,
    }


def edition_label(prov: dict) -> str:
    """One-line edition span for a group, e.g. '4.0.3 – 4.2' or '4.2'."""
    lo, hi = prov["span"]
    return lo if lo == hi else f"{lo} – {hi}"


# ---------------------------------------------------------------------------
# AGS data-type glossary
# ---------------------------------------------------------------------------
# Anchored entries the `/types/` page renders and the heading tables deep-link
# to. `summary` is the AGS standard's own wording (from the dictionary's TYPE
# group); `detail` adds laterite's canonical mapping + how the value is read.
# Parametric families (nDP / nSF / nSCI) are documented once; `glossary_key()`
# folds each concrete code (2DP, 3SF, …) onto its family anchor.

TYPE_GLOSSARY: list[dict] = [
    {
        "key": "id", "codes": ["ID"], "title": "ID — Unique identifier",
        "canonical": "string", "ags": "Unique Identifier",
        "detail": "A KEY-style text code that names a record (a borehole, a "
                  "sample, a test). Read as a string; never coerced to a number "
                  "even when it looks numeric.",
        "example": "`BH01`, `BH01/4.50`",
    },
    {
        "key": "x", "codes": ["X"], "title": "X — Text",
        "canonical": "string", "ags": "Text",
        "detail": "Free-form alphanumeric text — the catch-all type. Read "
                  "verbatim (whitespace trimmed).",
        "example": "`Made Ground`, `Light brown sandy CLAY`",
    },
    {
        "key": "xn", "codes": ["XN"], "title": "XN — Text or numeric",
        "canonical": "string", "ags": "Text/numeric",
        "detail": "Usually a number but may carry non-numeric tokens (`<0.05`, "
                  "`n/a`, `>1000`), so it is read as a string by default. The "
                  "opt-in `xn=\"numeric\"` read knob coerces the parseable cells "
                  "to floats.",
        "example": "`12.5`, `<0.05`, `n/a`",
    },
    {
        "key": "pa", "codes": ["PA"], "title": "PA — Abbreviation pick-list",
        "canonical": "string", "ags": "Text listed in ABBR Group",
        "detail": "A controlled value that must appear in the file's `ABBR` "
                  "group. Read as a string; the validator checks membership.",
        "example": "`CP` (cable percussion), `RC` (rotary cored)",
    },
    {
        "key": "pt", "codes": ["PT"], "title": "PT — Type pick-list",
        "canonical": "string", "ags": "Text listed in TYPE Group",
        "detail": "A controlled value drawn from the `TYPE` group — i.e. one of "
                  "the data-type codes on this page. Read as a string.",
        "example": "`2DP`, `DT`",
    },
    {
        "key": "pu", "codes": ["PU"], "title": "PU — Unit pick-list",
        "canonical": "string", "ags": "Text listed in UNIT Group",
        "detail": "A controlled value drawn from the file's `UNIT` group. Read "
                  "as a string.",
        "example": "`m`, `kN/m2`, `Mg/m3`",
    },
    {
        "key": "u", "codes": ["U"], "title": "U — Unit",
        "canonical": "string", "ags": "Unit (text)",
        "detail": "A unit string. Read as text. Note: `U` is used by headings in "
                  "the dictionary but is not itself defined in the standard "
                  "`TYPE` group — treat it as a free unit label.",
        "example": "`mm`, `%`",
    },
    {
        "key": "t", "codes": ["T"], "title": "T — Elapsed time",
        "canonical": "string", "ags": "Elapsed Time",
        "detail": "A duration / elapsed time written in a colon-separated form "
                  "(not a clock time-of-day). Read as a string.",
        "example": "`12:30` (12 h 30 m), `00:00:45`",
    },
    {
        "key": "mc", "codes": ["MC"], "title": "MC — Moisture content",
        "canonical": "string", "ags": "BS1377 reported moisture content",
        "detail": "A moisture content reported per BS 1377 : Part 2 conventions. "
                  "Read as a string.",
        "example": "`23`",
    },
    {
        "key": "dms", "codes": ["DMS"], "title": "DMS — Degrees:minutes:seconds",
        "canonical": "string", "ags": "Degrees:Minutes:Seconds",
        "detail": "A sexagesimal angle / geographic coordinate. Read as a string "
                  "(it is not a single decimal).",
        "example": "`51:28:38`",
    },
    {
        "key": "yn", "codes": ["YN"], "title": "YN — Yes / No",
        "canonical": "bool", "ags": "Yes or No",
        "detail": "A boolean. Read as `True`/`False`; the parser accepts "
                  "`Y`/`N`, `YES`/`NO`, `TRUE`/`FALSE`, `1`/`0` (case-insensitive).",
        "example": "`Y`, `N`",
    },
    {
        "key": "dt", "codes": ["DT"], "title": "DT — Date / date-time",
        "canonical": "datetime", "ags": "Date time in international format",
        "detail": "An ISO-8601-style date or date-time. Read as a `datetime`; a "
                  "date-only value is promoted to midnight. Accepted forms "
                  "include `YYYY-MM-DD`, `YYYY-MM-DD HH:MM[:SS]` and "
                  "`YYYY-MM-DDTHH:MM:SS`.",
        "example": "`2024-03-15`, `2024-03-15 09:30:00`",
    },
    {
        "key": "rl", "codes": ["RL"], "title": "RL — Record link",
        "canonical": "string", "ags": "Record Link",
        "detail": "A typed cross-reference to another group's KEY record "
                  "(heading-name / value pairs concatenated). Read as a string.",
        "example": "`LOCA_ID`",
    },
    {
        "key": "0dp", "codes": ["0DP"], "title": "0DP — Whole number",
        "canonical": "integer", "ags": "Value; required number of decimal places, 0",
        "detail": "An integer (zero decimal places). Read as an `int`; values "
                  "written `5.0` are tolerated and parsed via float.",
        "example": "`5`, `100`",
    },
    {
        "key": "ndp", "codes": ["1DP", "2DP", "3DP", "4DP"],
        "title": "nDP — Fixed decimal places",
        "canonical": "decimal", "ags": "Value; required number of decimal places, n",
        "detail": "A fixed-point number carrying exactly **n** decimal places "
                  "(`1DP` … `4DP`). The digit count is the declared precision the "
                  "validator enforces (Rule 8); read as a `float`.",
        "example": "`12.30` (2DP), `0.001` (3DP)",
    },
    {
        "key": "nsf", "codes": ["1SF", "2SF", "3SF"],
        "title": "nSF — Significant figures",
        "canonical": "decimal", "ags": "Value; required number of significant figures, n",
        "detail": "A number expressed to exactly **n** significant figures "
                  "(`1SF` … `3SF` are used). Read as a `float`.",
        "example": "`0.045` (2SF), `12300` (3SF)",
    },
    {
        "key": "nsci", "codes": ["1SCI", "2SCI", "3SCI", "4SCI"],
        "title": "nSCI — Scientific notation",
        "canonical": "decimal", "ags": "Scientific Notation; required number of decimal places, n",
        "detail": "A number in scientific notation with **n** decimal places in "
                  "the mantissa (`1SCI`, `2SCI` are used). Read as a `float`.",
        "example": "`1.2E-08` (1SCI), `4.50E+03` (2SCI)",
    },
]

# code -> glossary anchor key, including the parametric folds.
_GLOSSARY_BY_CODE: dict[str, str] = {
    code: entry["key"] for entry in TYPE_GLOSSARY for code in entry["codes"]
}


def glossary_key(type_code: str) -> str | None:
    """Anchor key for a type code, folding `2DP`→`ndp`, `3SF`→`nsf`, etc."""
    t = type_code.strip().upper()
    if t in _GLOSSARY_BY_CODE:
        return _GLOSSARY_BY_CODE[t]
    for suffix, key in (("SCI", "nsci"), ("SF", "nsf"), ("DP", "ndp")):
        if t.endswith(suffix) and t[: -len(suffix)].isdigit():
            return "0dp" if t == "0DP" else key
    return None


def used_type_codes() -> set[str]:
    """Every distinct heading `type` code that appears in the dictionary."""
    codes: set[str] = set()
    for g in load_dict()["groups"].values():
        for h in g["headings"]:
            codes.add(h["type"])
    return codes


# ---------------------------------------------------------------------------
# Family taxonomy — the catalogue's human-meaningful nav axis
# ---------------------------------------------------------------------------
# The PROJ parent tree is too lopsided to navigate (164/174 groups hang off
# PROJ; SAMP has 56 children, LOCA 50), so the catalogue groups by a flat,
# curated FAMILY layer instead.
#
# Curated taxonomy (#201 Q2) — produced by a propose -> reconcile -> critique
# pass over all 174 group descriptions (critique verdict: APPROVE; every code
# assigned exactly once, family sizes 5-25). Families read in
# site-investigation order: project/admin first, transfer + dictionaries last.
# `FAMILY_OF` (code -> family) and `FAMILIES` (ordered [(name, tagline)]) are the
# stable interface the generators + drift gate consume; `_FAMILY_CODES`
# (family -> its sorted codes) is the reviewable source.

FAMILIES: list[tuple[str, str]] = [
    ('Project & Administration', 'Project set-up, schedule, standards & remarks'),
    ('Exploratory Holes, Sampling & Field Records', 'Holes, construction, drilling records & samples'),
    ('Ground Description & Field Logging', 'Geological descriptions, strata, weathering & discontinuities'),
    ('In-Situ Penetration & Strength Tests', 'SPT, CPT, probing, vane & in-situ density'),
    ('In-Situ Loading, Deformation & Geophysics', 'Pressuremeter, dilatometer, plate, seismic & geophysics'),
    ('Groundwater & Monitoring', 'Permeability, pumping, water strikes & installations'),
    ('Soil Classification & Index Tests', 'Atterberg, moisture, density, particle size & index'),
    ('Soil Strength, Consolidation & Deformation Tests', 'Triaxial, shear box, oedometer & consolidation'),
    ('Rock Testing', 'Intact-rock strength, density, hardness & abrasiveness'),
    ('Aggregate, Earthworks & Pavement Materials', 'Aggregate suite, CBR, compaction, MCV & lime'),
    ('Geoenvironmental & Chemical Testing', 'Contamination, ground chemistry & corrosivity'),
    ('Data Transfer & Dictionaries', 'Transfer metadata, abbreviations, types & units'),
]

# family -> its group codes (sorted) — reviewable at a glance.
_FAMILY_CODES: dict[str, list[str]] = {
    'Project & Administration': [
        'LBSG', 'LBST', 'PREM', 'PROJ', 'STND',
    ],
    'Exploratory Holes, Sampling & Field Records': [
        'BKFL', 'CDIA', 'CHIS', 'CHOC', 'CORE', 'DOBS', 'DREM', 'ECTN',
        'FLSH', 'HDIA', 'HDPH', 'HORN', 'LOCA', 'PTIM', 'SAMP', 'TREM',
        'WADD', 'WINS',
    ],
    'Ground Description & Field Logging': [
        'DETL', 'DISC', 'DLOG', 'FRAC', 'GEOL', 'WETH',
    ],
    'In-Situ Penetration & Strength Tests': [
        'CPDG', 'CPDT', 'CPTG', 'CPTM', 'CPTP', 'CPTT', 'CPTY', 'CPTZ',
        'DCPG', 'DCPT', 'DPRB', 'DPRG', 'ICBR', 'IDEN', 'IPEN', 'ISPT',
        'IVAN', 'SCDG', 'SCDT', 'SCPG', 'SCPP', 'SCPT',
    ],
    'In-Situ Loading, Deformation & Geophysics': [
        'DMDG', 'DMDT', 'DMTG', 'DMTP', 'DMTT', 'DMTZ', 'ISTA', 'ISTG',
        'ISTR', 'ISTS', 'ITCH', 'PLTG', 'PLTT', 'PMMC', 'PMMD', 'PMMG',
        'PMTD', 'PMTG', 'PMTL', 'PMTP', 'PMTZ', 'WGPG', 'WGPT',
    ],
    'Groundwater & Monitoring': [
        'FGHG', 'FGHI', 'FGHS', 'FGHT', 'IPRG', 'IPRT', 'ISAG', 'ISAT',
        'MOND', 'MONG', 'MONS', 'PIPE', 'PUMG', 'PUMT', 'WSTD', 'WSTG',
    ],
    'Soil Classification & Index Tests': [
        'GRAG', 'GRAT', 'LDEN', 'LFCN', 'LLIN', 'LLPL', 'LNMC', 'LPDN',
        'LSLT', 'LSWL', 'LTCH', 'RELD', 'SUCT',
    ],
    'Soil Strength, Consolidation & Deformation Tests': [
        'CONG', 'CONS', 'CTRC', 'CTRD', 'CTRG', 'CTRP', 'CTRS', 'ESCG',
        'ESCT', 'LDYN', 'LPEN', 'LUCT', 'LVAN', 'PTST', 'RESC', 'RESD',
        'RESG', 'RESP', 'RESS', 'SHBG', 'SHBT', 'TREG', 'TRET', 'TRIG',
        'TRIT',
    ],
    'Rock Testing': [
        'RCAG', 'RCAT', 'RCCV', 'RDEN', 'RPLT', 'RSCH', 'RSHR', 'RTEN',
        'RUCS', 'RWCO',
    ],
    'Aggregate, Earthworks & Pavement Materials': [
        'AAVT', 'ACVT', 'AELO', 'AFLK', 'AIVT', 'ALOS', 'APSV', 'ARTW',
        'ASDI', 'ASNS', 'AWAD', 'CBRG', 'CBRP', 'CBRT', 'CMPG', 'CMPT',
        'FRST', 'LSTG', 'LSTT', 'MCVG', 'MCVT', 'TNPC',
    ],
    'Geoenvironmental & Chemical Testing': [
        'ELRG', 'ERES', 'GCHM', 'IFID', 'IPID', 'IRDX', 'IRES', 'LRES',
    ],
    'Data Transfer & Dictionaries': [
        'ABBR', 'DICT', 'FILE', 'TRAN', 'TYPE', 'UNIT',
    ],
}

FAMILY_OF: dict[str, str] = {
    code: fam for fam, codes in _FAMILY_CODES.items() for code in codes
}


def family(code: str) -> str:
    """The catalogue family a group belongs to."""
    return FAMILY_OF[code]
