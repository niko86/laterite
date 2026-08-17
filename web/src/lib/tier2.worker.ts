// The second engine worker (#354, ags-wiki/design/dec-engine-tiering.md): the
// Explore typed parse + its per-group Arrow pulls, and Tools → Excel's two
// conversions. It is created the first time one of those tabs is opened and
// never before, so a visitor who opens neither pays for one worker, not two.
//
// It runs **tier 2** (#355): the full engine, 1771 KiB gzipped against tier 1's
// 839. That artifact is deliberately NOT precached — it is `globIgnore`d in
// `vite.config.ts` and served by its own `CacheFirst` runtime rule — so it costs
// nothing until one of these two tabs is opened, and works offline after.
//
// Which makes this file the one place the whole tiering is spent: the import
// below is what decides whether most visitors download 839 KiB or 1771.
//
// `ParsedDataset` lives here now, in this worker's dispatch closure. It is the
// app's only stateful wasm handle and Explore is its only consumer, so it moved
// with the tab it belongs to: there is no state to migrate, and an Explore parse
// can no longer touch the worker a validate is running in.

// `ags4_wasm_full`, not `ags4_wasm` — a distinct `--out-name` on the second
// `wasm-pack` run, which is load-bearing rather than cosmetic: two artifacts
// sharing a fingerprinted stem would both match the tier-1 precache glob. The
// full reasoning, and the three other locks, are in `vite.config.ts` next to that
// glob and in ags-wiki/design/dec-engine-tiering.md.
import init from "../wasm-full/ags4_wasm_full.js";
import * as engine from "../wasm-full/ags4_wasm_full.js";
// Not a `?url` import of its own: the idle warm needs the SAME URL, so the two
// share one — see `tier2Asset.ts` for what drifting them would cost.
import { TIER2_WASM_URL } from "./tier2Asset";
import { createEngineDispatch } from "./engineDispatch";
import type { WorkerReq, WorkerRes } from "./engineDispatch";

// Same shape as `validator.worker.ts`, and for the same reasons — see its
// header for the transfer-list overload and the queue-behind-init contract.
// What a worker entry point owns is only ever those three things: the engine it
// instantiates, its readiness promise, and turning a thrown op into a reply
// (#351, which put everything else in `engineDispatch.ts`).
//
// Not factored into a shared `createWorkerEntry()`, though the two bootstraps
// are near-identical. #354 predicted they would diverge here, when this file took
// its own artifact, and they did not: the import above is static, like the other
// worker's, because the worker is ALREADY created lazily and a dynamic import
// inside it defers nothing a user waits on. So the honest reason is smaller than
// the predicted one — what a worker entry owns is which engine it instantiates,
// and a factory taking that as a parameter would leave each file a single call
// with the interesting part passed in. It stays worth revisiting if a third
// engine ever appears.
const ctx = self as unknown as Worker;
const reply = (msg: WorkerRes, transfer?: Transferable[]) => {
  if (transfer) ctx.postMessage(msg, transfer);
  else ctx.postMessage(msg);
};

const ready: Promise<void> = init({ module_or_path: TIER2_WASM_URL }).then(
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
