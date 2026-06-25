// laterite.transport — zstd + age file-envelope helpers, the Node port of
// laterite-py's `transport`. General-purpose: compress/encrypt ANY file. The age
// envelope is interoperable with the Python side (pyrage). The work is native;
// this is the typed TS face.
import {
  type PackStats,
  type UnpackStats,
  transportLock,
  transportPack,
  transportUnlock,
  transportUnpack,
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
 * @returns The output size, compression ratio vs source, and elapsed seconds.
 */
export function lock(src: string, dest: string, password: string, level?: number): PackStats {
  return transportLock(src, dest, password, level);
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
