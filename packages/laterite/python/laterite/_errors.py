"""Exception hierarchy + mapping from the native layer's error dicts.

The Rust module never raises for un-validatable input — it returns
``{"ok": False, "error_kind", "error", "exit_code"}``. The nice API
and ``compat`` translate that into Python exceptions here; the CLI
reads ``exit_code`` directly. ``exit_code`` mirrors the Rust
``lat`` binary (3 not-found/io, 4 not-utf8/not-ags4/
unsupported-edition, 5 bad-dict/bad-args).
"""

from __future__ import annotations


class Ags4Error(Exception):
    """Base for every laterite error. Carries the validator exit code."""

    exit_code: int = 1

    def __init__(self, message: str, *, exit_code: int | None = None) -> None:
        super().__init__(message)
        if exit_code is not None:
            self.exit_code = exit_code


class NotAgs4Error(Ags4Error):
    """Input has no GROUP rows — not a parseable AGS4 file."""

    exit_code = 4


class UnsupportedEditionError(Ags4Error):
    """A recognised but unsupported edition (AGS3). Clean-room refuses
    rather than silently validating it against an AGS4 schema (O-30)."""

    exit_code = 4


class BadDictError(Ags4Error):
    """Bad ``--dict-version`` / unimplemented external ``--dict`` (O-28)."""

    exit_code = 5


class StaleCertError(Ags4Error):
    """A passed ``index=`` certificate (``.ags.idx``) does not match the file it
    was read for — its size/SHA-256 differ, so its byte offsets and clean verdict
    are now lies. Raised by **both** doors that take ``index=`` — [`read`][laterite.read]
    and [`validate`][laterite.validate] — because naming one asserts "this cert is for
    this file", and a false assertion is an error rather than a silent fall-back to
    re-validation. Rebuild it (``read(p).validate().certify()``).

    NOT raised for a cert that is genuinely for these bytes but cannot answer *this*
    question — a different rule engine, a severity tier it never measured,
    ``check_files=True``. That is not a caller error: the rules run and
    [`Report.revalidate_reason`][laterite.Report.revalidate_reason] says why the cert
    did not stand in. A handle carrying a cert from ``read(index=)`` is in the same
    position: nothing was asserted at ``.validate()``, so nothing is raised there.
    """

    exit_code = 4


class WorldCheckRequiresSourceError(Ags4Error):
    """``check_files=True`` was asked of an input with no path — [`read`][laterite.read]
    of ``bytes`` or ``str``, where there is no directory for the sibling ``FILE/`` tree
    to be in.

    Rule 20's on-disk half is the one check that reads state the AGS4 bytes do not
    contain, so it is the one check that cannot be answered from content alone. The
    engine used to answer it anyway — by dropping the request and reporting Rule 20
    clean. Passing a path (``read("delivery.ags")``) makes the question answerable;
    dropping ``check_files`` makes it unasked. Both are honest; a silent clean was not.
    """

    exit_code = 5


class MergeConflictError(Ags4Error):
    """A [`merge`][laterite.merge] could not be reconciled. Either two files typed the
    same heading differently and ``on_type_clash="error"`` (the default) refused to
    guess — pass ``"promote"`` to keep the greatest ``nDP`` precision, or ``"widen"``
    to fall back to ``X`` text; or two files declared conflicting **UNITs**, which is
    fatal in every mode (no mode can absorb it — see below); or the merged output
    failed the emitter's own re-validation."""

    exit_code = 6


# error_kind (from the Rust layer) → exception. "not_found" maps to the
# builtin FileNotFoundError so callers can `except FileNotFoundError`.
_KIND_TO_EXC: dict[str, type[Ags4Error]] = {
    "not_ags4": NotAgs4Error,
    "not_utf8": NotAgs4Error,
    "unsupported_edition": UnsupportedEditionError,
    "bad_dict": BadDictError,
    "bad_args": BadDictError,
    "world_check_requires_source": WorldCheckRequiresSourceError,
    # Raised BEFORE the engine runs — the point of a named cert is to skip that
    # work, so a mismatch found afterwards would cost what the caller was saving.
    "stale_cert": StaleCertError,
    "type_conflict": MergeConflictError,
    # Fatal in EVERY merge mode — no `on_type_clash` value absorbs a unit clash (laterite-dev#501).
    "unit_conflict": MergeConflictError,
    "emit_error": MergeConflictError,
}


def raise_for(result: dict) -> dict:
    """Pass a successful native result through; raise the mapped
    exception for a failure dict."""
    if result.get("ok"):
        return result
    kind = result.get("error_kind", "")
    msg = result.get("error", "unknown error")
    code = int(result.get("exit_code", 1))
    if kind in ("not_found", "io"):
        raise FileNotFoundError(msg)
    exc = _KIND_TO_EXC.get(kind, Ags4Error)
    raise exc(msg, exit_code=code)
