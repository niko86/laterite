import { test, expect, type Page, type CDPSession } from "@playwright/test";
import { APP } from "./helpers";
import { fileURLToPath } from "node:url";
import path from "node:path";

// OPT-IN performance harness (PERF=1). It MEASURES each real user flow under
// emulated SLOW hardware and prints a table — it asserts nothing, so it never
// gates CI. The owner's report is "fast on my Mac, bad on a slower computer",
// so we emulate a weak machine two ways at once:
//   • CPU: CDP Emulation.setCPUThrottlingRate (1×/4×/6×) — wasm compile + JS.
//   • RAM/cores: navigator.deviceMemory / hardwareConcurrency overridden low,
//     so the device-capability prefetch gate (Tranche 1) sees "low-end" even
//     on a beefy dev Mac.
//
//   PERF=1 npx playwright test perf.spec.ts --workers=1
//   PERF=1 PERF_RATES=6 npx playwright test perf.spec.ts --workers=1   # quick
//
// Runs against the same `vite preview` of dist/ as the rest of the e2e suite
// (build first). Two metrics track the specific optimisations:
//   • "idle DuckDB pull" — MB of engine wasm fetched UNPROMPTED during a
//     validate-only idle window (Tranche 1: should drop to ~0 on low-end).
//   • "Explore revisit"  — re-entering Explore after a tab switch (Tranche 2:
//     should drop to ~instant once ingested).

const RUN = !!process.env.PERF;
const RATES = (process.env.PERF_RATES ?? "1,4,6")
  .split(",")
  .map((n) => parseInt(n.trim(), 10))
  .filter((n) => n > 0);
// Idle window (ms) after cold load to let the prefetch fire before we measure
// what it pulled. Generous so a throttled requestIdleCallback still runs.
const IDLE_MS = parseInt(process.env.PERF_IDLE_MS ?? "8000", 10);

const fixture = (name: string) =>
  path.join(path.dirname(fileURLToPath(import.meta.url)), "fixtures", name);

// Emulate a weak machine's RAM/core count (CPU is throttled separately via CDP).
const LOW_END_INIT = `try {
  Object.defineProperty(navigator, 'deviceMemory', { configurable: true, get: () => 2 });
  Object.defineProperty(navigator, 'hardwareConcurrency', { configurable: true, get: () => 2 });
} catch (e) {}`;

interface Metrics {
  coldMs: number;
  cpuProbeMs: number;
  idleDuckdbMB: number;
  validateMs: number;
  exploreMs: number;
  revisitMs: number;
  queryMs: number;
  chartMs: number;
}
const byRate = new Map<number, Metrics>();

