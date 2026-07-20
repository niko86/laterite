import { configDefaults, defineConfig } from "vitest/config";

// Unit tests cover the PURE logic — severity classification, CSV/analytics
// helpers, and the rest of the wasm-free modules. Component rendering (Solid +
// DOM) is exercised end-to-end by the Playwright suite (web/e2e), so these run
// in a plain node env with no plugins — fast, no JSDOM, no Rust/wasm toolchain.
//
// The tokenizer-dependent tests (agsline display helpers + fix-preview
// geometry) now call the wasm-backed tokenizer (#533), so they run in the
// separate wasm lane (vitest.wasm.config.ts, in the e2e job) and are excluded
// here to keep this lane pure + fast. The tokenizer's own invariants are pinned
// in Rust (laterite-ags4-parse's `tokenize_spans` proptest).
export default defineConfig({
  test: {
    environment: "node",
    include: ["src/**/*.test.ts"],
    exclude: [
      ...configDefaults.exclude,
      "src/lib/agsline.test.ts",
      "src/lib/fixpreview.test.ts",
    ],
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
