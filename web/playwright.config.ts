import { defineConfig } from "@playwright/test";

// End-to-end tests drive the REAL app (wasm validator in a Web Worker +
// DuckDB-wasm) in headless Chromium against a local `vite preview` of the
// production build — the same artefact that deploys. This replaces the
// fragile "verify only on the deployed site" loop: a synthetic AGS4 fixture
// exercises validate → fix → explore deterministically.
//
// Requires a prior `npm run build`: the webServer below runs `vite preview`,
// which serves an existing `dist/` at the deploy base `/laterite/` but
// does NOT build it (CI's e2e workflow runs build:wasm + build first).
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
    baseURL: "http://localhost:4173",
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
    command: "npm run preview -- --port 4173 --strictPort",
    url: "http://localhost:4173/laterite/",
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});