test.describe(
  RUN ? "perf (PERF=1, low-end emulation)" : "perf (skipped — set PERF=1)",
  () => {
    test.skip(
      !RUN,
      "opt-in: PERF=1 npx playwright test perf.spec.ts --workers=1",
    );
    test.describe.configure({ mode: "serial" });

    for (const rate of RATES) {
      test(`CPU ${rate}x`, async ({ browser }) => {
        test.setTimeout(900_000);
        // Fresh context per rate ⇒ genuine cold load (no warm HTTP/SW cache).
        const ctx = await browser.newContext();
        await ctx.addInitScript(LOW_END_INIT);
        const page: Page = await ctx.newPage();
        const cdp: CDPSession = await ctx.newCDPSession(page);
        await cdp.send("Emulation.setCPUThrottlingRate", { rate });

        // (1) Cold load → validator ready.
        const t0 = Date.now();
        await page.goto(APP);
        await expect(
          page.getByRole("button", { name: /Clean \(minimal\)/ }),
        ).toBeVisible({
          timeout: 180_000,
        });
        const coldMs = Date.now() - t0;

        // Sanity-check the throttle is live: a fixed busy loop should scale ~rate×.
        const cpuProbeMs = await page.evaluate(() => {
          const t = performance.now();
          let x = 0;
          for (let i = 0; i < 5e7; i++) x += i % 7;
          return Math.round(performance.now() - t) + (x < 0 ? 1 : 0);
        });

        // (2) T1: let the SW take control, idle WITHOUT touching Explore, then read
        //     how much engine wasm the prefetch pulled into the SW cache. The wasm
        //     is fetched inside the DuckDB worker (invisible to page CDP Network),
        //     but the SW intercepts every in-scope fetch — so the runtime cache is
        //     the reliable, context-independent witness of an unprompted warm.
        await page
          .waitForFunction(
            () => navigator.serviceWorker?.controller != null,
            undefined,
            {
              timeout: 60_000,
            },
          )
          .catch(() => {});
        await page.waitForTimeout(IDLE_MS);
        const idleDuckdbMB = await page.evaluate(async () => {
          try {
            const c = await caches.open("ags-duckdb-wasm");
            let bytes = 0;
            for (const req of await c.keys()) {
              const res = await c.match(req);
              if (res) bytes += (await res.arrayBuffer()).byteLength;
            }
            return +(bytes / 1024 / 1024).toFixed(1);
          } catch {
            return -1;
          }
        });

        // (3) Validate a dirty file (worker validate + report marshal + main-thread passes).
        const t1 = Date.now();
        await page
          .locator('input[type="file"]')
          .setInputFiles(fixture("many_findings.ags"));
        await expect(page.getByText("✗").first()).toBeVisible({
          timeout: 180_000,
        });
        const validateMs = Date.now() - t1;

        // (4) Explore: low-end emulation triggers the cold-engine gate (T1b) —
        //     accept it, then time the DuckDB instantiate + parse + ingest from
        //     the confirm (so my read-time isn't counted).
        await page
          .locator('input[type="file"]')
          .setInputFiles(fixture("fixable.ags"));
        await page.getByRole("tab", { name: /^Explore$/ }).click();
        const gate = page.getByRole("button", { name: /^Continue$/ });
        await gate.waitFor({ state: "visible", timeout: 5000 }).catch(() => {});
        const t2 = Date.now();
        if (await gate.isVisible().catch(() => false)) await gate.click();
        await expect(page.getByText(/data rows/)).toBeVisible({
          timeout: 300_000,
        });
        const exploreMs = Date.now() - t2;

        // (5) T2: leave Explore, come back — should be instant once ingested.
        await page.getByRole("tab", { name: /^Validate$/ }).click();
        await page.getByRole("tab", { name: /^Explore$/ }).click();
        const t3 = Date.now();
        await expect(page.getByText(/data rows/)).toBeVisible({
          timeout: 300_000,
        });
        const revisitMs = Date.now() - t3;

        // (6) SQL query.
        await page.getByRole("button", { name: "SQL" }).click();
        await page.locator("textarea").fill(`SELECT * FROM "LOCA"`);
        const t4 = Date.now();
        await page.getByRole("button", { name: /^Run/ }).click();
        await expect(page.getByText("LOCA_ID").last()).toBeVisible({
          timeout: 120_000,
        });
        const queryMs = Date.now() - t4;

        // (7) Chart render.
        await page.getByRole("button", { name: "Charts" }).click();
        await page.getByLabel("Table").selectOption("LOCA");
        const t5 = Date.now();
        await expect(page.locator("canvas").first()).toBeVisible({
          timeout: 120_000,
        });
        const chartMs = Date.now() - t5;

        byRate.set(rate, {
          coldMs,
          cpuProbeMs,
          idleDuckdbMB,
          validateMs,
          exploreMs,
          revisitMs,
          queryMs,
          chartMs,
        });
        await ctx.close();
      });
    }

    test.afterAll(() => {
      const s = (ms: number) => `${(ms / 1000).toFixed(1)}s`;
      const rows = [...byRate.entries()]
        .sort((a, b) => a[0] - b[0])
        .map(([rate, m]) => ({
          CPU: `${rate}x`,
          "cpu probe": `${m.cpuProbeMs}ms`,
          "cold→ready": s(m.coldMs),
          "idle DuckDB pull (T1)": `${m.idleDuckdbMB} MB`,
          validate: s(m.validateMs),
          "Explore 1st": s(m.exploreMs),
          "revisit (T2)": s(m.revisitMs),
          query: s(m.queryMs),
          chart: s(m.chartMs),
        }));
      // eslint-disable-next-line no-console
      console.log(
        `\n=== PERF (low-end emul: deviceMemory=2, cores=2; CPU throttle ${RATES.join("/")}×) ===`,
      );
      // eslint-disable-next-line no-console
      console.table(rows);
    });
  },
);
