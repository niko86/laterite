"""AGS4 → typed PROJ tree, in one call — on the DuckDB-free base path.

Companion to ``laterite.ags5db.read_db`` for callers who have an AGS4
transfer file (rather than a ``.ags5db``) and want the typed-graph form
directly.

**Why this is hand-rolled here** (W2, 2026-06-16). ``read_typed`` is a
*base* AGS4 API — it must work on a plain ``pip install laterite`` with no
``[ags5]`` extra. It used to route ``convert(ags4 → temp .ags5db)`` then
``read_db(tmp)``, dragging the whole DuckDB-bundled ``laterite-ags5``
companion wheel into what is a pure-AGS4 read. This builds the same PROJ
tree from the base parser (``parse_primitives``), the registry, and
``parse_value`` — no DuckDB, nothing imported from ``laterite.ags5db``.

**Parity is enforced, not assumed.** The tree returned here must be
identical to the ``convert → read_db`` reference. The linkage below is a
faithful port of the converter's parent resolution
(``laterite-ags5-db/src/convert.rs`` — ``insert_group_rows`` /
``resolve_parent_uuid``): topological group order, content-key row dedup,
and the *shared-keys intersection* — for each child group, the parent's KEY
headings whose names the child also KEYs on (in parent-KEY order), joined
into a tuple and matched between parent and child rows. (Pseudo-key drift
like MOND_REF↔PIPE_REF needs no special case: drifted names simply fall out
of the name-intersection, so linkage rides the keys both layers share. The
implicit PROJ↔LOCA edge works the same way — PROJ_ID isn't a LOCA heading,
so the intersection is empty and every LOCA attaches to the one PROJ.) A
parity test in the ``[ags5]`` suite asserts byte-equality against the DuckDB
path on a real multi-group fixture, so drift in *either* implementation
fails loud.

Custom / passthrough groups (present in the file but not in the 92-entry
dictionary) flow through the same ``laterite.dynamic`` factory ``read_db``
uses: parent defaults to LOCA, every heading is OTHER-status with the file's
declared AGS type — matching ``convert.rs::build_passthrough_descriptors``.
"""

from __future__ import annotations

from os import PathLike
from pathlib import Path
from typing import Any

from laterite import _laterite_native as _native
from laterite import dynamic
from laterite.ags_types import parse_value
from laterite.registry import GROUPS, GroupDescriptor, Heading

__all__ = ["read_typed"]

# An unknown group's parent defaults to LOCA — mirrors
# convert.rs::build_passthrough_descriptors. The AGS keys preserve the real
# relationship in the data; this hint only decides where it hangs in the tree.
_PASSTHROUGH_PARENT = "LOCA"

# Join shared-key values into a lookup tuple. Mirrors the Rust
# ``encode_shared_tuple`` (LF + NUL — can't occur inside an AGS4 KEY value),
# so a partial-key row can't collide with a complete-key one.
_KEY_SEP = "\n\0"


