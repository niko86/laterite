"""laterite.ags_types — Rust-backed AGS4 type system.

Exposes `parse_value` / `canonical_type` / `display_hint` semantics
backed by the Rust engine in `rust-packages/laterite-ags4-core/src/ags_types.rs`.

The `CanonicalType` StrEnum lives Python-side so existing
`canonical_type(x) is CanonicalType.DECIMAL` checks keep their
identity. Rust returns the lowercase label; we coerce back to the
enum.

`parse_value` returns native Python types — int / float / bool /
datetime.datetime / datetime.date / datetime.time / str / None —
matching the pure-Python implementation. Returns the trimmed string
for unknown AGS codes (passthrough), matching the prior behaviour.
"""

from __future__ import annotations

from enum import StrEnum
from typing import Any

from . import _laterite_native as _native


class CanonicalType(StrEnum):
    """Cross-system target types. Values match the lowercase labels the
    Rust side returns from `canonical_type` so the enum doubles as the
    string interchange representation (e.g. for ``_spec_headings``)."""

    STRING = "string"
    INTEGER = "integer"
    DECIMAL = "decimal"
    DATETIME = "datetime"
    DATE = "date"
    TIME = "time"
    BOOL = "bool"
    ENUM = "enum"


def canonical_type(ags_type: str) -> CanonicalType:
    """AGS spec type code → canonical category.

    Raises ``ValueError`` for unknown codes — the AGS4 codec coerces
    passthrough groups' heading types to ``'X'`` before calling, so
    in practice this never raises on dictionary-resident headings.
    """
    label = _native.canonical_type(ags_type)
    if label is None:
        raise ValueError(f"unknown AGS type code: {ags_type!r}")
    return CanonicalType(label)


def display_hint(ags_type: str) -> str | None:
    """Presentation hint for a numeric AGS type, or ``None``.

    ``'2DP'`` → ``'%.2f'``, ``'3SF'`` → ``'%.3g'``, ``'1SCI'`` →
    ``'%.1e'``. String / datetime / bool types return ``None``.
    """
    return _native.display_hint(ags_type)


def parse_value(raw: Any, ags_type: str) -> Any:
    """Parse an AGS4-shaped raw string into the canonical Python type.

    Permissive: unparseable values return ``None``. The single source
    of truth for AGS4 + ``.agsx`` ingest (both Rust paths); the Python
    ``laterite_ags5x._codec`` module is encode-only since Stage E3.
    """
    # Accept any input PyO3 won't coerce directly (e.g. an int passed
    # by a careless caller) — match the Python implementation's
    # "isinstance(s, str)" guards by stringifying first.
    if raw is None:
        return None
    if not isinstance(raw, str):
        raw = str(raw)
    return _native.parse_value(raw, ags_type)


__all__ = [
    "CanonicalType",
    "canonical_type",
    "display_hint",
    "parse_value",
]
