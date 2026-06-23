"""laterite.registry: read-only PyO3 surface over Rust's Registry.

Smoke-tests the Python-side mirror of the Rust dictionary: every
group / heading reachable from the typed surface lines up with what
the Rust crate's `to_groups_json` returns.
"""

from __future__ import annotations

import pytest
from laterite import registry as latreg


def test_groups_loaded() -> None:
    assert isinstance(latreg.GROUPS, dict)
    # The consolidated UNION of the official 4.0.3-4.2 dictionary (~174 groups).
    assert len(latreg.GROUPS) > 150, f"expected the full official union, got {len(latreg.GROUPS)}"
    for code in ("PROJ", "LOCA", "SAMP", "CPTG"):  # CPTG was missing from the old curated subset
        assert code in latreg.GROUPS


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


# `test_matches_ags5_models` retired with F2c-4: ags5-models gone,
# so there's no second registry to cross-check against. The single
# source of truth is `rust-packages/laterite-ags4-core/data/ags_dictionary.json`;
# `tests/test_pyi_stubs_match_generator.py` catches drift between
# that JSON and the typed-graph .pyi.
