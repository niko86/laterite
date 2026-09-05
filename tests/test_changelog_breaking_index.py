"""The breaking-changes index is the `breaking` flag, rendered.

"Pre-1.0 a minor may break you — read the changelog before upgrading" is the
compatibility promise this project makes. A promise that sends people to a 40KB
file only pays out if the file answers the question they arrived with, which is
not "what changed" but "does the step I am about to take break me".

The index at the top of `CHANGELOG.md` is that answer, and the reason it is
*generated* rather than written is that a hand-maintained second list is exactly
the thing that goes quietly out of date — the same argument that made
`observations.json` and `changelog.json` the SSOTs they are. So what these tests
pin is not the wording:

  * every entry declaring itself breaking is in the index (it cannot be flagged
    and omitted, because the index IS the flag);
  * every anchor the index emits lands on a heading the same render produced —
    the failure a reader meets as a link that scrolls nowhere;
  * the era clause is computed, not asserted, so it does not go false at 1.0
    while the advisor beside it keeps getting the answer right.

`tests/test_changelog_advisor.py` covers the other half of the same flag: which
bump it earns, and the bidirectional cross-check against the `**Breaking:**`
prose marker.
"""

from __future__ import annotations

import re
from pathlib import Path

from _tools import load_tool

REPO = Path(__file__).resolve().parents[1]


gc = load_tool("gen_changelog")

#: An index row: `| [0.9.0](#090--2026-07-30) | … |`
_ROW = re.compile(
    r"^\| \[(?P<label>[^\]]+)\]\(#(?P<anchor>[^)]+)\) \| (?P<what>.+) \|$"
)


def _index(rendered: str) -> list[re.Match[str]]:
    """The index's rows, parsed out of a full render."""
    body = rendered.split("## Breaking changes", 1)[1].split("\n## ", 1)[0]
    return [m for m in (_ROW.match(ln) for ln in body.splitlines()) if m]


def _e(text: str, breaking: bool = False) -> dict:
    entry: dict = {"text": text}
    if breaking:
        entry["breaking"] = True
    return entry


def _data(
    *, unreleased: dict | None = None, releases: list[dict] | None = None
) -> dict:
    return {
        "repo": "niko86/laterite",
        "unreleased": {c: (unreleased or {}).get(c, []) for c in gc.CATEGORIES},
        "releases": releases
        if releases is not None
        else [{"version": "0.10.0", "date": "2026-08-02"}],
    }


# --- completeness -----------------------------------------------------------


def test_the_real_changelog_indexes_every_declared_break() -> None:
    """The invariant that makes the promise satisfiable: flagged means listed.

    Falsify by adding `"breaking": true` to any entry in `changelog.json`
    without regenerating — the `--check` drift gate fails, and so does this.
    """
    data = gc._load()
    declared = gc._breaking_count(data.get("unreleased", {})) + sum(
        gc._breaking_count(rel) for rel in data.get("releases", [])
    )
    assert declared, "no entry declares itself breaking — this test proves nothing"
    assert len(_index(gc.render(data))) == declared


def test_the_committed_changelog_carries_the_index() -> None:
    """Asserted on the committed file, not the render, so deleting the section
    fails here rather than silently dropping it from every other check."""
    committed = (REPO / "CHANGELOG.md").read_text(encoding="utf-8")
    assert "## Breaking changes" in committed
    assert _index(committed), "the section is present but lists nothing"


def test_a_row_names_the_version_it_broke_in() -> None:
    data = _data(
        releases=[
            {
                "version": "0.9.0",
                "date": "2026-07-30",
                "changed": [_e("**Breaking: the door moved.**", True)],
            }
        ]
    )
    (row,) = _index(gc.render(data))
    assert row["label"] == "0.9.0"


def test_an_unreleased_break_is_listed_as_unreleased() -> None:
    """Queued breaks are in the index too — they are what the next minor will
    be — but labelled so nobody reads one as shipped."""
    data = _data(unreleased={"changed": [_e("**Breaking: the door moves.**", True)]})
    (row,) = _index(gc.render(data))
    assert row["label"] == "Unreleased"
    assert row["anchor"] == "unreleased"


