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
import re
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


def test_the_facade_crate_has_its_own_page() -> None:
    """The gap this file used to record, now closed.

    Three different things here are called `laterite`: the PyPI wheel
    (`packages/laterite`), the crates.io facade (`rust-packages/laterite`), and
    the Python import root. `ags-wiki/tools/laterite.md` roots at the WHEEL, so
    for a while the published facade crate had no page at all and a reader
    looking up "laterite" landed confidently on the wrong artifact.

    The stem is `laterite-crate` because `laterite` was taken. That stem is NOT a
    package name — asserted here so nobody later "fixes" it into one, which would
    put `cargo add laterite-crate` into the vault pointing at nothing.
    """
    page = gcg._page_for_crate().get("laterite")
    assert page is not None, "rust-packages/laterite has no tool page rooted at it"
    assert page.stem == "laterite-crate", (
        f"expected the facade page at laterite-crate.md, found {page.name}"
    )
    text = page.read_text(encoding="utf-8")
    assert "cargo add laterite-crate` resolves to nothing" in text, (
        "the facade page must say its own stem is not a package name"
    )


def test_every_shipped_readme_matches_its_manifest() -> None:
    """The README half of the gate.

    A crate README is frozen at publish time — a wrong install line on crates.io
    cannot be corrected retroactively, only superseded by another release. So a
    manifest change that leaves a README stale has to fail before the upload, not
    after.
    """
    stale = [
        p.relative_to(REPO).as_posix()
        for p, rendered in gcg.readmes().items()
        if p.read_text(encoding="utf-8") != rendered
    ]
    assert not stale, (
        "README availability blocks are stale — run "
        "`uv run --no-project python tools/gen_crate_graph.py`:\n  "
        + "\n  ".join(stale)
    )


def test_every_publishable_crate_readme_says_how_to_install_it() -> None:
    """Eleven crates.io READMEs carried no install line at all before this gate.

    A visitor landing on crates.io saw a description and no way to add the crate.
    Asserted on the committed text rather than the render, so deleting the region
    from a README fails here instead of silently dropping it from the gate.
    """
    man = gcg._manifests()
    missing = []
    for crate_dir in gcg._members():
        name = gcg._name_of(crate_dir)
        if (
            name is None
            or name not in man
            or not gcg.distribution(name, man)["crates_io"]
        ):
            continue
        readme = gcg.RUST / crate_dir / "README.md"
        if readme.exists() and f"cargo add {name}" not in readme.read_text(
            encoding="utf-8"
        ):
            missing.append(readme.relative_to(REPO).as_posix())
    assert not missing, (
        f"publishable crates whose README never says `cargo add`: {missing}"
    )


def _published(non_facade: bool = True) -> list[tuple[str, Path]]:
    """(crate name, its README) for every crate the manifests publish."""
    man = gcg._manifests()
    out = []
    for crate_dir in gcg._members():
        name = gcg._name_of(crate_dir)
        if name is None or name not in man:
            continue
        if not gcg.distribution(name, man)["crates_io"]:
            continue
        if non_facade and name == gcg.FACADE:
            continue
        readme = gcg.RUST / crate_dir / "README.md"
        if readme.exists():
            out.append((name, readme))
    return out


def test_every_engine_readme_carries_the_anti_promise() -> None:
    """The engine tier says what it is, on the page a `cargo add` decision is made.

    The facade's whole stated purpose is to absorb engine reshaping — which says
    nothing about the engine crates holding still, and a reader who found one on
    crates.io had nothing telling them otherwise. Asserted on the committed text
    rather than the render, so deleting the line from a README fails here instead
    of quietly leaving that crate out of the gate's reach.
    """
    missing = [
        readme.relative_to(REPO).as_posix()
        for _, readme in _published()
        if "**Engine crate, not a door.**" not in readme.read_text(encoding="utf-8")
    ]
    assert not missing, (
        "published engine crates whose README does not say it is one — run "
        "`uv run --no-project python tools/gen_crate_graph.py`:\n  "
        + "\n  ".join(missing)
    )


