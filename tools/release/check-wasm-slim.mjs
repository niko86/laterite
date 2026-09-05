#!/usr/bin/env node
///////////////////////////////////////////////////////////////////////////////
// check-wasm-slim.mjs — hold the published slim wasm to the two promises it
// was split out to make (#330).
//
// WHY THIS EXISTS
//   `laterite-ags4-wasm` used to ship one artifact carrying everything: the
//   Excel reader/writer, Arrow IPC, the certificate stack, diff, merge, censor.
//   A page that only wanted to validate a file downloaded all of it. The cargo
//   features split that up; this asserts the split is still doing its job.
//
//   This gate guards what NPM gets, and only that. The browser app's engine is
//   no longer the same artifact — see `check-wasm-tier1.mjs`, which guards the
//   one the app precaches (#338). Both drive `wasm-artifact-gate.mjs`, whose
//   header carries the why: two instruments per artifact, because a surface
//   check and a size check catch different silent failures, and two size axes,
//   because gzip and raw disagree exactly where it matters.
//
// USAGE
//   node tools/release/check-wasm-slim.mjs <pkg-dir>
//   Exit 0 = the artifact is the slim one; 1 = it is not, and why.
///////////////////////////////////////////////////////////////////////////////

import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { checkWasmArtifact, pkgDirArg } from "./wasm-artifact-gate.mjs";

// The slim surface: every export the facts table (modality.json
// `wasm_exports`, #926) says is UNGATED. Anything behind `excel` / `arrow` /
// `certify` / `diff` / `merge` / `censor` is absent on purpose. The question
// this list used to force — "should this ship to every consumer?" — is still
// answered once, deliberately: it is now the `features_required: []` cell on
// the export's table row, where registering the export happens anyway, instead
// of a second copy here that #883 showed drifts one hand-edit at a time.
const FACTS = JSON.parse(
  readFileSync(
    fileURLToPath(new URL("../../modality.json", import.meta.url)),
    "utf-8",
  ),
).wasm_exports.exports;
const EXPECTED_FUNCTIONS = FACTS.filter(
  (row) => row.features_required.length === 0,
).map((row) => row.name);
if (EXPECTED_FUNCTIONS.length === 0) {
  // A gate fed an empty expectation would "pass" any artifact at all.
  console.error(
    "check-wasm-slim: the export-facts table yielded no ungated exports",
  );
  process.exit(1);
}

// `ParsedDataset` is the only handle class the slim build hands out (the gated
// builds add `ExcelResult` and `MergeResult`), and its MEMBERS are listed for
// the reason the shared gate's header gives: `arrow`'s door into this build is
// `arrow_ipc`, a method on this class, which a top-level-only check cannot see.
const EXPECTED_CLASSES = {
  ParsedDataset: ["group_codes", "meta", "rows_json"],
};

// Measured 2026-08-16 (post-#342), by the shared gate's own `gzipSync(level: 9)`
// so the numbers below are the ones it compares — node's zlib runs ~8 KB above
// the `gzip -9` binary on this artifact, which is exactly the sort of gap that
// makes a hand-copied figure wrong. Each row is that feature alone on top of
// slim:
//
//                 gzip KiB            raw KiB
//     slim         757.3              1868.9
//     certify      771.2  (+13.9)     1908.5  (+  39.7)
//     diff         775.1  (+17.9)     1922.8  (+  53.9)
//     censor       788.5  (+31.2)     1944.7  (+  75.8)
//     merge        793.5  (+36.2)     1962.8  (+  93.9)
//     arrow       1174.3 (+417.0)     3643.7  (+1774.9)
//     excel       1286.1 (+528.8)     3218.9  (+1350.0)
//     full        1771.1              5189.8
//
// Read the four cheap gates honestly: at +14 to +36 KiB gzipped, NO ceiling with
// honest headroom catches them, and these two ceilings do not pretend to. The
// SURFACE check catches a flipped gate exactly, by name, in either direction —
// that is its whole job, and it does not degrade as the gated code gets lighter.
// (Before #342 `diff`, `censor` and `merge` each crossed 850 KiB, but only
// because they dragged in the duplicate dictionary. That was the size check
// covering for the surface check by accident, and it is not a property to
// preserve.)
//
// What the ceilings are for is the failure the surface check CANNOT see: a heavy
// dependency landing on an UNGATED path, where the export list is unchanged and
// only the weight moved. Both leave ~12% headroom over slim for honest growth.
const MAX_GZIP_BYTES = 850 * 1024; // slim 757.3
const MAX_RAW_BYTES = 2100 * 1024; // slim 1868.9

checkWasmArtifact({
  label: "wasm-slim",
  pkg: pkgDirArg("check-wasm-slim.mjs <pkg-dir>"),
  functions: EXPECTED_FUNCTIONS,
  classes: EXPECTED_CLASSES,
  maxGzipBytes: MAX_GZIP_BYTES,
  maxRawBytes: MAX_RAW_BYTES,
  rebuild:
    "`wasm-pack build … -- --no-default-features` (cargo flags go AFTER the --;\n" +
    "  wasm-pack exits 0 when they land in the wrong place).",
});
