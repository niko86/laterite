// Init the tiny AGS4 tokenizer wasm (laterite-dev#533) ONCE, up front, from the built
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
  // `cause` matters here specifically: the two failures this catch covers look
  // identical in the message but are nothing alike — a missing file (nobody ran
  // the build) versus a wasm that will not instantiate (a stale or corrupt
  // artifact). Interpolating `.message` and dropping the original threw away the
  // stack that tells them apart, in the one situation where you need it.
  throw new Error(
    "tokenizer wasm not built or failed to init — run 'npm run build:wasm-tokenizer' first",
    { cause: e },
  );
}
