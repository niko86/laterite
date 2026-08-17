// Main-thread handle to the transport worker (zstd + age lock/unlock). The
// Tools "Transport" pane calls lock()/unlock() and gets a Promise back; the
// worker owns the lazy-loaded libs.
//
// A `workerChannel` consumer since #379, not a hand-rolled copy of one. What
// the copy got wrong is what the channel exists to get right: its crash
// handler rejected the requests in flight but KEPT the dead worker, so the
// next lock()/unlock() posted into the corpse and its promise never settled —
// the pane's spinner span forever, disabled buttons and all, with the error
// branch never reached. The channel retires a dead worker, so the next request
// spawns a fresh one and the user's own retry is what recovers the tool. No
// pane change: it already renders whatever rejection it is handed.

import { createChannel } from "./workerChannel";
import type { TransportReq, TransportRes } from "./transport.worker";

interface Pending {
  resolve: (bytes: Uint8Array) => void;
  reject: (e: Error) => void;
}

// Spawned on first use, where the old copy spawned at module load. Nothing
// needs the scrypt/zstd stack until a lock or unlock is actually requested,
// and a request that lands before the wasm is up queues in the worker behind
// its own init.
const channel = createChannel<TransportRes, TransportReq, Pending>(
  () =>
    new Worker(new URL("./transport.worker.ts", import.meta.url), {
      type: "module",
    }),
  // Both reply kinds carry bytes and nothing else — the transport protocol's
  // whole settle is "hand them over".
  (msg, p) => {
    p.resolve(new Uint8Array(msg.bytes));
  },
);

function post(
  kind: "lock" | "unlock",
  bytes: Uint8Array,
  passphrase: string,
): Promise<Uint8Array> {
  return new Promise((resolve, reject) => {
    channel.post(
      { kind, bytes: new ArrayBuffer(0), passphrase }, // bytes replaced inside post()
      bytes,
      { resolve, reject },
    );
  });
}

/** Compress + passphrase-encrypt bytes to a `.zst.age` (byte-compatible with
 *  laterite `lock` — zstd 9 + age scrypt log_N 18). */
export function lock(
  bytes: Uint8Array,
  passphrase: string,
): Promise<Uint8Array> {
  return post("lock", bytes, passphrase);
}

/** Decrypt + decompress a `.zst.age`'s bytes. Rejects on a wrong passphrase or
 *  a non-age / non-zstd payload. */
export function unlock(
  bytes: Uint8Array,
  passphrase: string,
): Promise<Uint8Array> {
  return post("unlock", bytes, passphrase);
}
