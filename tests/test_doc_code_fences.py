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
markers = gen_doc_outputs.census_doc_markers


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


def test_an_unrecognised_marker_job_is_reported() -> None:
    """#543: `doc-snipet:` (typo) was ignored by every gate with no output —
    indistinguishable from a fence nobody looked at. The marker census names
    it, with its location."""
    md = "<!-- doc-snipet: skip — typo -->\n```python\nprint(1)\n```\n"
    total, problems = markers(md, "p.md")
    assert total == 1
    assert len(problems) == 1
    assert "doc-snipet" in problems[0]
    assert "p.md:1" in problems[0]


def test_an_unrecognised_verb_is_reported() -> None:
    """The convention has one verb; `doc-code: skpi` directly above a fence is
    positionally consumed by the fence walk yet acted on by nothing."""
    md = "<!-- doc-code: skpi — verb typo -->\n```bash\nx\n```\n"
    total, problems = markers(md, "p.md")
    assert total == 1
    assert len(problems) == 1


def test_the_known_markers_in_their_read_positions_pass() -> None:
    md = (
        "<!-- doc-code: skip — a reason -->\n"
        "```bash\nx\n```\n"
        "<!-- doc-snippet: skip — a reason -->\n"
        "```python\nprint(1)\n```\n"
    )
    total, problems = markers(md, "p.md")
    assert total == 2
    assert not problems


def test_a_snippet_marker_above_an_unscanned_language_is_reported() -> None:
    """The issue's own example: a correctly-spelled marker above a fence in a
    language neither snippet gate scans is an instruction nobody reads."""
    md = "<!-- doc-snippet: skip — for nobody -->\n```sql\nselect 1;\n```\n"
    total, problems = markers(md, "p.md")
    assert total == 1
    assert len(problems) == 1
    assert "doc-snippet" in problems[0]


def test_a_snippet_marker_outside_the_window_is_reported() -> None:
    """Both gates look exactly 300 characters behind a fence; a marker pushed
    beyond that by intervening prose falls out of every window."""
    md = (
        "<!-- doc-snippet: skip — too far -->\n"
        + ("prose " * 60).strip()
        + "\n```python\nprint(1)\n```\n"
    )
    total, problems = markers(md, "p.md")
    assert total == 1
    assert len(problems) == 1


def test_a_detached_doc_code_marker_is_reported() -> None:
    """`doc-code` is read only from the line(s) immediately above a fence;
    prose in between detaches it from any reader."""
    md = (
        "<!-- doc-code: skip — floats free -->\n"
        "\nSome prose.\n\n"
        "```python\nprint(1)\n```\n"
    )
    total, problems = markers(md, "p.md")
    assert total == 1
    assert len(problems) == 1


def test_a_doc_output_marker_outside_its_slot_is_reported() -> None:
    md = "<!-- doc-output: skip — floats -->\n\nProse only.\n"
    total, problems = markers(md, "p.md")
    assert total == 1
    assert len(problems) == 1


def test_a_snippet_marker_above_an_include_fence_is_reported() -> None:
    """Review-found hole: both snippet gates skip an include fence before
    consulting the marker window, so a window over one has no reader — the
    census must not count it as read."""
    md = (
        "<!-- doc-snippet: skip — nobody consults this -->\n"
        "```python\n"
        '--8<-- "python/x.py:code"\n'
        "```\n"
    )
    total, problems = markers(md, "p.md")
    assert total == 1
    assert len(problems) == 1


def test_the_mirrored_window_matches_both_real_gates() -> None:
    """SNIPPET_WINDOW and SNIPPET_LANGS restate the two snippet gates; a
    drifted copy would make the sweep vouch for windows nobody scans. Tie the
    number to each gate's own source, and the languages to their fence
    regexes."""
    py_gate = (ROOT / "tests" / "test_docs_snippets.py").read_text()
    ts_gate = (
        ROOT / "rust-packages" / "laterite-node" / "test" / "docs-snippets.test.ts"
    ).read_text()
    w = gen_doc_outputs.SNIPPET_WINDOW
    assert f"m.start() - {w}" in py_gate
    assert f"m.index - {w}" in ts_gate
    assert "```python" in py_gate
    assert "```(js|ts)" in ts_gate
    assert {"python", "js", "ts"} == gen_doc_outputs.SNIPPET_LANGS


