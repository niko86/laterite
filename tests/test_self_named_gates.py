"""A doc that names its own gate must name a gate this repo runs.

The worst kind of stale documentation is not the kind that is merely wrong — it
is the kind that says a safety exists. `tools/gen_reference_groups.py` opened
with "gated by tests/test_reference_groups_faithful.py"; no such file has ever
been in this repository. A reader deciding whether the 174 generated group pages
could drift from the dictionary got a confident yes from a sentence naming a file
that does not exist. `tools/gen_wiki_cli.py` went further and named a test that
was supposed to close a hole its own docstring described, and the hole was open.

Nobody wrote those lines dishonestly. Some were true in the dev satellite and
copied here; some described a gate that was planned and never built. Either way
the failure mode is the same as every other finding in the 2026-08-04 audit: the
code moved (or never arrived) and the prose stayed.

So: a `tests/test_*.py` token in a doc must resolve to a real test in this repo,
or disclaim itself. Two disclaimers, both meaning "I am not claiming this repo
runs this":

  dev satellite      — it runs in the private dev checkout, not here
  not in this repo   — it does not run here, and the sentence is about that

Direction, stated honestly. `tests/**` is in `ci.yml`'s `code` filter, so the
high-value direction — someone deletes or renames a test and the docs naming it
go false — fires the required `python` job and lands here. `ags-wiki/**` is
deliberately NOT in that filter (ci.yml says why in place), so a wiki-only edit
that types a fresh false gate name is caught on the next code-touching PR rather
than its own. That is the same accepted asymmetry as the crate cards.
"""

from __future__ import annotations

import re
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]

#: Where a self-named gate is a claim a reader could act on. Deliberately not the
#: whole tree: `CHANGELOG.md` narrates what was true at a release and `docs/`
#: tabulates other suites by full path, neither of which is asserting a live gate.
SCOPE = [
    "ags-wiki/**/*.md",
    "CLAUDE.md",
    "tools/**/*.py",
    ".github/workflows/*.yml",
    "web/docs-site/mkdocs.yml",
    "web/docs-site/scripts/*.py",
]

#: Where this repo's Python tests actually live. Both roots run in CI — the
#: `python` job invokes `packages/laterite/tests` and the root `tests/` together.
TEST_ROOTS = ["tests", "packages/laterite/tests"]

#: `tests/test_x.py`, `packages/laterite/tests/test_x.py`, or any other prefix —
#: only the basename decides, because a doc citing the right test under a stale
#: path is a link problem (`check_doc_refs.py`'s job), not a false-safety problem.
TOKEN = re.compile(r"(?:[\w./-]+/)?tests?/(test_[\w.-]*\.py)")

#: Matched as bare phrases, not as `(…)` parentheticals: the disclaimer has to
#: survive being written into a sentence, a table cell, a YAML comment and a
#: mermaid node label, and punctuation-anchoring it would fail the honest ones.
#: Within `REACH` of a test filename, neither phrase means anything else.
DISCLAIMERS = ("dev satellite", "not in this repo")

#: How far past the token a disclaimer may sit. Prose wraps, so this is measured
#: on the whitespace-collapsed text rather than per line; one short clause of
#: slack. It is a ceiling, not the whole rule — see `_window`.
REACH = 120


def _window(flat: str, end: int) -> str:
    """The text a disclaimer at `end` would be attached to.

    `REACH` characters, but **stopped at the next test filename**. Without that
    stop, one claim borrows its neighbour's disclaimer: `mkdocs.yml` names three
    gates in one comment, and 120 characters from the first reaches the third's
    `(dev satellite)`. That is not a corner case — it fired the first time the
    inverse check ran, on prose that is entirely correct.
    """
    nxt = TOKEN.search(flat, end)
    return flat[end : min(end + REACH, nxt.start() if nxt else len(flat))]


def _real_tests() -> set[str]:
    return {p.name for root in TEST_ROOTS for p in (REPO / root).rglob("test_*.py")}


