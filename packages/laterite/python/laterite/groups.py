"""laterite.groups — the AGS4 typed-graph classes (``PROJ``, ``LOCA``, ``SAMP``, …).

The 174 typed classes are compiled into the ``_laterite_native`` extension (one
``#[pyclass]`` per AGS group, generated from ``ags_dictionary.json`` at build
time). This module re-exports them so ``from laterite.groups import PROJ, LOCA``
works and editors autocomplete the codes — while keeping the 174 four-letter
names out of the top-level ``laterite`` namespace, which carries the
read / validate / build API. (Custom groups built at runtime live in the sibling
``laterite.dynamic``; this module is their static, dictionary-defined counterpart.)
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from . import _laterite_native as _native
from .registry import GROUPS as _GROUPS

# The class objects live in the compiled extension; this loop aliases them onto
# this submodule so they import from `laterite.groups`. (The package root used to
# carry these aliases — moved here to declutter the top-level namespace.)
__all__ = sorted(_GROUPS)
for _code in __all__:
    globals()[_code] = getattr(_native, _code)
del _code

# The runtime loop above is invisible to static analysers, so without this they
# flag `from laterite.groups import PROJ` as an unknown symbol — no autocomplete,
# spurious type errors. Re-importing the classes here (type-check time only,
# never executed) makes them statically visible; the star is restricted to the
# typed-graph classes by `_laterite_native.pyi`'s generated `__all__`.
if TYPE_CHECKING:
    from ._laterite_native import *  # noqa: F403
