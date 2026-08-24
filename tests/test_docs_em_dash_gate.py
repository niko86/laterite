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


def test_the_gate_reads_the_config_keys_that_reach_every_page(gate):
    """`site_description` is the search snippet and `copyright` is the footer.
    One string each, in a file a docs/**.md scan has no reason to open, and both
    on every built page: between them they were the largest single source in the
    built site while the source scan reported it clean."""
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
