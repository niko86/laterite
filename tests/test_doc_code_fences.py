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


# --- page programs: the executing half (#513 step 2) -------------------------


def test_a_page_program_concatenates_include_then_continuation() -> None:
    """The pairing IS the point.

    A continuation refers to names the include bound, so running them apart
    proves nothing — and running them together is what turns an unbound name
    into a NameError the gate can see.
    """
    md = (
        '```python\n--8<-- "python/ex01_read_typed.py:code"\n```\n'
        "\n```python\nprint(ags)\n```\n"
    )
    src, inline = gen_doc_outputs.page_program(md, "python")
    assert inline == 1
    assert "import laterite" in src, "the include's source was not pulled in"
    assert src.rstrip().endswith("print(ags)"), "the continuation must come last"


def test_document_order_is_preserved_across_a_tab_boundary() -> None:
    """A tabbed include and a later top-level fence are one program."""
    md = (
        '=== "Python"\n\n    ```python\n    first = 1\n    ```\n'
        "\n```python\nsecond = first + 1\n```\n"
    )
    src, _ = gen_doc_outputs.page_program(md, "python")
    assert src.index("first = 1") < src.index("second = first + 1")


def test_a_tabbed_fence_is_dedented() -> None:
    """Concatenating a four-space-indented fence verbatim is an IndentationError."""
    md = '=== "Python"\n\n    ```python\n    x = 1\n    ```\n'
    src, _ = gen_doc_outputs.page_program(md, "python")
    assert "\n    x = 1" not in f"\n{src}", "indent survived"
    compile(src, "<page>", "exec")  # the real assertion: it parses


def test_a_skipped_fence_is_left_out_of_the_program() -> None:
    md = "<!-- doc-code: skip — why -->\n```python\nboom(\n```\n"
    src, inline = gen_doc_outputs.page_program(md, "python")
    assert inline == 0
    assert not src.strip()


def test_a_page_of_only_includes_yields_no_inline() -> None:
    """`test_docs_examples.py` already runs those as files; re-running adds nothing."""
    md = '```python\n--8<-- "python/ex01_read_typed.py:code"\n```\n'
    _, inline = gen_doc_outputs.page_program(md, "python")
    assert inline == 0


def test_only_the_requested_language_is_collected() -> None:
    md = "```python\na = 1\n```\n\n```js\nconst b = 2;\n```\n"
    src, _ = gen_doc_outputs.page_program(md, "python")
    assert "const b" not in src


def test_resolve_include_extracts_the_named_section_only() -> None:
    src = gen_doc_outputs.resolve_include("python/ex01_read_typed.py:code")
    assert src is not None
    assert "--8<--" not in src, "the section markers leaked into the program"


def test_resolve_include_returns_none_for_a_missing_file() -> None:
    """Silently contributing nothing would make a broken include look green."""
    assert gen_doc_outputs.resolve_include("python/does_not_exist.py:code") is None


def test_the_delivery_fixture_exists_and_carries_geol() -> None:
    """The pages' narrative `delivery.ags` is seeded from this.

    `cookbook/sql-across-groups.md` documents a three-way join through GEOL; a
    fixture without it would make a working capability look broken.
    """
    text = gen_doc_outputs.DELIVERY_FIXTURE.read_text(encoding="utf-8")
    assert '"GROUP","GEOL"' in text
    assert "GEOL_GEOL" in text, "the join the docs select on needs this heading"
