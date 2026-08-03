// what this shows: init ONCE, at module scope, and let every call await that
// one promise. Every other example here takes this shape.
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
import init, { version } from "@laterite/ags4-wasm";

// Pass `module_or_path` EXPLICITLY. Left out, the glue falls back to fetching
// relative to `import.meta.url`, which breaks under a non-root `base` — the app
// hit exactly that. In a bundler this is an asset URL:
//
//     import wasmUrl from "@laterite/ags4-wasm/ags4_wasm_bg.wasm?url";
//     await init({ module_or_path: wasmUrl });
//
// Under Node there is no fetch for a file path, so hand it the bytes instead.
// The CALL is the same either way; only the argument differs.
const wasmPath = fileURLToPath(
  import.meta.resolve("@laterite/ags4-wasm/ags4_wasm_bg.wasm"),
);
const ready = init({ module_or_path: readFileSync(wasmPath) });

// One promise, awaited everywhere. Calls that arrive before the module is
// instantiated queue behind it rather than racing a live-before-ready export —
// every wasm function throws if it runs first.
await ready;

console.log(version());

assert.match(version(), /^\d+\.\d+\.\d+/);
