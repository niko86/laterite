"""The crate cards match the manifests, and the two Cargo readers agree.

`gen_crate_graph.crate_deps()` has claimed in its own docstring that
"`test_crate_graph_faithful` asserts this equals lint's copy" — a gate that did
not exist in this repo. The claim is load-bearing: two independent readers of the
same manifests, with nothing holding them together, is the multi-source-of-truth
drift the wiki machinery exists to catch. This is that gate.

The card assertions are the second half. Fifteen tool pages opened with
"Internal implementation detail — a workspace crate, not a public API"; nine of
those crates were `publish = true`. Nobody edited the pages — the manifests moved
underneath them. So the check that matters is not "is the page well-formed" but
"does the page still say what the manifest says", and it has to fail when a
manifest changes and the page does not.
"""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]


def _load(name: str, path: Path, *, tolerate_exit: bool = False):
    """Import a non-package script by path — same shape as the other loaders here.

    `tolerate_exit` is for `lint.py`, whose ~1450-line body is all top level and
    ends in `sys.exit()`; it has no `__main__` guard, so importing it RUNS the
    lint and then exits. Catching that leaves the module's top-level names bound,
    which is all this file needs. It is a wart, not a design: the fix is to wrap
    lint.py's body in a `main()`, which is a whole-file reindent and belongs in
    its own change.
    """
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec and spec.loader
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    try:
        spec.loader.exec_module(mod)
    except SystemExit:
        if not tolerate_exit:
            raise
    return mod


# `lint.py` may import a sibling by bare name, so the bootstrap directory has to
# be on the path before it loads — true whether or not the citation grammar has
# been split out into `refs.py` yet.
sys.path.insert(0, str(REPO / "ags-wiki" / ".bootstrap"))
gcg = _load("gen_crate_graph", REPO / "tools" / "gen_crate_graph.py")
lint = _load("lint", REPO / "ags-wiki" / ".bootstrap" / "lint.py", tolerate_exit=True)


def test_the_two_cargo_readers_agree() -> None:
    """The claim `crate_deps()`'s docstring makes about this file."""
    assert gcg.crate_deps() == lint._crate_deps()


def test_every_card_matches_its_manifest() -> None:
    """The gate: a card region that has drifted from its manifest fails CI.

    Falsify by flipping any `publish` value, renaming a crate, or adding a
    workspace dependency without regenerating.
    """
    stale = [
        p.relative_to(REPO).as_posix()
        for p, rendered in gcg.cards().items()
        if p.read_text(encoding="utf-8") != rendered
    ]
    assert not stale, (
        "crate-card regions are stale — run "
        "`uv run --no-project python tools/gen_crate_graph.py`:\n  "
        + "\n  ".join(stale)
    )


def test_every_workspace_crate_page_carries_a_card() -> None:
    """A page rooted at a workspace member opts in by carrying the markers.

    Without this, deleting a card region would silently remove the page from the
    gate's reach — the check would pass because it had nothing left to check.
    """
    rooted = set(gcg._page_for_crate())
    carded = {p.stem for p in gcg.cards()}
    missing = sorted(d for d in rooted if gcg._page_for_crate()[d].stem not in carded)
    assert not missing, f"tool pages rooted at a crate but carrying no card: {missing}"


def test_the_facade_crate_still_has_no_page() -> None:
    """Records a known gap so it cannot be forgotten or silently "fixed" wrong.

    Three different things here are called `laterite`: the PyPI wheel
    (`packages/laterite`), the crates.io facade (`rust-packages/laterite`), and
    the Python import root. `ags-wiki/tools/laterite.md` roots at the WHEEL, so
    the published facade crate — `publish = true`, on its own 0.1.x line — has no
    page at all, and a reader looking up "laterite" lands confidently on the
    wrong artifact.

    Creating that page needs a distinct stem and cross-links to all three, which
    is a documentation decision rather than a generated one. When it lands, this
    test flips to asserting the page exists.
    """
    assert "laterite" not in gcg._page_for_crate(), (
        "rust-packages/laterite now has a page — good. Replace this test with "
        "the positive assertion and add the crate to the parametrised list below."
    )


@pytest.mark.parametrize("crate", ["laterite-ags4-parse", "laterite-ags4-diff"])
def test_published_crates_are_not_called_internal(crate: str) -> None:
    """The defect class this PR exists to retire, asserted directly.

    A crate whose manifest publishes it must not have a page describing it as an
    internal detail — that text told Rust users a public, semver-committed crate
    was off-limits.
    """
    page = gcg._page_for_crate().get(crate)
    assert page is not None, f"{crate} has no tool page rooted at it"
    text = page.read_text(encoding="utf-8")
    assert "**Internal implementation detail**" not in text, (
        f"{page.name} still calls {crate} an internal implementation detail, "
        "but its manifest sets `publish = true`"
    )
