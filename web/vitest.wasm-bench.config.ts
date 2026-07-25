import { defineConfig } from "vitest/config";

// The wasm perf harness (`bench/wasm-read.bench.ts`) runs in its OWN lane, like
// node's `vitest bench`: it inits the browser cdylib from the built `.wasm` bytes
// under the node environment, and is NOT part of the unit `test` run. Needs the
// wasm built (`npm run build:wasm`) and the fixture generated
// (`tools/gen-bench-fixtures.sh`); the bench self-skips when either is absent.
export default defineConfig({
  test: {
    environment: "node",
    benchmark: {
      include: ["bench/wasm-read.bench.ts"],
    },
  },
});
