import { defineConfig } from "@playwright/test";

const BASE = process.env.VITE_BASE ?? "/";
// Overridable so two checkouts (a worktree and the main clone) can run e2e on
// one machine without reusing each other's preview server — reuseExistingServer
// makes a same-port collision silently test the OTHER checkout's build.
const PORT = Number(process.env.PW_PORT ?? 4173);

// End-to-end tests drive the REAL app (wasm validator in a Web Worker +
// DuckDB-wasm) in headless Chromium against a local `vite preview` of the
// production build — the same artefact that deploys. This replaces the
// fragile "verify only on the deployed site" loop: a synthetic AGS4 fixture
// exercises validate → fix → explore deterministically.
//
// Requires a prior `npm run build`: the webServer below runs `vite preview`,
// which serves an existing `dist/` at the deploy base but does NOT build it
// (CI's e2e workflow runs build:wasm + build first). The base comes from the
// same VITE_BASE knob vite.config.ts reads, so previewing a non-root build
// needs no second edit here.
export default defineConfig({
  testDir: "./e2e",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 1 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: "list",
  timeout: 60_000,
  expect: { timeout: 30_000 },
  use: {
    baseURL: `http://localhost:${PORT}`,
    trace: "on-first-retry",
  },
  projects: [
    {
      name: "chromium",
      use: { browserName: "chromium", viewport: { width: 1280, height: 800 } },
    },
    {
      // Responsive/layout checks at a phone viewport. Scoped to layout.spec so
      // the full suite isn't doubled — the other specs assume desktop width.
      // layout.spec is viewport-aware and also runs under `chromium`, so the
      // same assertions are checked at 1280 and 390.
      name: "mobile",
      testMatch: /layout\.spec\.ts$/,
      use: {
        browserName: "chromium",
        viewport: { width: 390, height: 844 },
        isMobile: true,
      },
    },
  ],
  webServer: {
    command: `npm run preview -- --port ${PORT} --strictPort`,
    url: `http://localhost:${PORT}${BASE}`,
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});
