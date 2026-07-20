"""Shared test fixtures for the laterite suite.

Two autouse isolation fixtures make every test self-contained with
respect to laterite's two process-global mutable states, so test
ordering can't leak state between tests. This is the prerequisite for
running under `pytest -n` (xdist), which re-groups tests across worker
processes and would otherwise expose any implicit ordering dependency.
"""

from __future__ import annotations

import pytest


@pytest.fixture(autouse=True)
def _restore_compat_backend():
    """Snapshot/restore the process-global compat backend.

    `laterite.compat.set_backend()` mutates `_frames._DEFAULT_BACKEND`; a
    test that changes it must not leak it into whatever test runs next on
    the same worker. Snapshot before, restore after.
    """
    from laterite import _frames

    saved = _frames._DEFAULT_BACKEND
    try:
        yield
    finally:
        _frames._DEFAULT_BACKEND = saved


@pytest.fixture(autouse=True)
def _clear_dynamic_cache():
    """Start each test with an empty dynamic-class cache.

    `laterite.dynamic` caches runtime-built passthrough classes process-
    wide, and that cache backs `isinstance` identity. Clearing it in setup
    (before the test body) makes identity assertions order-independent —
    the test repopulates the cache as it runs.
    """
    from laterite import dynamic

    dynamic.clear_cache()
    return
