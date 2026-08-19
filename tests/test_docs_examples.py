"""Runnable-guarantee gate for the docs site's Python snippets.

The twin of `rust-packages/laterite-node/test/docs-examples.test.ts`, which has
named this file in its header since #373 — but the file itself never existed, so
only the Node half of the guarantee was ever enforced. It bit exactly as you'd
expect: a behaviour change landed with the Node examples caught by CI and the
Python examples silently broken, because nothing ran them.

Executes every `web/docs-site/examples/python/*.py` as a real subprocess from the
repo root, so the `examples/sample_site.ags` relative paths inside them resolve.
Each example ends in `assert` statements, so a changed return shape, property
name or printed format turns a doc snippet red HERE rather than in a reader's
terminal. The doc pages `--8<--`-include these exact files: page and test are the
same bytes.

Discovery is by glob, deliberately — a new example is covered the moment it is
added, with no list to forget to update.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest

# Needs the built surfaces: this module executes the docs example scripts, which import `laterite`. The buildless
# `repo-gates` job deselects it; the `python` job runs it after the wheel
# and the CLI exist.
pytestmark = pytest.mark.needs_env


REPO_ROOT = Path(__file__).resolve().parents[1]
EXAMPLE_DIR = REPO_ROOT / "web" / "docs-site" / "examples" / "python"

EXAMPLES = sorted(EXAMPLE_DIR.glob("ex*.py"))


def test_examples_are_discovered() -> None:
    """Zero is a bad witness: an empty glob would make every case below vacuous.

    If the examples move, this fails loudly instead of the suite reporting a
    green run over nothing.
    """
    assert EXAMPLES, f"no docs examples found under {EXAMPLE_DIR}"


@pytest.mark.parametrize("example", EXAMPLES, ids=lambda p: p.name)
def test_docs_example_runs(example: Path) -> None:
    proc = subprocess.run(
        [sys.executable, str(example)],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        timeout=300,
    )
    assert proc.returncode == 0, (
        f"{example.name} is a published doc snippet and it does not run.\n"
        f"--- stdout ---\n{proc.stdout}\n--- stderr ---\n{proc.stderr}"
    )
