"""The demo state sweep's two refusals, which no real run can reach (#660).

`tools/gen_demo_state_map.py` writes the map only when it can explain every
difference in it. Two of those guards are unreachable from a live sweep against
today's engines — the map is clean, which is the whole point — so nothing else
would ever execute them, and a guard that has never run is a guard nobody knows
the state of.

Both fail the same way if they break: silently. An unexplained difference gets
written to the map and the demo says nothing about it, which on the page is
indistinguishable from a state that has nothing to explain.

Stdlib only, so this runs in the buildless subset beside the other tools tests.
"""

from __future__ import annotations

import importlib.util
import json
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]
MAP = REPO / "web" / "landing" / "demo" / "state-map.json"


def _load():
    """Import `tools/gen_demo_state_map.py` — `tools/` is not a package."""
    spec = importlib.util.spec_from_file_location(
        "gen_demo_state_map", REPO / "tools" / "gen_demo_state_map.py"
    )
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules["gen_demo_state_map"] = mod
    spec.loader.exec_module(mod)
    return mod


@pytest.fixture(scope="module")
def gen():
    return _load()


@pytest.fixture(scope="module")
def committed() -> dict:
    return json.loads(MAP.read_text(encoding="utf-8"))


def test_the_committed_map_needs_no_excuses(gen, committed):
    """The positive control. Without it the two guards below could pass because
    `build_notes` reports a gap for everything."""
    notes, gaps = gen.build_notes(committed)
    assert gaps == []
    assert notes["notes"], "the map has differences, so it must have notes"


def test_a_difference_shape_with_no_reader_note_stops_the_run(gen, committed):
    """The demo renders from these notes. A shape that reaches the page with no
    note renders as silence, and silence is what a state with NO difference
    looks like."""
    doc = json.loads(json.dumps(committed))
    doc["difference_shapes"].append(
        {
            "rust_only": ["Warning (Related to Rule 99)"],
            "python_only": [],
            "states": 1,
            "triage": "O-99",
            "why": "invented for this test",
            "example": "made-up",
        }
    )
    _, gaps = gen.build_notes(doc)
    assert any("O-99" in g for g in gaps), gaps


def test_a_per_rule_count_difference_stops_the_run(gen, committed):
    """The third case #660 asked the demo to explain, and the one it cannot:
    `count_differences` is not a difference shape, no O-N covers one, and none
    has ever occurred. Rather than ship a render path that has never rendered,
    the generator refuses — so this cannot arrive unnoticed."""
    doc = json.loads(json.dumps(committed))
    assert doc["count_differences"] == [], "the fixture assumes a clean map"
    doc["count_differences"] = [
        {"state": "set-LOCA-0-LOCA_ID-blank", "rule": "AGS Format Rule 10a"}
    ]
    _, gaps = gen.build_notes(doc)
    assert any("AGS Format Rule 10a" in g for g in gaps), gaps
