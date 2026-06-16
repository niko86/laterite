"""laterite.ags5db — convert AGS4 text ↔ ``.ags5db`` and query the DB.

The Python face of the Rust ``ags5db`` conversion engine
(``rust-packages/laterite-ags5-db/src/convert.rs``), exposed in-process through
``laterite._laterite_native`` — no subprocess, no temp files. Linking
the engine pulls bundled libduckdb into the wheel (the deliberate
"fatter wheel" of the staged-adoption plan); the validator half stays
lean. ``.agsx`` retired in Stage F2a — it is now a Python-only
inspection format produced by ``laterite_ags5x.ags4_to_agsx``.

Each call does the data work in Rust and returns a small stats dict.
Failures raise ``RuntimeError`` whose message carries the same exit
code the ``ags5db`` binary would exit with (e.g. ``7`` for an AGS4
Record Link the exporter can't faithfully round-trip).

    >>> from laterite import ags5db
    >>> ags5db.convert("delivery.ags", "delivery.ags5db")
    {'bytes': ..., 'mode': 'fresh', 'attachments': 0, ...}
    >>> ags5db.export("delivery.ags5db", "round-trip.ags")
    {'groups_emitted': ..., 'rows_emitted': ..., 'warnings': [...]}
"""

from __future__ import annotations

import json
from dataclasses import dataclass
from os import PathLike
from typing import Any, Literal

from . import _laterite_ags5_native as _native

__all__ = [
    "BlobAttachment",
    "DiffReport",
    "GroupDiff",
    "Predicate",
    "ValidationFinding",
    "attach_blobs",
    "convert",
    "count",
    "diff",
    "export",
    "groups",
    "headings",
    "info",
    "inspect",
    "list_blobs",
    "peek",
    "query",
    "read_db",
    "sql",
    "sum",
    "validate",
    "write_db",
]


# --- typed predicate API --------------------------------------------
#
# Stage F2a-2: ``Predicate`` lives here (moved from ``ags5_db.query``).
# The Rust engine still parses ``"FIELD<op>VALUE"`` strings; this class is
# the typed front-end. Constructing a ``Predicate`` validates the op
# (against the allowed set) and the value type; the field is checked
# against the file's registry on the Rust side at query time. Passing a
# Predicate (or list, ANDed) to ``count`` / ``sum`` / ``peek`` /
# ``query`` is identical in effect to passing the string form, just
# safer and IDE-autocompletable.

_OPS: frozenset[str] = frozenset({"<", "<=", "=", ">", ">=", "!="})


@dataclass(frozen=True, slots=True)
class Predicate:
    """A typed where-clause comparison applied to a group's field.

    The ``op`` must be one of ``<``, ``<=``, ``=``, ``!=``, ``>``,
    ``>=``. The value can be int, float, or str (AGS IDs pass through
    as text). The field is validated against the registry by the
    Rust engine when the query runs.
    """
    field: str
    op: Literal["<", "<=", "=", "!=", ">", ">="]
    value: float | int | str

    def __post_init__(self) -> None:
        if self.op not in _OPS:
            raise ValueError(f"disallowed predicate op: {self.op!r}")

    def to_where_string(self) -> str:
        """Render to the ``"FIELD<op>VALUE"`` form the Rust engine parses."""
        return f"{self.field}{self.op}{self.value}"


def _where_to_strings(
    where: Predicate | list[Predicate] | list[str] | None,
) -> list[str]:
    """Normalise the various accepted ``where`` shapes to a ``list[str]``
    for the underlying PyO3 fn. Accepts:
      * ``None`` → empty list (no filter)
      * a single ``Predicate`` → one string
      * a list of ``Predicate`` → many strings (ANDed)
      * a list of raw ``"FIELD<op>VALUE"`` strings → passed through
    """
    if where is None:
        return []
    if isinstance(where, Predicate):
        return [where.to_where_string()]
    out: list[str] = []
    for item in where:
        if isinstance(item, Predicate):
            out.append(item.to_where_string())
        else:
            out.append(str(item))
    return out


# --- conversion -----------------------------------------------------


