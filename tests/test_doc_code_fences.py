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


# --- which runner owns a language, and the SQL split it hands over -----------


def test_every_runnable_language_names_its_runner() -> None:
    """A language classified as runnable but absent from the routing table is
    the failure this whole exercise is about: counted as covered, run by nobody.

    Adding a language to `PAGE_SURFACE` is what makes its fences `inline`
    instead of prose — so the same edit has to say who executes them, even if
    the answer is `None` (pending), which the census then reports out loud.
    """
    assert set(gen_doc_outputs.PAGE_RUNNER) == set(gen_doc_outputs.PAGE_SURFACE)


def test_the_sql_runner_is_a_file_that_exists() -> None:
    """The census prints this path at readers. A moved module would make it a
    confident pointer at nothing — the census's whole job is to be believable."""
    where = gen_doc_outputs.PAGE_RUNNER["sql"]
    assert where and (ROOT / where).is_file()


def test_run_pages_claims_only_the_languages_routed_to_it() -> None:
    """`--run-pages` shells out per surface; SQL runs in-process under pytest.

    If it claimed sql it would need the DuckDB CLI, which `pip install duckdb`
    does not ship — so every run would print a SKIP line for a language that is
    in fact gated, which reads as a hole rather than a handoff.
    """
    here = gen_doc_outputs.HERE
    mine = {k for k, v in gen_doc_outputs.PAGE_RUNNER.items() if v == here}
    assert mine == {"python"}
    assert gen_doc_outputs.PAGE_RUNNER["sql"] != here


def test_sql_split_flags_only_statements_that_ask_for_rows() -> None:
    """`INSTALL`/`LOAD` return nothing by definition, so counting them as
    zero-row findings would bury a real one in its own preamble."""
    stmts = gen_doc_outputs.sql_statements(
        "INSTALL laterite_ags4 FROM community;\nLOAD laterite_ags4;\nSELECT 1;\n"
    )
    assert [asks for _, asks in stmts] == [False, False, True]


def test_sql_split_drops_blanks_but_keeps_a_comment_led_statement() -> None:
    """A trailing `;` is not a statement; a comment ABOVE one does not hide it.

    Dropping any chunk starting with `--` is the obvious way to skip comments,
    and it silently swallowed the query underneath — which is how these pages
    introduce one. The comment stays in the executed text, because what runs
    should be what the page shows.
    """
    stmts = gen_doc_outputs.sql_statements("-- just a note\nSELECT 1;\n\n")
    assert len(stmts) == 1
    stmt, asks = stmts[0]
    assert "SELECT 1" in stmt and "-- just a note" in stmt
    assert asks, "the comment masked a row-returning query"


def test_sql_split_drops_a_comment_only_chunk() -> None:
    stmts = gen_doc_outputs.sql_statements("-- nothing to run here\n")
    assert stmts == []


def test_sql_split_sees_a_leading_cte_as_asking_for_rows() -> None:
    """`WITH … SELECT` is the shape a cookbook query takes; missing it would
    silently exempt exactly the queries most worth watching."""
    ((stmt, asks),) = gen_doc_outputs.sql_statements(
        "WITH x AS (SELECT 1) SELECT * FROM x;"
    )
    assert asks, stmt
