"""Keep `needs_env` modules out of the buildless job's COLLECTION, not just its run.

`-m "not needs_env"` deselects, and deselection happens after import — pytest
has to import a module to find out what it is marked with. So a module whose
very first lines import something the buildless `repo-gates` job does not have
never reaches the deselect: it raises at collection, and a collection error
takes the whole session down (exit 2) rather than the one file.

That is not hypothetical. `test_vendored_authority_faithful` imports
`python_ags4` at module scope; the marker was correct, the mechanism was not,
and CI said `ModuleNotFoundError` where a developer's machine said 345 passed.

The fix keeps the marker as the single source and changes only WHEN it is read.
`pytest_ignore_collect` runs before import, so the marker is read the only way
it can be at that point — off the source text, with an AST walk. Deliberately
NOT `pytest.importorskip`: that turns a vanished dependency into a green skip in
the job that is supposed to be exercising it.
"""

from __future__ import annotations

import ast
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from pathlib import Path

MARKER = "needs_env"


def _declares_marker(path: Path) -> bool:
    """True if the module assigns `pytestmark` the marker, read without importing.

    An AST walk rather than a text search because this suite talks ABOUT the
    marker: tests/test_build_marker_faithful.py names it in assertion messages,
    and a grep would ignore the very file that polices it.
    """
    try:
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
    except (OSError, SyntaxError):
        return False
    for node in tree.body:
        if not isinstance(node, ast.Assign):
            continue
        if not any(
            isinstance(t, ast.Name) and t.id == "pytestmark" for t in node.targets
        ):
            continue
        values = (
            node.value.elts
            if isinstance(node.value, (ast.List, ast.Tuple))
            else [node.value]
        )
        for value in values:
            if isinstance(value, ast.Attribute) and value.attr == MARKER:
                return True
    return False


def pytest_ignore_collect(collection_path: Path, config) -> bool | None:
    """Skip the file outright when the run has asked to exclude the marker.

    Keyed on the mark expression rather than on whether the imports happen to
    resolve — a run that asks for these tests must fail loudly when it cannot
    have them.
    """
    if (config.option.markexpr or "").replace(" ", "") != f"not{MARKER}":
        return None
    if collection_path.suffix != ".py":
        return None
    return True if _declares_marker(collection_path) else None
