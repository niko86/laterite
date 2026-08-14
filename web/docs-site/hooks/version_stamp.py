"""Stamp the footer with the product version the docs were built from.

Ticket 03 settled that there are **no versioned docs before 1.0** — no `mike`,
one live version. The docs' integrity is that they are generated and
runtime-gated against HEAD: every snippet single-sourced from `examples/` with a
test asserting the page and the code are the same bytes. A frozen snapshot would
trade that (they are tested) for something weaker (they are archived), and add a
rot surface.

What a reader on an older release actually needs is not an archive but one fact:
that they are reading ahead of their install. So: a build-time stamp naming the
version, and a changelog link to see what moved.

Read from `packages/laterite/pyproject.toml` at build time rather than written
into `mkdocs.yml`, because a hand-typed version in the config would be a second
copy of a number that already exists — and a second copy is a thing that drifts.
Three clocks run in this repo (product, engine, facade); the docs document the
PRODUCT, which is the wheel's version.
"""

from __future__ import annotations

import tomllib
from pathlib import Path
from typing import Any

_PYPROJECT = (
    Path(__file__).resolve().parents[3] / "packages" / "laterite" / "pyproject.toml"
)
_CHANGELOG = "https://github.com/niko86/laterite/blob/main/CHANGELOG.md"
_BASE = "laterite — MIT-licensed AGS4 tooling"


def product_version() -> str:
    """The shipped wheel's version — the one a reader's `pip install` gave them."""
    return tomllib.loads(_PYPROJECT.read_text(encoding="utf-8"))["project"]["version"]


def stamp(version: str) -> str:
    return (
        f"{_BASE} · documents <strong>v{version}</strong> · "
        f'<a href="{_CHANGELOG}">changelog</a>'
    )


def on_config(config: Any, **_: Any) -> Any:
    # Fail the build rather than stamp a guess: an unreadable version here means
    # the path moved, and a footer quietly falling back to "unknown" is the kind
    # of silent degradation that survives for months.
    config.copyright = stamp(product_version())
    return config