def read_typed(
    ags4: str | PathLike[str],
    *,
    attachments_dir: str | PathLike[str] | None = None,
) -> Any:
    """Read an AGS4 transfer file and return its typed PROJ tree.

    Args:
        ags4: Path to the ``.ags`` source file.
        attachments_dir: Accepted for signature stability. The typed tree
            carries no binary side-channel (AGS4 Rule 20 FILE attachments
            only ever materialised in the ``.ags5db`` ``blob`` table, never
            in the returned tree), so this argument has no effect here.

    Returns:
        A PROJ instance — the same shape ``laterite.ags5db.read_db``
        returns. Standard groups are compiled ``#[pyclass]`` types; custom /
        passthrough groups are dynamic Python classes from
        ``laterite.dynamic``.

    Raises:
        FileNotFoundError: if ``ags4`` does not exist.
        RuntimeError: if the file fails to parse, or has no PROJ row.
    """
    ags4 = Path(ags4)
    if not ags4.exists():
        raise FileNotFoundError(str(ags4))
    _ = attachments_dir  # see docstring — retained, no-op for the tree

    parsed = _native.parse_primitives(path=str(ags4))
    if not parsed.get("ok"):
        raise RuntimeError(parsed.get("error") or f"failed to parse {ags4}")

    file_groups: dict[str, Any] = parsed["groups"]
    order: list[str] = parsed["group_order"]

    # Per-group descriptor: the dictionary entry for a standard group, a
    # synthesised passthrough descriptor for the rest. Plus its Python class.
    descriptors: dict[str, GroupDescriptor] = {
        code: _descriptor_for(code, file_groups[code]) for code in order
    }
    classes: dict[str, type] = {
        code: _class_for(code, descriptors[code]) for code in order
    }

    # (parent_code, child_code, shared_tuple) -> parent instance. Filled as
    # each parent row is built; topo order guarantees parents come first.
    lookup: dict[tuple[str, str, str], Any] = {}
    # code -> {content_key: instance}: identical raw rows collapse to one
    # instance (first occurrence wins) — matches the converter's dedup.
    dedup: dict[str, dict[tuple[tuple[str, str], ...], Any]] = {}
    root: Any = None

    for code in _topo_order(order, descriptors):
        desc = descriptors[code]
        fg = file_groups[code]
        cls = classes[code]
        file_headings: list[str] = fg["headings"]
        desc_names = [h.name for h in desc.headings]
        # The shared-key headings to index each of this group's rows under,
        # one shape per present child, so the child can find us.
        child_shared = _descendant_shared(code, desc, descriptors, order)
        group_dedup = dedup.setdefault(code, {})

        for raw_row in fg["rows"]:
            # strict=False: tolerate a ragged DATA row (fewer/more cells than
            # headings) — a missing cell reads as absent, matching the
            # converter's `row.get(...).unwrap_or("")`.
            cells = dict(zip(file_headings, raw_row["values"], strict=False))

            content_key = tuple((n, cells.get(n, "")) for n in sorted(desc_names))
            existing = group_dedup.get(content_key)
            if existing is not None:
                # Dup row: don't rebuild, but still index it so descendants
                # resolve to the original (matches the converter's re-index).
                _index_for_children(lookup, code, cells, child_shared, existing)
                continue

            kwargs = {
                h.name.lower(): parse_value(cells.get(h.name), h.type)
                for h in desc.headings
            }
            inst = cls(**kwargs)
            group_dedup[content_key] = inst

            parent = _resolve_parent(lookup, code, desc, descriptors, cells)
            if parent is not None:
                _attach_child(parent, code, inst)
            elif code == "PROJ" and root is None:
                root = inst
            # A non-PROJ row with no resolvable parent is an orphan: built (so
            # its own children could find it) but unreferenced — exactly as
            # read_db drops a row whose parent_id points to nothing.

            _index_for_children(lookup, code, cells, child_shared, inst)

    if root is None:
        raise RuntimeError(
            "no PROJ row found in file (every AGS4 transfer must have one)"
        )
    return root


# --- descriptors & classes -------------------------------------------


def _descriptor_for(code: str, fg: dict[str, Any]) -> GroupDescriptor:
    """Dictionary descriptor for a standard group; a synthesised one for a
    passthrough. The passthrough shape mirrors
    ``convert.rs::build_passthrough_descriptors``: parent LOCA, every heading
    OTHER-status with the file's declared AGS type (empty → ``X``)."""
    standard = GROUPS.get(code)
    if standard is not None:
        return standard
    # Keep EVERY heading, padding a missing/empty TYPE cell to 'X' — mirrors
    # convert.rs::build_passthrough_descriptors (`types.get(i)... unwrap_or("X")`).
    # (A plain zip would instead drop headings past a short TYPE row.)
    ftypes = fg["types"]
    headings = tuple(
        Heading(
            name=name,
            status="OTHER",
            type=(ftypes[i] if i < len(ftypes) and ftypes[i] else "X"),
        )
        for i, name in enumerate(fg["headings"])
    )
    return GroupDescriptor(
        code=code,
        contents=f"(passthrough) {code}",
        parent=_PASSTHROUGH_PARENT,
        headings=headings,
    )