def test_a_doc_output_marker_in_its_slot_passes() -> None:
    md = (
        "```python\n"
        '--8<-- "python/x.py:code"\n'
        "```\n"
        "\n"
        "<!-- doc-output: skip — no output shown -->\n"
        "```text\n"
        "```\n"
    )
    total, problems = markers(md, "p.md")
    assert total == 1
    assert not problems


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


def test_a_type_only_ts_fence_is_not_a_program() -> None:
    """The #519 decision: a fence that declares types and executes nothing is
    not a program, so it belongs with prose — not with runnable languages,
    where its only effect was a standing PENDING line for a runner nobody
    should build. Shaped like the corpus's one instance (wasm-api.md)."""
    md = (
        "```ts\n"
        "import type {\n"
        "  FindingDto,\n"
        "  RuleGroup,\n"
        "  ValidationReport,\n"
        '} from "@laterite/ags4-wasm";\n'
        "```\n"
    )
    counts, problems = census(md, "p.md")
    assert counts["prose"] == 1
    assert counts["inline"] == 0
    assert not problems


def test_an_executable_ts_fence_is_reported_not_absorbed() -> None:
    """The guard on the same decision: type-only is the ONLY ts class the
    census accepts. A ts fence with a real statement must not ride into prose
    on the tag — it is a new case #519 deliberately did not route, and the
    census has to say so rather than absorb it."""
    md = "```ts\nconst report = validate(bytes);\n```\n"
    counts, problems = census(md, "p.md")
    assert counts["prose"] == 0
    assert counts["inline"] == 0
    assert len(problems) == 1
    assert "executable" in problems[0]


def test_a_ts_fence_mixing_types_and_execution_is_reported() -> None:
    """One executable statement poisons the fence — the type-only judgement is
    per-fence, not per-line, and the safe direction is out of prose."""
    md = (
        "```ts\n"
        'import type { ValidationReport } from "@laterite/ags4-wasm";\n'
        "console.log(report.ok);\n"
        "```\n"
    )
    counts, problems = census(md, "p.md")
    assert counts["prose"] == 0
    assert len(problems) == 1


def test_a_declare_and_interface_block_is_type_only() -> None:
    md = (
        "```ts\n"
        "interface Row {\n"
        "  id: string;\n"
        "  depth(): number;\n"
        "}\n"
        "declare const VERSION: string;\n"
        "type Pair = [Row, Row];\n"
        "```\n"
    )
    counts, problems = census(md, "p.md")
    assert counts["prose"] == 1
    assert not problems


def test_a_brace_inside_a_string_cannot_hide_executable_code() -> None:
    """Review-found hole: `type X = "{";` used to leave the depth counter open,
    so every later line passed the depth>0 check and executable code rode into
    prose unreported. Strings are stripped before depth is counted."""
    md = '```ts\ntype X = "{";\nsomeCall();\n```\n'
    counts, problems = census(md, "p.md")
    assert counts["prose"] == 0
    assert len(problems) == 1


def test_code_trailing_a_type_import_on_one_line_is_refused() -> None:
    """Review-found hole: the opener check reads the START of a line, so
    `import type { Foo } from "x"; doSomething();` passed as type-only. A
    statement completing at depth zero with code after its `;` is refused."""
    md = '```ts\nimport type { Foo } from "x"; doSomething();\n```\n'
    counts, problems = census(md, "p.md")
    assert counts["prose"] == 0
    assert len(problems) == 1


def test_a_multiline_template_literal_refuses_the_fence() -> None:
    """A quote surviving the single-line strip is a string this reader cannot
    follow — refuse rather than guess, and refusal is a reported problem."""
    md = "```ts\ntype T = `\n{\n`;\nrun();\n```\n"
    counts, problems = census(md, "p.md")
    assert counts["prose"] == 0
    assert len(problems) == 1


def test_a_url_in_a_type_string_is_not_read_as_a_comment() -> None:
    """`//` inside a string must not truncate the line: strings are stripped
    before comments, so `type U = "https://x";` stays a clean declaration."""
    md = '```ts\ntype U = "https://example.org";\n```\n'
    counts, problems = census(md, "p.md")
    assert counts["prose"] == 1
    assert not problems


