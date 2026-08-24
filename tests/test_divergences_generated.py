"""`divergences.md` is rendered from `observations.json`, and cannot outlive it.

The page it replaced was hand-written against a generated, gated SSOT — the one
shape in this repo that has drifted every single time. It told readers the
`--dict` override was "deferred" a release after laterite-dev#568 shipped it, and it never
gained O-49 or O-50.

These tests hold the two halves that a stale-file `--check` alone does not: that
membership is a decision recorded ON the record, and that the two ways a
generated page can silently lie — rendering a resolved record, or dropping a
record whose axis nobody renders — are hard failures rather than warnings.
"""

from __future__ import annotations

import copy
import importlib.util
import json
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parent.parent

_spec = importlib.util.spec_from_file_location(
    "gen_observations", ROOT / "tools" / "gen_observations.py"
)
assert _spec and _spec.loader
gen = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(gen)


@pytest.fixture
def data() -> dict:
    return json.loads(gen.JSON_PATH.read_text(encoding="utf-8"))


def _records(d: dict):
    return [o for s in d["sections"] for o in s["observations"]]


def _find(d: dict, oid: str) -> dict:
    return next(o for o in _records(d) if o["id"] == oid)


def test_page_on_disk_is_the_render(data: dict) -> None:
    assert gen.DIVERGENCES.read_text(encoding="utf-8") == gen.render_divergences(data)


def test_every_user_facing_record_reaches_the_page(data: dict) -> None:
    page = gen.render_divergences(data)
    live = [o["id"] for o in _records(data) if o.get("user_facing")]
    assert live, "no user-facing records — the page would render empty"
    missing = [oid for oid in live if f"**{oid}**" not in page]
    assert not missing, f"user_facing records absent from the page: {missing}"


def test_the_stated_count_is_the_rendered_count(data: dict) -> None:
    """The page's headline number is the one thing a reader quotes."""
    page = gen.render_divergences(data)
    live = [o for o in _records(data) if o.get("user_facing")]
    assert f"**{len(live)} of them change what you see.**" in page
    assert page.count("| **O-") == len(live)


def test_a_resolved_record_cannot_stay_on_the_page(data: dict) -> None:
    """The live defect #320 was filed for, in its general form.

    O-28 shipped in laterite-dev#568 and the page still called it deferred. Marking a record
    resolved while leaving it user-facing must fail, not render.
    """
    mutated = copy.deepcopy(data)
    _find(mutated, "O-2")["status"] = "superseded"
    with pytest.raises(SystemExit, match=r"O-2 carries both `user_facing` and a"):
        gen.render_divergences(mutated)


def test_a_relation_outside_the_closed_set_fails_rather_than_dropping_the_record(
    data: dict,
) -> None:
    """The section is derived from `relation`, so an unrecognised value has no
    section to render under and the record would vanish silently."""
    mutated = copy.deepcopy(data)
    _find(mutated, "O-2")["relation"]["python"] = "nowhere"
    with pytest.raises(SystemExit, match=r"unknown relation.python \['nowhere'\]"):
        gen.render_divergences(mutated)


def test_a_user_facing_record_without_a_relation_fails(data: dict) -> None:
    """Absent is the failure that goes quiet: there is nothing to derive from,
    and a record with no section is a record off the page."""
    mutated = copy.deepcopy(data)
    del _find(mutated, "O-2")["relation"]
    with pytest.raises(SystemExit, match=r"O-2 is user-facing but carries no"):
        gen.render_divergences(mutated)


def test_the_two_silences_are_not_the_same_fact(data: dict) -> None:
    """The regression this exists for, caught while designing the closed set.

    "python-ags4 says nothing where we report" (O-52) and "python-ags4 reports
    where we say nothing" (O-53) are opposite facts. A first draft gave them one
    `silent` value, which filed O-53 under "Checks laterite adds" — the exact
    inverse of what O-53 says. They must land in different sections.
    """
    o52, o53 = _find(data, "O-52"), _find(data, "O-53")
    assert o52["relation"]["python"] == "reports-nothing"
    assert o53["relation"]["python"] == "reports-additionally"
    assert gen.axis_of(o52) == "laterite-adds"
    assert gen.axis_of(o53) == "vs-python"


def test_the_section_is_derived_and_not_stored(data: dict) -> None:
    """One authored fact, not two that can disagree. `axis` was the second, and
    it disagreed with its own record's body for O-45."""
    for rec in _records(data):
        if uf := rec.get("user_facing"):
            assert "axis" not in uf, (
                f"{rec['id']} still stores an axis — the section derives from "
                "`relation`, and a stored copy can contradict it"
            )


def test_resolved_records_carry_what_resolved_them(data: dict) -> None:
    """A status with no pointer is an assertion the reader cannot follow up."""
    for rec in _records(data):
        if status := rec.get("status"):
            assert rec.get("resolved_by"), (
                f"{rec['id']} is {status} but does not say what resolved it"
            )


def test_membership_is_not_derivable_from_kind(data: dict) -> None:
    """Guards the reason `user_facing` is a field at all.

    If this ever starts failing because the two sets converged, the field is
    still right — but the comment justifying it needs rewriting, and a future
    reader would otherwise "simplify" it into a `kind == VARIANCE` rule that had
    been wrong all along. The old page's 19 rows and the 19 VARIANCE records
    matching was a coincidence, not a rule.
    """
    recs = _records(data)
    variance = {o["id"] for o in recs if o["kind"] == "VARIANCE"}
    live = {o["id"] for o in recs if o.get("user_facing")}
    assert live != variance, (
        "user_facing now equals the VARIANCE set — re-read the AXES comment "
        "before concluding the field is redundant"
    )
