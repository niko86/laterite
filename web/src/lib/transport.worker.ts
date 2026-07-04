// The browser transport pipeline runs here, off the main thread, because the
// scrypt passphrase KDF is deliberately expensive (~256 MiB / seconds at
// log_N 18) and zstd of a big file is CPU-heavy — either would freeze the UI.
// The main thread talks to this worker only through `transportClient.ts`.
//
// Format is byte-compatible with laterite's `lock`/`unlock` (Rust `age` +
// `zstd`, and Python `pyrage`): zstd-compress THEN age-passphrase-encrypt, so a
// `.zst.age` produced here opens with `lat unlock` and vice-versa. The two knobs
// are pinned to laterite's: zstd level 9, and scrypt log_N 18 (matching
// laterite-transport::SCRYPT_LOG_N — conservative age decoders cap the factor,
// see that crate). `@bokuweb/zstd-wasm` + `age-encryption` (Filippo Sottile's
// official age TS impl) are lazy-loaded — only this worker pulls them in, so the
// validator path never pays their weight.

import { init as zstdInit, compress, decompress } from "@bokuweb/zstd-wasm";
import { Decrypter, Encrypter } from "age-encryption";

/** zstd level — laterite `lock`'s default. */
const ZSTD_LEVEL = 9;
/** scrypt work factor — MUST match laterite-transport::SCRYPT_LOG_N (18) so a
 *  browser-locked file opens in the CLI/library and vice-versa. */
const SCRYPT_LOG_N = 18;

export interface LockReq {
  id: number;
  kind: "lock";
  /** Transferred plaintext bytes. */
  bytes: ArrayBuffer;
  passphrase: string;
}
export interface UnlockReq {
  id: number;
  kind: "unlock";
  /** Transferred `.zst.age` bytes. */
  bytes: ArrayBuffer;
  passphrase: string;
}
export type TransportReq = LockReq | UnlockReq;

export type TransportRes =
  | { type: "ready" }
  | { type: "initError"; error: string }
  | { id: number; ok: true; kind: "locked" | "unlocked"; bytes: ArrayBuffer }
  | { id: number; ok: false; error: string };

const ctx = self as unknown as Worker;
const reply = (msg: TransportRes, transfer?: Transferable[]) =>
  transfer ? ctx.postMessage(msg, transfer) : ctx.postMessage(msg);

// zstd needs its wasm instantiated once before compress/decompress. age is pure
// JS (no init). Every handler awaits this, so a request that lands before the
// wasm is ready queues behind it rather than racing a live-before-ready call.
const ready: Promise<void> = zstdInit().then(() => undefined);
ready.then(
  () => reply({ type: "ready" }),
  (e) => reply({ type: "initError", error: String(e) }),
);

self.onmessage = async (e: MessageEvent<TransportReq>) => {
  const req = e.data;
  try {
    await ready;
    if (req.kind === "lock") {
      // compress-then-encrypt (zstd needs low-entropy input; ciphertext is
      // random). One-shot, not streamed — bounded by the caller's size cap.
      const compressed = compress(new Uint8Array(req.bytes), ZSTD_LEVEL);
      const enc = new Encrypter();
      enc.setPassphrase(req.passphrase);
      enc.setScryptWorkFactor(SCRYPT_LOG_N); // pin to laterite's factor
      const locked = await enc.encrypt(compressed);
      const buf = locked.slice().buffer;
      reply({ id: req.id, ok: true, kind: "locked", bytes: buf }, [buf]);
      return;
    }
    // unlock: decrypt-then-decompress. A wrong passphrase throws from age →
    // the outer catch turns it into an `ok: false` the client rejects.
    const dec = new Decrypter();
    dec.addPassphrase(req.passphrase);
    const compressed = await dec.decrypt(new Uint8Array(req.bytes), "uint8array");
    const plain = decompress(compressed);
    const buf = plain.slice().buffer;
    reply({ id: req.id, ok: true, kind: "unlocked", bytes: buf }, [buf]);
  } catch (err) {
    reply({ id: req.id, ok: false, error: String(err) });
  }
};
