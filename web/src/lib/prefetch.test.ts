import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

// The idle-warm POLICY: what gets primed, on which devices, and — for the two
// heavy engines — that priming stops at the bytes. It is decided entirely by
// `warmLazyAssets()`, so it is testable without a browser; what a browser is
// needed for (the bytes actually reaching the CacheFirst bucket, no worker being
// created, no refetch on the click that follows) is pinned in web/e2e/app.spec.ts.
//
// Every module `warmLazyAssets` reaches for is stubbed. Not for speed alone: the
// real ones are DuckDB, echarts and Arrow — megabytes of browser code — and
// `validatorClient` spawns a Worker at module scope, which a node lane has no
// way to honour.

const TIER2_URL = "/assets/ags4_wasm_full_bg-TEST.wasm";

const { warmFetch, tier2Started } = vi.hoisted(() => ({
  warmFetch: vi.fn<() => Promise<void>>(() => Promise.resolve()),
  tier2Started: { value: false },
}));

vi.mock("./arrowResult", () => ({}));
vi.mock("proj4", () => ({ default: {} }));
vi.mock("echarts/core", () => ({}));
vi.mock("./duck", () => ({ warmFetch }));
vi.mock("./tier2Asset", () => ({ TIER2_WASM_URL: TIER2_URL }));
vi.mock("./validatorClient", () => ({
  isTier2Started: () => tier2Started.value,
}));

interface Device {
  saveData?: boolean;
  effectiveType?: string;
  cores?: number;
  memory?: number;
}

// A hand-rolled recorder rather than `vi.fn()`, and that is load-bearing for the
// failure test below: vitest attaches its own handler to a mock's returned
// promise (to populate `settledResults`), so a rejection out of a `vi.fn` can
// never be an UNHANDLED one — which silently made "the warm swallows its
// failures" unfalsifiable. Verified by deleting the `.catch` under test.
let fetchCalls: string[];
let fetchImpl: (url: string) => Promise<unknown>;

/** Let the fire-and-forget warms settle. Each is a dynamic import followed by a
 *  promise chain, so a single microtask drain is not enough — and the negative
 *  assertions ("this was NOT warmed") cannot wait on an event that never comes,
 *  which rules out polling for a condition. */
async function flush() {
  for (let i = 0; i < 5; i++) await new Promise((r) => setTimeout(r, 0));
}

/** Present `device` to the app, run `warmLazyAssets()` with a
 *  requestIdleCallback that fires synchronously, and settle the promises it
 *  started. A fresh module registry each time — `warmed` is module state, and
 *  the once-only guard is one of the things under test. */
async function warm(device: Device = { cores: 8, memory: 8 }) {
  vi.resetModules();
  vi.stubGlobal("window", {
    requestIdleCallback: (cb: () => void) => {
      cb();
    },
  });
  vi.stubGlobal("navigator", {
    hardwareConcurrency: device.cores ?? 8,
    deviceMemory: device.memory ?? 8,
    connection:
      (device.saveData ?? device.effectiveType)
        ? { saveData: device.saveData, effectiveType: device.effectiveType }
        : undefined,
  });
  vi.stubGlobal("fetch", (url: string) => {
    fetchCalls.push(url);
    return fetchImpl(url);
  });
  const { warmLazyAssets } = await import("./prefetch");
  warmLazyAssets();
  await flush();
  return warmLazyAssets;
}

