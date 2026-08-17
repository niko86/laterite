// One worker's lifecycle — spawning it lazily, correlating its replies, and
// retiring it when its engine never arrives. Nothing here knows what a reply
// MEANS; that mapping stays with each consumer (`validatorClient.ts` for the
// two engine workers, `transportClient.ts` for the transport one) beside the
// typed API it resolves into.
//
// It lived inline there until #357, and moved for a reason the split makes
// plain: `validatorClient` creates the always-on worker at module load, so a
// unit test cannot import it without spawning a worker in a runtime that has
// none. A channel that takes `spawn` as a parameter is drivable by a fake, and
// the behaviour worth testing on its own — a dead worker being dropped rather
// than kept — is exactly the behaviour that file could not reach.

/** The wire shape every worker of ours speaks — two lifecycle messages, then
 *  id-correlated replies that either carry a kind-specific payload or a bare
 *  error. A channel is generic over a protocol EXTENDING this (#379): the
 *  engine protocol (`engineDispatch`'s `WorkerRes`) and the transport protocol
 *  (`transport.worker`'s `TransportRes`) are both instances, which is what
 *  lets one channel implementation drive either worker. */
export type WorkerEnvelope =
  | { type: "ready" }
  | { type: "initError"; error: string }
  | { id: number; ok: true }
  | { id: number; ok: false; error: string };

/** A SUCCESSFUL reply to a request — all `settle` ever sees. The channel
 *  handles the other three shapes itself: the two lifecycle messages, and a
 *  failed op, which needs no per-kind knowledge to reject. */
export type OkReply<Res extends WorkerEnvelope> = Extract<
  Res,
  { id: number; ok: true }
>;

/** The whole of what a channel needs from a pending entry: a way to fail it.
 *  Resolving is per-kind and therefore the caller's business. */
interface Failable {
  reject: (e: Error) => void;
}

/** No engine is running — as distinct from an op that ran and failed. Either
 *  way the channel has retired the worker, so the next request starts a fresh
 *  one: that is what makes this the failure a RETRY can clear, and what the
 *  tabs read to offer one (#357, widened to the crash in #363).
 *
 *  `reason` exists because the two are equally retryable and not equally
 *  explicable. "Check your connection" is the useful thing to say about an
 *  engine that never downloaded and a false lead about one that died holding a
 *  file, so a pane that showed one message for both would be guessing at the
 *  reader's expense. */
export class EngineUnavailableError extends Error {
  readonly reason: "load" | "crash";

  constructor(message: string, reason: "load" | "crash") {
    super(message);
    // Set explicitly: subclassing a built-in leaves `name` as "Error", and this
    // is the string that reaches a console log when nothing catches it.
    this.name = "EngineUnavailableError";
    this.reason = reason;
  }
}

// Omit over a discriminated union must DISTRIBUTE, else only the keys
// common to every member survive (dropping `dict`/`fixes`/`code`/… from the
// per-kind requests). The built-in Omit doesn't distribute, so spell it out.
type DistributiveOmit<T, K extends keyof never> = T extends unknown
  ? Omit<T, K>
  : never;

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
export function createChannel<
  Res extends WorkerEnvelope,
  Req extends { id: number },
  P extends Failable,
>(spawn: () => Worker, settle: (msg: OkReply<Res>, p: P) => void) {
  type ReqInit = DistributiveOmit<Req, "id">;
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

    // Assigned synchronously by the `ready` executor below, and only called
    // from `retire` — which cannot run before that, since every path to it is a
    // worker event.
    let failReady!: (e: Error) => void;

    // This worker is finished — its engine never arrived (#357) or it died
    // (#363). Fail everything it was carrying and DROP the handle, so the next
    // request spawns a fresh worker. Without the drop that request is posted
    // into a dead worker: a failed engine re-reads its settled rejection and
    // fails identically however long ago the cause was fixed, and a crashed one
    // does not reply at all. Both are a failure outliving what caused it, which
    // is #339's lesson one layer up. Terminating is what makes "dropped" true
    // rather than nominal: the old worker stops answering into a table nobody
    // reads.
    //
    // `failReady` is not a tidy extra. A worker whose SCRIPT fails to load — a
    // stale chunk after a deploy — fires `error` and never sends `initError`,
    // so readiness is the one thing no other path can settle. App reads it to
    // report a dead engine at page level, and an unsettled promise there is a
    // page that neither reports the failure nor ever warms anything: the silent
    // state again, at the one altitude that is supposed to catch it.
    const retire = (err: Error) => {
      // Identity, not truthiness: `error` can fire again on a worker already
      // retired, and by then `live` may be its replacement — which this must
      // not take down with it.
      if (live?.worker === worker) live = null;
      failReady(err);
      for (const [, p] of pending) p.reject(err);
      pending.clear();
      worker.terminate();
    };

    // Resolves when the worker has instantiated the wasm; rejects if it never
    // does. NOTHING gates its first render on this any more (#353) — App only
    // uses it to report a dead engine and to sequence the idle warm, and an op
    // that arrives first queues in the worker behind the same promise. Kept
    // because "engine is up" still has one consumer, and because a caller that
    // ignores it is not silently racing.
    const ready = new Promise<void>((resolve, reject) => {
      failReady = reject;
      const onInit = (e: MessageEvent<WorkerEnvelope>) => {
        const msg = e.data;
        if ("type" in msg && msg.type === "ready") {
          worker.removeEventListener("message", onInit);
          resolve();
          // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- worker messages are a runtime boundary; keep the explicit check though the type narrows to it
        } else if ("type" in msg && msg.type === "initError") {
          worker.removeEventListener("message", onInit);
          // `retire` rejects `ready` as well. The worker replies `{ ok: false }`
          // to each queued op too, but those land after this and find their
          // entries already gone — failing them HERE is what gives them the
          // typed error rather than a bare string, which is what a pane reads
          // to offer a retry.
          retire(new EngineUnavailableError(msg.error, "load"));
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

    worker.addEventListener("message", (e: MessageEvent<WorkerEnvelope>) => {
      const msg = e.data;
      if ("type" in msg) return; // ready / initError handled above
      const p = pending.get(msg.id);
      if (!p) return; // superseded + already dropped, or a retired generation
      pending.delete(msg.id);
      // The one assertion the generics cost. postMessage is untyped at runtime,
      // so the listener sees the ENVELOPE; the constraint guarantees every
      // `ok: true` member of `Res` is such a reply, and narrowing back to the
      // caller's protocol is exactly what this channel exists to centralise.
      if (!msg.ok) p.reject(new Error(msg.error));
      else settle(msg as OkReply<Res>, p);
    });

    worker.addEventListener("error", (e) => {
      // A worker that crashed, or whose script never loaded. Retired like a
      // failed engine and for the same reason (#363): rejecting the requests in
      // flight was never the whole job — it left the handle pointing at a dead
      // worker, so those requests reported and every request AFTER them was
      // posted into silence. A hang never rejects, so that was the permanent
      // silent state with no error branch ever reached.
      //
      // Only this worker's requests: the other one's are unaffected by it
      // crashing, which is half of why the split is a process boundary.
      retire(
        new EngineUnavailableError(
          e.message || "engine worker crashed",
          "crash",
        ),
      );
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
