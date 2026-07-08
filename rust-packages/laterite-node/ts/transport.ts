// laterite.transport — zstd + age file-envelope helpers, the Node port of
// laterite-py's `transport`. General-purpose: compress/encrypt ANY file. The age
// envelope is interoperable with the Python side (pyrage). The work is native;
// this is the typed TS face.
import {
  type PackStats,
  type UnpackStats,
  transportLock,
  transportLockBytes,
  transportPack,
  transportPackBytes,
  transportUnlock,
  transportUnlockBytes,
  transportUnpack,
  transportUnpackBytes,
} from "./native";

export type { PackStats, UnpackStats };

/**
 * zstd-compress `src` → `dest` for transport — content-agnostic, so it works on
 * any file (an `.ags` transfer, an `.ags5db`, anything), not just AGS data.
 *
 * @param src - Path to the file to compress.
 * @param dest - Path the compressed output is written to.
 * @param level - zstd level 1–22 (default 9, the empirical sweet spot on AGS
 * data — higher levels buy minutes, not bytes).
 * @returns The output size, compression ratio vs source, and elapsed seconds.
 */
export function pack(src: string, dest: string, level?: number): PackStats {
  return transportPack(src, dest, level);
}

/**
 * zstd-decompress a `.zst` produced by {@link pack} back to its original bytes.
 *
 * @param src - Path to the compressed `.zst` file.
 * @param dest - Path the decompressed output is written to.
 * @returns The output size and elapsed seconds.
 */
export function unpack(src: string, dest: string): UnpackStats {
  return transportUnpack(src, dest);
}

/**
 * zstd-compress, then age-passphrase-encrypt `src` → `dest`. Compress-then-
 * encrypt is load-bearing: zstd needs low-entropy input, and ciphertext is
 * random — so the order can't flip. The age envelope is interoperable with the
 * Python side (pyrage) and `lat-db lock`, all linking the same Rust `age` crate.
 *
 * @param src - Path to the file to compress and encrypt.
 * @param dest - Path the encrypted output is written to.
 * @param password - Passphrase for the age envelope (scrypt + ChaCha20-Poly1305).
 * @param level - zstd level 1–22 (default 9).
 * @param logN - scrypt work factor (`log2(N)`); omit for the pinned default (18,
 *   openable everywhere). Lower is faster but weaker; must be `1..=20`.
 * @returns The output size, compression ratio vs source, and elapsed seconds.
 */
export function lock(
  src: string,
  dest: string,
  password: string,
  level?: number,
  logN?: number,
): PackStats {
  return transportLock(src, dest, password, level, logN);
}

/**
 * age-passphrase-decrypt, then zstd-decompress a `.zst.age` produced by
 * {@link lock} back to its original bytes.
 *
 * @param src - Path to the encrypted `.zst.age` file.
 * @param dest - Path the recovered output is written to.
 * @param password - Passphrase the envelope was sealed with.
 * @returns The output size and elapsed seconds.
 * @throws If the passphrase is wrong or the input is not a passphrase envelope.
 */
export function unlock(src: string, dest: string, password: string): UnpackStats {
  return transportUnlock(src, dest, password);
}

// --- in-memory (bytes) forms -------------------------------------------------
// The filesystem-free counterparts of pack/unpack/lock/unlock — the Node mirror
// of laterite-py's `pack_bytes`/…. They package a value you already hold in
// memory (e.g. `read(...).fix().bytes`) straight to an upload; crucially
// `lockBytes` never writes the plaintext to disk. Each produces the SAME
// envelope as its file form, so a `*Bytes` blob interops with the file API
// (write it out, then unpack/unlock) and with `pyrage`/the browser.

/**
 * zstd-compress `data` → bytes in memory (zstd only) — no filesystem. The
 * in-memory form of {@link pack}; the output is a standard zstd frame, so it
 * opens with {@link unpackBytes}, {@link unpack} (write it out first), or stock
 * `zstd`.
 *
 * @param data - The bytes to compress.
 * @param level - zstd level 1–22 (default 9).
 * @returns The compressed bytes.
 */
export function packBytes(data: Uint8Array, level?: number): Buffer {
  return transportPackBytes(data, level);
}

/**
 * zstd-decompress bytes → bytes in memory — the in-memory form of {@link unpack}.
 *
 * @param data - The zstd-compressed bytes (e.g. from {@link packBytes}).
 * @returns The decompressed bytes.
 */
export function unpackBytes(data: Uint8Array): Buffer {
  return transportUnpackBytes(data);
}

/**
 * zstd-compress + age-passphrase-encrypt `data` → bytes in memory — no plaintext
 * on disk. The in-memory form of {@link lock}, ideal for sealing sensitive data
 * you hold in memory (e.g. a fixed `Ags4File`'s `.bytes`) without ever writing
 * the plaintext out. The `.zst.age` envelope matches {@link lock}'s, so the
 * result opens with {@link unlockBytes}, {@link unlock}, `pyrage`, or the
 * browser, given the passphrase.
 *
 * @param data - The bytes to compress and encrypt.
 * @param password - The age passphrase. Required — there is no agent-key path.
 * @param level - zstd level 1–22 (default 9).
 * @param logN - scrypt work factor (`log2(N)`); omit for the pinned default (18).
 *   Lower is faster but weaker; must be `1..=20`.
 * @returns The sealed bytes.
 */
export function lockBytes(
  data: Uint8Array,
  password: string,
  level?: number,
  logN?: number,
): Buffer {
  return transportLockBytes(data, password, level, logN);
}

/**
 * age-passphrase-decrypt + zstd-decompress `.zst.age` bytes → bytes in memory —
 * the in-memory form of {@link unlock}.
 *
 * @param data - The sealed bytes (e.g. from {@link lockBytes}).
 * @param password - The age passphrase the envelope was sealed with.
 * @returns The original bytes.
 * @throws If the passphrase is wrong or the input is not a passphrase envelope.
 */
export function unlockBytes(data: Uint8Array, password: string): Buffer {
  return transportUnlockBytes(data, password);
}
