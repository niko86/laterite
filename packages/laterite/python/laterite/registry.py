"""laterite.registry — Rust-backed AGS group registry (read-only).

Exposes typed group/heading descriptors, populated from the Rust
crate's `OnceLock` singleton via PyO3 (no second JSON parse in
Python).

Read-only: dictionary content is established at Rust crate load
time. Runtime extension for AGS4 passthrough groups (groups present
in a file but not in the bundled dictionary) lives in
`laterite.dynamic`.
"""

from __future__ import annotations

import json
from dataclasses import dataclass
from typing import Literal

from . import _laterite_native as _native

# The official AGS dictionary uses combined statuses (e.g. "KEY+REQUIRED")
# and a "DEPRECATED" marker alongside the base KEY/REQUIRED/OTHER.
HeadingStatus = Literal["KEY", "REQUIRED", "OTHER", "KEY+REQUIRED", "DEPRECATED"]


@dataclass(frozen=True, slots=True)
class Heading:
    """One heading (column) of an AGS group, as the dictionary defines it.

    Frozen because the registry is read-only — a ``Heading`` is a fact
    about the AGS spec, not mutable state. ``status`` keeps the
    dictionary's own vocabulary verbatim, including the combined
    ``"KEY+REQUIRED"`` and the ``"DEPRECATED"`` marker; use ``is_key``
    rather than an ``==`` test so those combined forms are handled.
    """

    name: str
    status: HeadingStatus
    type: str
    unit: str | None = None
    description: str = ""
    indexed: bool | None = None

    @property
    def py_name(self) -> str:
        """The heading name lower-cased — the identifier the typed
        classes and DuckDB columns use, where the dictionary's
        upper-case form isn't a legal attribute/column name."""
        return self.name.lower()

    @property
    def is_key(self) -> bool:
        """Whether this heading is a KEY column. Tests the
        ``+``-separated parts of ``status`` so the combined
        ``"KEY+REQUIRED"`` form counts, where a plain ``== "KEY"``
        would miss it."""
        return any(p.strip().upper() == "KEY" for p in self.status.split("+"))


@dataclass(frozen=True, slots=True)
class GroupDescriptor:
    """The full spec of one AGS group: its 4-letter ``code``, the
    dictionary's prose ``contents``, the ``parent`` code that places it
    in the PROJ tree (``None`` at the root), and its ``headings`` in
    dictionary order.

    Frozen for the same reason as ``Heading`` — this describes the AGS
    dictionary, which is fixed at Rust crate load. The ``table``/``view``
    getters give the conventional DuckDB names a ``.ags5db`` store uses
    for the group.
    """

    code: str
    contents: str
    parent: str | None
    headings: tuple[Heading, ...]
    index_parent: bool | None = None

    @property
    def table(self) -> str:
        """The group's physical table name in a ``.ags5db`` store —
        ``g_<code>`` lower-cased."""
        return f"g_{self.code.lower()}"

    @property
    def view(self) -> str:
        """The group's view name in a ``.ags5db`` store — ``v_<code>``
        lower-cased — the parent-joined view that complements the raw
        ``table``."""
        return f"v_{self.code.lower()}"

    @property
    def key_headings(self) -> tuple[Heading, ...]:
        """The KEY headings — the columns that identify a row in this
        group — in dictionary order."""
        return tuple(h for h in self.headings if h.is_key)

    @property
    def non_key_headings(self) -> tuple[Heading, ...]:
        """The non-KEY headings — every column that doesn't take part
        in identifying a row — in dictionary order."""
        return tuple(h for h in self.headings if not h.is_key)


def _load() -> dict[str, GroupDescriptor]:
    """Decode the Rust-served JSON into frozen dataclasses once, at
    import time. F2c-6 swapped this off msgspec to stdlib so the
    laterite runtime carries one less wheel."""
    payload = json.loads(_native.registry_groups_json())
    out: dict[str, GroupDescriptor] = {}
    for g in payload:
        headings = tuple(
            Heading(
                name=h["name"],
                status=h["status"],
                type=h["type"],
                unit=h.get("unit"),
                description=h.get("description", ""),
                indexed=h.get("indexed"),
            )
            for h in g["headings"]
        )
        desc = GroupDescriptor(
            code=g["code"],
            contents=g["contents"],
            parent=g.get("parent"),
            headings=headings,
            index_parent=g.get("index_parent"),
        )
        out[desc.code] = desc
    return out


GROUPS: dict[str, GroupDescriptor] = _load()


def get(code: str) -> GroupDescriptor | None:
    """Single-group lookup. ``None`` for unknown codes (mirrors
    ``GROUPS.get(code)``)."""
    return GROUPS.get(code)


def ancestor_chain(code: str) -> list[str]:
    """Parent chain from ``code`` to root: ``[code, parent, ..., root]``.

    Raises ``ValueError`` if ``code`` isn't in the registry (so callers
    can distinguish "no parent" — root groups return ``[code]`` — from
    "unknown code"). Computed in Rust against the static registry.
    """
    return _native.registry_ancestor_chain(code)


def inherited_key_names(code: str) -> set[str]:
    """The KEY heading names this group shares with its direct
    parent — the keys it inherits rather than declaring fresh.

    The intersection of this group's KEY headings with its immediate
    parent's (only the direct parent, not the whole ancestor chain),
    computed in Rust against the static registry. Rust returns a
    sorted list for determinism; we hand it back as a set since order
    carries no meaning to callers. Empty when the group is a root or
    its parent is unknown.
    """
    return set(_native.registry_inherited_key_names(code))


def child_groups(parent_code: str) -> list[GroupDescriptor]:
    """Every direct child group of ``parent_code``, in alphabetical
    order. Same shape ``ags5_models.child_groups`` returned (replaces
    that helper as of F2c-4)."""
    return sorted(
        (g for g in GROUPS.values() if g.parent == parent_code),
        key=lambda g: g.code,
    )


__all__ = [
    "Heading",
    "HeadingStatus",
    "GroupDescriptor",
    "GROUPS",
    "get",
    "ancestor_chain",
    "inherited_key_names",
    "child_groups",
]
