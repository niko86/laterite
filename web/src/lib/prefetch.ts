// Warm the lazy-loaded assets during browser IDLE time after the app is up — so
// switching to Explore / Charts / Coordinates / Tools feels instant instead of
// kicking off a multi-MB download on the click. Everything here is
// fire-and-forget + idempotent (each loader caches its result) and runs only
// when the main thread is idle (so it never delays the validate-first path).
//
// CRITICAL for low-end hardware: we NEVER compile the 36 MB DuckDB engine on
// idle. On a fast Mac that compile is invisible; on a 2-core / 2 GB machine it
// stole seconds of CPU + tens of MB of RAM from a user who may only ever click
// Validate. So the heavy engine is only ever *warm-fetched* (cache-primed,
// no compile) on a clearly-capable device, and skipped entirely on a low-end
// one — the compile is deferred to real Explore intent either way (see
// duck.ts:warmFetch + the EngineGate confirmation).
//
// This module is safe to static-import from App: it contains only DYNAMIC
// imports, so none of the heavy deps land in the entry chunk.

import { isLowEndDevice } from "./device";

interface NetInfo {
  saveData?: boolean;
  effectiveType?: string;
}

let warmed = false;

function onIdle(fn: () => void): void {
  const ric = (
    window as unknown as {
      requestIdleCallback?: (cb: () => void, opts?: { timeout: number }) => void;
    }
  ).requestIdleCallback;
  if (ric) ric(fn, { timeout: 4000 });
  else setTimeout(fn, 1500);
}

/** Kick off the idle prefetch once. Call after the validator is ready. */
export function warmLazyAssets(): void {
  if (warmed) return;
  warmed = true;

  const conn = (navigator as unknown as { connection?: NetInfo }).connection;
  if (conn?.saveData) return; // respect Data Saver — download nothing eagerly

  // Cheap warms (a few hundred kB total): the chart / coord / arrow JS chunks +
  // the static reference JSONs. Each in its OWN idle tick so a burst of
  // dynamic-import parse/eval can't blow a single requestIdleCallback deadline
  // on a slow CPU.
  onIdle(() => void import("./arrowResult").catch(() => {}));
  onIdle(() => void import("proj4").catch(() => {}));
  onIdle(() => void import("echarts/core").catch(() => {}));
  onIdle(() => {
    const base = import.meta.env.BASE_URL;
    void fetch(`${base}ags_dictionary.json`).catch(() => {});
    void fetch(`${base}rules-catalogue.json`).catch(() => {});
  });

  // The 36/41 MB DuckDB engine: on a low-end device, warm NOTHING — Explore will
  // ask + download+compile on demand (EngineGate). On a capable device, prime
  // the wasm into the HTTP/SW cache WITHOUT compiling (warmFetch), so a later
  // Explore click downloads nothing yet a validate-only session never pays the
  // wasm-compile / worker / engine-heap cost.
  if (!isLowEndDevice()) {
    onIdle(() => void import("./duck").then((m) => m.warmFetch()).catch(() => {}));
  }
}
