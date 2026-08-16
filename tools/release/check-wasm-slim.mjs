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
// WHY GZIP AND NOT RAW BYTES
//   Raw size is the wrong axis here, and measurably so. Turning on `diff` adds
//   1.47 MB raw but only 82 KB brotli — it is dictionary-shaped static data
//   that the compressor eats. A raw-byte ceiling would therefore fire on
//   changes nobody downloads and stay quiet on ones they do. Gzip is what a
//   client actually pays (a CDN's brotli lands ~10% under it, so this is a
//   conservative read of delivery, not an inflated one), it is deterministic at
//   a fixed level, and node's zlib gives it to us with no dependency.
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

// 850 KiB gzipped. Measured 2026-08-16 (post-#336), by THIS script's own
// `gzipSync(level: 9)` so the numbers below are the ones it compares — node's
// zlib runs ~8 KB above the `gzip -9` binary on this artifact, which is exactly
// the sort of gap that makes a hand-copied figure wrong:
//
//     slim      757.4 KiB      arrow    1273.0 KiB   (+515.6)
//     certify   771.2 (+13.8)  excel    1286.1 (+528.7)
//     diff      872.9 (+115.5)
//     censor    886.9 (+129.5)
//     merge     892.0 (+134.6)
//
// So the ceiling leaves ~92 KiB (12%) of headroom for honest growth while
// sitting below every heavy gate — `diff` is the tightest at 872.9 and still
// crosses. `certify` (+13.8) does NOT, deliberately: no honest headroom is
// tight enough to catch a 1.8% change. The SURFACE check catches that one
// exactly, which is the division of labour these two checks are for.
const MAX_GZIP_BYTES = 850 * 1024;

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

const diff = (label, actual, expected) => {
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
  diff("exports", functions, EXPECTED_FUNCTIONS),
  diff("classes", classes, EXPECTED_CLASSES),
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
      `  The surface check passed, so no feature gate flipped — a heavy dependency has\n` +
      `  landed on an UNGATED path, where no feature can turn it off. Find it with\n` +
      `  \`twiggy top\` on the artifact before raising this number.`,
  );
}

console.log(
  `[wasm-slim] OK: ${EXPECTED_FUNCTIONS.length} exports + ${EXPECTED_CLASSES.length} class; ` +
    `${kib(raw)} raw, ${kib(gzip)} gzip (ceiling ${kib(MAX_GZIP_BYTES)}), ${kib(brotli)} brotli.`,
);