def convert(
    ags4: str | PathLike[str],
    db: str | PathLike[str],
    *,
    append: bool = False,
    no_compact: bool = False,
    attachments_dir: str | PathLike[str] | None = None,
) -> dict[str, Any]:
    """Convert an AGS4 transfer file to a ``.ags5db`` (DuckDB).

    ``append`` merges into an existing DB instead of overwriting it;
    ``no_compact`` skips the final CTAS rewrite; ``attachments_dir``
    resolves AGS4 Rule 20 FILE references (defaults to the ``.ags``'s
    parent directory). Returns ``{bytes, mode, attachments,
    attachment_bytes, warnings}``.
    """
    return _native.ags5db_convert(
        str(ags4),
        str(db),
        append,
        no_compact,
        None if attachments_dir is None else str(attachments_dir),
    )


def export(
    db: str | PathLike[str],
    ags4: str | PathLike[str],
) -> dict[str, Any]:
    """Export a ``.ags5db`` back to an AGS4 transfer file.

    The data export only — FILE attachment unspooling and post-write
    validation live in the ``ags5db`` CLI, not here. Raises
    ``RuntimeError`` (exit 7) if the DB holds RL-typed (Record Link)
    headings, which can't be faithfully round-tripped. Returns
    ``{groups_emitted, rows_emitted, warnings}``.
    """
    return _native.ags5db_export(str(db), str(ags4))


# `.agsx` ↔ `.ags5db` conversion retired in Stage F2a; `.agsx` is now
# a Python-only inspection helper via `laterite_ags5x.ags4_to_agsx`.


# --- read-side query API --------------------------------------------


def count(
    db: str | PathLike[str],
    group: str,
    where: Predicate | list[Predicate] | list[str] | None = None,
) -> int:
    """``COUNT(*)`` of a group's rows, optionally filtered by ``where``.

    ``where`` accepts a single ``Predicate``, a list of them (ANDed),
    or raw ``"FIELD<op>VALUE"`` strings.
    """
    return _native.ags5db_count(str(db), group, _where_to_strings(where))


def sum(  # noqa: A001 - deliberately shadows builtins.sum in this namespace
    db: str | PathLike[str],
    group: str,
    field: str,
    where: Predicate | list[Predicate] | list[str] | None = None,
) -> float:
    """``SUM(field)`` over a numeric heading, optionally filtered. Raises
    ``RuntimeError`` (exit 5) if ``field`` is unknown or non-numeric;
    an empty sum is ``0.0``.

    ``where`` accepts a single ``Predicate``, a list of them (ANDed),
    or raw ``"FIELD<op>VALUE"`` strings.
    """
    return _native.ags5db_sum(str(db), group, field, _where_to_strings(where))


def sql(
    db: str | PathLike[str],
    statement: str,
    *,
    limit: int = 1000,
    explain: bool = False,
) -> dict[str, Any]:
    """Run a read-only ``SELECT`` and return ``{"columns": [...],
    "records": [{...}, ...]}`` (record key order preserved). ``limit``
    is appended unless the statement already names one (``0`` disables);
    ``explain`` returns the query plan instead."""
    return json.loads(_native.ags5db_sql(str(db), statement, limit, explain))


def peek(
    db: str | PathLike[str],
    group: str,
    *,
    fields: str | None = None,
    where: Predicate | list[Predicate] | list[str] | None = None,
    limit: int = 50,
    offset: int = 0,
    drop_null_cols: bool = False,
) -> dict[str, Any]:
    """Browse a group's view, returning ``{"columns": [...], "records":
    [{...}, ...]}``. ``fields`` is a comma-separated heading list
    (default: all); ``drop_null_cols`` removes columns that are NULL
    across every returned row.

    ``where`` accepts a single ``Predicate``, a list of them (ANDed),
    or raw ``"FIELD<op>VALUE"`` strings.
    """
    return json.loads(
        _native.ags5db_peek(
            str(db),
            group,
            fields,
            _where_to_strings(where),
            limit,
            offset,
            drop_null_cols,
        )
    )


