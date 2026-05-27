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

HeadingStatus = Literal["KEY", "REQUIRED", "OTHER"]


@dataclass(frozen=True, slots=True)
class Heading:
    name: str
    status: HeadingStatus
    type: str
    unit: str | None = None
    description: str = ""
    indexed: bool | None = None

    @property
    def py_name(self) -> str:
        return self.name.lower()


@dataclass(frozen=True, slots=True)
class GroupDescriptor:
    code: str
    contents: str
    parent: str | None
    headings: tuple[Heading, ...]
    is_high_volume: bool = False
    index_parent: bool | None = None

    @property
    def table(self) -> str:
        return f"g_{self.code.lower()}"

    @property
    def view(self) -> str:
        return f"v_{self.code.lower()}"

    @property
    def key_headings(self) -> tuple[Heading, ...]:
        return tuple(h for h in self.headings if h.status == "KEY")

    @property
    def non_key_headings(self) -> tuple[Heading, ...]:
        return tuple(h for h in self.headings if h.status != "KEY")


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
            is_high_volume=g.get("is_high_volume", False),
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
    """KEY heading names a group inherits from its parent (matches
    ``ags5_db._ddl._inherited_key_names``). Returns a set; the Rust
    side gives a sorted list for determinism, which we wrap."""
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
