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

from os import PathLike
from pathlib import Path

from . import _laterite_native as _native

__all__ = [
    "lock",
    "lock_bytes",
    "pack",
    "pack_bytes",
    "unlock",
    "unlock_bytes",
    "unpack",
    "unpack_bytes",
]


def _src_path(src, *, fn: str) -> Path:
    """Normalise the first argument to a ``Path``, rejecting non-paths early — a
    non-path arg (e.g. an ``Ags4File``) fails *here* with an actionable message
    rather than deep in Rust."""
    if not isinstance(src, (str, PathLike)):
        raise TypeError(
            f"{fn}() expects a file path (str or os.PathLike), not "
            f"{type(src).__name__!r}. transport works on any file by PATH — if you "
            f"have an Ags4File, call .save(path) first, then {fn}(path)."
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
    src: str | PathLike[str],
    *,
    level: int = 9,
    dest: str | PathLike[str] | None = None,
) -> Path:
    """Compress any file to ``<src>.zst`` for transport (zstd only).

    Works on **any file** — ``.ags``, ``.ags5db``, anything. ``level`` is the zstd
    level (1=fastest, 22=highest ratio); default 9 is the sweet spot on AGS data.
    ``dest`` overrides the default ``<src>.zst`` output path.

    Args:
        src: Path to the file to compress (``str`` or ``os.PathLike``). An
            ``Ags4File`` is rejected early — call ``.save(path)`` first.
        level: zstd compression level, 1 (fastest) to 22 (highest ratio).
            Defaults to 9.
        dest: Output path. Defaults to ``<src>.zst`` (original suffix preserved,
            ``.zst`` appended).

    Returns:
        The ``Path`` written.

    Raises:
        TypeError: If ``src`` is not a path-like (e.g. an ``Ags4File``).
        RuntimeError: If the underlying zstd operation fails.
    """
    src_path = _src_path(src, fn="pack")
    out = Path(dest) if dest is not None else _default_pack_out(src_path)
    _native.transport_pack(str(src_path), str(out), level)
    return out


def unpack(
    src: str | PathLike[str],
    *,
    dest: str | PathLike[str] | None = None,
) -> Path:
    """Decompress a ``.zst`` produced by ``pack`` back to the original file.

    By default the output strips the ``.zst`` suffix; pass ``dest`` to override.

    Args:
        src: Path to the ``.zst`` file to decompress.
        dest: Output path. Defaults to ``src`` with its ``.zst`` suffix stripped
            (or ``<src>.unpacked`` if it has no ``.zst`` suffix).

    Returns:
        The ``Path`` written.

    Raises:
        TypeError: If ``src`` is not a path-like.
        RuntimeError: If the input is not valid zstd or the operation fails.
    """
    src_path = _src_path(src, fn="unpack")
    out = Path(dest) if dest is not None else _default_unpack_out(src_path)
    _native.transport_unpack(str(src_path), str(out))
    return out


def lock(
    src: str | PathLike[str],
    *,
    password: str,
    level: int = 9,
    log_n: int | None = None,
    dest: str | PathLike[str] | None = None,
) -> Path:
    """Compress + age-passphrase-encrypt any file to ``<src>.zst.age``.

    Zstd first (low-entropy data compresses well), then age (scrypt +
    ChaCha20-Poly1305). The envelope is interoperable with ``lat-db lock`` and
    ``pyrage`` — both use the same Rust ``age`` crate. ``password`` is required (no
    agent-default path). ``level`` is the zstd level (default 9); ``dest`` overrides
    the default ``<src>.zst.age`` output path.

    Args:
        src: Path to the file to compress and encrypt.
        password: The age passphrase. Required — there is no agent-key path.
        level: zstd compression level, 1 (fastest) to 22 (highest ratio).
            Defaults to 9.
        log_n: scrypt work factor (``log2(N)``) for the age passphrase KDF.
            ``None`` uses the pinned default (18 — age's standard tier, openable
            everywhere). A lower value is faster but weaker; must be ``1..=20``
            (``>20`` produces a file the browser age decoder refuses).
        dest: Output path. Defaults to ``<src>.zst.age`` (flagging both the
            compression and encryption layers).

    Returns:
        The ``Path`` written.

    Raises:
        TypeError: If ``src`` is not a path-like.
        RuntimeError: If the underlying zstd or age operation fails, or ``log_n``
            is outside ``1..=20``.
    """
    src_path = _src_path(src, fn="lock")
    out = Path(dest) if dest is not None else _default_lock_out(src_path)
    _native.transport_lock(str(src_path), str(out), password, level, log_n)
    return out


