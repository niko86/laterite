"""The demo state sweep's refusals, which no real run can reach (#660, #673).

`tools/gen_demo_state_map.py` writes the map only when it can explain every
difference in it, and the python-count table only when the lookup it feeds is a
function. Those guards are unreachable from a live sweep against today's engines
— the map is clean and the one collision resolves, which is the whole point — so
nothing else would ever execute them, and a guard that has never run is a guard
nobody knows the state of.

Both fail the same way if they break: silently. An unexplained difference gets
written to the map and the demo says nothing about it, which on the page is
indistinguishable from a state that has nothing to explain.

Stdlib only, so this runs in the buildless subset beside the other tools tests.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest
from _tools import load_tool

REPO = Path(__file__).resolve().parents[1]
MAP = REPO / "web" / "landing" / "demo" / "state-map.json"


@pytest.fixture(scope="module")
def gen():
    return load_tool("gen_demo_state_map")


@pytest.fixture(scope="module")
def committed() -> dict:
    return json.loads(MAP.read_text(encoding="utf-8"))


def test_the_committed_map_needs_no_excuses(gen, committed):
    """The positive control. Without it the two guards below could pass because
    `build_notes` reports a gap for everything."""
    notes, gaps = gen.build_notes(committed)
    assert gaps == []
    assert notes["notes"], "the map has differences, so it must have notes"


def test_a_difference_shape_with_no_reader_note_stops_the_run(
    gen, committed, monkeypatch
):
    """The demo renders from these notes. A shape that reaches the page with no
    note renders as silence, and silence is what a state with NO difference
    looks like. Notes are built per triaged COMPONENT of a state's shape since
    #1024, so the gap is a triaged entry with no `reader`, carried by a state."""
    shape = (("Warning (Related to Rule 99)",), ())
    monkeypatch.setitem(
        gen.KNOWN, shape, {"triage": "O-99", "why": "invented for this test"}
    )
    doc = json.loads(json.dumps(committed))
    doc["states"].append(
        {
            "id": "made-up",
            "reached_by": {},
            "difference": {
                "rust_only": list(shape[0]),
                "python_only": [],
                "triage": "O-99",
            },
        }
    )
    _, gaps = gen.build_notes(doc)
    assert any("O-99" in g for g in gaps), gaps


def test_a_composing_finding_is_peeled_off_and_nothing_else_is(gen):
    """#1024: O-61's did-you-mean rides on top of every other difference, so a
    shape carrying it is explained by peeling it off and matching the rest
    exactly. Only an entry marked `composes` may be peeled, so two ordinary
    shapes never combine into one that nothing triaged."""
    o61 = ("FYI (Related to Rule 16)",)
    o52 = ("Warning (Related to Rule 10c)",)
    assert gen.components(((*o61, *o52), ())) == [(o61, ()), (o52, ())]
    assert gen.triage_of(((*o61, *o52), ())) == "O-61 + O-52"
    assert gen.components((o61, ())) == [(o61, ())]
    # O-52 and O-45's warning are each known alone; together they are not.
    assert gen.components(((*o52, "Warning (Related to Rule 14)"), ("FYI",))) is None
    # And the rest must still match whole: O-61 cannot excuse an unknown rule.
    assert gen.components(((*o61, "Warning (Related to Rule 99)"), ())) is None


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


def test_every_signature_in_the_committed_map_carries_a_python_total(gen, committed):
    """The positive control for the three below. Without it they could all pass
    on a table that was empty for some unrelated reason."""
    counts, problems = gen.build_python_counts(committed)
    assert problems == []
    assert counts["signatures"], "the map has states, so it must have signatures"
    totals = {e["signature"]: e["python"] for e in counts["signatures"]}
    assert len(totals) == len(counts["signatures"]), "a signature is listed twice"


