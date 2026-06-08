"""Shared test fixtures for the laterite-ags5 suite.

Same autouse isolation fixtures as the laterite suite (conftest.py is
not importable across package test roots, so the two are intentionally
parallel) — they make ordering-independent test runs safe under
`pytest -n` (xdist). The `dynamic` cache one matters here because the
typed-graph passthrough tests assert `isinstance` identity off it.
"""

from __future__ import annotations

import pytest


@pytest.fixture(autouse=True)
def _restore_compat_backend():
    """Snapshot/restore the process-global compat backend (see the laterite
    suite's conftest for the rationale)."""
    from laterite import _frames

    saved = _frames._DEFAULT_BACKEND
    try:
        yield
    finally:
        _frames._DEFAULT_BACKEND = saved


@pytest.fixture(autouse=True)
def _clear_dynamic_cache():
    """Start each test with an empty dynamic-class cache so passthrough
    `isinstance` identity assertions are order-independent."""
    from laterite import dynamic

    dynamic.clear_cache()
    yield
