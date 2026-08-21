"""The input half of the docs' runnable guarantee — `census_code_fences`.

`gen_doc_outputs.py` gates the `text` block showing what an example PRINTS.
This covers the mirror added for the fence showing what a reader RUNS: every
code fence is an include, an inline snippet in a language meant to run, an
explicit opt-out with a reason, or prose.

Each assertion below was A/B'd against the real corpus before being written —
removing a reason and removing a marker each turn the live gate red. These pin
that behaviour so it cannot quietly stop happening.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import pytest

ROOT = Path(__file__).resolve().parents[1]

_spec = importlib.util.spec_from_file_location(
    "gen_doc_outputs", ROOT / "tools" / "gen_doc_outputs.py"
)
assert _spec and _spec.loader
gen_doc_outputs = importlib.util.module_from_spec(_spec)
sys.modules["gen_doc_outputs"] = gen_doc_outputs
_spec.loader.exec_module(gen_doc_outputs)

census = gen_doc_outputs.census_code_fences


def test_an_include_is_gated_elsewhere_and_raises_nothing() -> None:
    md = '```python\n--8<-- "python/ex01_read_typed.py:code"\n```\n'
    counts, problems = census(md, "p.md")
    assert counts["included"] == 1
    assert not problems


def test_an_inline_snippet_is_counted_not_faulted() -> None:
    """Inline is the WAITING state, not a failure.

    Until the per-surface runners land there is nothing to execute these with, so
    the gate reports them and moves on. Faulting here would force a marker onto
    every fence we actually intend to run, and those markers would have to come
    straight back off again.
    """
    counts, problems = census("```python\nprint(x)\n```\n", "p.md")
    assert counts["inline"] == 1
    assert not problems


def test_an_excluded_language_must_say_why() -> None:
    counts, problems = census("```bash\npip install laterite\n```\n", "p.md")
    assert counts["inline"] == 0
    assert len(problems) == 1
    assert "never executed" in problems[0]


def test_an_excluded_language_with_a_reason_passes() -> None:
    md = (
        "<!-- doc-code: skip — installs packages -->\n"
        "```bash\npip install laterite\n```\n"
    )
    counts, problems = census(md, "p.md")
    assert counts["skipped"] == 1
    assert not problems


def test_a_reasonless_opt_out_is_rejected() -> None:
    """An escape hatch whose use is not on the record is just a silence."""
    md = "<!-- doc-code: skip -->\n```bash\npip install laterite\n```\n"
    counts, problems = census(md, "p.md")
    assert counts["skipped"] == 1
    assert len(problems) == 1
    assert "no reason" in problems[0]


def test_an_em_dash_reason_and_a_hyphen_reason_both_count() -> None:
    """The output half accepts either; accepting only one would be a trap."""
    for dash in ("—", "-"):
        md = f"<!-- doc-code: skip {dash} because -->\n```bash\nnpm i laterite\n```\n"
        _, problems = census(md, "p.md")
        assert not problems, f"{dash!r} reason was rejected"


def test_a_text_fence_belongs_to_the_output_half() -> None:
    counts, problems = census("```text\nsome output\n```\n", "p.md")
    assert counts == {"included": 0, "inline": 0, "skipped": 0, "prose": 0}
    assert not problems


def test_an_unrunnable_language_is_prose_not_a_problem() -> None:
    counts, problems = census('```json\n{"a": 1}\n```\n', "p.md")
    assert counts["prose"] == 1
    assert not problems


def test_an_indented_fence_inside_a_tab_is_seen() -> None:
    """Cookbook pages put fences inside `pymdownx.tabbed` blocks, at an indent.

    A fence regex anchored at column zero would silently skip every tabbed page —
    which is most of the cookbook, and precisely where the known defects live.
    """
    md = '=== "Python"\n\n    ```bash\n    pip install laterite\n    ```\n'
    _, problems = census(md, "p.md")
    assert len(problems) == 1, "an indented excluded fence was not classified"


def test_the_problem_message_carries_a_line_number() -> None:
    md = "intro\n\nmore\n\n```bash\npip install laterite\n```\n"
    _, problems = census(md, "p.md")
    assert problems[0].startswith("p.md:5"), problems[0]


@pytest.mark.parametrize("lang", sorted(gen_doc_outputs.PAGE_SURFACE))
def test_every_runnable_language_maps_to_a_real_surface(lang: str) -> None:
    """A typo here would silently demote a language to prose."""
    assert gen_doc_outputs.PAGE_SURFACE[lang] in gen_doc_outputs.SURFACES


def test_the_two_language_sets_do_not_overlap() -> None:
    """A language in both would be runnable AND require an opt-out — unresolvable."""
    assert not (set(gen_doc_outputs.PAGE_SURFACE) & gen_doc_outputs.EXCLUDED_LANGS)
