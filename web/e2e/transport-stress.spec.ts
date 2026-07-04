import { test, type CDPSession, type Page } from "@playwright/test";

// OPT-IN transport stress probe (STRESS=1). Measures the compress + encrypt
// pipeline a browser "pack/lock" would run, across file sizes and CPU throttle
// rates, to find where the TIME wall is (→ the size cap for the tool, #295).
//
//   STRESS=1 npx playwright test transport-stress.spec.ts --workers=1
//   STRESS=1 STRESS_RATES=6 STRESS_SIZES=50,100 npx playwright test transport-stress.spec.ts --workers=1
//
// Uses native primitives (CompressionStream gzip + Web Crypto AES-GCM) — the
// FASTEST option, so the numbers are an optimistic floor: a wasm zstd/age lib
// would be slower, so the real cap should be more conservative than this.
//
// LIMITATION: CDP throttles CPU for real, but the "low-end RAM" init only spoofs
// navigator.deviceMemory (for the app's heuristic) — it does NOT cap actual
// memory. So this measures TIME/throughput reliably; real OOM on a 2 GB device
// needs a physical/limited machine. Peak heap is reported best-effort.

const RUN = !!process.env.STRESS;
const RATES = (process.env.STRESS_RATES ?? "1,4,6").split(",").map(Number);
const SIZES_MB = (process.env.STRESS_SIZES ?? "25,50,100,200").split(",").map(Number);
const APP = "/laterite/"; // localhost = secure context → crypto.subtle available

const LOW_END_INIT = `try {
  Object.defineProperty(navigator, 'deviceMemory', { configurable: true, get: () => 2 });
  Object.defineProperty(navigator, 'hardwareConcurrency', { configurable: true, get: () => 2 });
} catch (e) {}`;

test.describe(RUN ? "transport stress (STRESS=1)" : "transport stress (skipped — set STRESS=1)", () => {
  test.skip(!RUN, "opt-in: STRESS=1 npx playwright test transport-stress.spec.ts --workers=1");
  test.describe.configure({ mode: "serial" });

  for (const rate of RATES) {
    test(`CPU ${rate}x`, async ({ browser }) => {
      test.setTimeout(1_800_000);
      const ctx = await browser.newContext();
      await ctx.addInitScript(LOW_END_INIT);
      const page: Page = await ctx.newPage();
      const cdp: CDPSession = await ctx.newCDPSession(page);
      await cdp.send("Emulation.setCPUThrottlingRate", { rate });
      await page.goto(APP);

      for (const mb of SIZES_MB) {
        const r = await page.evaluate(async (mb) => {
          const N = mb * 1024 * 1024;
          // Semi-compressible buffer, built fast: repeat a 64 KB random block.
          // Real AGS4 compresses far better (repetitive quoted text) → less
          // encrypt work, so this is a conservative proxy.
          const block = new Uint8Array(65536);
          crypto.getRandomValues(block);
          const buf = new Uint8Array(N);
          for (let o = 0; o < N; o += block.length)
            buf.set(block.subarray(0, Math.min(block.length, N - o)), o);

          const out: Record<string, unknown> = { ok: false };
          try {
            let t = performance.now();
            const compStream = new Blob([buf])
              .stream()
              .pipeThrough(new CompressionStream("gzip"));
            const compressed = new Uint8Array(
              await new Response(compStream).arrayBuffer(),
            );
            out.compressMs = Math.round(performance.now() - t);
            out.ratio = +(compressed.length / N).toFixed(3);

            const key = await crypto.subtle.generateKey(
              { name: "AES-GCM", length: 256 },
              false,
              ["encrypt"],
            );
            const iv = crypto.getRandomValues(new Uint8Array(12));
            t = performance.now();
            const enc = await crypto.subtle.encrypt(
              { name: "AES-GCM", iv },
              key,
              compressed,
            );
            out.encryptMs = Math.round(performance.now() - t);
            out.encBytes = enc.byteLength;
            out.ok = true;
          } catch (e) {
            out.error = String(e);
          }
          const perf = performance as Performance & {
            memory?: { usedJSHeapSize: number };
          };
          out.heapMB = perf.memory
            ? Math.round(perf.memory.usedJSHeapSize / 1048576)
            : null;
          return out;
        }, mb);
        const total = ((r.compressMs as number) || 0) + ((r.encryptMs as number) || 0);
        console.log(
          `rate=${rate}x  size=${String(mb).padStart(3)}MB  ` +
            `compress=${String(r.compressMs ?? "-").padStart(6)}ms (ratio ${r.ratio ?? "-"})  ` +
            `encrypt=${String(r.encryptMs ?? "-").padStart(5)}ms  ` +
            `TOTAL=${String(total).padStart(6)}ms  heap=${r.heapMB}MB  ok=${r.ok}` +
            (r.error ? `  ERR=${r.error}` : ""),
        );
      }
      await ctx.close();
    });
  }
});
