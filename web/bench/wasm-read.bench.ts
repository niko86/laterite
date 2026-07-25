// The first perf harness for the wasm ENGINE. The browser surface had NO
// benchmark of any kind — the last surface with no perf floor (Rust has the
// criterion benches, Python `bench-vs-python-ags4.py`, node `read.bench.ts` since
// #6/T6). This mirrors them: it materializes a 25 MB forge fixture through the
// SAME browser cdylib the app loads — the glue instantiated straight from the
// built `.wasm` BYTES (no fetch), the pattern `src/lib/content-hash.test.ts` uses
// under node — so the numbers sit on the same axis as the other surfaces.
//
// wasm carries an intrinsic cost the native surfaces do not: every call COPIES
// the input across the JS→wasm linear-memory boundary, and the typed columns
// come back as Arrow IPC bytes to be decoded host-side. That copy is real and
// part of what a browser pays, so it stays in the measured path.
//
// The wasm init is async (`import()` the glue, then instantiate from bytes), but
// vitest's benchmark runner does NOT await a `beforeAll` before the first
// iteration — so the setup runs at MODULE TOP-LEVEL (ESM top-level await),
// exactly as node's `read.bench.ts` reads its fixture synchronously before
// registering benches. By the time `describe` runs, `glue`/`bytes` are ready.
//
// `web/src/wasm` is gitignored (built by `npm run build:wasm`); the fixture is
// built by `tools/gen-bench-fixtures.sh`. Self-skips when either is absent — a
// skipped bench, like the Rust/node ones, not a hard failure.
//
// Run: `npm run bench:wasm`
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { bench, describe } from "vitest";

const here = path.dirname(fileURLToPath(import.meta.url));
const wasmDir = path.join(here, "..", "src", "wasm");
const wasmBinPath = path.join(wasmDir, "ags4_wasm_bg.wasm");
const fixture = path.join(
  here,
  "..",
  "..",
  "output",
  "bench-fixtures",
  "large.ags",
);
const ready = existsSync(wasmBinPath) && existsSync(fixture);

// The generated wasm-bindgen glue is gitignored and its `.d.ts` isn't in the type
// graph, so the exact surface this bench drives is declared locally — mirroring
// node's `read.bench.ts`, which imports a fully-typed `read` rather than reaching
// through `any`. Kept minimal on purpose: only what the four cases below call.
interface WasmDataset {
  group_codes(): string[];
  arrow_ipc(code: string, withKeys: boolean): Uint8Array;
  free(): void;
}
interface WasmGlue {
  default(init: { module_or_path: Uint8Array }): Promise<unknown>;
  read(bytes: Uint8Array): WasmDataset;
  validate(
    bytes: Uint8Array,
    dictVersion: string | undefined,
    includeWarnings: boolean,
    includeFyi: boolean,
    encoding: string,
    maxPerRule: number | undefined,
    dictBytes: Uint8Array | undefined,
    dictReplace: boolean,
  ): unknown;
}

// Async wasm init at module top-level so the glue is live before any bench body
// runs (see header). Guarded by `ready` so a clean checkout collects fast and
// skips, rather than throwing on the missing build. `glue`/`bytes` are only read
// inside the benches, which `describe.skipIf(!ready)` gates — hence the definite-
// assignment assertion: when a bench runs, both are set.
let glue!: WasmGlue;
let bytes: Uint8Array = new Uint8Array();
if (ready) {
  // Computed specifier (not a literal) + /* @vite-ignore */ so Vite's
  // import-analysis doesn't try to resolve the gitignored glue at transform time.
  const specifier = pathToFileURL(path.join(wasmDir, "ags4_wasm.js")).href;
  glue = (await import(/* @vite-ignore */ specifier)) as WasmGlue;
  await glue.default({ module_or_path: readFileSync(wasmBinPath) });
  bytes = readFileSync(fixture);
}

describe.skipIf(!ready)("wasm/read", () => {
  // Parse into the typed dataset (columns still lazy) — the floor the
  // materialization builds on. `free()` returns the wasm-side allocation so a
  // tight loop over 25 MB doesn't grow linear memory unboundedly.
  bench("read [large]", () => {
    const ds = glue.read(bytes);
    ds.free();
  });

  // The full typed read: parse + build every group's Arrow IPC, keys-less (the
  // node default post-#6). This is the browser explorer's actual read cost.
  bench("read + arrow_ipc(all groups) [large]", () => {
    const ds = glue.read(bytes);
    for (const code of ds.group_codes()) ds.arrow_ipc(code, false);
    ds.free();
  });

  // Keyed variant — keeps `_id`/`_parent_id`, so it pays the content-key chain;
  // the gap to the keys-less build above is that keychain cost on the wasm path.
  bench("read + arrow_ipc(all, keys) [large]", () => {
    const ds = glue.read(bytes);
    for (const code of ds.group_codes()) ds.arrow_ipc(code, true);
    ds.free();
  });

  // The validator path, both tier gates off — comparable to the Rust
  // `check_file/large` baseline (269 ms @ 25 MB, native).
  bench("validate [large]", () => {
    glue.validate(
      bytes,
      undefined,
      false,
      false,
      "utf-8",
      undefined,
      undefined,
      false,
    );
  });
});
