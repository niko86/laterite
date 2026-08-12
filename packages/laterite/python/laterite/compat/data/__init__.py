"""Mirrors ``python_ags4.data`` — a sample AGS4 file and the loader for it.

``load_test_data`` is taught in upstream's own README, so it is part of the
surface a port has to carry, not an internal.

**The file here is ours.** Upstream's ``test_data.ags`` is LGPL-3.0 and must not
enter this tree, so ``test_data.ags`` beside this module is clean-room: generated
by ``laterite-ags4-forge`` (``gen --scaffold loca-samp``) from laterite's own
bundled v4.2 dictionary, then verified to validate clean under BOTH the Rust
engine and real python-ags4 1.2.0. It keeps upstream's *filename* so that code
reaching for ``DATA_DIR / "test_data.ags"`` ports unchanged — but the contents
are a different (8-group: PROJ, TRAN, LOCA, SAMP, GEOL, ABBR, UNIT, TYPE) file,
so anything asserting on specific upstream values will not match, by design.
"""

from pathlib import Path
from typing import Any

from .._impl import AGS4_to_dataframe

DATA_DIR = Path(__file__).parent

# Individual files
TEST_DATA = DATA_DIR / "test_data.ags"


def load_test_data(*args: Any, **kwargs: Any) -> tuple:
    """Load test data.

    Note
    ----
    This wraps a call to ``AGS4.AGS4_to_dataframe``. All arguments and keyword
    arguments are passed directly to that method.

    """
    return AGS4_to_dataframe(TEST_DATA, *args, **kwargs)


__all__ = ["DATA_DIR", "TEST_DATA", "load_test_data"]