def query(
    db: str | PathLike[str],
    group: str,
    *,
    fields: list[str] | None = None,
    where: Predicate | list[Predicate] | list[str] | None = None,
    limit: int = 1000,
    offset: int = 0,
) -> Any:  # polars.DataFrame at runtime; quoted to keep import lazy
    """Query a group's view and return a polars DataFrame.

    Stage F2a-2: Rust-backed replacement for the retired
    ``ags5_db.query.query_readings``. Wraps the existing ``peek`` Rust
    primitive (which returns dict records) and assembles a polars
    DataFrame on the Python side. ``fields`` defaults to every heading
    on the group.
    """
    import polars as pl

    fields_arg = ",".join(fields) if fields else None
    payload = json.loads(
        _native.ags5db_peek(
            str(db),
            group,
            fields_arg,
            _where_to_strings(where),
            limit,
            offset,
            False,
        )
    )
    return pl.DataFrame(payload["records"], orient="row", schema=payload["columns"])


def list_blobs(
    db: str | PathLike[str],
    *,
    parent_code: str | None = None,
    kind: str | None = None,
) -> list[dict[str, Any]]:
    """List blob rows with optional filters. Returns a list of dicts
    with ``{id, parent_code, parent_id, kind, mime_type, filename,
    sha256, byte_length}`` per row. The ``data`` BLOB column is excluded
    — call ``sql("SELECT data FROM blob WHERE id=...")`` to fetch the
    bytes themselves.

    Stage F2a-2: Rust-backed replacement for ``ags5_db.blobs.list_blobs``.
    """
    payload = json.loads(
        _native.ags5db_list_blobs(str(db), parent_code, kind)
    )
    return payload["records"]


# --- structural + data-correctness validator ------------------------


@dataclass(frozen=True, slots=True)
class ValidationFinding:
    """One issue surfaced by :func:`validate`.

    ``code`` is one of ``"abbr_unknown"`` or ``"dt_invalid"`` today;
    more codes may be added without breaking the contract. ``where`` is
    a free-form location string like ``"g_loca[row 3].LOCA_TYPE"``.
    """
    severity: Literal["error", "warning"]
    code: str
    where: str
    message: str

    def __str__(self) -> str:
        return f"{self.code} @ {self.where}: {self.message}"


# --- file introspection ---------------------------------------------


def info(db: str | PathLike[str]) -> dict[str, Any]:
    """File-level summary: ``{file, size_mb, format_version,
    library_version, n_groups, n_nonempty, groups: [{code, rows,
    parent}, ...]}``. Stage F2a-2e."""
    return json.loads(_native.ags5db_info(str(db)))


def read_db(db: str | PathLike[str]) -> Any:
    """Read a ``.ags5db`` file and return its typed PROJ tree.

    Standard AGS groups (the 92 codegen'd into ``laterite._laterite_native``
    at Rust build time) are returned as their compiled ``#[pyclass]``
    instances. Custom / passthrough groups encountered in the file's
    ``_spec_groups`` are materialised as dynamic Python classes built
    from the file's own ``_spec_headings`` schema (cached process-wide
    by ``(code, sorted-heading-tuple)``); their instances attach to
    their declared parent via ``setattr`` and are reachable via
    ``getattr(parent, child_field, [])``.
    """
    return _native.ags5db_read_db(str(db))


# --- F2c-2: blob attachments ---------------------------------------


