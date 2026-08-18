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
// It runs **tier 1** (#355): the engine minus `arrow` and `excel`, 839 KiB
// gzipped against the full build's 1771. That is the artifact the service worker
// precaches, so it is what a first visit downloads and — for most visitors —
// the only engine they ever download.
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

// Re-exported so `validatorClient.ts` keeps importing the two result shapes it
// names from a worker it talks to, rather than reaching past it into the shared
// dispatch. `WorkerReq`/`WorkerRes` were forwarded here for the same reason and
// are not any more: since #357 the wire types are read by `workerChannel.ts`,
// which takes them from `engineDispatch` directly. That is not inconsistency —
// the channel must not import this module even for a type, because a unit test
// importing the channel would then load a file that spawns wasm.
export type { ReportMeta, CensorTally } from "./engineDispatch";

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

// The ops tier 1 serves, named one by one rather than handed the whole
// module. This build *has* `read`, but its `ParsedDataset` has no `arrow_ipc`
// door and it has no Excel conversions at all — so listing what this worker can
// serve is what makes the tier boundary a compile error rather than a runtime
// surprise: name an op here that tier 1 drops and this file stops typechecking.
const dispatch = createEngineDispatch(
  {
    validate: engine.validate,
    certify: engine.certify,
    compute_fixes: engine.compute_fixes,
    apply_fixes: engine.apply_fixes,
    diff: engine.diff,
    merge: engine.merge,
    censor: engine.censor,
    build_ags4: engine.build_ags4,
  },
  reply,
);

self.onmessage = async (e: MessageEvent<WorkerReq>) => {
  const req = e.data;
  try {
    await ready;
    await dispatch(req);
  } catch (err) {
    reply({ id: req.id, ok: false, error: String(err) });
  }
};
