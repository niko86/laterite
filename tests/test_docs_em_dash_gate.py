"""The docs em-dash gate, and the one string it could not see (#588).

Two things are pinned here, and the second is the reason the first is not
enough on its own.

The gate reads source, because building the site to find a character would put
it behind the docs job. That makes its SCOPE the whole question: every class of
input it skips is a place the policy can be broken with a green tick over it.
So the skips are tested, not just written down.

And the footer. `mkdocs.yml` carries a `copyright`, and the docs site's
`hooks/version_stamp.py` carried its own copy of the same text, appending the
version to it at build time. The hook's copy won, so the config's value rendered nowhere, and the two
drifted exactly as that module's own docstring warns a second copy will. A gate
reading the config therefore read a string nobody sees. There is one copy now,
and this holds it to one.

Stdlib only, so it runs in the buildless subset beside the other tools tests.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]


def _load(rel: str, name: str):
    spec = importlib.util.spec_from_file_location(name, REPO / rel)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


@pytest.fixture(scope="module")
def gate():
    return _load("tools/check_docs_em_dash.py", "check_docs_em_dash")


@pytest.fixture(scope="module")
def hook():
    return _load("web/docs-site/hooks/version_stamp.py", "version_stamp")


def test_prose_is_reported(gate):
    """The positive control. Without it every skip test below could pass on a
    scanner that reports nothing at all."""
    hits, _ = gate.scan("A sentence — with a dash in it.\n")
    assert [n for n, _ in hits] == [1]


def test_a_fenced_block_is_skipped_and_counted(gate):
    """An output capture prints an em dash because the tool did. Rewriting one
    would make the page a claim about output nobody can reproduce."""
    hits, skipped = gate.scan("before\n```\nlat: a — b\n```\nafter\n")
    assert hits == []
    assert skipped == 1


def test_inline_code_is_skipped(gate):
    hits, skipped = gate.scan("The flag `--x — y` is quoted output.\n")
    assert hits == []
    assert skipped == 1


def test_an_html_comment_is_skipped(gate):
    """`doc-code: skip — why` markers spell their reason after an em dash, and
    mkdocs renders none of it. A policy about what a reader sees has nothing to
    say about text no reader sees."""
    hits, skipped = gate.scan("<!-- doc-code: skip — installs packages -->\ntext\n")
    assert hits == []
    assert skipped == 1


def test_a_multi_line_html_comment_is_skipped_without_moving_line_numbers(gate):
    """The comment is blanked across the whole text, so a dash after a
    multi-line comment must still be reported against its own line."""
    hits, _ = gate.scan("<!-- one —\ntwo — three -->\nreal — prose\n")
    assert [n for n, _ in hits] == [3]


CONFIG = """\
site_name: laterite
site_description: >-
  Read, validate and query AGS4 data — born-typed, fluent, one engine.
site_url: https://docs.laterite.dev/
# A comment — not rendered, so not this gate's business.
copyright: laterite — MIT-licensed AGS4 tooling
theme:
  features:
    - navigation.tabs — not a reader key, and indented besides
