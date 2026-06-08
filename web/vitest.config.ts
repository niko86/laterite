import { defineConfig } from "vitest/config";

// Unit tests cover the PURE logic — the AGS4 line tokenizer's lossless
// invariant (load-bearing, and previously only a dev-only console.error that
// was silent in prod), severity classification, CSV/quoting helpers. Component
// rendering (Solid + DOM) is exercised end-to-end by the Playwright suite
// (web/e2e), so these run in a plain node env with no plugins — fast, no JSDOM.
export default defineConfig({
  test: {
    environment: "node",
    include: ["src/**/*.test.ts"],
    coverage: {
      // v8 provider — no Istanbul instrumentation step, and matches the
      // engine the bundle ships on. Scope is the files the unit suite
      // actually imports (the pure-logic modules); components + the worker
      // are covered by the Playwright e2e suite, not measured here.
      provider: "v8",
      // text → CI log; lcov → web/coverage/lcov.info for the Codecov upload
      // (web flag). Only uploaded from the public repo (see e2e.yml guard).
      reporter: ["text", "lcov"],
      // Floor gate, not a target. Introduced at 65 (68.76% baseline);
      // RATCHETED to 95 after agsline + analytics got full unit suites
      // (now 97.97% lines). A new untested pure module — or deleting a tested
      // one — drops this and fails the `unit` job, which is the point.
      thresholds: { lines: 95 },
    },
  },
});
