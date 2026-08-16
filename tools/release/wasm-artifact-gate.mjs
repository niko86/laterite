///////////////////////////////////////////////////////////////////////////////
// wasm-artifact-gate.mjs — the shared instrument behind the per-artifact wasm
// gates. One artifact per caller: `check-wasm-slim.mjs` (what npm publishes),
// `check-wasm-tier1.mjs` (what the browser precaches, #338).
//
// WHY A SHARED MODULE AND NOT A COPY
//   The two gates differ in exactly three things — which artifact, which
//   exports it promises, and where its ceilings sit. Everything else is the
//   same instrument: read the generated `.d.ts`, compare the surface by name,
//   then weigh the `.wasm` on both axes. Copied, the next fix to the parsing
//   (wasm-pack has changed its `.d.ts` shape before) lands in one file and
//   silently rots the other, and a gate that has quietly stopped measuring is
//   worse than no gate — it reports green.
//
// WHY TWO CHECKS PER ARTIFACT
//   A feature-gated build can stop being the build it claims to be in two ways,
//   and both are SILENT:
//
//     1. A gate gets switched back on — a `default`-features build sneaks into
//        the step, or a feature grows a dependency edge that pulls another one
//        in. Nothing fails; the artifact just gets bigger.
//     2. A heavy dependency lands on an UNGATED path, where no feature can turn
//        it off. This is the failure the tokenizer gate's comment names
//        ("laterite-ags4-types' optional `arrow` feature getting turned on"),
//        and it is why that gate exists at all.
//
//   One instrument cannot see both. The SURFACE check catches (1) exactly — it
//   reads the `.d.ts` and compares the exported names against the caller's
//   list, so a gate flipping in either direction fails BY NAME rather than by
//   byte count, and keeps working however light the gated code gets. The SIZE
//   check catches (2), where the surface is unchanged and only the weight
//   moved.
//
//   The surface is compared down to CLASS MEMBERS, not just top-level exports,
//   because the engine's heaviest gate hides inside one: `arrow` adds
//   `ParsedDataset::arrow_ipc`, a method, and a top-level-only check would see
//   an unchanged `ParsedDataset` and pass.
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
//   as fine. Raw is also the honest cost on the wasm path specifically — the
//   decode-and-compile work on every cold load, and what a service worker's
//   precache budget counts.
//
// USAGE
//   Callers hand `checkWasmArtifact` one profile and get an exit code: 0 = the
//   artifact is the one it claims to be; 1 = it is not, and why, on stderr.
///////////////////////////////////////////////////////////////////////////////

import { readFileSync, statSync } from "node:fs";
import { gzipSync, brotliCompressSync, constants } from "node:zlib";
import { basename, join } from "node:path";

/** The one positional argument every gate takes: the wasm-pack output dir. */
export function pkgDirArg(usage) {
  const pkg = process.argv[2];
  if (!pkg) {
    console.error(`usage: ${usage}`);
    process.exit(1);
  }
  return pkg;
}

// wasm-pack emits `init` / `initSync` as its own loader plumbing rather than as
// our API, and gives every handle class a `free()` + `[Symbol.dispose]()` +
// `private constructor()` from the same generator. None of them says anything
// about which features are on, so none of them belongs in a caller's list.
const LOADER_FUNCTIONS = new Set(["init", "initSync"]);
const GENERATED_MEMBERS = new Set(["free", "constructor"]);

