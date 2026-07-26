"""Dynamic typed-graph classes for custom AGS groups.

Stage F2b-4. Built on demand by ``laterite.ags4.read_typed`` when a file
declares a group not in the compiled-in typed-graph registry.

The factory caches classes process-wide by
``(code, sorted-heading-tuple)`` so subsequent reads with the same
shape reuse the same class object — ``isinstance`` works as expected.
Conflicting shapes get a disambiguated suffix
(``MYCUSTOM__<8-char-hash>``) so both shapes remain registered and
importable.

Once registered, classes are importable::

    from laterite.dynamic import MYCUSTOM
    MYCUSTOM(field1="x", field2=1.5)

The classes have full parity with the compiled ``#[pyclass]`` types
for the 92 standard groups: kwargs constructor with field validation
(unknown kwargs raise TypeError), per-instance attribute access,
``__repr__``, and a ``walk(code)`` method (returns ``[]`` — dynamic
classes are passthrough leaves; their children, if any, attach via
``setattr`` from ``read_db`` and are reachable via
``getattr(parent, child_field, [])``).
"""

from __future__ import annotations

import hashlib
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from collections.abc import Callable

__all__ = ["clear_cache", "get_or_register", "registered_classes"]


# (code, sorted-heading-tuple) → class
_CACHE: dict[tuple[str, tuple[tuple[str, str], ...]], type] = {}

# code → list of (cache_key, class) for shape-conflict disambiguation
_BY_CODE: dict[str, list[tuple[Any, type]]] = {}


def _shape_key(headings: list[dict[str, Any]]) -> tuple[tuple[str, str], ...]:
    """Cache key from a heading list.

    Sorted by name; includes the AGS type so a rename or type change
    surfaces as a distinct shape (and therefore a distinct class).
    """
    return tuple(sorted((h["name"], h.get("type", "X")) for h in headings))


def _make_init(field_names: tuple[str, ...]) -> Callable[..., None]:
    """Build an ``__init__`` that accepts kwargs matching field_names
    only — anything else raises ``TypeError`` (mirrors the compiled
    ``#[pyclass]`` contract). Missing fields default to None."""
    field_set = frozenset(field_names)

    def __init__(self: object, **kwargs: object) -> None:
        unknown = [k for k in kwargs if k not in field_set]
        if unknown:
            raise TypeError(
                f"{type(self).__name__}.__init__() got unexpected "
                f"keyword arguments: {sorted(unknown)}"
            )
        for name in field_names:
            object.__setattr__(self, name, kwargs.get(name))

    return __init__


def _make_repr(name: str) -> Callable[..., str]:
    def __repr__(self: object) -> str:
        return f"{name}(...)"

    return __repr__


def _walk(self: object, code: str) -> list:
    """Dynamic classes are passthrough — by policy ``walk`` returns
    ``[]``. Passthrough children attached via ``setattr`` from
    ``read_db`` are reachable via ``getattr(parent, child_field, [])``,
    not via ``walk``."""
    return []


def get_or_register(
    code: str,
    headings: list[dict[str, Any]],
) -> type:
    """Return the cached class for ``(code, headings)`` or build one.

    ``headings`` is a list of dicts with at least ``name`` and
    ``type`` keys (the shape produced by reading ``_spec_headings``).
    """
    code_upper = code.upper()
    key = (code_upper, _shape_key(headings))
    cached = _CACHE.get(key)
    if cached is not None:
        return cached

    by_code = _BY_CODE.setdefault(code_upper, [])
    if by_code:
        # A class for this code already exists with a different
        # heading set — disambiguate the new one's __name__ with a
        # stable hash of the shape tuple. Both classes coexist; both
        # are importable from `laterite.dynamic`.
        digest = hashlib.sha1(
            repr(key[1]).encode("utf-8"),
            usedforsecurity=False,
        ).hexdigest()[:8]
        cls_name = f"{code_upper}__{digest}"
    else:
        cls_name = code_upper

    field_names = tuple(h["name"].lower() for h in headings)
    # Store the full ordered (UPPERCASE name, AGS type) tuples so the
    # F2b-5b write_db path can rebuild a GroupDescriptor for the
    # session-extended registry without re-reading the file. Order
    # matches the source headings list (NOT sorted by name).
    heading_specs = tuple((h["name"].upper(), h.get("type", "X")) for h in headings)
    namespace: dict[str, Any] = {
        "__init__": _make_init(field_names),
        "__repr__": _make_repr(cls_name),
        "walk": _walk,
        "_ags_code": code_upper,
        "_ags_headings": field_names,
        "_ags_heading_specs": heading_specs,
        "__slots__": (),  # let __dict__ hold the per-instance fields
    }
    # `__slots__ = ()` is dropped because we need __dict__ for the
    # dynamic attribute assignment in `__init__`. Remove the namespace
    # key entirely; type() without __slots__ gives a default __dict__.
    namespace.pop("__slots__", None)

    cls = type(cls_name, (), namespace)
    cls.__module__ = "laterite.dynamic"

    _CACHE[key] = cls
    by_code.append((key, cls))
    return cls


def registered_classes() -> dict[str, list[type]]:
    """Snapshot of currently-registered dynamic classes.

    Returns a dict ``{code: [class, class_variant_2, ...]}``. Useful
    for introspection in tests / debug sessions.
    """
    return {code: [cls for _, cls in variants] for code, variants in _BY_CODE.items()}


def clear_cache() -> None:
    """Drop every cached class. Used by tests that need an isolated
    registration namespace; production callers shouldn't normally
    need this."""
    _CACHE.clear()
    _BY_CODE.clear()


def __getattr__(name: str) -> type:
    """Module-level ``__getattr__`` (PEP 562). Enables
    ``from laterite.dynamic import MYCUSTOM`` after registration."""
    for variants in _BY_CODE.values():
        for _, cls in variants:
            if cls.__name__ == name:
                return cls
    raise AttributeError(
        f"module 'laterite.dynamic' has no attribute {name!r} "
        f"(no class with that name registered yet)"
    )


def __dir__() -> list[str]:
    """Make registered class names visible to ``dir()`` and to
    tooling that introspects the module."""
    base = [*list(__all__), "__name__", "__doc__"]
    base.extend(cls.__name__ for variants in _BY_CODE.values() for _, cls in variants)
    return sorted(set(base))
