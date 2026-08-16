#!/usr/bin/env node
///////////////////////////////////////////////////////////////////////////////
// check-wasm-tier1.mjs — hold the browser's TIER-1 engine to the size the
// four-tier design (#338, `ags-wiki/design/dec-engine-tiering.md`) depends on.
//
// WHAT TIER 1 IS
//   The engine minus `arrow` and `excel`: everything Validate, Fix, Export and
//   ALL of Tools need, and nothing else. It is what the service worker
//   precaches, so it is the artifact most visitors download and the only one
//   most visitors ever download. Tier 2 (the full engine) is warm-fetched on
//   idle and compiled only on intent; tier 0 (the ~30 KB tokenizer, gated by
//   `web/scripts/check-wasm-tokenizer-size.mjs`) is what first render waits on.
//
//   The boundary is `arrow` + `excel` and nothing else because those two are
//   the only heavy gates: `certify`, `diff`, `merge` and `censor` cost +82 KiB
//   gzipped BETWEEN them, while `arrow` and `excel` account for 932 KiB of the
//   1,014 KiB gap to the full build. Buying Tools-offline and the Validate
//   tab's certificate button for 82 KiB is the whole trade.
//
// WHY IT NEEDS ITS OWN GATE
//   `check-wasm-slim.mjs` guards what NPM ships and no longer describes what
//   the app precaches — the two artifacts have different feature sets and
//   different ceilings. Without this gate nothing watches the artifact the
//   whole of #338 rests on, and it can only get heavier silently: no user-
//   visible symptom, no test failure, just a slower first visit.
//
// WHY THE RAW CEILING MATTERS HERE SPECIFICALLY
//   Raw is the decode-and-compile work on the first-render path, and it is what
//   a service worker's precache budget counts. Gzip says what the visitor
//   downloads; raw says what their device then has to chew through before the
//   engine can answer anything. Tier 1 is on the critical path for both.
//
// FALSIFIED BEFORE TRUSTED (#352)
//   A gate nobody has seen go red is a claim, not a check, so each instrument
//   was shown to fire on its own before this was believed. The regression used
//   throughout is `arrow` switched back on — the likeliest way this artifact
//   ever regresses, since it is one word in a feature list.
//
//     surface, extra    that build fails naming `build_ags4_ipc` AND
//                       `ParsedDataset.arrow_ipc` — the member catch is the
//                       one a top-level check would have missed
//     surface, missing  the slim artifact fails here naming `certify`, `diff`,
//                       `merge`, `censor` and `MergeResult`
//     gzip ceiling      with the surface widened to ADMIT arrow's two names, so
//                       only the weight is left to judge: 1246.9 KiB, over 940
//     raw ceiling       same artifact with the gzip ceiling lifted out of the
//                       way — 3849.7 KiB over 2350, reported as the redundant-
//                       data shape. Without this run the raw axis would be an
//                       untested claim, since gzip is checked first and would
//                       always have fired before it on this input.
//
// USAGE
//   node tools/release/check-wasm-tier1.mjs <pkg-dir>
//   Exit 0 = the artifact is the tier-1 one; 1 = it is not, and why.
//   `npm run build:wasm-tier1 && npm run check:wasm-tier1` from `web/` does both.
///////////////////////////////////////////////////////////////////////////////

import { checkWasmArtifact, pkgDirArg } from "./wasm-artifact-gate.mjs";

// The tier-1 surface, in full: the slim ten plus the four cheap gates. What is
// absent is what defines the tier — `build_ags4_ipc` (arrow), `ags4_to_xlsx` /
// `xlsx_to_ags4` (excel) — and it is absent by cost, not by taste. Adding a
// name here means deciding that every first-time visitor should download it.
const EXPECTED_FUNCTIONS = [
  "apply_fixes",
  "build_ags4",
  "censor",
  "certify",
  "compute_fixes",
  "dictionary",
  "diff",
  "engine_fingerprint",
  "engine_version",
  "list_rules",
  "merge",
  "read",
  "validate",
  "version",
];

// Two handle classes: `ParsedDataset` (ungated) and `MergeResult` (from
// `merge`). `excel`'s `ExcelResult` is the third, and its absence is half of
// what makes this tier 1.
//
// The members matter as much as the class names. `arrow`'s only door into
// `ParsedDataset` is `arrow_ipc`, a METHOD — turn that feature on and the class
// list is unchanged, so a top-level-only check would pass while the artifact
// carried the entire Arrow stack.
const EXPECTED_CLASSES = {
  MergeResult: ["bytes", "revisions_json", "warnings_json"],
  ParsedDataset: ["group_codes", "meta", "rows_json"],
};

// Measured 2026-08-16 by the shared gate's own `gzipSync(level: 9)`, on the
// artifact `npm run build:wasm-tier1` produces:
//
//                          gzip KiB     raw KiB
//     slim (npm)             757.3      1868.9
//     TIER 1                 839.2      2093.9
//     tier 1 + arrow        1246.9      3849.7
//     tier 2 / full         1771.1      5189.8
//
// ~12% headroom on both axes over the measured figures — the same margin the
// slim gate leaves, and enough for ordinary growth in the validator's own rules
// without admitting a dependency. A breach means a heavy dependency reached an
// UNGATED path, because a flipped feature is the surface check's job and it
// runs first.
const MAX_GZIP_BYTES = 940 * 1024; // tier 1 839.2
const MAX_RAW_BYTES = 2350 * 1024; // tier 1 2093.9

checkWasmArtifact({
  label: "wasm-tier1",
  pkg: pkgDirArg("check-wasm-tier1.mjs <pkg-dir>"),
  functions: EXPECTED_FUNCTIONS,
  classes: EXPECTED_CLASSES,
  maxGzipBytes: MAX_GZIP_BYTES,
  maxRawBytes: MAX_RAW_BYTES,
  rebuild:
    "`npm run build:wasm-tier1` from `web/`, which is the one place the\n" +
    "  tier-1 feature list is written down (cargo flags go AFTER the `--`;\n" +
    "  wasm-pack exits 0 when they land in the wrong place).",
});
