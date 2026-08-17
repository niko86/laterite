// Warm the lazy-loaded assets during browser IDLE time after the app is up — so
// switching to Explore / Charts / Coordinates / Tools feels instant instead of
// kicking off a multi-MB download on the click. Everything here is
// fire-and-forget + idempotent (each loader caches its result) and runs only
// when the main thread is idle (so it never delays the validate-first path).
//
// CRITICAL for low-end hardware: we NEVER compile a heavy engine on idle. On a
// fast Mac that compile is invisible; on a 2-core / 2 GB machine it stole
// seconds of CPU + tens of MB of RAM from a user who may only ever click
// Validate. So the heavy engines are only ever *warm-fetched* (cache-primed,
// no compile) on a clearly-capable device, and skipped entirely on a low-end
// one — the compile is deferred to real intent either way (see duck.ts:warmFetch
// + the EngineGate confirmation for DuckDB; the second worker's own creation for
// the tier-2 engine).
//
// This module is safe to static-import from App: its only static imports are two
// tiny modules — a capability predicate and a URL string — so none of the heavy
// deps land in the entry chunk.

import { isLowEndDevice } from "./device";
import { TIER2_WASM_URL } from "./tier2Asset";
import { isTier2Started } from "./validatorClient";

interface NetInfo {
  saveData?: boolean;
  effectiveType?: string;
}

let warmed = false;

function onIdle(fn: () => void): void {
  const ric = (
    window as unknown as {
      requestIdleCallback?: (
        cb: () => void,
        opts?: { timeout: number },
      ) => void;
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

  // The two heavy engines. On a low-end device, warm NEITHER — the tab that
  // needs one downloads it on demand, behind a loading state (and, for DuckDB,
  // behind the EngineGate confirmation). On a capable device, prime the bytes
  // into the HTTP/SW cache WITHOUT compiling, so a later click downloads nothing
  // yet a validate-only session never pays a wasm-compile / worker / engine-heap
  // cost it had no use for. ONE gate for both, because that is the policy —
  // "this machine cannot afford speculation" is a fact about the machine, not
  // about which engine is being speculated on.
  if (isLowEndDevice()) return;

  // Tier 2 — the full engine (#356, ags-wiki/design/dec-engine-tiering.md), 5.2
  // MB, the artifact Explore and Tools → Excel need and nothing else does. FETCH
  // ONLY: the bytes land in the `ags-engine-tier2` CacheFirst bucket and stop
  // there. Instantiating it here would compile ~5 MB of wasm for two tabs most
  // visitors never open — handing back a good part of what the tiering just won
  // — so the compile still waits for the second worker, which only opening one
  // of those tabs creates.
  //
  // Queued ahead of DuckDB deliberately: at 5.2 MB against 36 it is the one that
  // can plausibly finish, and Tools → Excel is the only tier-2 consumer that
  // never touches DuckDB — the one place a skipped warm is the whole delay
  // rather than a rounding error on a 36 MB wait.
  onIdle(() => {
    // Explore or Excel may have been opened inside the idle window, in which
    // case that worker is already fetching this exact URL — and CacheFirst does
    // not coalesce, so a second request here is a second 5.2 MB download, not a
    // cache hit. Read at fire time, not at call time, so the whole window counts.
    if (isTier2Started()) return;
    void fetch(TIER2_WASM_URL).catch(() => {});
  });

  // The 36/41 MB DuckDB engine (see duck.ts:warmFetch — it primes only the
  // variant selectBundle would later pick, and no-ops once instantiated).
  onIdle(
    () => void import("./duck").then((m) => m.warmFetch()).catch(() => {}),
  );
}