@dataclass(frozen=True, slots=True)
class BlobAttachment:
    """A binary attachment to an AGS group row.

    ``target_keys`` must contain every cascaded AGS KEY heading of
    ``target_code`` — for a CORE blob: ``LOCA_ID`` + ``CORE_TOP``;
    for a SAMP blob: ``LOCA_ID`` + the SAMP_* tuple. ``from_model``
    is the easy path when you have an instance — it pulls the
    KEY values off the model itself.

    ``kind`` is a free-form discriminator. Common values: ``"photo"``,
    ``"video"``, ``"pdf"``, ``"other"``.
    """

    target_code: str
    target_keys: dict[str, Any]
    kind: str
    data: bytes
    mime_type: str | None = None
    filename: str | None = None

    @classmethod
    def from_model(
        cls,
        model: Any,
        *,
        kind: str,
        data: bytes,
        mime_type: str | None = None,
        filename: str | None = None,
    ) -> BlobAttachment:
        """Build a BlobAttachment by introspecting `model`'s class name
        (the AGS group code) and pulling KEY heading values directly
        off the model. Works with the compiled `laterite.*`
        `#[pyclass]` types and with `laterite.dynamic.*` classes —
        the only contract is that the AGS code is the class's
        `__name__` and KEY fields are reachable via `getattr`."""
        import mimetypes as _mimetypes

        from laterite.registry import GROUPS

        code = type(model).__name__
        if code not in GROUPS:
            raise ValueError(
                f"model class {code!r} is not a registered AGS group"
            )
        g = GROUPS[code]
        target_keys: dict[str, Any] = {}
        for h in g.headings:
            if h.status != "KEY":
                continue
            target_keys[h.name] = getattr(model, h.py_name, None)
        if mime_type is None and filename:
            mime_type, _ = _mimetypes.guess_type(filename)
        return cls(
            target_code=code,
            target_keys=target_keys,
            kind=kind,
            data=data,
            mime_type=mime_type,
            filename=filename,
        )


def attach_blobs(
    db: str | PathLike[str],
    blobs: list[BlobAttachment],
) -> int:
    """Attach a list of binary blobs to rows in an existing `.ags5db`.

    Each ``BlobAttachment`` carries ``target_keys`` for the row it
    should attach to. This call resolves the target row's UUID by
    querying the file's ``v_<target_code>`` view (which exposes
    inherited KEYs from ancestors) and bulk-inserts into the file's
    `blob` table.

    Returns the number of blobs attached. Raises ``ValueError`` if a
    target row can't be found.

    Stage F2c-2: Rust-backed replacement for the soon-to-retire
    ``ags5_db.write_ags5db(proj, db, blobs=...)`` side-channel.
    """
    if not blobs:
        return 0
    # Convert each dataclass to the dict shape the Rust side expects.
    payload = [
        {
            "target_code": b.target_code,
            "target_keys": dict(b.target_keys),
            "kind": b.kind,
            "data": b.data,
            "mime_type": b.mime_type,
            "filename": b.filename,
        }
        for b in blobs
    ]
    return _native.ags5db_attach_blobs(str(db), payload)


def write_db(
    proj: Any,
    db: str | PathLike[str],
    *,
    append: bool = False,
) -> None:
    """Write a typed PROJ tree to a ``.ags5db`` file.

    Walks the tree, extracts heading values via Python attribute
    access, and reuses the Rust converter's bucket-writer (the same
    code path AGS4 ingest goes through) — UUID7 minting,
    content-hash dedup, parent-id linkage, DDL build, and the
    self-describing ``_spec_*`` tables all match the AGS4 path
    byte-for-byte.

    Args:
        proj: A typed PROJ tree (compiled ``#[pyclass]`` instances
            and/or ``laterite.dynamic.*`` instances).
        db: Destination path. Overwritten unless ``append=True``.
        append: If True, merge into an existing ``.ags5db`` —
            pseudo-key dedup against the file's existing rows keeps
            re-ingest idempotent and pulls in only rows new to this
            write. The destination MUST already exist when
            ``append=True``.
    """
    _native.ags5db_write_db(proj, str(db), append)


def groups(
    db: str | PathLike[str],
    *,
    nonempty: bool = False,
) -> list[dict[str, Any]]:
    """Every registered group in the file with row count, parent and
    contents description. ``nonempty`` filters to groups that actually
    carry rows. Stage F2a-2e."""
    return json.loads(_native.ags5db_groups(str(db), nonempty))


def headings(
    db: str | PathLike[str],
    group: str,
) -> list[dict[str, Any]]:
    """Schema dump for one group: ``[{name, status, ags_type,
    canonical_type, unit, hint}, ...]``. Stage F2a-2e."""
    return json.loads(_native.ags5db_headings(str(db), group))


