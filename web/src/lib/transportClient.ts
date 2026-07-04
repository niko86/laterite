// Main-thread handle to the transport worker (zstd + age lock/unlock). The
// Tools "Transport" pane calls lock()/unlock() and gets a Promise back; the
// worker owns the lazy-loaded libs. Same request/response-by-id shape as
// validatorClient.ts.

import type { TransportReq, TransportRes } from "./transport.worker";

type Pending = {
  resolve: (bytes: Uint8Array) => void;
  reject: (e: Error) => void;
};

const worker = new Worker(new URL("./transport.worker.ts", import.meta.url), {
  type: "module",
});

let nextId = 1;
const pending = new Map<number, Pending>();

// Resolves once zstd's wasm is instantiated; the pane can gate its first action
// on it (age is pure-JS, no init).
const readyPromise = new Promise<void>((resolve, reject) => {
  const onInit = (e: MessageEvent<TransportRes>) => {
    const msg = e.data;
    if ("type" in msg && msg.type === "ready") {
      worker.removeEventListener("message", onInit);
      resolve();
    } else if ("type" in msg && msg.type === "initError") {
      worker.removeEventListener("message", onInit);
      reject(new Error(msg.error));
    }
  };
  worker.addEventListener("message", onInit);
});

worker.addEventListener("message", (e: MessageEvent<TransportRes>) => {
  const msg = e.data;
  if ("type" in msg) return; // ready / initError handled above
  const p = pending.get(msg.id);
  if (!p) return;
  pending.delete(msg.id);
  if (msg.ok) p.resolve(new Uint8Array(msg.bytes));
  else p.reject(new Error(msg.error));
});

worker.addEventListener("error", (e) => {
  const err = new Error(e.message || "transport worker crashed");
  for (const [, p] of pending) p.reject(err);
  pending.clear();
});

export function ready(): Promise<void> {
  return readyPromise;
}

function post(kind: "lock" | "unlock", bytes: Uint8Array, passphrase: string): Promise<Uint8Array> {
  return new Promise((resolve, reject) => {
    const id = nextId++;
    // Transfer a copy so the caller's Uint8Array stays intact.
    const buf = bytes.slice().buffer;
    pending.set(id, { resolve, reject });
    worker.postMessage({ id, kind, bytes: buf, passphrase } as TransportReq, [buf]);
  });
}

/** Compress + passphrase-encrypt bytes to a `.zst.age` (byte-compatible with
 *  laterite `lock` — zstd 9 + age scrypt log_N 18). */
export function lock(bytes: Uint8Array, passphrase: string): Promise<Uint8Array> {
  return post("lock", bytes, passphrase);
}

/** Decrypt + decompress a `.zst.age`'s bytes. Rejects on a wrong passphrase or
 *  a non-age / non-zstd payload. */
export function unlock(bytes: Uint8Array, passphrase: string): Promise<Uint8Array> {
  return post("unlock", bytes, passphrase);
}