// Doc comments carry prose that looks like code — `` `meta()` ``, `arrow_ipc()`
// — so strip them before matching anything, or a gate can pass on a mention.
const stripComments = (src) => src.replace(/\/\*[\s\S]*?\*\//g, "");

function readSurface(dts) {
  const src = stripComments(dts);
  const functions = [...src.matchAll(/^export function (\w+)/gm)]
    .map((m) => m[1])
    .filter((n) => !LOADER_FUNCTIONS.has(n));

  // Each `export class X { … }` body, closed by a `}` in column 0 — which is
  // how wasm-pack formats them, and the only structure available without a TS
  // parser. Members are `name(`, `readonly name:`, or a getter/setter.
  const classes = {};
  for (const m of src.matchAll(/^export class (\w+) \{\n([\s\S]*?)^\}/gm)) {
    classes[m[1]] = [
      ...m[2].matchAll(
        /^\s+(?:private\s+|static\s+|readonly\s+|get\s+|set\s+)*([A-Za-z_$][\w$]*)\s*[(:]/gm,
      ),
    ]
      .map((mm) => mm[1])
      .filter((n) => !GENERATED_MEMBERS.has(n));
  }
  return { functions, classes };
}

// Not named `diff`: that is one of the gated surfaces AND a verb this engine
// exports, so a local `diff` here reads as the AGS4 comparison rather than a
// set difference over export names.
//
// Reports the two directions separately because they mean opposite things — an
// EXTRA name is a feature that should be off being on, a MISSING one is the
// artifact losing something it promised — and the advice for each is written
// once by the caller, after every mismatch has had its say.
function mismatch(label, actual, expected) {
  const extra = actual.filter((n) => !expected.includes(n)).sort();
  const missing = expected.filter((n) => !actual.includes(n)).sort();
  return {
    extra: extra.length ? `unexpected ${label}: ${extra.join(", ")}` : null,
    missing: missing.length ? `missing ${label}: ${missing.join(", ")}` : null,
  };
}

const kib = (n) => `${(n / 1024).toFixed(1)} KiB`;

/**
 * Hold one wasm-pack output to the surface and the weight it promises.
 *
 * @param {object} profile
 * @param {string} profile.label     log prefix, e.g. `wasm-tier1`
 * @param {string} profile.pkg       the wasm-pack `--out-dir`
 * @param {string[]} profile.functions  every top-level export, in full
 * @param {Record<string, string[]>} profile.classes  each handle class and its
 *   members, in full — the members are where `arrow` hides
 * @param {number} profile.maxGzipBytes
 * @param {number} profile.maxRawBytes
 * @param {string} profile.rebuild   the command that produces this artifact,
 *   quoted back to whoever tripped the surface check
 */
export function checkWasmArtifact({
  label,
  pkg,
  functions: expectedFunctions,
  classes: expectedClasses,
  maxGzipBytes,
  maxRawBytes,
  rebuild,
}) {
  const fail = (msg) => {
    console.error(`[${label}] FAIL: ${msg}`);
    process.exit(1);
  };

  const dtsPath = join(pkg, "ags4_wasm.d.ts");
  const wasmPath = join(pkg, "ags4_wasm_bg.wasm");

  let dts;
  try {
    dts = readFileSync(dtsPath, "utf8");
  } catch {
    fail(`${dtsPath} not found — did the wasm-pack build succeed?`);
  }

  const found = readSurface(dts);
  const problems = [
    mismatch("exports", found.functions, expectedFunctions),
    mismatch(
      "classes",
      Object.keys(found.classes),
      Object.keys(expectedClasses),
    ),
  ];
  // Members only for the classes BOTH sides agree exist — a class that is
  // wholly unexpected has already been reported by name, and listing its
  // members again says nothing new.
  for (const [name, members] of Object.entries(expectedClasses)) {
    if (found.classes[name])
      problems.push(mismatch(`${name} members`, found.classes[name], members));
  }

  const surface = problems.flatMap((p) => [p.extra, p.missing]).filter(Boolean);
  if (surface.length) {
    if (problems.some((p) => p.extra))
      surface.push(
        `A gated surface is switched on. Build this artifact with\n  ${rebuild}`,
      );
    if (problems.some((p) => p.missing))
      surface.push(
        `This artifact lost part of its surface. If that was deliberate, update\n` +
          `  EXPECTED_* in the gate that names it, and say so where the surface is\n` +
          `  documented.`,
      );
    fail(surface.join("\n  "));
  }

  let bytes;
  try {
    bytes = readFileSync(wasmPath);
  } catch {
    fail(`${wasmPath} not found — did the wasm-pack build succeed?`);
  }

  const raw = statSync(wasmPath).size;
  const gzip = gzipSync(bytes, { level: 9 }).length;
  // A ceiling breach names the axis it was caught on, because the two mean
  // different things: gzip says the download got heavier, raw says the artifact
  // did — and raw firing alone is the signature of redundant data.
  const OVER_CEILING =
    `  The surface check passed, so no feature gate flipped — a heavy dependency has\n` +
    `  landed on an UNGATED path, where no feature can turn it off. Find it with\n` +
    `  \`twiggy top\` on the artifact before raising this number.`;

  if (gzip > maxGzipBytes) {
    fail(
      `${basename(wasmPath)} is ${kib(gzip)} gzipped, over the ${kib(maxGzipBytes)} ceiling.\n` +
        OVER_CEILING,
    );
  }

  if (raw > maxRawBytes) {
    fail(
      `${basename(wasmPath)} is ${kib(raw)} raw, over the ${kib(maxRawBytes)} ceiling ` +
        `(gzipped it is ${kib(gzip)}, under its own ceiling).\n` +
        `  Raw over budget while gzip is fine means REDUNDANT data — something large and\n` +
        `  repetitive that the compressor eats, so delivery looks healthy and cold-start\n` +
        `  decode does not. #342 is the worked example: the dictionary embedded twice.\n` +
        OVER_CEILING,
    );
  }

  // Reported, never gated: brotli at max quality is what the artifact COULD
  // compress to, not what a CDN serves, so it belongs in the log rather than in
  // a threshold.
  const brotli = brotliCompressSync(bytes, {
    params: { [constants.BROTLI_PARAM_QUALITY]: 11 },
  }).length;

  console.log(
    `[${label}] OK: ${expectedFunctions.length} exports + ` +
      `${Object.keys(expectedClasses).length} class(es); ` +
      `${kib(raw)} raw (ceiling ${kib(maxRawBytes)}), ` +
      `${kib(gzip)} gzip (ceiling ${kib(maxGzipBytes)}), ${kib(brotli)} brotli.`,
  );
}
