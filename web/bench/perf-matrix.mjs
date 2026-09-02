// The wasm leg of the cross-surface performance matrix (#824) — the sibling
// of `laterite-ags4-perf` (rust) and `laterite-node/bench/perf-matrix.mjs`
// (node), emitting the same uniform per-surface schema (schema 2:
// `{surface, results:[{op, rung, bytes, median_ms, throughput_mb_s, mem?}],
// skipped:[{rung, reason}]}`) into `output/perf-results/wasm.json` for
// `tools/perf-matrix.py` to merge. Where `bench/wasm-read.bench.ts` is the
// regression guard (vitest, one rung, read only), this is the comparison
// lane: the campaign's three ops over the whole forge ladder
// (`tools/perf-ladder.py` → `output/perf-ladder/manifest.json`), driving the
// SAME browser cdylib the app loads (the wasm-full build — `arrow_ipc` and
// `build_ags4_ipc` are the `arrow` feature, absent from the tier-1 artifact).
//
// The ops go through the PUBLIC wasm surface, default call shapes — the
// JS→wasm boundary copy of the input stays in the measured path, because a
// browser pays it for real:
//   validate        `validate(bytes)` (surface defaults: warnings on, FYI
//                   off, utf-8 — each lane measures its own default gate).
//   parse-to-typed  `read(bytes)` + `arrow_ipc(code)` for every group,
//                   keys-less default. The op ends at the Arrow IPC bytes —
//                   the surface's contract; decoding them is the host app's
//                   business (duckdb-wasm or arrow-js), where node's twin op
//                   includes its own arrow-js decode.
//   write           `build_ags4_ipc(groups)` (default autofix mode), the
//                   `{code, ipc}` input prepared outside the timed loop
//                   through the read door, ParsedDataset freed first — a
//                   browser caller builds from held IPC (e.g. a duckdb-wasm
//                   result), not from a live parse handle.
//
// THE MEMORY INSTRUMENT (the #824 decision — recorded in the perf-campaign
// protocol, rule 13): `mem` is the **high-water of the module's linear
// memory** — `WebAssembly.Memory.buffer.byteLength` read after one
// end-to-end op in a FRESH child process (fresh instantiation per cell:
// wasm32 linear memory only ever grows, so the size at exit IS the peak).
// This is a DIFFERENT CLAIM from the other surfaces' fresh-child peak RSS
// and is labelled per cell (`instrument: "wasm-linear-memory"`, key
// `peak_linear_memory_bytes`) so no reader or merger can fold the two into
// one column — the same two-claims rule as RSS vs dhat. What it buys:
// deterministic (byte-identical run to run, engine-independent — the number
// a browser's wasm heap would show, not this harness's V8). What it gives
// up: JS-side holds (the input bytes, the returned IPC/text) are out of
// claim; they belong to the host, not the engine. The rejected candidate —
// the JS heap's own measurement APIs — is engine-dependent, GC-timing
// non-deterministic, and in node does not even count wasm linear memory
// (an external ArrayBuffer), so the "broader" instrument would measure
// less of the engine, less reproducibly.
//
// Refusal semantics are the campaign's (`beyond-mem-cap` / `failed`,
// recorded, never silently skipped) with the same 265 MB cap constant as
// the rust/node/python harnesses — the cap is rung policy (epic #820
// decision 7), kept here so every lane admits and refuses the same pinned
// rungs. There is NO `swapped` refusal in this lane: host paging cannot
// move a linear-memory byte count, so a swap watch would veto nothing the
// instrument can see. The whole memory pass still runs BEFORE the timed
// loops, while this parent is small — the #823 GC'd-host ordering lesson.
// A write cell's high-water includes parsing and framing the input — you
// cannot write what you do not hold — so attribute it against the same
// rung's parse-to-typed cell.
//
// Run (needs the built wasm — `npm run build:wasm-full` — and the ladder
// manifest):
//   npm run bench:matrix          # or: node bench/perf-matrix.mjs [args]
//   args: [--manifest <p>] [--out <p>] [--iters N] [--skip-mem]
//
// Pure seams are exported for `bench/perf-matrix-harness.test.mjs`; the
// wasm glue is imported lazily so the unit tests need no built artifact.
import { spawnSync } from "node:child_process";
import {
  mkdirSync,
  readFileSync,
  realpathSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const SELF = fileURLToPath(import.meta.url);
const REPO = resolve(dirname(SELF), "..", "..");
const WASM_DIR = join(REPO, "web", "src", "wasm-full");

/** Epic #820 decision 7: memory columns stop at the 265 MB rung. The same
 * value as the rust/node/python harnesses' `MEM_CAP_BYTES`, so the four
 * lanes admit and refuse the same pinned rungs — a policy constant here,
 * not a swap guard (this lane's instrument cannot swap — see header). */
export const MEM_CAP_BYTES = 300_000_000;

/** The epic-#820 cap: memory measurement stops at the 265 MB rung.
 * @param {number} rungBytes @returns {boolean} */
export function memRungAllowed(rungBytes) {
  return rungBytes <= MEM_CAP_BYTES;
}

/** Upper-middle sample of the sorted values (the rust bin's len/2 pick).
 * @param {number[]} samples @returns {number} */
export function median(samples) {
  const s = [...samples].sort((a, b) => a - b);
  return s[s.length >> 1];
}

/** Decimal MB/s (MB = 1e6 bytes, matching forge's parse_size): the
 * cross-surface throughput headline. Guards the degenerate timing.
 * @param {number} bytes @param {number} medianMs @returns {number} */
export function throughputMbS(bytes, medianMs) {
  if (medianMs <= 0) return 0;
  return bytes / (medianMs * 1000);
}

/** One measured mem cell, labelled by instrument (the #824 two-claims rule:
 * linear-memory high-water never shares a key or a column with peak RSS).
 * `x_output` is the campaign's headline unit — peak as a multiple of the
 * operation's output (or input) size, comparable across rungs.
 * @param {number} peakBytes @param {number} denomBytes
 * @returns {{instrument: string, peak_linear_memory_bytes: number, x_output: number}} */
export function memCell(peakBytes, denomBytes) {
  return {
    instrument: "wasm-linear-memory",
    peak_linear_memory_bytes: peakBytes,
    x_output: Math.round((peakBytes / denomBytes) * 100) / 100,
  };
}

/** A recorded refusal — shape-distinguishable from a measurement on purpose,
 * so no reader (human or script) can mistake a vetoed run for a small number.
 * @param {string} reason @param {string} detail
 * @returns {{refusal: string, detail: string}} */
export function refusalCell(reason, detail) {
  return { refusal: reason, detail };
}

/** The matrix's uniform per-surface result document. `skipped` serialises even
 * when empty — a positive statement that nothing was dropped, because a filter
 * nobody can see is a blind spot.
 * @param {number} iters @param {object[]} results @param {object[]} skipped */
export function buildOutput(iters, results, skipped) {
  return {
    schema: 2,
    surface: "wasm",
    tool: "web/bench/perf-matrix.mjs",
    iters,
    results,
    skipped,
  };
}

/** Instantiate a fresh wasm module from the built artifact's bytes (the
 * pattern the read bench uses — no fetch, no bundler) and hand back the glue
 * plus the live `WebAssembly.Memory` the instrument reads. */
async function initWasm() {
  const specifier = pathToFileURL(join(WASM_DIR, "ags4_wasm_full.js")).href;
  const glue = await import(specifier);
  const initOut = await glue.default({
    module_or_path: readFileSync(join(WASM_DIR, "ags4_wasm_full_bg.wasm")),
  });
  return { glue, memory: initOut.memory };
}

/** Warm up untimed runs, then the median wall time (ms) over the timed ones. */
function medianMs(warmup, iters, f) {
  for (let i = 0; i < warmup; i++) f();
  const samples = [];
  for (let i = 0; i < iters; i++) {
    const t = process.hrtime.bigint();
    f();
    samples.push(Number(process.hrtime.bigint() - t) / 1e6);
  }
  return median(samples);
}

/** parse-to-typed, defined once so the timed loop and the memory worker
 * measure the same work by construction: parse + keys-less default
 * `arrow_ipc()` for every group. `free()` returns the wasm-side parse so a
 * tight loop doesn't ratchet linear memory past one iteration's peak. */
function typeAllGroups(glue, bytes) {
  const ds = glue.read(bytes);
  for (const code of ds.group_codes()) ds.arrow_ipc(code);
  ds.free();
}

/** The write axis's held input: every group framed to `{code, ipc}` through
 * the read door — built once, outside the timed loop, the parse handle freed
 * before the door runs (see header: a browser caller builds from held IPC,
 * not from a live parse). */
function prepareItems(glue, bytes) {
  const ds = glue.read(bytes);
  const items = ds
    .group_codes()
    .map((code) => ({ code, ipc: ds.arrow_ipc(code) }));
  ds.free();
  return items;
}

function measurement(op, rung, bytes, ms) {
  return {
    op,
    rung,
    bytes,
    median_ms: ms,
    throughput_mb_s: throughputMbS(bytes, ms),
  };
}

/** The `--mem-worker` child: fresh module, one operation, once, end-to-end;
 * report the linear-memory high-water on stdout (no library on this path
 * prints, so stdout is a safe channel). A throw exits non-zero and becomes
 * the parent's `failed` refusal cell. */
async function memWorker(op, filePath) {
  const { glue, memory } = await initWasm();
  const data = readFileSync(filePath);
  let outBytes = null;
  if (op === "validate") {
    glue.validate(data);
  } else if (op === "parse-to-typed") {
    typeAllGroups(glue, data);
  } else if (op === "write") {
    // Parse + frame + emit: you cannot write what you do not hold, so the
    // write cell's high-water includes the input framing — attribute it
    // against the same rung's parse-to-typed cell.
    const items = prepareItems(glue, data);
    const report = glue.build_ags4_ipc(items);
    outBytes = Buffer.byteLength(report.text, "utf8");
  } else {
    throw new Error(`unknown mem-worker op: ${op}`);
  }
  // Linear memory never shrinks, so the size now IS the peak — including the
  // module's own baseline pages, exactly as an RSS cell includes its
  // interpreter floor: the 5 MB rung prices it, rank from the big rungs.
  const report = {
    linear_memory_bytes: memory.buffer.byteLength,
    out_bytes: outBytes,
  };
  process.stdout.write(`${JSON.stringify(report)}\n`);
}

/** One (op, rung) memory cell: fresh child (this same script). Every veto is
 * a recorded refusal, never a silent skip. No swap watch — see header. */
function measureMem(op, filePath, inputBytes) {
  if (!memRungAllowed(inputBytes)) {
    return refusalCell(
      "beyond-mem-cap",
      `${inputBytes}-byte rung is past the ${MEM_CAP_BYTES}-byte cap ` +
        "(epic #820 decision 7: the campaign's memory columns stop there)",
    );
  }
  const out = spawnSync(
    process.execPath,
    [SELF, "--mem-worker", op, "--mem-file", filePath],
    { encoding: "utf8" },
  );
  if (out.error) return refusalCell("failed", `spawn: ${out.error.message}`);
  if (out.status !== 0) {
    // A signal kill leaves status null and out.error unset, and a crash
    // dump's last lines are stack frames + the Node banner — so name the
    // signal, and prefer the first Error-bearing stderr line (the message)
    // over a blind tail. This detail is the only record the artifact keeps.
    const stderr = (out.stderr ?? "").trim();
    const errLine = stderr.split("\n").find((l) => l.includes("Error"));
    const cause = out.signal ? `killed by ${out.signal}` : `exit ${out.status}`;
    const hint = errLine?.trim() ?? stderr.split("\n").at(-1)?.trim();
    return refusalCell("failed", hint ? `${cause}: ${hint}` : cause);
  }
  try {
    const report = JSON.parse(out.stdout);
    return memCell(
      report.linear_memory_bytes,
      // `||` not `??`: a zero-byte output must fall back to the input size,
      // never become a 1-byte denominator inflating x_output by ~10^8.
      Math.max(report.out_bytes || inputBytes, 1),
    );
  } catch (e) {
    return refusalCell("failed", `unreadable worker report: ${e.message}`);
  }
}

async function main() {
  let manifestPath = join(REPO, "output", "perf-ladder", "manifest.json");
  let outPath = join(REPO, "output", "perf-results", "wasm.json");
  let iters = 10;
  let skipMem = false;
  let memWorkerOp = null;
  let memFile = null;

  const args = process.argv.slice(2);
  for (let i = 0; i < args.length; i++) {
    const a = args[i];
    const next = () => {
      i += 1;
      if (i >= args.length) throw new Error(`${a} needs a value`);
      return args[i];
    };
    if (a === "--manifest") manifestPath = resolve(next());
    else if (a === "--out") outPath = resolve(next());
    else if (a === "--iters") {
      // Loud like the rust bin's `.parse()?`: NaN or 0 would otherwise time
      // zero samples and silently DROP the median_ms key from the artifact.
      iters = Number.parseInt(next(), 10);
      if (!Number.isInteger(iters) || iters < 1)
        throw new Error("--iters needs a positive integer");
    } else if (a === "--skip-mem") skipMem = true;
    else if (a === "--mem-worker") memWorkerOp = next();
    else if (a === "--mem-file") memFile = next();
    else if (a === "-h" || a === "--help") {
      console.error(
        "usage: node bench/perf-matrix.mjs [--manifest <p>] [--out <p>] [--iters N] [--skip-mem]",
      );
      return;
    } else throw new Error(`unknown arg: ${a}`);
  }

  if (memWorkerOp !== null) {
    if (memFile === null) throw new Error("--mem-worker needs --mem-file");
    await memWorker(memWorkerOp, memFile);
    return;
  }

  let manifest;
  try {
    manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
  } catch (e) {
    throw new Error(
      `read ladder manifest ${manifestPath}: ${e.message} — run ` +
        "`uv run python tools/perf-ladder.py` first",
    );
  }

  // Resolve the ladder once: a rung missing on disk is recorded, not silently
  // dropped, and both passes below walk the same surviving list.
  const rungs = [];
  const skipped = [];
  for (const rung of manifest.rungs) {
    try {
      rungs.push({ ...rung, bytes: statSync(rung.path).size });
    } catch {
      console.error(
        `perf-matrix.mjs: rung ${rung.label} missing (${rung.path}) — ` +
          "skipping (re-run `uv run python tools/perf-ladder.py`)",
      );
      skipped.push({
        rung: rung.label,
        reason: `missing on disk: ${rung.path}`,
      });
    }
  }

  // The memory pass runs FIRST, while this parent holds nothing beyond the
  // manifest — the #823 ordering lesson for GC'd-host parents. The cells
  // themselves are fresh children and swap-proof, but the timed loops that
  // follow are not, and a bloated parent is exactly what squeezed the node
  // lane's top rung into the pager.
  const memCells = new Map();
  if (!skipMem) {
    for (const rung of rungs) {
      console.error(`perf-matrix.mjs: ${rung.label} memory children`);
      const cells = {};
      for (const op of ["validate", "parse-to-typed", "write"]) {
        cells[op] = measureMem(op, rung.path, rung.bytes);
      }
      memCells.set(rung.label, cells);
    }
  }

  const { glue } = await initWasm();
  const results = [];
  for (const rung of rungs) {
    const bytes = rung.bytes;
    const data = readFileSync(rung.path);
    console.error(
      `perf-matrix.mjs: ${rung.label} (${bytes} bytes) × ${iters} iters`,
    );

    // The timed pass shares ONE instance across the ladder (an ESM module is
    // cached by specifier, so a "fresh" re-import returns the same singleton)
    // and wasm32 linear memory only ever grows — so a big-enough rung can
    // ratchet the instance into a failed `memory.grow`, which throws inside
    // the wasm call. The default ladder fits comfortably; a deliberate
    // `--rungs …,524MB` run may not. A thrown rung is RECORDED in `skipped`
    // and the walk continues, so one rung past the surface's ceiling cannot
    // destroy every measurement already taken — the artifact is written once,
    // at the end, and a crash here would lose it all.
    const rungResults = [];
    try {
      rungResults.push(
        measurement(
          "validate",
          rung.label,
          bytes,
          medianMs(1, iters, () => glue.validate(data)),
        ),
      );
      rungResults.push(
        measurement(
          "parse-to-typed",
          rung.label,
          bytes,
          medianMs(2, iters, () => typeAllGroups(glue, data)),
        ),
      );
      const items = prepareItems(glue, data);
      rungResults.push(
        measurement(
          "write",
          rung.label,
          bytes,
          medianMs(1, iters, () => glue.build_ags4_ipc(items)),
        ),
      );
    } catch (e) {
      console.error(`perf-matrix.mjs: ${rung.label} timed pass failed: ${e}`);
      skipped.push({
        rung: rung.label,
        reason:
          `timed pass failed after ${rungResults.length} op(s) ` +
          `(completed ops kept): ${e.message}`,
      });
    }
    const cells = memCells.get(rung.label);
    if (cells) {
      for (const m of rungResults) {
        m.mem = cells[m.op];
      }
    }
    results.push(...rungResults);
  }

  const output = buildOutput(iters, results, skipped);
  mkdirSync(dirname(outPath), { recursive: true });
  writeFileSync(outPath, `${JSON.stringify(output, null, 2)}\n`);
  console.error(
    `perf-matrix.mjs: wrote ${results.length} measurements ` +
      `(${skipped.length} rung(s) skipped) → ${outPath}`,
  );
}

// CLI entry — the pure exports above stay importable without running it.
// Both sides realpath'd: node resolves the ESM main module to its real path
// (SELF is already dereferenced), so a raw argv[1] reached through a symlink
// would mismatch and turn the run into a clean-looking no-op — the worst
// failure shape for a harness.
function isCliEntry(argv1) {
  if (!argv1) return false;
  try {
    return SELF === realpathSync(resolve(argv1));
  } catch {
    return false;
  }
}
if (isCliEntry(process.argv[1])) {
  await main();
}
