// One engine worker's lifecycle — spawning it lazily, correlating its replies,
// and retiring it when its engine never arrives. Nothing here knows what a
// reply MEANS; that mapping stays in `validatorClient.ts` beside the typed API
// it resolves into.
//
// It lived inline there until #357, and moved for a reason the split makes
// plain: `validatorClient` creates the always-on worker at module load, so a
// unit test cannot import it without spawning a worker in a runtime that has
// none. A channel that takes `spawn` as a parameter is drivable by a fake, and
// the behaviour worth testing on its own — a dead worker being dropped rather
// than kept — is exactly the behaviour that file could not reach.

import type { WorkerReq, WorkerRes } from "./engineDispatch";

/** A SUCCESSFUL reply to a request — all `settle` ever sees. The channel
 *  handles the other three shapes itself: the two lifecycle messages, and a
 *  failed op, which needs no per-kind knowledge to reject. */
export type WorkerReply = Extract<WorkerRes, { id: number; ok: true }>;

/** The whole of what a channel needs from a pending entry: a way to fail it.
 *  Resolving is per-kind and therefore the caller's business. */
interface Failable {
  reject: (e: Error) => void;
}

/** The engine wasm never arrived, or never instantiated — as distinct from an
 *  op that ran and failed. The distinction is the one a user can act on: this
 *  is the failure a RETRY can clear once its cause is fixed, and the tabs that
 *  need the tier-2 engine tell the two apart to say so (#357). */
export class EngineLoadError extends Error {
  constructor(message: string) {
    super(message);
    // Set explicitly: subclassing a built-in leaves `name` as "Error", and this
    // is the string that reaches a console log when nothing catches it.
    this.name = "EngineLoadError";
  }
}

// Omit over a discriminated union must DISTRIBUTE, else only the keys
// common to every member survive (dropping `dict`/`fixes`/`code`/… from the
// per-kind requests). The built-in Omit doesn't distribute, so spell it out.
type DistributiveOmit<T, K extends keyof never> = T extends unknown
  ? Omit<T, K>
  : never;
type ReqInit = DistributiveOmit<WorkerReq, "id">;

// One id space across every channel. Correlation never needs that — a reply
// only ever meets its own worker's pending table — but two workers each
// answering "id 3" in a console log is a debugging trap bought for nothing.
let nextId = 1;

/** One worker, plus the pending table that correlates its replies.
 *
 *  `spawn` runs on the first request (or `start()`) and not again while that
 *  worker lives, which is what lets a caller hold a channel for a worker that
 *  is never created — the whole point of the app's second one (#354).
 *
 *  `settle` matches a reply to the pending entry waiting on its id. It is only
 *  called for replies that survived correlation, so it may assume both exist. */
