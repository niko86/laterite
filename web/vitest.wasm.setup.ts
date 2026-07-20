// Init the tiny AGS4 tokenizer wasm (#533) ONCE, up front, from the built
// artifact — so the wasm-backed `splitAgsFields`/`quoteAgsField` run
// synchronously in the tests of this lane (agsline display helpers +
// fix-preview geometry). In the browser the app awaits `tokenizerReady()`
// (async init); here we init synchronously from disk with the glue's
// `initSync`, which takes the bytes directly (no fetch).
//
// Requires `npm run build:wasm-tokenizer` first — the e2e CI job builds it
// before running this lane; locally, run it once.
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { initSync } from "./src/wasm-tokenizer/ags4_tokenizer.js";

const wasm = fileURLToPath(
  new URL("./src/wasm-tokenizer/ags4_tokenizer_bg.wasm", import.meta.url),
);

try {
  initSync({ module: readFileSync(wasm) });
} catch (e) {
  throw new Error(
    "tokenizer wasm not built or failed to init — run 'npm run build:wasm-tokenizer' first " +
      `(${(e as Error).message})`,
  );
}