def unlock(
    src: str | PathLike[str],
    *,
    password: str,
    dest: str | PathLike[str] | None = None,
) -> Path:
    """Decrypt + decompress a ``.zst.age`` produced by ``lock`` back to the original.

    Wrong passphrase or non-passphrase envelopes raise ``RuntimeError``. By default
    the output strips ``.age`` then ``.zst``.

    Args:
        src: Path to the ``.zst.age`` file to decrypt and decompress.
        password: The age passphrase used by ``lock``.
        dest: Output path. Defaults to ``src`` with ``.age`` then ``.zst``
            stripped (or ``<src>.unlocked`` if neither suffix is present).

    Returns:
        The ``Path`` written.

    Raises:
        TypeError: If ``src`` is not a path-like.
        RuntimeError: If the passphrase is wrong, the envelope is not a
            passphrase envelope, or decompression fails.
    """
    src_path = _src_path(src, fn="unlock")
    out = Path(dest) if dest is not None else _default_unlock_out(src_path)
    _native.transport_unlock(str(src_path), str(out), password)
    return out


# --- in-memory (bytes) forms -------------------------------------------------
# The filesystem-free counterparts of pack/unpack/lock/unlock, for packaging a
# value you already hold in memory (e.g. `read(...).fix(...).bytes`) straight to
# an upload — crucially, `lock_bytes` never writes the plaintext to disk. Each
# produces the same envelope as its file form, so a `*_bytes` blob interops with
# the file API (write it out, then unpack/unlock) and vice versa.


def pack_bytes(data: bytes, *, level: int = 9) -> bytes:
    """Compress bytes → bytes in memory (zstd only) — no filesystem.

    The in-memory form of :func:`pack`. Output is a standard zstd frame, so it
    opens with :func:`unpack_bytes`, :func:`unpack` (write it to a file first),
    or stock ``zstd``.

    Args:
        data: The bytes to compress.
        level: zstd level, 1 (fastest) to 22 (highest ratio). Defaults to 9.

    Returns:
        The compressed bytes.

    Raises:
        RuntimeError: If the underlying zstd operation fails.
    """
    return _native.transport_pack_bytes(data, level)


def unpack_bytes(data: bytes) -> bytes:
    """Decompress zstd bytes → bytes in memory — the in-memory form of :func:`unpack`.

    Args:
        data: The zstd-compressed bytes (e.g. from :func:`pack_bytes`).

    Returns:
        The decompressed bytes.

    Raises:
        RuntimeError: If the input is not valid zstd or the operation fails.
    """
    return _native.transport_unpack_bytes(data)


def lock_bytes(data: bytes, *, password: str, level: int = 9, log_n: int | None = None) -> bytes:
    """Compress + age-passphrase-encrypt bytes → bytes in memory — no plaintext on disk.

    The in-memory form of :func:`lock` (zstd, then age scrypt +
    ChaCha20-Poly1305). Ideal for sealing sensitive data you hold in memory —
    e.g. a fixed ``Ags4File``'s ``.bytes`` — without ever writing the plaintext
    out. The ``.zst.age`` envelope matches :func:`lock`'s, so the result opens
    with :func:`unlock_bytes`, :func:`unlock` (write it out first), ``pyrage``,
    or the browser, given the passphrase.

    Args:
        data: The bytes to compress and encrypt.
        password: The age passphrase. Required — there is no agent-key path.
        level: zstd level, 1 (fastest) to 22 (highest ratio). Defaults to 9.
        log_n: scrypt work factor (``log2(N)``). ``None`` uses the pinned default
            (18); a lower value is faster but weaker; must be ``1..=20``.

    Returns:
        The sealed bytes.

    Raises:
        RuntimeError: If the underlying zstd or age operation fails, or ``log_n``
            is outside ``1..=20``.
    """
    return _native.transport_lock_bytes(data, password, level, log_n)


def unlock_bytes(data: bytes, *, password: str) -> bytes:
    """Decrypt + decompress ``.zst.age`` bytes → bytes — the in-memory form of :func:`unlock`.

    Args:
        data: The sealed bytes (e.g. from :func:`lock_bytes`).
        password: The age passphrase used to seal them.

    Returns:
        The original bytes.

    Raises:
        RuntimeError: If the passphrase is wrong, the envelope is not a
            passphrase envelope, or decompression fails.
    """
    return _native.transport_unlock_bytes(data, password)