def inspect(
    db: str | PathLike[str],
    *,
    group: str | None = None,
) -> dict[str, Any]:
    """Dump the file's ``_spec_*`` self-describing tables.

    With ``group=None`` returns scalar meta + counts: ``{format_version,
    library_version, written_at, note, n_groups, n_headings}``.
    With ``group=<CODE>``, also fills in ``group`` (the group's block
    from ``_spec_groups``) and ``headings`` (its rows from
    ``_spec_headings``). Unknown group raises ``RuntimeError`` (exit 4).

    The ``index_parent`` + ``indexed`` keys only appear on files
    written by ≥6.5.2 (compatibility — pre-6.5.2 files omit the
    columns entirely). Stage F2a-2f.
    """
    return json.loads(_native.ags5db_inspect(str(db), group))


# --- cross-file diff ------------------------------------------------


@dataclass(frozen=True, slots=True)
class GroupDiff:
    """Per-group diff state. Sample tuples are lists of raw values
    (str / int / float / None), one per KEY column."""
    code: str
    added: int
    removed: int
    modified: int
    unchanged: int
    sample_added: list[list[Any]]
    sample_removed: list[list[Any]]
    sample_modified: list[list[Any]]


@dataclass(frozen=True, slots=True)
class DiffReport:
    """Whole-file diff between two `.ags5db` files.

    `has_changes` is True if any group has add/remove/modify rows OR
    if any group exists in only one file. Same semantics as the Rust
    binary's exit-1 signal.
    """
    changed_groups: list[GroupDiff]
    groups_only_in_a: list[str]
    groups_only_in_b: list[str]

    @property
    def has_changes(self) -> bool:
        return bool(
            self.changed_groups or self.groups_only_in_a or self.groups_only_in_b
        )


def diff(
    a: str | PathLike[str],
    b: str | PathLike[str],
    *,
    samples: int = 3,
) -> DiffReport:
    """Diff two ``.ags5db`` files: per-group added/removed/modified row
    counts plus sample KEY tuples per change category.

    Identity is the AGS KEY tuple, not the UUID surrogate (UUIDs are
    random per write; the KEY tuple is the only cross-file-stable id).
    ``samples`` caps the per-group sample tuples (0 to suppress).

    Stage F2a-2d: Rust-backed replacement for ``ags5_db.diff.diff_dbs``.
    """
    raw = _native.ags5db_diff(str(a), str(b), samples)
    payload = json.loads(raw)
    changed = [
        GroupDiff(
            code=gd["code"],
            added=gd["added"],
            removed=gd["removed"],
            modified=gd["modified"],
            unchanged=gd["unchanged"],
            sample_added=gd["sample_added"],
            sample_removed=gd["sample_removed"],
            sample_modified=gd["sample_modified"],
        )
        for gd in payload["changed_groups"]
    ]
    return DiffReport(
        changed_groups=changed,
        groups_only_in_a=payload["groups_only_in_a"],
        groups_only_in_b=payload["groups_only_in_b"],
    )


def validate(
    db: str | PathLike[str],
    *,
    check_abbr: bool = True,
    check_dt: bool = True,
) -> list[ValidationFinding]:
    """Validate a ``.ags5db`` file's spec-correctness.

    * ``check_abbr``: every PA-typed value must appear in the file's
      own ABBR group. The file's ABBR rows declare what abbreviations
      THIS delivery uses; values absent from ABBR signal a typo or
      out-of-spec entry.
    * ``check_dt``: every non-empty DT-typed value must parse via the
      same coercion the AGS4 ingest uses, so the validator never
      disagrees with the ingest path.

    Stage F2a-2b: Rust-backed replacement for the retired
    ``laterite_ags5x.validation.validate_ags5db``.
    """
    raw = _native.ags5db_validate(str(db), check_abbr, check_dt)
    findings_data = json.loads(raw)
    return [
        ValidationFinding(
            severity=f["severity"],
            code=f["code"],
            where=f["where"],
            message=f["message"],
        )
        for f in findings_data
    ]
