// Size gate for the tiny AGS4 tokenizer wasm (#533).
//
// The tokenizer wasm exists to give the browser the shared Rust
// tokenizer/quoter WITHOUT the 6.9 MB engine — its whole justification is that
// it stays tiny. That premise must be PROVEN, not asserted: this gate trips if
// the artifact balloons, which is exactly what an accidental heavy dependency
// (e.g. laterite-types' optional `arrow` feature getting turned on, or a
// non-leaf crate creeping into the dep graph) would cause.
//
// Ceiling is generous (~5x the ~30 KB baseline) so ordinary growth is fine, but
// any order-of-magnitude regression — the only kind that matters here — fails
// loudly. Run after `build:wasm-tokenizer`.

import { statSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const wasm = join(
  here,
  "..",
  "src",
  "wasm-tokenizer",
  "ags4_tokenizer_bg.wasm",
);

// 150 KiB. Baseline at introduction was ~30 KB (13 KB gzipped); the engine wasm
// it replaces on the main thread is ~6.9 MB, so this ceiling is ~45x under the
// thing we're avoiding while leaving the tokenizer room to grow.
const MAX_BYTES = 150 * 1024;

let size;
try {
  size = statSync(wasm).size;
} catch {
  console.error(
    `[wasm-tokenizer-size] ${wasm} not found — run 'npm run build:wasm-tokenizer' first.`,
  );
  process.exit(1);
}

const kib = (size / 1024).toFixed(1);
if (size > MAX_BYTES) {
  console.error(
    `[wasm-tokenizer-size] FAIL: ags4_tokenizer_bg.wasm is ${kib} KiB, over the ${MAX_BYTES / 1024} KiB ceiling.\n` +
      `  The tiny tokenizer wasm must stay tiny — a jump this size means a heavy dependency crept in\n` +
      `  (check laterite-types' 'arrow' feature is OFF and only parse+types are in the dep graph).`,
  );
  process.exit(1);
}

console.log(
  `[wasm-tokenizer-size] OK: ags4_tokenizer_bg.wasm is ${kib} KiB (ceiling ${MAX_BYTES / 1024} KiB).`,
);
