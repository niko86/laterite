// Cross-surface golden for the wasm ENGINE leg (#448, PR D2 — the browser
// half of the `_content_hash` rollout). Drives the SAME browser cdylib
// `tools/xcheck/emit_js.mjs`'s `runWasm` uses under node — the glue
// instantiated straight from the built `.wasm` BYTES (no fetch), the
// identical artifact the browser loads — and decodes the Arrow IPC it
// returns with `apache-arrow` (a real web dependency; duckdb-wasm consumes
// the same stream via `arrowResult.ts`).
//
// SAME fixture + golden UUIDv8 hash values as Node's
// `rust-packages/laterite-node/test/p-content-hash.test.ts` (itself pinned
// to a release build of the Python wheel). wasm, Node and Python all route
// through the ONE shared `keychain::group_content_hashes`, so matching those
// values here IS the cross-surface parity proof.
//
// Against the **full** build (`web/src/wasm-full`), which is the one that still
// carries this door: `arrow_ipc` is the `arrow` feature, and since #355 the app
// splits its engine in two — tier 1 for Validate/Fix/Export/Tools, the full build
// for Explore and Excel. So the artifact under test here is still exactly the one
// the browser instantiates to produce these columns, which is what makes it a
// cross-surface proof rather than a lab result.
//
// Both wasm dirs are gitignored, built only by `wasm-pack build … --out-dir
// web/src/wasm[-full]` (the `e2e` job in e2e.yml, before `npm run typecheck`/
// `npm run build`). The FAST `unit` lane in the same workflow deliberately
// runs vitest with NO wasm build ("no wasm, no browser" — its own header
// comment), so this suite self-skips when the artifact is absent — the same
// guard `coords.test.ts` uses for the optional OSTN15 grid file.
import { existsSync, readFileSync } from "node:fs";
import { fileURLToPath, pathToFileURL } from "node:url";
import path from "node:path";
import { beforeAll, describe, expect, it } from "vitest";
import { tableFromIPC, type Table } from "apache-arrow";

const here = path.dirname(fileURLToPath(import.meta.url));
const wasmDir = path.join(here, "..", "wasm-full");
const wasmBinPath = path.join(wasmDir, "ags4_wasm_full_bg.wasm");
const hasWasm = existsSync(wasmBinPath);

// Two deliveries of one project — identical to p-content-hash.test.ts.
//   D1: BH01 (GL 10.00), BH02 (GL 12.00).            NO LOCA_REM column at all.
//   D2: BH01 UNCHANGED but the level is re-emitted "10.0" (formatting only),
//       BH02 REVISED (12.00 -> 12.75), BH03 new.     PLUS a blank LOCA_REM column.
const D1 =
  '"GROUP","PROJ"\r\n' +
  '"HEADING","PROJ_ID","PROJ_NAME"\r\n' +
  '"UNIT","",""\r\n' +
  '"TYPE","ID","X"\r\n' +
  '"DATA","P100","Demo"\r\n' +
  '"GROUP","LOCA"\r\n' +
  '"HEADING","LOCA_ID","LOCA_NATE","LOCA_GL"\r\n' +
  '"UNIT","","m","m"\r\n' +
  '"TYPE","ID","2DP","2DP"\r\n' +
  '"DATA","BH01","523400.00","10.00"\r\n' +
  '"DATA","BH02","523500.00","12.00"\r\n';
const D2 =
  '"GROUP","PROJ"\r\n' +
  '"HEADING","PROJ_ID","PROJ_NAME"\r\n' +
  '"UNIT","",""\r\n' +
  '"TYPE","ID","X"\r\n' +
  '"DATA","P100","Demo"\r\n' +
  '"GROUP","LOCA"\r\n' +
  '"HEADING","LOCA_ID","LOCA_NATE","LOCA_GL","LOCA_REM"\r\n' +
  '"UNIT","","m","m",""\r\n' +
  '"TYPE","ID","2DP","2DP","X"\r\n' +
  '"DATA","BH01","523400.00","10.0",""\r\n' +
  '"DATA","BH02","523500.00","12.75",""\r\n' +
  '"DATA","BH03","523600.00","9.25",""\r\n';

