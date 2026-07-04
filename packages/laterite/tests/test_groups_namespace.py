"""The 174 typed-graph classes live in ``laterite.groups``, NOT on the top-level
``laterite`` namespace — so ``laterite.<TAB>`` and ``from laterite import *`` show
only the read / validate / build API, not 174 four-letter AGS codes.

Renamed: ``from laterite import PROJ`` → ``from laterite.groups import PROJ``. A
clean break (the old top-level alias is gone); this gate guards both the new home
(every code present, re-exported not redefined) and the break (no code creeps
back onto the root).
"""

from __future__ import annotations

import laterite
import laterite.groups as groups
import pytest
from laterite import _laterite_native as _native
from laterite.registry import GROUPS

# The real API surface — must stay on the root regardless of the group move.
_API = ("read", "validate", "build_ags4", "fix", "diff", "list_rules", "fixable_rules")


def test_every_group_class_imports_from_laterite_groups():
    """One class per registry group, and each IS the compiled ``_laterite_native``
    class — the submodule re-exports, it does not redefine (so ``read_typed`` and
    the build walker, which key off ``__module__``, are unaffected)."""
    for code in GROUPS:
        assert getattr(groups, code) is getattr(_native, code)
    from laterite.groups import LOCA, PROJ  # the canonical spelling resolves

    assert PROJ.__name__ == "PROJ" and LOCA.__name__ == "LOCA"


def test_groups_dunder_all_equals_the_registry():
    """``__all__`` is exactly the dictionary's group set, so ``from laterite.groups
    import *`` pulls the typed classes and the set can't drift from the dict."""
    assert set(groups.__all__) == set(GROUPS)
    assert len(groups.__all__) == 174


def test_group_codes_are_absent_from_the_top_level_namespace():
    """The clean break: no four-letter code is an attribute of ``laterite`` or in
    its ``__all__`` — the top namespace carries only the read/validate/build API."""
    for code in GROUPS:
        assert not hasattr(laterite, code), f"{code} leaked back onto laterite"
        assert code not in laterite.__all__
    for name in _API:
        assert hasattr(laterite, name) and name in laterite.__all__


def test_from_laterite_import_a_group_raises():
    """``from laterite import PROJ`` is gone — importing a code from the root fails
    (no silent fall-through shim)."""
    with pytest.raises(ImportError):
        from laterite import PROJ  # noqa: F401


def test_laterite_groups_is_reachable_after_a_bare_import():
    """``import laterite`` makes ``laterite.groups`` available without a separate
    submodule import (the way ``laterite.registry`` is)."""
    import laterite as _l

    assert _l.groups.PROJ is _native.PROJ
