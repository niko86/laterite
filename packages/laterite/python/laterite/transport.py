"""laterite.transport — zstd compression + age passphrase encryption for any file.

The four operations are **content-agnostic**: they read a file's bytes,
(de)compress and optionally (de)encrypt them, and write the result. They work on
**any file** — an AGS4 ``.ags`` transfer file, an ``.ags5db``, anything — not only
``.ags5db`` (the historical framing; the Rust core just runs zstd/age over raw
bytes).

Two pairs of operations:

* ``pack`` / ``unpack`` — zstd only. The default level 9 is the empirical sweet
  spot on AGS data (~10% ratio in a few seconds; higher levels buy minutes not
  bytes).
* ``lock`` / ``unlock`` — zstd + age passphrase encryption. The age envelope is
  interoperable with ``pyrage`` and the ``lat-db lock`` binary subcommand — both
  link the same Rust ``age`` crate.

    >>> from laterite import transport
    >>> transport.pack("delivery.ags")
    PosixPath('delivery.ags.zst')
    >>> transport.lock("delivery.ags5db", password="hunter2")
    PosixPath('delivery.ags5db.zst.age')
"""

from __future__ import annotations

import warnings
from os import PathLike
from pathlib import Path

from . import _laterite_native as _native

__all__ = ["lock", "pack", "unlock", "unpack"]


def _src_path(src, *, fn: str, legacy=None) -> Path:
    """Normalise the first argument to a ``Path``, rejecting non-paths early.

    ``legacy`` is the ``(old_name, value)`` of the keyword each op used to expose
    (``db`` / ``zst`` / ``file``) before they were unified to ``src``; passing it
    warns and is honoured for one deprecation cycle. A non-path arg (e.g. an
    ``Ags4File``) fails *here* with an actionable message rather than deep in Rust.
    """
    if legacy is not None:
        old_name, old_val = legacy
        if old_val is not None:
            if src is not None:
                raise TypeError(
                    f"{fn}() received both 'src' and the deprecated '{old_name}'; "
                    "pass only 'src'."
                )
            warnings.warn(
                f"{fn}()'s '{old_name}=' keyword is renamed 'src='; "
                f"'{old_name}=' will be removed in a future release.",
                DeprecationWarning,
                stacklevel=3,
            )
            src = old_val
    if src is None:
        raise TypeError(f"{fn}() missing required argument: 'src' (a file path).")
    if not isinstance(src, (str, PathLike)):
        raise TypeError(
            f"{fn}() expects a file path (str or os.PathLike), not "
            f"{type(src).__name__!r}. transport works on any file by PATH — if you "
            f"have an Ags4File, call .write(path) first, then {fn}(path)."
        )
    return Path(src)


def _default_pack_out(src: Path) -> Path:
    """`<src>.zst` — preserves the original suffix and appends `.zst`."""
    return src.with_suffix(src.suffix + ".zst")


def _default_unpack_out(src: Path) -> Path:
    """Strip `.zst` if present, else append `.unpacked`."""
    if src.suffix == ".zst":
        return src.with_suffix("")
    return src.with_suffix(src.suffix + ".unpacked")


def _default_lock_out(src: Path) -> Path:
    """`<src>.zst.age` — flags both the compression + encryption layers."""
    return src.with_suffix(src.suffix + ".zst.age")


def _default_unlock_out(src: Path) -> Path:
    """Strip `.age`, then `.zst`. Fall back to `.unlocked` if neither."""
    if src.suffix == ".age":
        stripped = src.with_suffix("")
        if stripped.suffix == ".zst":
            return stripped.with_suffix("")
        return stripped
    return src.with_suffix(src.suffix + ".unlocked")


def pack(
    src: str | PathLike[str] | None = None,
    *,
    level: int = 9,
    dest: str | PathLike[str] | None = None,
    db: str | PathLike[str] | None = None,
) -> Path:
    """Compress any file to ``<src>.zst`` for transport (zstd only).

    Works on **any file** — ``.ags``, ``.ags5db``, anything. ``level`` is the zstd
    level (1=fastest, 22=highest ratio); default 9 is the sweet spot on AGS data.
    ``dest`` overrides the default ``<src>.zst`` output path. (``db=`` is the
    deprecated former name of ``src``.)
    """
    src_path = _src_path(src, fn="pack", legacy=("db", db))
    out = Path(dest) if dest is not None else _default_pack_out(src_path)
    _native.transport_pack(str(src_path), str(out), level)
    return out


def unpack(
    src: str | PathLike[str] | None = None,
    *,
    dest: str | PathLike[str] | None = None,
    zst: str | PathLike[str] | None = None,
) -> Path:
    """Decompress a ``.zst`` produced by ``pack`` back to the original file.

    By default the output strips the ``.zst`` suffix; pass ``dest`` to override.
    (``zst=`` is the deprecated former name of ``src``.)
    """
    src_path = _src_path(src, fn="unpack", legacy=("zst", zst))
    out = Path(dest) if dest is not None else _default_unpack_out(src_path)
    _native.transport_unpack(str(src_path), str(out))
    return out


def lock(
    src: str | PathLike[str] | None = None,
    *,
    password: str,
    level: int = 9,
    dest: str | PathLike[str] | None = None,
    db: str | PathLike[str] | None = None,
) -> Path:
    """Compress + age-passphrase-encrypt any file to ``<src>.zst.age``.

    Zstd first (low-entropy data compresses well), then age (scrypt +
    ChaCha20-Poly1305). The envelope is interoperable with ``lat-db.exe lock`` and
    ``pyrage`` — both use the same Rust ``age`` crate. ``password`` is required (no
    agent-default path). ``level`` is the zstd level (default 9); ``dest`` overrides
    the default ``<src>.zst.age`` output path. (``db=`` is the deprecated former
    name of ``src``.)
    """
    src_path = _src_path(src, fn="lock", legacy=("db", db))
    out = Path(dest) if dest is not None else _default_lock_out(src_path)
    _native.transport_lock(str(src_path), str(out), password, level)
    return out


def unlock(
    src: str | PathLike[str] | None = None,
    *,
    password: str,
    dest: str | PathLike[str] | None = None,
    file: str | PathLike[str] | None = None,
) -> Path:
    """Decrypt + decompress a ``.zst.age`` produced by ``lock`` back to the original.

    Wrong passphrase or non-passphrase envelopes raise ``RuntimeError``. By default
    the output strips ``.age`` then ``.zst``. (``file=`` is the deprecated former
    name of ``src``.)
    """
    src_path = _src_path(src, fn="unlock", legacy=("file", file))
    out = Path(dest) if dest is not None else _default_unlock_out(src_path)
    _native.transport_unlock(str(src_path), str(out), password)
    return out
