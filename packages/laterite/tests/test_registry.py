"""laterite.registry: read-only PyO3 surface over Rust's Registry.

Smoke-tests the Python-side mirror of the Rust dictionary: every
group / heading reachable from the typed surface lines up with what
the Rust crate's `to_groups_json` returns.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest
from laterite import registry as latreg

# The dictionary JSON the union GROUPS + per-edition views are all generated from —
# its `editions` list is the authority for which editions the union spans.
_CORE_DICT = (
    Path(__file__).resolve().parents[3]
    / "rust-packages"
    / "laterite-ags4-reference"
    / "data"
    / "ags_dictionary.json"
)


def test_groups_is_read_only_but_still_a_dict() -> None:
    # GROUPS is a process-global singleton projected from the single-source
    # dictionary — sealed against mutation (the Python analogue of laterite-node's
    # Object.freeze on its own GROUPS), while STAYING an isinstance(dict) so the
    # type + the ty gate stay honest. Reads are unaffected; every mutator raises.
    assert isinstance(latreg.GROUPS, dict)
    assert latreg.GROUPS.get("NOPE") is None  # read surface intact
    for mutate in (
        lambda: latreg.GROUPS.__setitem__("XXXX", None),
        lambda: latreg.GROUPS.__delitem__("PROJ"),
        latreg.GROUPS.clear,
        lambda: latreg.GROUPS.update({"XXXX": None}),
        lambda: latreg.GROUPS.setdefault("XXXX", None),
        lambda: latreg.GROUPS.pop("PROJ"),
    ):
        with pytest.raises(TypeError, match="read-only"):
            mutate()
    # The registry survived every rejected mutation untouched.
    assert "XXXX" not in latreg.GROUPS
    assert "PROJ" in latreg.GROUPS


def test_groups_is_exactly_the_union_of_the_per_edition_codes() -> None:
    # GROUPS is DEFINED as the union across editions 4.0.3-4.2, so pin that exact
    # relationship — `set(GROUPS) == ⋃ per-edition codes` — rather than a `>150`
    # magic number that neither proves the union is complete nor catches a group
    # that leaks into GROUPS without belonging to any edition. This is the real
    # invariant the union codegen must uphold, and it drifts loudly if it breaks.
    assert isinstance(latreg.GROUPS, dict)
    editions = json.loads(_CORE_DICT.read_text(encoding="utf-8"))["editions"]
    union = set().union(
        *({g["code"] for g in latreg.dictionary(ed)["groups"]} for ed in editions)
    )
    assert set(latreg.GROUPS) == union
    # A couple of anchors so a total collapse (empty union == empty GROUPS) still fails.
    for code in ("PROJ", "LOCA", "SAMP", "CPTG"):
        assert code in latreg.GROUPS


def test_dictionary_per_edition() -> None:
    # The per-edition STANDARD dictionary accessor (#294 F#6) — distinct from the
    # union GROUPS. Shape mirrors the browser/Node dictionary().
    d = latreg.dictionary("4.2")
    assert d["ags_edition"] == "4.2"
    assert isinstance(d["groups"], list) and d["groups"]
    proj = next(g for g in d["groups"] if g["code"] == "PROJ")
    assert proj["contents"]  # the group's standard description
    h0 = proj["headings"][0]
    assert h0["name"] == "PROJ_ID"
    assert {"name", "status", "type", "description"} <= set(
        h0
    )  # `type`, not `ags_type`

    # Editions genuinely differ (4.0.3 has fewer groups than 4.2).
    assert len(latreg.dictionary("4.0.3")["groups"]) < len(d["groups"])
    # None / "auto" fall back to the default edition; both agree.
    assert (
        latreg.dictionary()["ags_edition"] == latreg.dictionary("auto")["ags_edition"]
    )


def test_dictionary_rejects_unknown_edition() -> None:
    with pytest.raises(ValueError, match="unknown edition"):
        latreg.dictionary("9.9")


def test_abbreviations_flat_shape() -> None:
    rows = latreg.abbreviations("4.1.1")
    assert isinstance(rows, list) and rows
    assert all(set(r) == {"heading", "code", "description"} for r in rows)
    keys = [(r["heading"], r["code"]) for r in rows]
    assert keys == sorted(keys), "not ordered by heading then code"
    # The default shape is flat, and None/"auto" resolve like dictionary().
    assert latreg.abbreviations("4.1.1", shape="flat") == rows
    assert latreg.abbreviations() == latreg.abbreviations("auto")


def test_abbreviations_nested_round_trips_to_flat() -> None:
    flat = latreg.abbreviations("4.1.1")
    nested = latreg.abbreviations("4.1.1", shape="nested")
    assert isinstance(nested, dict)
    back = [
        {"heading": h, "code": c["code"], "description": c["description"]}
        for h, codes in nested.items()
        for c in codes
    ]
    assert back == flat


def _desc(edition: str, heading: str, code: str) -> str | None:
    return next(
        (
            r["description"]
            for r in latreg.abbreviations(edition)
            if r["heading"] == heading and r["code"] == code
        ),
        None,
    )


def test_abbreviations_are_per_edition() -> None:
    # CBRP_END "BASE" is bundled for 4.2 only.
    assert _desc("4.2", "CBRP_END", "BASE") is not None
    assert _desc("4.1.1", "CBRP_END", "BASE") is None
    # A per-edition description override: 4.1.1 spells this lower-case.
    assert _desc("4.1.1", "ELRG_CODE", "100-75-4") == "n-nitrosopiperidine"
    assert _desc("4.2", "ELRG_CODE", "100-75-4") == "n-Nitrosopiperidine"


def test_abbreviations_keep_case_only_pairs() -> None:
    codes = {
        r["code"] for r in latreg.abbreviations("4.1.1") if r["heading"] == "PTST_TYPE"
    }
    assert {"CONSTANT HEAD", "Constant Head"} <= codes


def test_abbreviations_reject_bad_edition_and_shape() -> None:
    # The same error type dictionary() raises for the same edition.
    with pytest.raises(ValueError, match="unknown edition"):
        latreg.dictionary("9.9")
    with pytest.raises(ValueError, match="unknown edition"):
        latreg.abbreviations("9.9")
    for bad in ("Flat", "tree", None, 1):
        with pytest.raises(ValueError, match="flat\\|nested"):
            latreg.abbreviations("4.2", shape=bad)  # type: ignore[call-overload]


@pytest.mark.parametrize("edition", json.loads(_CORE_DICT.read_text())["editions"])
def test_every_listed_abbreviation_is_accepted_by_the_o43_fyi(edition: str) -> None:
    # The API and the validator must share one data source: declaring every
    # listed code in ABBR raises no "not a recognised standard abbreviation"
    # FYI (O-43) — and no description-drift FYI either, since each row carries
    # the standard description for this edition.
    import laterite

    def q(v: str) -> str:
        return '"' + v.replace('"', '""') + '"'

    lines = [
        '"GROUP","ABBR"',
        '"HEADING","ABBR_HDNG","ABBR_CODE","ABBR_DESC"',
        '"UNIT","","",""',
        '"TYPE","X","X","X"',
    ]
    lines += [
        f'"DATA",{q(r["heading"])},{q(r["code"])},{q(r["description"])}'
        for r in latreg.abbreviations(edition)
    ]
    rep = laterite.validate(
        text="\r\n".join(lines) + "\r\n", fyi=True, dict_version=edition
    )
    fyi16 = [
        f
        for rule, items in rep.by_rule().items()
        if "Rule 16" in rule
        for f in items
        if f.get("severity") == "fyi"
    ]
    assert fyi16 == []


def test_group_descriptor_shape() -> None:
    proj = latreg.GROUPS["PROJ"]
    assert isinstance(proj, latreg.GroupDescriptor)
    assert proj.code == "PROJ"
    assert proj.parent is None  # root
    assert proj.table == "g_proj"
    assert proj.view == "v_proj"
    assert proj.headings, "PROJ must have headings"
    h0 = proj.headings[0]
    assert isinstance(h0, latreg.Heading)
    assert h0.name == "PROJ_ID"
    assert h0.status == "KEY+REQUIRED"  # official status; still a KEY (below)
    assert h0.is_key
    assert h0.py_name == "proj_id"


def test_key_vs_non_key_partitioning() -> None:
    # SAMP carries the LOCA+SAMP cascaded KEY tuple; non-keys are the
    # SAMP-specific properties.
    samp = latreg.GROUPS["SAMP"]
    key_names = {h.name for h in samp.key_headings}
    non_key_names = {h.name for h in samp.non_key_headings}
    assert "SAMP_TOP" in key_names
    assert "SAMP_ID" in key_names
    assert "LOCA_ID" in key_names  # inherited KEY
    assert key_names.isdisjoint(non_key_names)


def test_ancestor_chain_root_down_order() -> None:
    # LLPL is a deep group; chain returned in [code, ..., root] order.
    chain = latreg.ancestor_chain("LLPL")
    assert chain[0] == "LLPL"
    assert chain[-1] == "PROJ"
    # LLPL → SAMP → LOCA → PROJ (per the dictionary)
    assert chain == ["LLPL", "SAMP", "LOCA", "PROJ"]


def test_ancestor_chain_root_group() -> None:
    assert latreg.ancestor_chain("PROJ") == ["PROJ"]


def test_ancestor_chain_unknown_code_raises() -> None:
    with pytest.raises(ValueError, match="unknown group code"):
        latreg.ancestor_chain("ZZZZ")


def test_inherited_key_names_samp_inherits_loca_id() -> None:
    inh = latreg.inherited_key_names("SAMP")
    assert "LOCA_ID" in inh, f"SAMP must inherit LOCA_ID from LOCA, got {inh}"
    # SAMP's own KEYs (SAMP_TOP, SAMP_ID etc.) are NOT inherited
    assert "SAMP_TOP" not in inh
    assert "SAMP_ID" not in inh


def test_inherited_key_names_root_is_empty() -> None:
    assert latreg.inherited_key_names("PROJ") == set()


# `test_matches_models` retired with F2c-4: the old model-gen path is
# gone, so there's no second registry to cross-check against. The single
# source of truth is `rust-packages/laterite-ags4-reference/data/ags_dictionary.json`;
# `tests/test_pyi_stubs_match_generator.py` catches drift between
# that JSON and the typed-graph .pyi.
