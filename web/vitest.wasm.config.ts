import { defineConfig } from "vitest/config";

// The tokenizer-dependent tests — the agsline GROUP-block/alignment display
// helpers and the fix-preview geometry — call the wasm-backed
// `splitAgsFields`/`quoteAgsField` (#533). They run in THIS lane, which inits
// the tiny tokenizer wasm from disk in a setup file, rather than the fast
// pure-node `unit` lane (vitest.config.ts). CI runs this in the e2e job, after
// building the wasm; locally, `npm run build:wasm-tokenizer` first, then
// `npm run test:wasm`.
//
// No coverage gate here: the tokenizer's own invariants are pinned
// authoritatively in Rust (laterite-ags4-parse's `tokenize_spans` proptest),
// and the fast `unit` lane keeps the lines floor for the pure modules.
export default defineConfig({
  test: {
    environment: "node",
    include: ["src/lib/agsline.test.ts", "src/lib/fixpreview.test.ts"],
    setupFiles: ["./vitest.wasm.setup.ts"],
  },
});