"""


def test_the_config_values_that_reach_every_page_are_reported(gate):
    """The defect the source-only scan had. `site_description` is the search
    snippet and `copyright` is the footer: one string each, in a file a
    docs/**.md walk has no reason to open, and between them they were the
    largest single source in the BUILT site while the source scan called it
    clean."""
    found = {key: (n, v) for n, key, v in gate.scan_config(CONFIG)}
    assert sorted(found) == ["copyright", "site_description"]
    assert found["copyright"][0] == 6
    # The wrapped value is followed to its end, or the dash on its second line
    # would be missed for exactly the key that matters most.
    assert "born-typed" in found["site_description"][1]


def test_the_config_scan_ignores_what_no_reader_sees(gate):
    """A YAML comment and an indented value are not reader copy, and a key that
    is not in the reader set is not this gate's business however it is written.
    Without this, the gate would demand rewrites of the config's own notes."""
    clean = CONFIG.replace(" — born-typed, fluent, one engine.", ".").replace(
        "copyright: laterite — MIT-licensed AGS4 tooling", "copyright: laterite"
    )
    assert gate.scan_config(clean) == []


def test_the_reader_keys_are_the_ones_that_render(gate):
    assert "site_description" in gate.MKDOCS_READER_KEYS
    assert "copyright" in gate.MKDOCS_READER_KEYS
    assert gate.MKDOCS.is_file()


def test_the_footer_is_built_from_the_config_not_a_second_copy(hook):
    """The defect this test exists for. A literal base in the hook is a second
    copy of `mkdocs.yml`'s `copyright` that WINS at build time, so the config's
    value renders nowhere and any gate reading it is checking a string nobody
    sees. Reintroducing one turns this red."""
    stamped = hook.stamp("A DISTINCT BASE", "9.9.9")
    assert stamped.startswith("A DISTINCT BASE"), (
        "the footer ignored the copyright it was given, so something in the "
        "hook is supplying its own text again"
    )
    assert "9.9.9" in stamped


def test_a_fence_marker_inside_a_comment_does_not_swallow_prose(gate):
    """The bug review found. The fence test read the RAW line while the prose
    test read the comment-masked one, so a fence-shaped line inside a comment
    opened a block that was never open: the real prose after it was passed AND
    counted as skipped, so the gate under-reported and misdescribed why."""
    hits, skipped = gate.scan("<!--\n```\n-->\nreal — prose here\n")
    assert [n for n, _ in hits] == [4]
    assert skipped == 0


def test_a_mismatched_fence_token_does_not_close_the_block(gate):
    """``` and ~~~ do not close each other. A `~~~` printed inside a ``` block
    is content, and closing on it would end the fence early and report the code
    after it as prose."""
    hits, _ = gate.scan("```\nx\n~~~\nstill — code\n```\nreal — prose\n")
    assert [n for n, _ in hits] == [6]


def test_the_footer_guard_refuses_a_missing_copyright(hook):
    """The hook's own comment refuses to stamp a guessed VERSION; the base is
    the other half of the same footer. mkdocs defaults `copyright` to None, and
    stamping it renders the word "None" in front of the version on every page."""

    class Config:
        copyright = None
        extra = {}  # noqa: RUF012 - a stand-in for mkdocs' config object

    with pytest.raises(SystemExit, match=r"no `copyright`"):
        hook.on_config(Config())


# --- the built half, which is the gate proper (#588) --------------------------
#
# `--built` reads what `mkdocs build` produced, because three of this site's
# page families are emitted by web/docs-site/scripts/ and have no Markdown at
# all. The source-only draft reported clean while several hundred sat in the
# built site, so these hold the parts of the HTML walk that were wrong at least
# once while it was being written.


def test_built_prose_is_reported(gate):
    """The positive control for everything below."""
    excerpts, _ = gate.scan_html("<p>A sentence — with a dash.</p>")
    assert len(excerpts) == 1
    assert "with a dash" in excerpts[0]


def test_built_code_is_skipped_and_counted(gate):
    """The built-site counterpart of the fence rule: a `<pre>` holds what a tool
    printed, so rewriting it would misquote the tool."""
    excerpts, skipped = gate.scan_html("<pre>lat: a — b</pre><p>fine</p>")
    assert excerpts == []
    assert skipped == 1


def test_a_pre_wrapping_a_code_block_is_counted_once(gate):
    """Every highlighted block is `<pre><code>`, so double-counting here would
    inflate the skip total on essentially every page — and the skip total is the
    number this gate publishes to make its own blind spot visible."""
    _, skipped = gate.scan_html("<pre><code>a — b</code></pre>")
    assert skipped == 1


def test_a_code_block_closes_on_its_own_tag_not_the_first_one_seen(gate):
    """The shape every highlighted block on the site actually has. Pygments
    wraps each token in a `<span>`, so a close-on-any-tag pattern ends the block
    at the first `</span>` and reports the REST of the code as reader prose.
    Nothing else here would catch that: it needs a nested element and a dash on
    each side of it."""
    excerpts, skipped = gate.scan_html(
        '<pre><code>a — b <span class="k">let</span> c — d</code></pre>'
    )
    assert excerpts == [], excerpts
    assert skipped == 2


def test_code_leaves_a_mark_so_the_excerpt_stays_greppable(gate):
    """The excerpt is the only locator this half gives: generated HTML has no
    line a person edits. Deleting code spans outright produced excerpts like
    "Stack / — nothing runs" that match nothing in any source file."""
    (excerpt,) = gate.scan_html("<p>Stack <code>a</code>/<code>b</code> — go.</p>")[0]
    assert "`…`" in excerpt


def test_an_entity_encoded_dash_reaches_a_reader_too(gate):
    """`&mdash;` renders as an em dash. A byte-level search for U+2014 would
    call this page clean while a reader looks straight at one."""
    excerpts, _ = gate.scan_html("<p>A sentence &mdash; with a dash.</p>")
    assert len(excerpts) == 1


def test_an_entity_encoded_dash_inside_code_still_counts_as_skipped(gate):
    """The other side of the same coin. Counting the raw match rather than the
    unescaped one understates the skip total, which is the one number here whose
    whole job is to be honest about what went unread."""
    excerpts, skipped = gate.scan_html("<pre>a &mdash; b</pre>")
    assert excerpts == []
    assert skipped == 1


def test_the_meta_description_is_read_although_it_is_an_attribute(gate):
    """`site_description` renders on every page as `<meta content=…>`, so it has
    no text node: stripping tags drops it. It is also the search snippet and the
    link preview, which makes it the most-read string on the site."""
    excerpts, _ = gate.scan_html(
        '<meta name="description" content="Read AGS4 — born-typed."><p>ok</p>'
    )
    assert len(excerpts) == 1
    assert "born-typed" in excerpts[0]


def test_the_meta_description_is_found_whichever_order_it_is_written_in(gate):
    """Attribute order is the theme template's business, not this gate's."""
    excerpts, _ = gate.scan_html('<meta content="Read AGS4 — fast" name="description">')
    assert len(excerpts) == 1


def test_a_meta_tag_that_is_not_the_description_is_left_alone(gate):
    """Without this the `name=description` test passes on a scanner that reads
    the content of every meta tag, including generator and theme-colour ones."""
    assert gate.scan_html('<meta name="generator" content="mkdocs — 1.6">')[0] == []


def test_stripped_tags_do_not_glue_neighbouring_words(gate):
    """Substituting nothing for a tag runs the last word of one block into the
    first of the next, which corrupts every excerpt that spans a tag boundary."""
    (excerpt,) = gate.scan_html("<p>alpha</p><p>beta — gamma</p>")[0]
    assert "alpha beta" in excerpt


def test_a_site_that_did_not_build_is_not_a_pass(gate, tmp_path):
    """A gate that sees no input has checked nothing. An empty site dir means
    the build failed or the path is wrong, and returning 0 there would report
    green over an unscanned site."""
    assert gate.check_built(tmp_path, False) == 1


def test_an_excluded_family_is_counted_and_reported_not_passed_over(
    gate, tmp_path, capsys
):
    """The exclusions are a decision, so they are printed with their counts on
    every run: a declared exclusion nobody can see is a blind spot with a green
    tick on it (CLAUDE.md, Conventions)."""
    (tmp_path / "reference" / "api").mkdir(parents=True)
    (tmp_path / "reference" / "api" / "index.html").write_text(
        "<p>a docstring — rendered by mkdocstrings</p>", encoding="utf-8"
    )
    (tmp_path / "index.html").write_text("<p>clean</p>", encoding="utf-8")

    assert gate.check_built(tmp_path, False) == 0
    out = capsys.readouterr().out
    assert "reference/api/ — 1 occurrence(s)" in out
    assert "read 1 built page(s)" in out, "the excluded page must not be counted twice"


def test_a_page_outside_the_excluded_families_still_fails(gate, tmp_path):
    """The negative control for the test above: the exclusions must be the two
    named prefixes, not a rule that quietly swallows the whole reference tree."""
    (tmp_path / "reference" / "groups" / "LOCA").mkdir(parents=True)
    (tmp_path / "reference" / "groups" / "LOCA" / "index.html").write_text(
        "<h1>LOCA — General information</h1>", encoding="utf-8"
    )
    assert gate.check_built(tmp_path, False) == 1


def test_every_exclusion_carries_a_reason_and_a_blind_spot_is_named(gate):
    """A path list with no reasons is the same silent scope the discipline is
    against, one indirection later."""
    assert set(gate.BUILT_SKIP) == {
        "reference/api/",
        "reference/modules/",
    }
    assert all(len(r) > 40 for r in gate.BUILT_SKIP.values())
    assert gate.BUILT_BLIND_SPOTS, "a gate that drops input says what it dropped"
    assert all(len(s) > 40 for s in gate.BUILT_BLIND_SPOTS)


def test_the_meta_description_is_found_when_the_theme_leaves_it_unquoted(gate):
    """Attribute quoting is the template author's business too, not this gate's.
    A pattern requiring quotes reads clean on a theme that omits them, which is
    the same silent miss the source-only scan had for this exact string."""
    excerpts, _ = gate.scan_html("<meta name=description content='AGS4 — fast'>")
    assert len(excerpts) == 1


def test_the_shipped_cli_guide_is_gated_rather_than_excused(gate, tmp_path):
    """#681 decided the policy covers what a shipped program prints, so the page
    that IS `lat --readme` is read like any other.

    The predecessor of this test pinned that the exclusion's reason cited #681,
    on the reasoning that an unmet criterion has to point somewhere a reader can
    go. That is now settled, so this asserts the settlement functionally rather
    than asserting the absence of a string: a dash in that page's prose has to
    make the gate RED. An exclusion re-added by path would pass a
    `"reference/cli/" not in BUILT_SKIP` check for exactly as long as someone
    spelled the prefix differently."""
    assert "reference/cli/" not in gate.BUILT_SKIP
    (tmp_path / "reference" / "cli").mkdir(parents=True)
    (tmp_path / "reference" / "cli" / "index.html").write_text(
        "<p>Errors decide the verdict — the table answers something else</p>",
        encoding="utf-8",
    )
    assert gate.check_built(tmp_path, False) == 1


def test_the_gen_cli_note_is_no_longer_a_declared_blind_spot(gate):
    """The note web/docs-site/scripts/gen_cli.py writes above the shipped guide
    is OURS, and it has no `.md` for the source half to read — so while the page
    was excluded by path, nothing held it at all. Gating the page holds the
    note, and the tuple must stop saying otherwise: a blind spot that has been
    closed is worse than one that was never listed, because it is a live claim
    that something is unchecked when it is."""
    assert not any("gen_cli.py" in s for s in gate.BUILT_BLIND_SPOTS)


def test_the_two_halves_are_not_described_as_one_subsuming_the_other(gate):
    """The claim a review caught: the source half was documented as a strict
    subset of the built half, kept only for speed. It is not. `reference/api/`
    and `reference/modules/` are excluded from the built scan BY PATH, and both
    pages carry hand-written prose around their generated parts, so the source
    half is the only gate those paragraphs have."""
    doc = gate.__doc__ or ""
    assert "strict SUBSET" not in doc
    assert "COMPLEMENTARY" in doc
    for prefix in ("reference/api/", "reference/modules/"):
        assert "hand-written" in gate.BUILT_SKIP[prefix], (
            f"{prefix} is excluded wholesale but is a MIX; the reason must say "
            "so, or the printed count reads as 'all generated'"
        )


def test_every_reason_names_where_the_excluded_prose_actually_lives(gate):
    """Triage of #681 found a reason asserting a file topology this repo does not
    have. Reasons print on every CI run, pass or fail, so a wrong one is
    published on every run — and the specific failure was a reader being sent to
    the wrong file to edit. The CLI entry that carried it is gone (the page is
    gated now), and this generalises what it was worth to the two that remain:
    each has to name the source its prose comes from."""
    for prefix, reason in gate.BUILT_SKIP.items():
        assert "docs/reference/" in reason, (
            f"{prefix} must name the `.md` whose prose the source half reads, "
            "so a reader knows which file to edit"
        )


def test_an_excluded_page_reports_prose_only_not_its_code(gate, tmp_path):
    """What the printed count MEANS, pinned because the reason now says it. An
    excluded page's dashes are counted through the same prose filter as any
    other page, so a guide that is mostly quoted `--help` output does not have
    that output inflating the number a reader is asked to judge."""
    (tmp_path / "reference" / "api").mkdir(parents=True)
    (tmp_path / "reference" / "api" / "index.html").write_text(
        "<p>real — prose</p><pre>lat: a — b — c</pre>", encoding="utf-8"
    )
    (tmp_path / "index.html").write_text("<p>clean</p>", encoding="utf-8")
    assert gate.check_built(tmp_path, False) == 0
    # One prose dash, not three: the two inside <pre> are code on this page too.
    assert "reference/api/ — 1 occurrence(s)" in _capture(gate, tmp_path)


def _capture(gate, site) -> str:
    import contextlib
    import io

    buf = io.StringIO()
    with contextlib.redirect_stdout(buf):
        gate.check_built(site, False)
    return buf.getvalue()