beforeEach(() => {
  fetchCalls = [];
  fetchImpl = () => Promise.resolve();
  warmFetch.mockClear().mockResolvedValue(undefined);
  tier2Started.value = false;
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("warmLazyAssets", () => {
  it("primes the tier-2 engine on a capable device, and only fetches it", async () => {
    await warm();

    expect(fetchCalls).toContain(TIER2_URL);
    // The DuckDB warm is unchanged, and rides the same gate.
    expect(warmFetch).toHaveBeenCalledOnce();
    // NO assertion here that the warm didn't COMPILE tier 2. The obvious one —
    // `expect(startTier2Worker).not.toHaveBeenCalled()` — was written, and is
    // worthless: this module never imports that function, so it passes on every
    // build including one that instantiated wasm directly. Compilation means a
    // real Worker, which this lane cannot observe, so the check lives in
    // web/e2e/app.spec.ts, where it counts live workers and goes red when the
    // warm is made to start one.
  });

  it("skips both heavy engines on a low-end device, keeping the cheap warms", async () => {
    // 2 logical cores — `isLowEndDevice()` reads that as constrained.
    await warm({ cores: 2 });

    expect(fetchCalls).not.toContain(TIER2_URL);
    expect(warmFetch).not.toHaveBeenCalled();
    // The few-hundred-kB warms are NOT what the device gate is protecting
    // against, and dropping them would make Charts and Coordinates slower for
    // no saving worth having.
    expect(fetchCalls).toEqual(
      expect.arrayContaining([
        expect.stringContaining("ags_dictionary.json"),
        expect.stringContaining("rules-catalogue.json"),
      ]),
    );
  });

  it("downloads nothing at all under Data Saver", async () => {
    await warm({ saveData: true });

    expect(fetchCalls).toEqual([]);
    expect(warmFetch).not.toHaveBeenCalled();
  });

  it("skips the tier-2 warm when Explore or Excel already started that worker", async () => {
    // The user reached one of those tabs inside the idle window, so its engine
    // is already in flight. CacheFirst does not coalesce, so warming here would
    // be a second 5.2 MB download rather than a cache hit.
    tier2Started.value = true;
    await warm();

    expect(fetchCalls).not.toContain(TIER2_URL);
    expect(warmFetch).toHaveBeenCalledOnce();
  });

  it("warms once per session, however often it is called", async () => {
    const warmLazyAssets = await warm();
    const first = fetchCalls.length;

    warmLazyAssets();
    warmLazyAssets();

    expect(fetchCalls.length).toBe(first);
    expect(warmFetch).toHaveBeenCalledOnce();
  });

  // Named for what it asserts and no more: that a failed warm stays quiet. The
  // other half of "silent AND harmless" — the tab that needs the asset still
  // fetching it after a failed warm — is #357's ticket, and nothing here checks
  // it. A test named for a claim it doesn't make is how a gap gets counted as
  // covered.
  it("swallows every failure rather than surfacing an unhandled rejection", async () => {
    // All of it fails at once: the fetches reject, the DuckDB warm rejects, and
    // the three chunk imports throw on evaluation. NOT ONE may surface as an
    // unhandled rejection — a speculative download that reports its own failure
    // is noise a user cannot act on, and the tab that needs the asset fetches it
    // again anyway. Asserted on a listener rather than left to the runner:
    // vitest passes a suite that logs one, so "the test still went green" is no
    // evidence at all here (dropping the tier-2 `.catch` was invisible until
    // this listener existed).
    const unhandled: unknown[] = [];
    const record = (reason: unknown) => unhandled.push(reason);
    process.on("unhandledRejection", record);

    fetchImpl = () => Promise.reject(new Error("offline"));
    warmFetch.mockRejectedValue(new Error("no engine"));
    vi.resetModules();
    vi.doMock("proj4", () => {
      throw new Error("chunk 404");
    });
    vi.doMock("echarts/core", () => {
      throw new Error("chunk 404");
    });
    vi.doMock("./arrowResult", () => {
      throw new Error("chunk 404");
    });

    await warm();
    await flush(); // node reports an unhandled rejection a turn late

    process.off("unhandledRejection", record);
    // Re-mocked, NOT unmocked: `doUnmock` would hand the following tests the
    // real modules, and the real `arrowResult` pulls Arrow into a node lane
    // that has no use for it (and into the coverage denominator, at 0%).
    vi.doMock("proj4", () => ({ default: {} }));
    vi.doMock("echarts/core", () => ({}));
    vi.doMock("./arrowResult", () => ({}));

    expect(unhandled).toEqual([]);
    // …and it did try, rather than passing by never having warmed anything.
    expect(fetchCalls).toContain(TIER2_URL);
  });

  it("falls back to a timer where requestIdleCallback is missing", async () => {
    // Safari has no requestIdleCallback. Without the fallback the warm would
    // silently never run there — no error, just a slower first Explore/Excel
    // click on every Apple device.
    vi.useFakeTimers();
    vi.resetModules();
    vi.stubGlobal("window", {});
    vi.stubGlobal("navigator", { hardwareConcurrency: 8, deviceMemory: 8 });
    vi.stubGlobal("fetch", (url: string) => {
      fetchCalls.push(url);
      return fetchImpl(url);
    });

    const { warmLazyAssets } = await import("./prefetch");
    warmLazyAssets();
    expect(fetchCalls).toEqual([]);

    await vi.advanceTimersByTimeAsync(1500);
    vi.useRealTimers();
    await flush();

    expect(fetchCalls).toContain(TIER2_URL);
    expect(warmFetch).toHaveBeenCalledOnce();
  });
});