def test_nothing_breaking_says_so_out_loud() -> None:
    """An empty index must state its emptiness. A section that vanishes when it
    has nothing to say is indistinguishable from one nobody rendered."""
    rendered = gc.render(_data(unreleased={"added": [_e("A new flag.")]}))
    assert _index(rendered) == []
    assert "None — no release has declared a breaking change." in rendered


# --- the anchors ------------------------------------------------------------


def test_every_anchor_lands_on_a_heading_the_same_render_produced() -> None:
    """The reader-visible failure: a row whose link scrolls nowhere.

    Both sides come from one render, so this fires if the heading format is
    changed without `_heading`/`_anchor` moving with it.
    """
    rendered = gc.render(gc._load())
    headings = {
        gc._anchor(ln.removeprefix("## "))
        for ln in rendered.splitlines()
        if ln.startswith("## ")
    }
    missing = [m["anchor"] for m in _index(rendered) if m["anchor"] not in headings]
    assert not missing, f"index anchors with no matching heading: {missing}"


def test_anchor_follows_githubs_slug_rules() -> None:
    """Pinned by hand, because the rules belong to the renderer rather than to us.

    The double hyphen is the load-bearing case: GitHub does not collapse runs of
    spaces, so dropping the em dash between `]` and the date leaves two. It reads
    like a typo, and a "fix" to a single hyphen would break every link in the
    index at once — python-markdown's slugify is the one that collapses, and it
    does not render this file.
    """
    assert gc._anchor("[0.10.0] — 2026-08-02") == "0100--2026-08-02"
    assert gc._anchor("[Unreleased]") == "unreleased"
    assert gc._anchor("Breaking changes") == "breaking-changes"


def test_the_heading_and_its_anchor_come_from_one_string() -> None:
    """`_heading` is what both the `## ` line and the anchor are built from."""
    rel = {"version": "1.2.3", "date": "2027-01-05"}
    rendered = gc.render(_data(releases=[{**rel, "changed": [_e("x")]}]))
    assert f"## {gc._heading(rel)}" in rendered


# --- the row text -----------------------------------------------------------


def test_the_row_quotes_the_entrys_own_headline() -> None:
    """Not a paraphrase: a summary written twice can disagree with itself."""
    data = _data(
        unreleased={
            "changed": [
                _e(
                    "**Breaking: `diff` takes an options object.** Followed by "
                    "several paragraphs of rationale that do not belong in a table.",
                    True,
                )
            ]
        }
    )
    (row,) = _index(gc.render(data))
    assert row["what"] == "`diff` takes an options object."


def test_a_bare_breaking_marker_falls_back_to_the_first_sentence() -> None:
    """The 0.7.0 / 0.8.0 house style: summary first, `**Breaking:**` mid-entry.

    Stripping the redundant prefix off `**Breaking:**` leaves nothing, so an
    empty cell was the natural bug here.
    """
    data = _data(
        unreleased={
            "changed": [_e("**Breaking:** callers must pass a flag. More.", True)]
        }
    )
    (row,) = _index(gc.render(data))
    assert row["what"] == "callers must pass a flag."


def test_a_pipe_in_an_entry_cannot_end_the_cell() -> None:
    data = _data(
        unreleased={"changed": [_e("**Breaking: `a | b` is refused.**", True)]}
    )
    (row,) = _index(gc.render(data))
    assert row["what"] == r"`a \| b` is refused."


# --- the era clause ---------------------------------------------------------


def test_the_era_clause_is_computed_not_asserted() -> None:
    """Pre-1.0 a break takes the MINOR; past 1.0 it takes the MAJOR. Hard-coding
    the first would leave the promise's own page contradicting the advisor beside
    it on the day 1.0 ships."""
    entry = {"changed": [_e("**Breaking: it moved.**", True)]}
    pre = gc.render(
        _data(releases=[{"version": "0.10.0", "date": "2026-08-02", **entry}])
    )
    post = gc.render(
        _data(releases=[{"version": "1.2.3", "date": "2027-01-05", **entry}])
    )
    assert "takes the **MINOR**" in pre and "**MAJOR**" not in pre
    assert "takes the **MAJOR**" in post and "**MINOR**" not in post
