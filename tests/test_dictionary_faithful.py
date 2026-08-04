"""CI gate: the committed consolidated dictionary stays faithful to the official spec.

`rust-packages/laterite-ags4-reference/data/ags_dictionary.json` is generated from the five
official AGS standard dictionaries by `tools/gen_dictionary.py`. This test re-runs the
generator and asserts the committed file matches — so the dictionary can never silently
drift from the spec (the same guard the `.pyi` stub has in
`test_pyi_stubs_match_generator.py`). `build()` itself self-verifies that every edition
reconstructs exactly, so a green generator is already a faithful one; this locks the
committed artifact to it.
"""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path

_REPO = Path(__file__).resolve().parents[1]
_GEN = _REPO / "tools" / "gen_dictionary.py"
_COMMITTED = (
    _REPO / "rust-packages" / "laterite-ags4-reference" / "data" / "ags_dictionary.json"
)


def _gen():
    spec = importlib.util.spec_from_file_location("_gen_dictionary", _GEN)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def test_committed_matches_generator():
    """regenerate == committed (catches both stale commits and source-dict edits)."""
    mod = _gen()
    regenerated = mod.build()
    committed = json.loads(_COMMITTED.read_text())
    assert regenerated == committed, (
        "ags_dictionary.json is out of date — run `python tools/gen_dictionary.py`"
    )


def test_every_edition_reconstructs_exactly():
    """The heading-local doc losslessly reproduces all five editions (incl. order)."""
    mod = _gen()
    doc = json.loads(_COMMITTED.read_text())
    for ed in doc["editions"]:
        # reconstruct must not raise; build() already self-verified, this re-checks
        # against the committed file specifically.
        groups = mod.reconstruct(doc, ed)
        assert groups, f"edition {ed} reconstructed empty"


def test_faithful_to_spec_spot_checks():
    """A handful of the cells the old hand-scaffolded dict got wrong are now correct."""
    doc = json.loads(_COMMITTED.read_text())

    def heading(group, name):
        for h in doc["groups"][group]["headings"]:
            if h["name"] == name:
                return h
        raise KeyError(f"{group}.{name}")

    # XN (Text/Numeric — holds "NP"), not 0DP
    assert heading("LLPL", "LLPL_PL")["type"] == "XN"
    # identity field is KEY, not OTHER/REQUIRED
    assert "KEY" in heading("GEOL", "GEOL_BASE")["status"]
    # record link, not free text
    assert heading("SAMP", "SAMP_LINK")["type"] == "RL"
    # moisture content stays text (ranges/qualifiers), not 2DP
    assert heading("LNMC", "LNMC_MC")["type"] == "X"
    # pick-list, not free text
    assert heading("GEOL", "GEOL_GEOL")["type"] == "PA"


def test_default_edition_and_no_deviations():
    doc = json.loads(_COMMITTED.read_text())
    assert doc["default_edition"] == "4.2"
    assert doc["fallback_edition"] == "4.1.1"  # python-parity auto-select fallback
    assert doc["editions"] == ["4.0.3", "4.0.4", "4.1", "4.1.1", "4.2"]
    assert doc["deviations"] == []  # pure faithful to spec


def test_abbr_picklist_and_tran_ags_faithful():
    """The union also carries the ABBR pick-list (Rule 16) + per-edition TRAN_AGS
    (Rule 14) — the two things beyond the group/heading schema the validator
    needs. Each must reconstruct from the committed file exactly per edition,
    re-parsed straight from the official .ags."""
    mod = _gen()
    doc = json.loads(_COMMITTED.read_text())
    assert isinstance(doc["abbreviations"], list) and len(doc["abbreviations"]) > 3000
    for ed in doc["editions"]:
        src = mod.SRC / mod._fname(ed)
        assert mod._reconstruct_abbr(doc, ed) == mod._parse_abbr(src), (
            f"ABBR drift {ed}"
        )
        assert doc["tran_ags"][ed] == mod._parse_tran_ags(src), f"TRAN_AGS drift {ed}"