def _scope_files() -> list[Path]:
    seen: dict[Path, None] = {}
    for pattern in SCOPE:
        for p in sorted(REPO.glob(pattern)):
            if p.is_file():
                seen[p] = None
    return list(seen)


def test_every_named_gate_exists_or_disclaims_itself() -> None:
    """Falsify by deleting any test named in a doc, or by naming a new one.

    Both halves matter. Deleting `tests/test_issue_tracker.py` must fail here
    (nightly.yml names it); so must writing "gated by tests/test_nope.py" into
    any page in scope.
    """
    real = _real_tests()
    assert "test_self_named_gates.py" in real, (
        "the resolver cannot see this file, so it can see nothing — "
        f"TEST_ROOTS is wrong (searched {TEST_ROOTS})"
    )

    bad: list[str] = []
    for path in _scope_files():
        flat = " ".join(path.read_text(encoding="utf-8").split())
        rel = path.relative_to(REPO).as_posix()
        for m in TOKEN.finditer(flat):
            name = m.group(1)
            if name in real:
                continue
            window = _window(flat, m.end())
            if any(d in window for d in DISCLAIMERS):
                continue
            bad.append(f"{rel}: {name}")

    assert not bad, (
        "these docs name a gate this repo does not run, and do not say so:\n  "
        + "\n  ".join(sorted(set(bad)))
        + "\n\nEither name the gate that actually runs, or append one of "
        + f"{list(DISCLAIMERS)} to the claim."
    )


def test_a_gate_that_runs_here_is_not_disclaimed_as_absent() -> None:
    """The inverse defect, which the rule above cannot see.

    `test_every_named_gate_exists_or_disclaims_itself` passes any name that
    resolves — so a gate that DOES run here, wrongly marked `(dev satellite)`,
    sails through it. That is not hypothetical: `ags-wiki/concepts/docs-site.md`
    disclaimed `tests/test_docs_examples.py` as satellite-only while the file sat
    in this repo, wired into the required `python` job. It understates the repo's
    own safety, which is the same class of harm as overstating it — a reader
    deciding whether a change is covered gets the wrong answer either way.

    Falsify by writing `(dev satellite)` beside any test in `tests/`.
    """
    real = _real_tests()
    wrong: list[str] = []
    for path in _scope_files():
        flat = " ".join(path.read_text(encoding="utf-8").split())
        rel = path.relative_to(REPO).as_posix()
        for m in TOKEN.finditer(flat):
            name = m.group(1)
            if name not in real:
                continue
            if "dev satellite" in _window(flat, m.end()):
                wrong.append(f"{rel}: {name}")

    assert not wrong, (
        "these call a gate `dev satellite` when it runs in THIS repo:\n  "
        + "\n  ".join(sorted(set(wrong)))
        + "\n\nDrop the disclaimer — the test is here."
    )


def test_the_disclaimer_is_not_a_way_to_keep_a_dead_name() -> None:
    """A `dev satellite` disclaimer must be about a test that exists there.

    Without this the escape hatch retires the rule: append four words and any
    invented filename passes forever. Skips when the satellite is not checked
    out — a contributor without it is not the person this can catch.
    """
    satellite = REPO.parent / "laterite-dev"
    if not (satellite / ".git").exists():
        import pytest

        pytest.skip("dev satellite not checked out beside this repo")

    there = {p.name for p in satellite.rglob("test_*.py") if "/target/" not in str(p)}
    real = _real_tests()
    phantom: list[str] = []
    for path in _scope_files():
        flat = " ".join(path.read_text(encoding="utf-8").split())
        rel = path.relative_to(REPO).as_posix()
        for m in TOKEN.finditer(flat):
            name = m.group(1)
            if name in real or name in there:
                continue
            if "dev satellite" in _window(flat, m.end()):
                phantom.append(f"{rel}: {name}")

    assert not phantom, (
        "these claim a test runs in the dev satellite, which has no such file:\n  "
        + "\n  ".join(sorted(set(phantom)))
    )
