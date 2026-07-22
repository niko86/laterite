import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["test/**/*.test.ts"],
    // The napi loader (`index.js`) and the `.node` binary are native — keep Vite
    // from transforming them; require() them as-is in the node runtime.
    server: { deps: { external: [/index\.js$/, /\.node$/] } },
    // Coverage for the TS wrapper. The tests import the SOURCE (`../ts/index`),
    // so v8 instruments `ts/**` directly — no dist/sourcemap hop. The native addon
    // is a `.node` binary vitest can't instrument (the Rust engine's coverage is
    // the nightly `cargo llvm-cov` run); the `*.generated.ts` files are codegen,
    // not hand-tested, so they're excluded. lcov feeds the Codecov `node` flag.
    coverage: {
      provider: "v8",
      reporter: ["text", "lcov"],
      // Measure only files the tests actually execute — not every ts/ file (some
      // top-level-import the native addon; instrumenting those for a baseline is a
      // known flakiness source). include/exclude still scope the executed set.
      all: false,
      include: ["ts/**"],
      exclude: ["ts/**/*.generated.ts"],
      // Regression floor: node is the lowest-covered surface (~59% lines at
      // 2026-07) and is being raised (see the tracking issue). The floor sits a
      // couple points under current so a genuine drop reds the `node` job
      // without a normal-fluctuation false-red — RATCHET UP as coverage climbs,
      // like the rust (88) and python (80) floors.
      thresholds: { lines: 57 },
    },
  },
});
