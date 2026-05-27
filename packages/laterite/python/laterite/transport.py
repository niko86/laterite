"""laterite.transport — zstd + age passphrase encryption for ``.ags5db``.

Stage F2a-2c: Rust-backed replacement for the retired
``ags5_db.transport`` Python module. All four operations go through
the same `ags5db::transport` lib API the ``ags5db`` Rust binary
itself uses (since F2a-2c made `commands/{pack,unpack,lock,unlock}.rs`
thin shims).

Two pairs of operations:

* ``pack`` / ``unpack`` — zstd only. The default level 9 is the
  empirical sweet spot on AGS data (~10% ratio in a few seconds;
  higher levels buy minutes not bytes).
* ``lock`` / ``unlock`` — zstd + age passphrase encryption. The age
  envelope is interoperable with ``pyrage`` and the ``ags5db lock``
  binary subcommand — both link the same Rust ``age`` crate.

    >>> from laterite import transport
    >>> transport.pack("delivery.ags5db")
    PosixPath('delivery.ags5db.zst')
    >>> transport.lock("delivery.ags5db", password="hunter2")
    PosixPath('delivery.ags5db.zst.age')
"""

from __future__ import annotations

from os import PathLike
from pathlib import Path

from . import _laterite_native as _native

__all__ = ["lock", "pack", "unlock", "unpack"]


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
    db: str | PathLike[str],
    *,
    level: int = 9,
    dest: str | PathLike[str] | None = None,
) -> Path:
    """Compress a ``.ags5db`` to ``.ags5db.zst`` for transport.

    ``level`` is the zstd level (1=fastest, 22=highest ratio); default 9
    is the empirical sweet spot on AGS data. ``dest`` overrides the
    default ``<db>.zst`` output path.
    """
    db_path = Path(db)
    out = Path(dest) if dest is not None else _default_pack_out(db_path)
    _native.ags5db_pack(str(db_path), str(out), level)
    return out


def unpack(
    zst: str | PathLike[str],
    *,
    dest: str | PathLike[str] | None = None,
) -> Path:
    """Decompress a ``.ags5db.zst`` back to a working ``.ags5db``.

    By default the output strips the ``.zst`` suffix; pass ``dest`` to
    override.
    """
    zst_path = Path(zst)
    out = Path(dest) if dest is not None else _default_unpack_out(zst_path)
    _native.ags5db_unpack(str(zst_path), str(out))
    return out


def lock(
    db: str | PathLike[str],
    *,
    password: str,
    level: int = 9,
    dest: str | PathLike[str] | None = None,
) -> Path:
    """Compress + age-passphrase-encrypt a ``.ags5db`` to
    ``.ags5db.zst.age``.

    Zstd first (low-entropy data compresses well), then age
    (scrypt + ChaCha20-Poly1305). The envelope is interoperable with
    ``ags5db.exe lock`` and ``pyrage`` — both use the same Rust
    ``age`` crate.

    ``password`` is required (no agent-default path). ``level`` is the
    zstd compression level (default 9). ``dest`` overrides the default
    ``<db>.zst.age`` output path.
    """
    db_path = Path(db)
    out = Path(dest) if dest is not None else _default_lock_out(db_path)
    _native.ags5db_lock(str(db_path), str(out), password, level)
    return out


def unlock(
    file: str | PathLike[str],
    *,
    password: str,
    dest: str | PathLike[str] | None = None,
) -> Path:
    """Decrypt + decompress a ``.ags5db.zst.age`` back to a working
    ``.ags5db``.

    Wrong passphrase or non-passphrase envelopes raise ``RuntimeError``.
    By default the output strips ``.age`` then ``.zst``.
    """
    src = Path(file)
    out = Path(dest) if dest is not None else _default_unlock_out(src)
    _native.ags5db_unlock(str(src), str(out), password)
    return out
