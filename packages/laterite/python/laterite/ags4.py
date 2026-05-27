"""AGS4 → typed PROJ tree, in one call.

Stage F2b-4b. Companion to ``laterite.ags5db.read_db`` for callers
who have an AGS4 transfer file (rather than a ``.ags5db``) and want
the typed-graph form directly.

Implementation routes via a temp ``.ags5db`` under the hood:
``convert(ags4, tmp)`` then ``read_db(tmp)`` then unlink. This is
the same shortcut ``ags5_xml.ags4_to_agsx`` already uses — robust,
reuses every tested code path, and ~100 ms overhead on a 23 MB
file (acceptable for an inspection-style helper).

Custom / passthrough groups in the AGS4 source flow through the
exact same dynamic-class machinery as ``read_db`` (the Rust
converter writes ``_spec_*`` rows for unknowns; ``read_db`` reads
them and calls the ``laterite.dynamic`` factory).
"""

from __future__ import annotations

import tempfile
from os import PathLike
from pathlib import Path
from typing import Any

from laterite.ags5db import convert, read_db

__all__ = ["read_typed"]


def read_typed(
    ags4: str | PathLike[str],
    *,
    attachments_dir: str | PathLike[str] | None = None,
) -> Any:
    """Read an AGS4 transfer file and return its typed PROJ tree.

    Args:
        ags4: Path to the ``.ags`` source file.
        attachments_dir: Optional AGS4 Rule 20 FILE-attachment root
            (defaults to the source file's parent).

    Returns:
        A PROJ instance (the same shape ``laterite.ags5db.read_db``
        returns). Standard groups are compiled ``#[pyclass]`` types;
        custom / passthrough groups are dynamic Python classes from
        ``laterite.dynamic``.
    """
    ags4 = Path(ags4)
    with tempfile.NamedTemporaryFile(
        suffix=".ags5db", delete=False,
    ) as tmp:
        tmp_db = Path(tmp.name)
    try:
        convert(ags4, tmp_db, attachments_dir=attachments_dir)
        return read_db(tmp_db)
    finally:
        tmp_db.unlink(missing_ok=True)
