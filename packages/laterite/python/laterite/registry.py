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
from typing import TYPE_CHECKING, Any, Literal

from . import _laterite_native as _native

if TYPE_CHECKING:
    from . import Edition

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
    getters give the conventional DuckDB table/view names for the
    group.
    """

    code: str
    contents: str
    parent: str | None
    headings: tuple[Heading, ...]

    @property
    def table(self) -> str:
        """The group's physical table name (``g_<code>`` lower-cased)
        in a DuckDB store."""
        return f"g_{self.code.lower()}"

    @property
    def view(self) -> str:
        """The group's view name (``v_<code>`` lower-cased) in a DuckDB
        store — the parent-joined view that complements the raw
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
            )
            for h in g["headings"]
        )
        desc = GroupDescriptor(
            code=g["code"],
            contents=g["contents"],
            parent=g.get("parent"),
            headings=headings,
        )
        out[desc.code] = desc
    return out


class _ReadOnlyGroups(dict):
    """A read-only ``dict`` for the module-level registry.

    It **is** a ``dict`` (so ``isinstance(GROUPS, dict)`` holds and the
    ``dict[str, GroupDescriptor]`` type stays honest for callers + the ``ty``
    gate) but every mutator raises — sealing the process-global registry *in
    place*, the same guarantee ``laterite-node``'s ``registry.ts`` gives with
    ``Object.freeze``. ``GROUPS`` is the projection of the single-source
    ``ags_dictionary.json``; a stray ``GROUPS[code] = …`` / ``.clear()`` by any
    importer would silently corrupt it for the whole process. The values are
    already ``frozen=True`` dataclasses, so this shallow container seal is
    effectively deep. (It does not stop *rebinding the name* ``registry.GROUPS``
    — nothing can — only mutating the mapping's contents.)
    """

    __slots__ = ()

    def _readonly(self, *_args: Any, **_kwargs: Any) -> Any:
        raise TypeError(
            "laterite.registry.GROUPS is read-only (the union AGS4 registry, "
            "projected from ags_dictionary.json). Register a custom group at "
            "runtime with laterite.dynamic.get_or_register() instead."
        )

    __setitem__ = _readonly
    __delitem__ = _readonly
    __ior__ = _readonly
    clear = _readonly
    pop = _readonly
    popitem = _readonly
    setdefault = _readonly
    update = _readonly

    def __reduce__(self) -> tuple[Any, ...]:
        # Preserve the read-only type across pickle: dict.__init__ populates at
        # the C level, bypassing the sealed __setitem__.
        return (self.__class__, (dict(self),))


GROUPS: dict[str, GroupDescriptor] = _ReadOnlyGroups(_load())


def get(code: str) -> GroupDescriptor | None:
    """Single-group lookup. ``None`` for unknown codes (mirrors
    ``GROUPS.get(code)``)."""
    return GROUPS.get(code)


def dictionary(edition: Edition | None = None) -> dict[str, Any]:
    """The bundled STANDARD dictionary for one AGS **edition** — the per-edition
    view of the official dictionary.

    Where the module-level :data:`GROUPS` is the *union* registry across all
    editions (the typed-graph / DDL model, and the default), this is a single
    edition's standard dictionary: canonical group + heading names, descriptions,
    UNIT/TYPE, and status. It's the same content the browser's dictionary
    reference and Node's ``registry.dictionary()`` render, built from one shared
    Rust builder (``dict::dictionary_dto``).

    The shape is ``{ags_edition, groups: [{code, contents, parent, headings:
    [{name, status, type, unit?, description}]}]}`` — groups sorted by code, each
    group's headings in canonical dictionary order.

    Args:
        edition: One of ``"4.0.3" | "4.0.4" | "4.1" | "4.1.1" | "4.2"``. ``None``
            (or ``"auto"``) uses the fallback edition.

    Returns:
        The dictionary snapshot as a nested ``dict``.

    Raises:
        ValueError: If ``edition`` is not a recognised edition.
    """
    return json.loads(_native.registry_dictionary_json(edition))


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
    order."""
    return sorted(
        (g for g in GROUPS.values() if g.parent == parent_code),
        key=lambda g: g.code,
    )


__all__ = [
    "GROUPS",
    "GroupDescriptor",
    "Heading",
    "HeadingStatus",
    "ancestor_chain",
    "child_groups",
    "get",
    "inherited_key_names",
]
