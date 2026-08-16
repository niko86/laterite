#!/usr/bin/env node
///////////////////////////////////////////////////////////////////////////////
// check-wasm-slim.mjs — hold the published slim wasm to the two promises it
// was split out to make (#330).
//
// WHY THIS EXISTS
//   `laterite-ags4-wasm` used to ship one artifact carrying everything: the
//   Excel reader/writer, Arrow IPC, the certificate stack, diff, merge, censor.
//   A page that only wanted to validate a file downloaded all of it. The cargo
//   features split that up; this asserts the split is still doing its job,
//   because both ways it can stop are SILENT:
//
//     1. A gate gets switched back on — a `default`-features build sneaks into
//        the release step, or a feature grows a dependency edge that pulls
//        another one in. Nothing fails; the package just doubles.
//     2. A heavy dependency lands on an UNGATED path, where no feature can
//        turn it off. This is the failure the tokenizer gate's comment names
//        ("laterite-ags4-types' optional `arrow` feature getting turned on"),
//        and it is why that gate exists at all.
//
//   So there are two checks, because one instrument cannot see both. The
//   SURFACE check catches (1) exactly — it reads the generated `.d.ts` and
//   compares the exported names against the list below, so a gate flipping in
//   either direction fails by name rather than by byte count. The SIZE check
//   catches (2), where the surface is unchanged and only the weight moved.
//
// WHY BOTH GZIP AND RAW BYTES
//   They answer different questions and neither subsumes the other.
//
//   GZIP is what a client actually pays (a CDN's brotli lands ~10% under it, so
//   this is a conservative read of delivery, not an inflated one), it is
//   deterministic at a fixed level, and node's zlib gives it to us with no
//   dependency. It is the axis that matters for "is this page fast".
//
//   RAW is the axis on which REDUNDANT data is unmistakable, and #342 is the
//   worked example: the dictionary was embedded twice, and the duplicate was
//   +1375 KiB raw against +98 KiB gzip — a 14:1 ratio, because JSON that
//   repetitive compresses ~18:1. Every compressed-size check we had reported it
//   as fine. Re-embedding it today would land the slim build at ~855 KiB
//   gzipped, which clears the ceiling below by five, so the gzip axis would
//   catch it on luck rather than on margin. Raw catches it by 1.4 MB. Raw is
//   also the honest cost on the wasm path specifically — decode-and-compile
//   work on every cold load, and what a service worker's precache budget counts.
//
// USAGE
//   node tools/release/check-wasm-slim.mjs <pkg-dir>
//   Exit 0 = the artifact is the slim one; 1 = it is not, and why.
///////////////////////////////////////////////////////////////////////////////

import { readFileSync, statSync } from "node:fs";
import { gzipSync, brotliCompressSync, constants } from "node:zlib";
import { basename, join } from "node:path";

const pkg = process.argv[2];
if (!pkg) {
  console.error("usage: check-wasm-slim.mjs <pkg-dir>");
  process.exit(1);
}

// The slim surface, in full. Every name here is ungated in `lib.rs`; anything
// behind `excel` / `arrow` / `certify` / `diff` / `merge` / `censor` is absent
// on purpose. Adding a genuinely new ungated export means adding it here — that
// edit is the point, since it forces the question "should this ship to every
// consumer?" to be answered once, deliberately, rather than by whoever's build
// happened to include it.
const EXPECTED_FUNCTIONS = [
  "apply_fixes",
  "build_ags4",
  "compute_fixes",
  "dictionary",
  "engine_fingerprint",
  "engine_version",
  "list_rules",
  "read",
  "validate",
  "version",
];

// `ParsedDataset` is the only handle class the slim build hands out. The gated
// builds add `ExcelResult` (excel) and `MergeResult` (merge).
const EXPECTED_CLASSES = ["ParsedDataset"];

// Measured 2026-08-16 (post-#342), by THIS script's own `gzipSync(level: 9)` so
// the numbers below are the ones it compares — node's zlib runs ~8 KB above the
// `gzip -9` binary on this artifact, which is exactly the sort of gap that makes
// a hand-copied figure wrong. Each row is that feature alone on top of slim:
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

function fail(msg) {
  console.error(`[wasm-slim] FAIL: ${msg}`);
  process.exit(1);
}

const dts = join(pkg, "ags4_wasm.d.ts");
const wasm = join(pkg, "ags4_wasm_bg.wasm");

