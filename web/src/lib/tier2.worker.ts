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
// `wasm-pack` run, which is load-bearing rather than cosmetic. Both builds emit
// `ags4_wasm_bg.wasm` by default; the bundler would then fingerprint them to two
// hashes with the same STEM, the precache glob `assets/ags4_wasm_bg-*.wasm` would
// match both, and the install would quietly carry the full engine again with
// nothing erroring and every size gate green.
import init from "../wasm-full/ags4_wasm_full.js";
import * as engine from "../wasm-full/ags4_wasm_full.js";
import wasmUrl from "../wasm-full/ags4_wasm_full_bg.wasm?url";
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