def _class_for(code: str, desc: GroupDescriptor) -> type:
    """Compiled ``#[pyclass]`` for a standard group; a ``laterite.dynamic``
    class for a passthrough (the same factory ``read_db`` uses)."""
    if code in GROUPS:
        return getattr(_native, code)
    return dynamic.get_or_register(
        code, [{"name": h.name, "type": h.type} for h in desc.headings]
    )


# --- topological order ------------------------------------------------


def _topo_order(order: list[str], descriptors: dict[str, GroupDescriptor]) -> list[str]:
    """Present groups, parents before children (DFS up the parent chain).
    The only invariant that matters is that a parent group is fully built
    before any child group references it."""
    visited: set[str] = set()
    out: list[str] = []

    def visit(code: str) -> None:
        if code in visited or code not in descriptors:
            return
        parent = descriptors[code].parent
        if parent is not None:
            visit(parent)
        visited.add(code)
        out.append(code)

    for code in order:
        visit(code)
    return out


# --- shared-keys parent resolution -----------------------------------


def _shared_headings(parent: GroupDescriptor, child: GroupDescriptor) -> list[Heading]:
    """The shared-key intersection: the parent's KEY headings whose names the
    child also KEYs on, in parent-KEY order. Index-time (on the parent) and
    lookup-time (on the child) call this with the same two descriptors, so the
    encoded tuples match. Name-based — which is what makes pseudo-key drift,
    and the keyless implicit PROJ↔LOCA edge, fall out for free."""
    child_key_names = {h.name for h in child.key_headings}
    return [h for h in parent.key_headings if h.name in child_key_names]


def _descendant_shared(
    code: str,
    desc: GroupDescriptor,
    descriptors: dict[str, GroupDescriptor],
    order: list[str],
) -> list[tuple[str, list[Heading]]]:
    """For each present child of ``code``, its shared-key headings (computed
    with ``code`` as the parent)."""
    return [
        (child_code, _shared_headings(desc, descriptors[child_code]))
        for child_code in order
        if descriptors[child_code].parent == code
    ]


def _encode_tuple(cells: dict[str, str], headings: list[Heading]) -> str:
    return _KEY_SEP.join(cells.get(h.name, "") for h in headings)


def _index_for_children(
    lookup: dict[tuple[str, str, str], Any],
    code: str,
    cells: dict[str, str],
    child_shared: list[tuple[str, list[Heading]]],
    inst: Any,
) -> None:
    """Index this row under each descendant's shared-key shape, first-wins
    (matches the converter's ``lookup.entry(...).or_insert``)."""
    for child_code, headings in child_shared:
        lookup.setdefault((code, child_code, _encode_tuple(cells, headings)), inst)


def _resolve_parent(
    lookup: dict[tuple[str, str, str], Any],
    code: str,
    desc: GroupDescriptor,
    descriptors: dict[str, GroupDescriptor],
    cells: dict[str, str],
) -> Any:
    """This child row's parent instance via the shared-keys lookup, or None
    (orphan — kept out of the tree, exactly as read_db skips an unresolved
    parent_id)."""
    parent_code = desc.parent
    if parent_code is None:
        return None
    parent_desc = descriptors.get(parent_code) or GROUPS.get(parent_code)
    if parent_desc is None:
        return None
    shared = _shared_headings(parent_desc, desc)
    return lookup.get((parent_code, code, _encode_tuple(cells, shared)))


def _attach_child(parent: Any, child_code: str, child: Any) -> None:
    """Append ``child`` to the parent's ``<child_code lower>s`` list. For a
    compiled parent the list field already exists; for a passthrough parent we
    create it via setattr — both match read.rs::attach_child_to_parent."""
    field = f"{child_code.lower()}s"
    existing = getattr(parent, field, None)
    if existing is not None:
        existing.append(child)
    else:
        setattr(parent, field, [child])
