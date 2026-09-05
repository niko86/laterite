"""The catalogue lookup ranks, caps and labels — on a vault built here, not ours.

Every assertion runs against a synthetic vault written into `tmp_path`. That is
deliberate. A test in a required job that asserted live pairs — "dict.rs returns
edition-resolution" — would go red the day someone legitimately moved a citation,
turning a lookup convenience into a brake on wiki edits. The behaviours below are
properties of the ranking, and they hold whatever the vault happens to contain.

What is being pinned is the honesty of the output. Only ~1 in 5 tracked non-wiki
files carries an exact `repo:` ref; the rest of what looks like coverage comes
from a page citing an ancestor directory (`repo:web/` alone reaches 323 files
from two pages that describe none of them). If those printed unmarked, the tool
would certify most of the tree as documented.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

import pytest
from _tools import load_tool

if TYPE_CHECKING:
    from pathlib import Path

lib = load_tool("librarian")


def _page(vault: Path, rel: str, *, kind: str, title: str, refs: list[str]) -> None:
    body = "\n".join(f"- see `repo:{r}`" for r in refs)
    (vault / rel).parent.mkdir(parents=True, exist_ok=True)
    (vault / rel).write_text(
        f"---\ntype: {kind}\ntitle: {title}\n---\n# heading\n{body}\n",
        encoding="utf-8",
    )


@pytest.fixture
def vault(tmp_path: Path) -> Path:
    v = tmp_path / "wiki"
    v.mkdir()
    # Cites the file itself.
    _page(
        v,
        "concepts/exactly-this.md",
        kind="concept",
        title="The concept that names the file",
        refs=["src/engine/parse.rs"],
    )
    # Cites only the directory above it.
    _page(
        v,
        "design/dec-sweeping.md",
        kind="decision",
        title="A decision that sweeps the directory",
        refs=["src/engine"],
    )
    # A generated reference page that also names the file exactly.
    _page(
        v,
        "groups/AAVT.md",
        kind="group",
        title="AAVT — generated reference",
        refs=["src/engine/parse.rs"],
    )
    # A hand-written tool page naming the file exactly.
    _page(
        v,
        "tools/engine.md",
        kind="tool",
        title="engine — the tool page",
        refs=["src/engine/parse.rs"],
    )
    # Nothing to do with the query.
    _page(
        v,
        "concepts/unrelated.md",
        kind="concept",
        title="Unrelated",
        refs=["docs/readme.md"],
    )
    # Skipped by construction: a `_`-prefixed meta page (AGS-WIKI.md §2).
    _page(
        v,
        "templates/_template-concept.md",
        kind="concept",
        title="Template",
        refs=["src/engine/parse.rs"],
    )
    return v


def _hits(vault: Path, target: str) -> list:
    return lib.lookup(target, lib.catalogue(vault))


def test_an_exact_citation_outranks_a_directory_sweep(vault: Path) -> None:
    order = [h.rel for h in _hits(vault, "src/engine/parse.rs")]
    assert order.index("design/dec-sweeping.md") == len(order) - 1, (
        f"the directory-only hit must sort last, got {order}"
    )


def test_an_explanatory_page_outranks_a_generated_reference_page(vault: Path) -> None:
    """Both cite the file exactly; only the class ordering separates them.

    `groups/` is written wholesale by `tools/gen_reference_groups.py`. 177 of the
    184 pages citing `ags_dictionary.json` live there, so without this the pages
    that explain the dictionary are unreachable behind AAVT.md and its siblings.
    """
    order = [h.rel for h in _hits(vault, "src/engine/parse.rs")]
    assert order.index("tools/engine.md") < order.index("groups/AAVT.md"), order


def test_a_directory_only_hit_says_so(vault: Path) -> None:
    """The load-bearing label. Falsify by dropping the note from `report()`."""
    out = lib.report(["src/engine/parse.rs"], limit=10, show_all=False, wiki=vault)
    sweeping = next(ln for ln in out.splitlines() if "design/dec-sweeping.md" in ln)
    assert "directory only" in sweeping, sweeping
    exact = next(ln for ln in out.splitlines() if "tools/engine.md" in ln)
    assert "directory only" not in exact, (
        "an exact citation must not be labelled a directory sweep"
    )


def test_meta_pages_are_not_catalogued(vault: Path) -> None:
    """AGS-WIKI.md §2's leading `_` is what excludes them — not the directory.

    An earlier version also skipped anything under `templates/`. Mutation testing
    could not make that check fail: every file there already carries the prefix.
    """
    assert not any("templates/" in h.rel for h in _hits(vault, "src/engine/parse.rs"))
    _page(
        vault,
        "concepts/_scratch.md",
        kind="concept",
        title="A meta page outside templates/",
        refs=["src/engine/parse.rs"],
    )
    assert not any("_scratch" in h.rel for h in _hits(vault, "src/engine/parse.rs"))


def test_the_title_is_printed_not_the_stem(vault: Path) -> None:
    """79 of 138 hand-written pages carry a title that differs from the filename.

    That gap is the whole reason a stem index cannot answer this question, so
    printing the stem twice would reproduce the problem the tool exists to solve.
    """
    out = lib.report(["src/engine/parse.rs"], limit=10, show_all=False, wiki=vault)
    assert "The concept that names the file" in out


def test_the_cap_holds_and_declares_what_it_dropped(vault: Path) -> None:
    """A silent truncation reads as 'that's everything'."""
    out = lib.report(["src/engine/parse.rs"], limit=2, show_all=False, wiki=vault)
    assert "… +2 more (--all)" in out, out
    every = lib.report(["src/engine/parse.rs"], limit=2, show_all=True, wiki=vault)
    assert "more (--all)" not in every
    assert every.count("\n  ") == 4


def test_a_dotfile_path_is_not_mangled(vault: Path) -> None:
    """Regression: `lstrip("./")` ate the dot and reported the file as uncited.

    `.github/workflows/ci.yml` became `github/workflows/ci.yml`, which matches
    nothing. A character-set strip is the wrong tool for a prefix.
    """
    _page(
        vault,
        "concepts/ci.md",
        kind="concept",
        title="CI",
        refs=[".github/workflows/ci.yml"],
    )
    assert lib._norm(".github/workflows/ci.yml") == ".github/workflows/ci.yml"
    assert lib._norm("./tools/x.py") == "tools/x.py"
    assert [h.rel for h in _hits(vault, ".github/workflows/ci.yml")] == [
        "concepts/ci.md"
    ]


def test_an_uncited_path_says_so_rather_than_printing_nothing(vault: Path) -> None:
    out = lib.report(["src/nowhere.rs"], limit=5, show_all=False, wiki=vault)
    assert "no page cites this path" in out


def test_it_is_a_lookup_not_a_gate(vault: Path, capsys) -> None:
    """Exit 0 for a path no page covers, and for one that does not exist.

    Nothing in CI depends on this tool. If it ever starts returning non-zero,
    something has quietly turned a convenience into a check.
    """
    assert lib.main(["--paths", "src/nowhere.rs"]) == 0
    assert lib.main(["--paths", "does/not/exist/at/all.rs"]) == 0
    capsys.readouterr()
