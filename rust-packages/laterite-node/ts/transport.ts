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

/** zstd-compress `src` → `dest`. `level` 1–22 (default 9). */
export function pack(src: string, dest: string, level?: number): PackStats {
  return transportPack(src, dest, level);
}

/** zstd-decompress `src` → `dest`. */
export function unpack(src: string, dest: string): UnpackStats {
  return transportUnpack(src, dest);
}

/** zstd-compress + age-passphrase-encrypt `src` → `dest`. */
export function lock(src: string, dest: string, password: string, level?: number): PackStats {
  return transportLock(src, dest, password, level);
}

/** age-passphrase-decrypt + zstd-decompress `src` → `dest`. Throws on a wrong
 * passphrase / non-passphrase envelope. */
export function unlock(src: string, dest: string, password: string): UnpackStats {
  return transportUnlock(src, dest, password);
}
