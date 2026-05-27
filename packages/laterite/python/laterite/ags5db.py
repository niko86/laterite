"""laterite.ags5db — lazy re-export from ``laterite_ags5``.

The AGS5 (``.ags5db``) read/write/query surface lives in the separate
``laterite-ags5`` companion wheel (it bundles DuckDB, ~50 MB linked).
Install it via the ``[ags5]`` extra::

    pip install "laterite[ags5]"

This module is a thin re-export so ``from laterite.ags5db import …``
keeps working — even though the implementation now ships in the
``laterite_ags5`` package — and raises an informative
``ModuleNotFoundError`` if the extra isn't installed.
"""
from __future__ import annotations

try:
    from laterite_ags5 import *  # noqa: F401,F403
    from laterite_ags5 import __all__  # noqa: F401
except ModuleNotFoundError as e:
    raise ModuleNotFoundError(
        "laterite.ags5db requires the 'ags5' extra. "
        "Install with: pip install 'laterite[ags5]'"
    ) from e
