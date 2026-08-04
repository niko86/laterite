"""Docs that quote the wheel's own metadata must quote what it says.

`ags-wiki/tools/laterite.md` carried five false `narwhals` claims, including a
VERBATIM quote of `pyproject.toml`'s description that the file has not said for
some time — the pyproject even carries a comment reading "narwhals is GONE". The
page also advertised a `[pandas]` extra that does not exist and described
`[compat]` as pulling pyarrow, which it deliberately does not.

Nobody edited that page wrongly. The packaging moved and the prose stayed, which
is the same mechanism behind every finding in the 2026-08-04 audit.

Two checks, both cheap and both zero-false-positive by construction: a quoted
description must match, and an advertised extra must exist. Neither obliges a doc
to mention either thing — the gate is agreement, not coverage.
"""

from __future__ import annotations

import re
import tomllib
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
PYPROJECT = REPO / "packages" / "laterite" / "pyproject.toml"

#: Docs a reader could act on. Deliberately not the whole tree: a changelog or a
#: decision record may quote an OLD description correctly, as a record of what it
#: said then.
DOCS = [
    REPO / "ags-wiki" / "tools" / "laterite.md",
    REPO / "packages" / "laterite" / "README.md",
    REPO / "COMPAT.md",
]

#: `pip install "laterite[compat,pyarrow]"` and friends.
EXTRA_RE = re.compile(r"laterite\[([a-z0-9,\s-]+)\]")


def _project() -> dict:
    return tomllib.loads(PYPROJECT.read_text(encoding="utf-8"))["project"]


def _flat(text: str) -> str:
    """Collapse wrapping so a quote broken across lines still compares."""
    return " ".join(text.split())


def test_a_quoted_description_is_the_real_one() -> None:
    """If a doc quotes the description, it must be the current one.

    Falsify by editing `description` in pyproject.toml without touching the
    pages that quote it.
    """
    desc = _flat(_project()["description"])
    # The distinctive tail is enough to tell "quoting it" from "mentioning it".
    fingerprint = "drop-in replacement for python-ags4"
    stale = []
    for doc in DOCS:
        if not doc.exists():
            continue
        flat = _flat(doc.read_text(encoding="utf-8"))
        if fingerprint in flat and desc not in flat:
            stale.append(doc.relative_to(REPO).as_posix())
    assert not stale, (
        "these docs quote pyproject's description but not what it currently "
        "says:\n  " + "\n  ".join(stale) + f"\n\ncurrent: {desc!r}"
    )


def test_every_advertised_extra_exists() -> None:
    """A doc telling a user to `pip install laterite[x]` must name a real extra.

    Falsify by renaming an extra in pyproject.toml, or by advertising one that
    was never added.
    """
    real = set(_project().get("optional-dependencies", {}))
    bad: list[str] = []
    for doc in DOCS:
        if not doc.exists():
            continue
        rel = doc.relative_to(REPO).as_posix()
        for m in EXTRA_RE.finditer(doc.read_text(encoding="utf-8")):
            bad.extend(
                f"{rel}: laterite[{name}]"
                for name in (p.strip() for p in m.group(1).split(","))
                if name and name not in real
            )
    assert not bad, (
        f"docs advertise extras that do not exist (real: {sorted(real)}):\n  "
        + "\n  ".join(sorted(set(bad)))
    )


def test_the_base_install_is_described_as_what_it_is() -> None:
    """The wheel page must not claim a base dependency the wheel does not have.

    Narrow on purpose: it asserts the page names every real base dependency and
    names no package as a base dependency that is only an extra. DuckDB is
    load-bearing in the base — it is the pyarrow-free dataframe bridge — so a
    page that omits it understates what a plain `pip install` pulls.

    An earlier version of this test asserted only that each dependency name
    appeared SOMEWHERE on the page. That passed when the base-install line was
    changed from duckdb to narwhals, because "duckdb" still occurred further
    down — a test that looked like it checked something and did not. So it now
    builds the expected phrase from the manifest and requires that exact phrase,
    which is what makes a dependency change force a prose change.
    """
    project = _project()
    base = [re.split(r"[><=!\[ ]", d)[0].strip() for d in project["dependencies"]]
    page = _flat(
        (REPO / "ags-wiki" / "tools" / "laterite.md").read_text(encoding="utf-8")
    )
    expected = " + ".join(f"`{p}`" for p in base)
    assert expected in page, (
        f"laterite.md must name the base install as {expected} — built from "
        f"pyproject's `dependencies`. If the dependencies changed, the prose has "
        f"to change with them."
    )