// Pinned from the SAME release build of the Python wheel Node pins
// (rust-packages/laterite-node/test/p-content-hash.test.ts).
const D1_BH01_HASH = "1bd2eb52-b18a-8427-b176-86d9881b1119";
const D1_BH02_HASH = "462fff84-6fb9-8631-a5d1-e728db23568d";
const D2_BH02_HASH = "28b39176-eb14-8a42-86b9-c9e7b6b74e39";
const D2_BH03_HASH = "accb778f-a86b-88c7-b28d-344a6e61c631";

/** Read one row's `column` value by `LOCA_ID`, from an already-decoded Table. */
function cellFor(table: Table, locaId: string, column: string): unknown {
  const ids = table.getChild("LOCA_ID")!;
  const values = table.getChild(column)!;
  for (let i = 0; i < table.numRows; i++) {
    if (ids.get(i) === locaId) return values.get(i);
  }
  throw new Error(`${locaId} not found`);
}

describe.skipIf(!hasWasm)("_content_hash (#448, wasm engine)", () => {
  // Typed `any`: the glue is generated (gitignored) — its `.d.ts` may not
  // exist at TS-analysis time in every context this file is parsed from, and
  // the import path below is deliberately non-literal (see beforeAll) so
  // neither tsc nor Vite's import-analysis eagerly resolves it when absent.
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  let glue: any;

  beforeAll(async () => {
    // A computed specifier (not a string literal) + /* @vite-ignore */: Vite's
    // import-analysis plugin would otherwise try to resolve this dynamic
    // import at TRANSFORM time (independent of describe.skipIf, which only
    // skips *running* the test bodies) and fail the whole file when the wasm
    // build is absent (the `unit` CI lane — see the file header).
    const specifier = pathToFileURL(
      path.join(wasmDir, "ags4_wasm_full.js"),
    ).href;
    glue = await import(/* @vite-ignore */ specifier);
    const wasmBytes = readFileSync(wasmBinPath);
    await glue.default({ module_or_path: wasmBytes });
  });

  function locaTable(text: string, contentHash: boolean): Table {
    const ds = glue.read(new TextEncoder().encode(text), "utf-8");
    const ipc = ds.arrow_ipc("LOCA", true, contentHash) as Uint8Array;
    return tableFromIPC(ipc);
  }

  it("contentHash=false carries no _content_hash column", () => {
    const table = locaTable(D1, false);
    expect(table.schema.fields.map((f) => f.name)).not.toContain(
      "_content_hash",
    );
  });

  it("contentHash=true adds a _content_hash column, a non-empty string per row", () => {
    const table = locaTable(D1, true);
    expect(table.schema.fields.map((f) => f.name)).toContain("_content_hash");
    const hashes = table.getChild("_content_hash")!;
    expect(table.numRows).toBeGreaterThan(0);
    for (let i = 0; i < table.numRows; i++) {
      const v = hashes.get(i);
      expect(typeof v).toBe("string");
      expect((v as string).length).toBeGreaterThan(0);
    }
  });

  it("golden hash values match Node/Python for both deliveries — the cross-surface proof", () => {
    const a = locaTable(D1, true);
    const b = locaTable(D2, true);
    expect(cellFor(a, "BH01", "_content_hash")).toBe(D1_BH01_HASH);
    expect(cellFor(a, "BH02", "_content_hash")).toBe(D1_BH02_HASH);
    // BH01 unchanged: a formatting-only reemit ("10.0" vs "10.00") and a new
    // blank LOCA_REM column do NOT change the hash (typed + blank-insensitive).
    expect(cellFor(b, "BH01", "_content_hash")).toBe(D1_BH01_HASH);
    // BH02 revised (12.00 -> 12.75): the hash MUST move.
    expect(cellFor(b, "BH02", "_content_hash")).toBe(D2_BH02_HASH);
    expect(cellFor(b, "BH03", "_content_hash")).toBe(D2_BH03_HASH);
  });
});
