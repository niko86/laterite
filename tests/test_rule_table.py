"""The validator README's rule table is the catalogue's, not a second copy.

The README ships to crates.io. Its rule table was hand-written and drifted until
about ten rows described a different rule from the one numbered, with nothing
failing. `tools/gen_rule_table.py` now renders it from `rules_meta.json`, and
these hold the gate itself: that the committed README matches, and that the gate
goes red on the two edits that produced the original defect — a row reworded by
hand, and a row for a rule the catalogue does not have.
"""

from __future__ import annotations

import json

import pytest
from _tools import load_tool

gen = load_tool("gen_rule_table")


def test_committed_readme_matches_the_catalogue() -> None:
    assert gen.README.read_text(encoding="utf-8") == gen.render()


def test_every_catalogue_rule_has_exactly_one_row() -> None:
    rules = json.loads(gen.CATALOGUE.read_text(encoding="utf-8"))["rules"]
    table = gen.render_table(rules)
    assert [row.split(" | ")[0].removeprefix("| ") for row in table[2:]] == [
        r["rule"] for r in rules
    ]


def _between_markers(md: str) -> list[str]:
    lines = md.split("\n")
    b = next(
        i for i, ln in enumerate(lines) if ln.startswith("<!-- BEGIN GENERATED: rules")
    )
    e = next(
        i for i, ln in enumerate(lines) if ln.startswith("<!-- END GENERATED: rules")
    )
    return lines[b + 1 : e]


@pytest.mark.parametrize(
    "edit",
    [
        # the original defect: a row that says something the catalogue does not
        lambda md: md.replace(
            "| 4 | Field Count and GROUP Row Format |", "| 4 | Quoting |"
        ),
        # and its sibling: a row for a rule the catalogue has no entry for
        lambda md: md.replace("| 13 |", "| 12 | DICT cross-checks |\n| 13 |"),
    ],
)
def test_a_hand_edit_inside_the_markers_is_undone_by_render(edit) -> None:
    good = gen.README.read_text(encoding="utf-8")
    bad = edit(good)
    assert bad != good, "the fixture edit did not apply — the table moved"
    rules = json.loads(gen.CATALOGUE.read_text(encoding="utf-8"))["rules"]
    assert gen.splice(bad, gen.render_table(rules)) == good
    assert _between_markers(bad) != _between_markers(good)


def test_a_readme_that_lost_a_marker_is_refused() -> None:
    md = gen.README.read_text(encoding="utf-8").replace(
        "<!-- END GENERATED: rules -->\n", ""
    )
    with pytest.raises(ValueError, match="expected one"):
        gen.splice(md, [])
