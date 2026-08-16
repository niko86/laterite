// The second engine worker (#354, ags-wiki/design/dec-engine-tiering.md): the
// Explore typed parse + its per-group Arrow pulls, and Tools → Excel's two
// conversions. It is created the first time one of those tabs is opened and
// never before, so a visitor who opens neither pays for one worker, not two.
//
// It is named for the engine it is FOR, not the one it runs. Both workers still
// instantiate the SAME artifact here, deliberately: this ticket proves the
// two-worker shape and the lazy creation on their own, so that #355 — where this
// one takes the full engine and the other drops to tier 1 — changes one thing
// rather than three.
//
// `ParsedDataset` lives here now, in this worker's dispatch closure. It is the
// app's only stateful wasm handle and Explore is its only consumer, so it moved
// with the tab it belongs to: there is no state to migrate, and an Explore parse
// can no longer touch the worker a validate is running in.

import init from "../wasm/ags4_wasm.js";
import * as engine from "../wasm/ags4_wasm.js";
import wasmUrl from "../wasm/ags4_wasm_bg.wasm?url";
import { createEngineDispatch } from "./engineDispatch";
import type { WorkerReq, WorkerRes } from "./engineDispatch";

// Same shape as `validator.worker.ts`, and for the same reasons — see its
// header for the transfer-list overload and the queue-behind-init contract.
// What a worker entry point owns is only ever those three things: the engine it
// instantiates, its readiness promise, and turning a thrown op into a reply
// (#351, which put everything else in `engineDispatch.ts`).
//
// Not factored into a shared `createWorkerEntry()`, though it reads as
// duplication today: in #355 this file acquires its engine by DYNAMIC import of
// a separate artifact, so its readiness promise spans that import and the two
// bootstraps stop being the same shape. Sharing them now would buy one ticket of
// deduplication and then need an async variant grafted on.
const ctx = self as unknown as Worker;
const reply = (msg: WorkerRes, transfer?: Transferable[]) => {
  if (transfer) ctx.postMessage(msg, transfer);
  else ctx.postMessage(msg);
};

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