export function createChannel<P extends Failable>(
  spawn: () => Worker,
  settle: (msg: WorkerReply, p: P) => void,
) {
  // The pending table belongs to a worker GENERATION, not to the channel: a
  // retirement clears one worker's table without reaching into the requests its
  // replacement is already carrying.
  let live: {
    worker: Worker;
    ready: Promise<void>;
    pending: Map<number, P>;
  } | null = null;

  const start = () => {
    if (live) return live;
    const worker = spawn();
    const pending = new Map<number, P>();

    // The engine never arrived. Fail everything this worker was carrying and
    // DROP the handle, so the next request spawns a fresh worker that fetches
    // again (#357). Without the drop a retry posts into the same dead worker,
    // re-reads its settled rejection, and fails identically however long ago
    // the cause was fixed — a failure outliving what caused it, which is #339's
    // lesson one layer up. Terminating is what makes "dropped" true rather than
    // nominal: the old worker stops answering into a table nobody reads.
    const retire = (err: Error) => {
      if (live?.worker === worker) live = null;
      for (const [, p] of pending) p.reject(err);
      pending.clear();
      worker.terminate();
    };

    // Resolves when the worker has instantiated the wasm; rejects if init
    // failed. NOTHING gates its first render on this any more (#353) — App only
    // uses it to sequence the idle warm, and an op that arrives first queues in
    // the worker behind the same promise. Kept because "engine is up" still has
    // one consumer, and because a caller that ignores it is not silently racing.
    const ready = new Promise<void>((resolve, reject) => {
      const onInit = (e: MessageEvent<WorkerRes>) => {
        const msg = e.data;
        if ("type" in msg && msg.type === "ready") {
          worker.removeEventListener("message", onInit);
          resolve();
          // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- worker messages are a runtime boundary; keep the explicit check though the type narrows to it
        } else if ("type" in msg && msg.type === "initError") {
          worker.removeEventListener("message", onInit);
          const err = new EngineLoadError(msg.error);
          reject(err);
          // The worker replies `{ ok: false }` to each queued op as well, but
          // those land after this and find their entries already gone. Failing
          // them HERE is what gives them the load error rather than a bare
          // string, which is what a pane reads to offer a retry.
          retire(err);
        }
      };
      worker.addEventListener("message", onInit);
    });
    // Only the always-on worker's readiness has a reader (App's). An unread
    // rejection is an unhandled-rejection log, and a dead second engine is
    // already reported where it matters — every Explore/Excel op rejects with
    // that same init error, and both panes display it. Marking it handled here
    // doesn't take it from `ready()`, which still returns the real promise.
    void ready.catch(() => undefined);

    worker.addEventListener("message", (e: MessageEvent<WorkerRes>) => {
      const msg = e.data;
      if ("type" in msg) return; // ready / initError handled above
      const p = pending.get(msg.id);
      if (!p) return; // superseded + already dropped, or a retired generation
      pending.delete(msg.id);
      if (!msg.ok) p.reject(new Error(msg.error));
      else settle(msg, p);
    });

    worker.addEventListener("error", (e) => {
      // A hard worker error rejects everything in flight rather than hanging.
      // Only this worker's: the other one's requests are unaffected by it
      // crashing, which is half of why the split is a process boundary.
      //
      // Deliberately NOT a `retire()` — the handle stays live, so every request
      // posted after a crash still hangs. That is #363, which owns the change
      // and the e2e that has to fail without it.
      const err = new Error(e.message || "engine worker crashed");
      for (const [, p] of pending) p.reject(err);
      pending.clear();
    });

    live = { worker, ready, pending };
    return live;
  };

  return {
    start: () => {
      start();
    },
    ready: () => start().ready,

    // Whether this worker exists yet — WITHOUT creating it, which `start()` and
    // `ready()` both do. The idle warm asks before priming an engine the worker
    // may already be fetching.
    started: () => live !== null,

    // Send `bytes` to the worker as a transferable. We transfer a *copy*
    // (`slice()`) so the caller's original Uint8Array stays intact — the main
    // thread still needs it to decode the editor text + finding snippets.
    post(req: ReqInit, bytes: Uint8Array, p: P) {
      const ch = start();
      const id = nextId++;
      const copy = bytes.slice().buffer;
      // Registered AFTER the send, as it always has been: a reply can only
      // arrive on a later task, and a postMessage that throws then leaves no
      // entry waiting on an id the worker never saw.
      ch.worker.postMessage({ ...req, id, bytes: copy }, [copy]);
      ch.pending.set(id, p);
    },

    // For requests that carry no bytes (e.g. arrowIpc, which reads the worker-
    // held dataset). No transfer list.
    postBare(req: ReqInit, p: P) {
      const ch = start();
      const id = nextId++;
      ch.worker.postMessage({ ...req, id });
      ch.pending.set(id, p);
    },

    // For requests carrying TWO byte buffers (the revision diff). Transfer
    // copies so the caller's originals stay intact (same rationale as post()).
    postDual(req: ReqInit, a: Uint8Array, b: Uint8Array, p: P) {
      const ch = start();
      const id = nextId++;
      const aCopy = a.slice().buffer;
      const bCopy = b.slice().buffer;
      ch.worker.postMessage({ ...req, id, aBytes: aCopy, bBytes: bCopy }, [
        aCopy,
        bCopy,
      ]);
      ch.pending.set(id, p);
    },
  };
}