let declared;
try {
  declared = readFileSync(dts, "utf8");
} catch {
  fail(`${dts} not found — did the wasm-pack build succeed?`);
}

// wasm-pack emits one `export function <name>(` per export, and `init` /
// `initSync` are its own loader plumbing rather than our API.
const LOADER = new Set(["init", "initSync"]);
const found = (re) => [...declared.matchAll(re)].map((m) => m[1]);
const functions = found(/^export function (\w+)/gm).filter(
  (n) => !LOADER.has(n),
);
const classes = found(/^export class (\w+)/gm);

// Not named `diff`: that is one of the gated surfaces AND a verb this engine
// exports, so a local `diff` here reads as the AGS4 comparison rather than a
// set difference over export names.
const mismatch = (label, actual, expected) => {
  const extra = actual.filter((n) => !expected.includes(n)).sort();
  const missing = expected.filter((n) => !actual.includes(n)).sort();
  if (!extra.length && !missing.length) return null;
  const parts = [];
  // An EXTRA name is the interesting direction: it means a feature that should
  // be off is on, so name the likely culprit rather than just the symbol.
  if (extra.length)
    parts.push(
      `unexpected ${label}: ${extra.join(", ")}\n` +
        `    A gated surface is switched on. Build the slim artifact with\n` +
        `    \`wasm-pack build … -- --no-default-features\` (cargo flags go AFTER the --;\n` +
        `    wasm-pack exits 0 when they land in the wrong place).`,
    );
  if (missing.length)
    parts.push(
      `missing ${label}: ${missing.join(", ")}\n` +
        `    The slim surface lost an export. If that was deliberate, update\n` +
        `    EXPECTED_* in this file and say so in the crate README.`,
    );
  return parts.join("\n  ");
};

const surface = [
  mismatch("exports", functions, EXPECTED_FUNCTIONS),
  mismatch("classes", classes, EXPECTED_CLASSES),
].filter(Boolean);
if (surface.length) fail(surface.join("\n  "));

let bytes;
try {
  bytes = readFileSync(wasm);
} catch {
  fail(`${wasm} not found — did the wasm-pack build succeed?`);
}

const raw = statSync(wasm).size;
const gzip = gzipSync(bytes, { level: 9 }).length;
// A ceiling breach names the axis it was caught on, because the two mean
// different things: gzip says the download got heavier, raw says the artifact
// did — and raw firing alone is the signature of redundant data.
const OVER_CEILING =
  `  The surface check passed, so no feature gate flipped — a heavy dependency has\n` +
  `  landed on an UNGATED path, where no feature can turn it off. Find it with\n` +
  `  \`twiggy top\` on the artifact before raising this number.`;
// Reported, never gated: brotli at max quality is what the artifact COULD
// compress to, not what a CDN serves, so it belongs in the log rather than in
// a threshold.
const brotli = brotliCompressSync(bytes, {
  params: { [constants.BROTLI_PARAM_QUALITY]: 11 },
}).length;
const kib = (n) => `${(n / 1024).toFixed(1)} KiB`;

if (gzip > MAX_GZIP_BYTES) {
  fail(
    `${basename(wasm)} is ${kib(gzip)} gzipped, over the ${kib(MAX_GZIP_BYTES)} ceiling.\n` +
      OVER_CEILING,
  );
}

if (raw > MAX_RAW_BYTES) {
  fail(
    `${basename(wasm)} is ${kib(raw)} raw, over the ${kib(MAX_RAW_BYTES)} ceiling ` +
      `(gzipped it is ${kib(gzip)}, under its own ceiling).\n` +
      `  Raw over budget while gzip is fine means REDUNDANT data — something large and\n` +
      `  repetitive that the compressor eats, so delivery looks healthy and cold-start\n` +
      `  decode does not. #342 is the worked example: the dictionary embedded twice.\n` +
      OVER_CEILING,
  );
}

console.log(
  `[wasm-slim] OK: ${EXPECTED_FUNCTIONS.length} exports + ${EXPECTED_CLASSES.length} class; ` +
    `${kib(raw)} raw (ceiling ${kib(MAX_RAW_BYTES)}), ` +
    `${kib(gzip)} gzip (ceiling ${kib(MAX_GZIP_BYTES)}), ${kib(brotli)} brotli.`,
);