def test_ts_is_not_a_runnable_language_nothing_runs() -> None:
    """AC1 of #519 at the seam the pending line is derived from: membership in
    PAGE_SURFACE is the claim a language is meant to be run, and PAGE_RUNNER
    is what the census prints — ts must be in neither."""
    assert "ts" not in gen_doc_outputs.PAGE_SURFACE
    assert "ts" not in gen_doc_outputs.PAGE_RUNNER


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
    assert mine == {"python", "js", "javascript"}
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


# --- one surface, several fence tags (step 4) --------------------------------


def test_two_tags_for_one_language_make_one_program() -> None:
    """```js and ```javascript are the same language to Node.

    Split into two programs, the second half runs without the first half's
    imports and fails for a reason that is about the gate, not the page.
    """
    md = (
        '```js\nimport { read } from "laterite";\n```\n'
        "\n```javascript\nconst f = read('delivery.ags');\n```\n"
    )
    src, inline = gen_doc_outputs.page_program(md, "js", "javascript")
    assert inline == 2
    assert src.index("import { read }") < src.index("const f =")


def test_one_tag_still_collects_only_that_tag() -> None:
    """The single-tag call is what the SQL and Python runners make; widening the
    signature must not widen what they collect."""
    md = "```js\nconst a = 1;\n```\n\n```javascript\nconst b = 2;\n```\n"
    src, inline = gen_doc_outputs.page_program(md, "js")
    assert inline == 1
    assert "const b" not in src


def test_every_surface_knows_all_the_tags_that_reach_it() -> None:
    """The runner groups by SURFACE, so the tags a surface answers for have to be
    derivable from the table rather than listed a second time somewhere."""
    node_tags = {k for k, v in gen_doc_outputs.PAGE_SURFACE.items() if v == "node"}
    assert node_tags == {"js", "javascript"}


def test_the_node_surface_prepares_its_temp_directory() -> None:
    """A page program lives nowhere, and ESM resolution walks UP from the file.

    The node EXAMPLES resolve `import … from "laterite"` through a symlink beside
    them in the repo; a program written into a temp directory has no such
    neighbour, so the surface has to build one. Python needs nothing — the
    interpreter running the gate already has the wheel — which is why this is a
    per-surface hook and not a step in `seed_workdir`.
    """
    assert gen_doc_outputs.SURFACES["node"].prepare is not None
    assert gen_doc_outputs.SURFACES["python"].prepare is None


def test_a_failure_excerpt_keeps_both_ends() -> None:
    """Python puts the exception last; Node puts it first and ends in loader
    frames. Keeping one end hid the `SyntaxError` behind four lines of
    `node:internal/modules/esm/loader`."""
    err = "\n".join(["FIRST", *(f"noise{i}" for i in range(20)), "LAST"])
    out = gen_doc_outputs._excerpt(err)
    assert out.startswith("FIRST")
    assert out.rstrip().endswith("LAST")


def test_a_failure_excerpt_says_how_much_it_dropped() -> None:
    """A silent trim is the blind spot this repo keeps re-finding: the reader
    cannot tell a short failure from a truncated one."""
    err = "\n".join(f"line{i}" for i in range(30))
    assert "elided" in gen_doc_outputs._excerpt(err)


def test_a_short_failure_is_not_elided() -> None:
    """Nothing was dropped, so nothing should claim it was."""
    err = "one\ntwo\nthree"
    out = gen_doc_outputs._excerpt(err)
    assert "elided" not in out
    assert "three" in out


def test_a_runner_that_finds_no_pages_fails(tmp_path, monkeypatch) -> None:
    """Zero is the one result a green run cannot mean.

    `test_docs_examples.py` guards its glob against a moved directory making
    every example "pass" by not running, and #513 names that guard as the
    precedent the page half had not inherited. With no pages to find, nothing is
    executed — so this exercises the guard itself rather than any surface.
    """
    monkeypatch.setattr(gen_doc_outputs, "DOCS", tmp_path)
    with pytest.raises(SystemExit) as e:
        gen_doc_outputs.run_page_programs(gen_doc_outputs.SURFACES["node"], ["js"])
    assert "discovery is broken" in str(e.value)