def test_the_anti_promise_names_the_door() -> None:
    """A crate told "not a door" and not told where the door is has been sent
    nowhere. The link is to crates.io rather than to a repo path because the
    reader this is for is on crates.io."""
    name, readme = _published()[0]
    text = readme.read_text(encoding="utf-8")
    assert f"`{name}` is machinery inside the laterite" in text
    assert "https://crates.io/crates/laterite" in text


def test_the_facade_makes_its_own_promise_by_hand() -> None:
    """`laterite` is the door, so it must NOT be stamped with the anti-promise.

    Its README's stability statement is hand-written and says something
    different in kind from the engine crates' generated line. Until phase 8 of
    dec-facade-parity (2026-09-04) that statement was the "not yet at parity"
    caveat; the jump retired it, and what stands in its place is the parity
    claim itself — the crate on the product version, in beta with every other
    surface. This holds the README to carrying it.
    """
    facade = gcg.RUST / gcg.FACADE / "README.md"
    text = facade.read_text(encoding="utf-8")
    assert "**Engine crate, not a door.**" not in text
    assert "not yet at parity" not in text, (
        "the pre-parity caveat is back in the facade README — phase 8 retired "
        "it, and the version gate now asserts the parity it hedged against"
    )
    assert "carries the **product\nversion**" in text or (
        "carries the **product version**" in text
    ), "the facade README has lost the hand-written parity/product-line promise"


def test_the_card_and_the_readme_cannot_disagree() -> None:
    """One computation, two renderings — the point of factoring `distribution()`.

    Half the audited defects lived outside `ags-wiki/`. If the wiki card and the
    shipped README derived their facts separately they could contradict each
    other about the same crate, which is the drift this whole change exists to
    remove.

    This used to assert both stated the same VERSION. #783 removed the version
    from both, so that specific disagreement is now structurally impossible; what
    is left to check is that they agree on the two facts still rendered.
    """
    man = gcg._manifests()
    for name in man:
        dist = gcg.distribution(name, man)
        if not dist["crates_io"]:
            continue
        page = gcg._page_for_crate().get(name)
        if page is None:
            continue
        card = page.read_text(encoding="utf-8")
        assert "**Cleared for crates.io**" in card, (
            f"{page.name}'s card does not say {name} is published, but its "
            "manifest sets `publish = true`"
        )
        tier = (
            "versioned with the workspace"
            if dist["inherited"]
            else ("versioned on its own line")
        )
        assert tier in card, (
            f"{page.name}'s card does not state {name}'s version tier ({tier})"
        )


def test_no_generated_block_restates_a_version_number() -> None:
    """The invariant #783 bought, asserted so the number cannot come back.

    A version in these documents cannot go stale — they are generated — so the
    argument was never accuracy. It is that thirty of the thirty-five files in an
    engine bump existed to restate one line, and that a reader resolving a
    version wants the registry's answer, which the unpinned `cargo add` already
    gives them. Re-adding a number would be invisible except as diff weight,
    which is exactly the sort of regression only a test catches.

    Asserted against what the GENERATOR emits, not against the files: a README's
    hand-written prose may legitimately name a version (an MSRV, a dependency
    pin), and this invariant is about the block the generator owns.
    """
    stamped = re.compile(r"\bv?\d+\.\d+\.\d+\b")
    man = gcg._manifests()
    for name in man:
        dist = gcg.distribution(name, man)
        for label, lines in (
            ("card", gcg._card_lines(name, man, [])),
            ("README availability block", gcg._availability_lines(dist)),
        ):
            body = "\n".join(lines)
            assert not stamped.search(body), (
                f"{name}'s {label} restates a version — `cargo add` is unpinned "
                "and the registry is what answers that (#783)"
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
