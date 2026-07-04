"""AGS4 → typed PROJ tree, in one call — on the DuckDB-free base path.

The base AGS4 typed reader: hand it an AGS4 transfer file, get the
typed-graph form directly.

**Why this is hand-rolled here** (W2, 2026-06-16). ``read_typed`` is a
*base* AGS4 API — it must work on a plain ``pip install laterite``. It used
to route ``convert(ags4 → temp .ags5db)`` then ``read_db(tmp)``, dragging a
DuckDB-bundled companion wheel into what is a pure-AGS4 read. This builds the
PROJ tree from the base parser (``parse_primitives``), the registry, and
``parse_value`` — no DuckDB. (Since #177 the experimental ``.ags5db`` engine
is fully decoupled to the dormant ``ags5/`` holding folder, so the base
genuinely cannot reach it.)

**Parity with the converter.** The linkage below is a faithful port of the
``.ags5db`` converter's parent resolution (``insert_group_rows`` /
``resolve_parent_uuid`` in the decoupled ``ags5/`` ``convert.rs``):
topological group order, content-key row dedup, and the *shared-keys
intersection* — for each child group, the parent's KEY headings whose names
the child also KEYs on (in parent-KEY order), joined into a tuple and matched
between parent and child rows. (Pseudo-key drift like MOND_REF↔PIPE_REF needs
no special case: drifted names simply fall out of the name-intersection, so
linkage rides the keys both layers share. The implicit PROJ↔LOCA edge works
the same way — PROJ_ID isn't a LOCA heading, so the intersection is empty and
every LOCA attaches to the one PROJ.) The byte-equality parity test against
the DuckDB path moved with the AGS5 engine to ``ags5/`` (dormant); the
algorithm here remains the reference for the base typed read.

Custom / passthrough groups (present in the file but not in the standard
dictionary) flow through the same ``laterite.dynamic`` factory the converter
uses: parent defaults to LOCA, every heading is OTHER-status with the file's
declared AGS type — matching ``convert.rs::build_passthrough_descriptors``.
"""

from __future__ import annotations

from os import PathLike
from typing import Any

from laterite import _laterite_native as _native
from laterite import _resolve_source, dynamic, raise_for
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
    source: Any = None,
    *,
    path: str | PathLike[str] | None = None,
    text: str | None = None,
    data: bytes | bytearray | memoryview | None = None,
    encoding: str | None = None,
) -> Any:
    """Read an AGS4 transfer — from a path, a file-like, raw bytes, or in-memory
    text — and return its typed PROJ tree.

    The base AGS4 typed read: hand it AGS4 and get the whole transfer back as a
    single live object graph, rooted at its PROJ row, with every group already cast
    to its AGS data type. This is the *base* door — it stands the PROJ tree up from
    the pure-string parser, the dictionary registry, and ``parse_value`` alone, so it
    works on a plain ``pip install laterite`` with no DuckDB on the path.

    Input is resolved exactly as [`read`][laterite.read]: one positional ``source``
    is auto-detected (path / file-like / bytes / AGS4 text), and the keyword-only
    ``path=`` / ``text=`` / ``data=`` doors name an ambiguous input to skip the
    sniff. ``encoding`` (a WHATWG label, default UTF-8) governs how bytes / path
    input is decoded — ``text=`` is already a string and is untouched.

    The shape it hands back mirrors the standard's parent/child model rather than the
    file's flat ``GROUP`` / ``HEADING`` / ``DATA`` blocks: the one PROJ instance is the
    root, and each group hangs off its parent under a ``<child code lower>s`` list
    (``root.locas``, a LOCA's ``samps``, and so on). Edges are recovered from the
    groups' shared KEY headings; identical raw rows collapse to one instance (first
    occurrence wins), and a non-PROJ row whose parent can't be resolved is built but
    left out of the tree as an orphan — the same way the converter drops a row whose
    parent reference points at nothing.

    Groups present in the file but absent from the standard dictionary flow through
    the `laterite.dynamic` factory: they become runtime classes parented to
    LOCA, every heading OTHER-status carrying the file's declared AGS type (an empty
    type padded to ``X``).

    Args:
        source: The AGS4 to read, auto-detected as a path, a file-like, raw bytes,
            or in-memory AGS4 text. Leave as ``None`` and use one of the keyword
            doors below to be explicit.
        path: Explicit on-disk path to an AGS4 file (keyword-only).
        text: Explicit already-decoded AGS4 text (keyword-only); not subject to
            ``encoding``.
        data: Explicit raw AGS4 bytes (keyword-only).
        encoding: WHATWG encoding label for bytes / path input (keyword-only);
            defaults to UTF-8. Ignored for ``text=``.

    Returns:
        The root PROJ instance, with descendant groups attached under their
        ``<child code lower>s`` lists. Standard groups are the compiled
        ``#[pyclass]`` types; custom / passthrough groups are dynamic classes minted
        via `laterite.dynamic.get_or_register`.

    Raises:
        FileNotFoundError: if a resolved path does not exist.
        NotAgs4Error: if the input is not valid AGS4.
        RuntimeError: if the file parses but contains no PROJ row (every AGS4
            transfer must have exactly one).
    """
    p, txt, raw = _resolve_source(source, path=path, text=text, data=data)
    parsed = raise_for(
        _native.parse_primitives(path=p, text=txt, data=raw, encoding=encoding)
    )

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
