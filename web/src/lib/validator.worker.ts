// The validator runs here, off the main thread, so a pathologically
// dirty file (millions of findings) can churn for tens of seconds without
// ever freezing the UI. The worker owns its own wasm instance; the
// main thread talks to it only through `validatorClient.ts`.
//
// The always-on one of the app's two workers (#354): Validate, Fix, Export and
// every tool but Excel. Explore and Excel are served by `tier2.worker.ts`, which
// is created only if one of those tabs is opened — so a parse can no longer
// arrive behind a validate, in either direction.
//
// Protocol: every request carries a monotonic `id`; every response echoes
// it so the client can correlate (and discard superseded runs). The wasm
// `validate()` is synchronous and uninterruptible once entered, so
// "cancellation" is necessarily *discard the stale result*, not *abort
// mid-rule* — but because it runs here, a superseded run never blocks the
// next paint.
//
// The ops themselves live in `engineDispatch.ts`, parameterised by the engine
// module (#351), so a second worker can serve Explore and Excel from a
// different build without duplicating them. What stays here is everything that
// is genuinely about being *this* worker: which engine it instantiates, its
// readiness promise, and turning a thrown op into an `{ ok: false }` reply.

import init from "../wasm/ags4_wasm.js";
import * as engine from "../wasm/ags4_wasm.js";
import wasmUrl from "../wasm/ags4_wasm_bg.wasm?url";
import { createEngineDispatch } from "./engineDispatch";
import type { WorkerReq, WorkerRes } from "./engineDispatch";

// Re-exported so `validatorClient.ts` keeps importing the protocol from a worker
// it talks to, rather than reaching past it into the shared dispatch. These four
// are what it imports; the other thirteen per-op request types were forwarded
// here too until #354, read by nobody — they are live members of `WorkerReq` in
// `engineDispatch.ts` either way, and types erase, so the bundle never knew.
export type {
  WorkerReq,
  WorkerRes,
  ReportMeta,
  CensorTally,
} from "./engineDispatch";

// Under the DOM lib (tsconfig), the dedicated-worker global's
// transfer-list `postMessage(message, transfer)` overload isn't visible;
// the `Worker` type carries exactly that signature, so route every reply
// through it (also gives a single typed choke point for `WorkerRes`).
const ctx = self as unknown as Worker;
const reply = (msg: WorkerRes, transfer?: Transferable[]) => {
  if (transfer) ctx.postMessage(msg, transfer);
  else ctx.postMessage(msg);
};

// Instantiate once. Passing the bundled-asset URL explicitly avoids the
// import.meta.url fetch fallback (which breaks under a non-root `base`).
// Every handler awaits this, so requests that arrive before init simply
// queue behind it rather than racing a live-before-ready `validate`.
const ready: Promise<void> = init({ module_or_path: wasmUrl }).then(
  () => undefined,
);

ready.then(
  () => {
    reply({ type: "ready" });
  },
  (e: unknown) => {
    reply({ type: "initError", error: String(e) });
  },
);

const dispatch = createEngineDispatch(engine, reply);

self.onmessage = async (e: MessageEvent<WorkerReq>) => {
  const req = e.data;
  try {
    await ready;
    await dispatch(req);
  } catch (err) {
    reply({ id: req.id, ok: false, error: String(err) });
  }
};