def test_the_one_real_collision_resolves_on_a_cell(gen, committed):
    """Not a hypothetical: clearing `PROJ_ID` or any of several `TRAN` cells gives
    laterite the same findings, and only a blank `TRAN_AGS` earns python-ags4's
    extra FYI (O-53). The page has to be able to tell those apart with one
    string comparison, because the alternative is a second validator in the
    browser disagreeing with the first."""
    counts, _ = gen.build_python_counts(committed)
    resolved = [e for e in counts["signatures"] if "when_cell_is" in e]
    assert len(resolved) == 1, [e["signature"] for e in resolved]
    (entry,) = resolved
    (override,) = entry["when_cell_is"]
    assert override["heading"] == "TRAN_AGS"
    assert override["value"] == ""
    assert override["python"] != entry["python"], (
        "an override that agrees with the default resolves nothing"
    )


def test_a_second_collision_stops_the_run(gen, committed):
    """A signature with two python answers is not a function, and a page cannot
    show a number it has two of. One that no single cell can tell apart must
    fail here rather than reach the demo, which would show one of the two and
    be wrong about the other."""
    doc = json.loads(json.dumps(committed))
    twin = json.loads(json.dumps(doc["states"][0]))
    twin["id"] = "invented-twin"
    # Same laterite signature, a different python total, and reached by a lever
    # that is not a cell edit — so there is nothing exact to discriminate on.
    twin["lever"] = "deleteGroup"
    twin["reached_by"] = {"group": "LLPL"}
    twin["python_rule_counts"] = {"AGS Format Rule 99": 99}
    doc["states"].append(twin)
    _, problems = gen.build_python_counts(doc)
    assert any("invented-twin" in p for p in problems), problems


def test_a_collision_hidden_behind_a_shared_cell_stops_the_run(gen, committed):
    """The subtler half. A minority state reached by a cell edit the MAJORITY
    also reaches resolves nothing: the override would fire on both, so the page
    would show the minority's number for every one of them."""
    doc = json.loads(json.dumps(committed))
    victim = next(st for st in doc["states"] if st["lever"] == "setCell")
    twin = json.loads(json.dumps(victim))
    twin["id"] = "invented-shared-cell"
    twin["python_rule_counts"] = {"AGS Format Rule 99": 99}
    doc["states"].append(twin)
    _, problems = gen.build_python_counts(doc)
    assert any("invented-shared-cell" in p for p in problems), problems


def test_a_laterite_fyi_is_counted_not_refused(gen, committed):
    """Until #1024 the demo left FYI off and a state raising a laterite FYI
    stopped the run. The demo now asks for FYI, so an FYI is part of what the
    page holds: it keys the lookup rather than refusing it."""
    doc = json.loads(json.dumps(committed))
    state = doc["states"][0]
    state["rust_rule_counts"]["FYI (Related to Rule 99)"] = 1
    counts, problems = gen.build_python_counts(doc)
    assert problems == [], problems
    assert any(
        "FYI (Related to Rule 99)=1" in e["signature"] for e in counts["signatures"]
    )


def test_the_signature_is_what_the_demo_can_see(gen):
    """The signature has to be computable from the findings the page holds, or
    the lookup keys off something the browser does not have. The page asks for
    FYI since #1024, so every tier is in it."""
    assert (
        gen.laterite_signature(
            {
                "AGS Format Rule 8": 2,
                "FYI (Related to Rule 16)": 1,
                "AGS Format Rule 1": 1,
            }
        )
        == "AGS Format Rule 1=1|AGS Format Rule 8=2|FYI (Related to Rule 16)=1"
    )


def test_a_collision_needing_two_overrides_stops_the_run(gen, committed):
    """A resolved collision can still be wrong. The browser takes the first
    override that MATCHES, and two overrides name two different cells — so a
    delivery holding both shows whichever was written first, silently. That
    delivery is one edit away from each of them, so it is reachable the moment
    there are two. The collision that exists today needs exactly one override,
    which is why it is safe."""
    doc = json.loads(json.dumps(committed))
    victim = next(
        st
        for st in doc["states"]
        if st["lever"] == "setCell" and "value" in st["reached_by"]
    )
    for n in range(2):
        twin = json.loads(json.dumps(victim))
        twin["id"] = f"invented-two-overrides-{n}"
        twin["reached_by"] = {**victim["reached_by"], "heading": f"MADE_UP_{n}"}
        twin["python_rule_counts"] = {"AGS Format Rule 99": 99}
        doc["states"].append(twin)
    _, problems = gen.build_python_counts(doc)
    assert any("hold both at once" in p for p in problems), problems
