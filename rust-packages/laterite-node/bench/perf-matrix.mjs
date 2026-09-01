// The Node leg of the cross-surface performance matrix (#823) — the sibling of
// `laterite-ags4-perf` (rust), emitting the same uniform per-surface schema
// (schema 2: `{surface, results:[{op, rung, bytes, median_ms, throughput_mb_s,
// mem?}], skipped:[{rung, reason}]}`) into `output/perf-results/node.json` for
// `tools/perf-matrix.py` to merge. Where `bench/read.bench.ts` is the
// regression guard (vitest, one rung, read only), this is the comparison lane:
// the campaign's three ops over the whole forge ladder
// (`tools/perf-ladder.py` → `output/perf-ladder/manifest.json`).
//
// The ops go through the PUBLIC surface — what a Node caller actually pays,
// napi marshalling and arrow-js decode included, not the native engine alone:
//   validate        `validate(path)` (the engine reads the path itself; the OS
//                   page cache makes the repeated read negligible).
//   parse-to-typed  `read(bytes)` + `table(code)` for every group — bytes read
//                   once OUTSIDE the timed loop, keys-less default tables.
//   write           `buildAgs4([[code, Table], …])`, the input prepared outside
//                   the timed loop, so the timed cost is the write door's:
//                   arrow-js IPC serialise + the native emit engine. Unlike the
//                   rust leg this door dictionary-fills UNIT/TYPE rather than
//                   carrying the source's — that IS the surface's write path.
//
// `mem` is the campaign's peak-RSS instrument (epic #820 decision 1): each
// (op, rung) cell is one FRESH child process (`--mem-worker`, this same
// script) running the operation once end-to-end and reporting its own peak RSS
// at exit. Same 265 MB cap and refusal semantics as the rust/python harnesses
// (`beyond-mem-cap` / `swapped` / `failed`) — recorded, never silently skipped.
// The whole memory pass runs BEFORE the timed loops, while this parent is
// still small — the reordering that keeps the swap watch honest (see main).
// A write cell's peak includes reading and typing the input — you cannot write
// what you do not hold — so attribute it against the same rung's
// parse-to-typed cell.
//
// Run (needs the built package — `npm run build` — and the ladder manifest):
//   npm run bench:matrix          # or: node bench/perf-matrix.mjs [args]
//   args: [--manifest <p>] [--out <p>] [--iters N] [--skip-mem]
//
// Pure seams are exported for `test/perf-matrix-harness.test.ts`; the shipped
// surface is imported lazily so the unit tests need no native addon.
import { spawnSync } from "node:child_process";
import { mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SELF = fileURLToPath(import.meta.url);
const REPO = resolve(dirname(SELF), "..", "..", "..");

/** Epic #820 decision 7: memory columns stop at the 265 MB rung — a run that
 * pushes the machine into swap measures the pager, not the library. The same
 * value as the rust bin's `MEM_CAP_BYTES` and the python lane's, so the three
 * harnesses admit and refuse the same pinned rungs. */
export const MEM_CAP_BYTES = 300_000_000;

/** Swap growth past this during a child's run marks the cell `swapped` —
 * small enough to catch a real spill, large enough that unrelated background
 * paging does not veto a clean run (the rust/python harnesses' threshold). */
export const SWAP_REFUSAL_BYTES = 64 * 1024 * 1024;

/** libuv normalises `ru_maxrss` to kibibytes on EVERY platform (darwin's raw
 * bytes are divided down inside uv_getrusage), so unlike the rust bin there is
 * no OS branch here — and a slip would move every cell 1024×.
 * @param {number} maxRssKb @returns {number} */
export function maxRssToBytes(maxRssKb) {
  return maxRssKb * 1024;
}

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

/** One measured mem cell. `x_output` is the campaign's headline unit — peak as
 * a multiple of the operation's output (or input) size, comparable across
 * rungs where raw MB is not.
 * @param {number} peakBytes @param {number} denomBytes
 * @returns {{peak_rss_bytes: number, x_output: number}} */
export function memCell(peakBytes, denomBytes) {
  return {
    peak_rss_bytes: peakBytes,
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

/** The `used = 512.50M` field of darwin's `vm.swapusage` sysctl, in bytes.
 * @param {string} text @returns {number | null} */
export function parseSwapUsedDarwin(text) {
  const m = /used\s*=\s*(\d+(?:\.\d+)?)([KMG])/.exec(text);
  if (!m) return null;
  const scale = { K: 1024, M: 1024 * 1024, G: 1024 * 1024 * 1024 }[m[2]];
  return Math.trunc(Number(m[1]) * scale);
}

/** `SwapTotal - SwapFree` from `/proc/meminfo` text (kB fields), in bytes.
 * @param {string} text @returns {number | null} */
export function parseMeminfoSwap(text) {
  const field = (name) => {
    const line = text.split("\n").find((l) => l.startsWith(name));
    if (!line) return null;
    const v = Number(line.split(/\s+/)[1]);
    return Number.isFinite(v) ? v : null;
  };
  const total = field("SwapTotal:");
  const free = field("SwapFree:");
  if (total === null || free === null) return null;
  return Math.max(0, total - free) * 1024;
}

/** The matrix's uniform per-surface result document. `skipped` serialises even
 * when empty — a positive statement that nothing was dropped, because a filter
 * nobody can see is a blind spot.
 * @param {number} iters @param {object[]} results @param {object[]} skipped */
export function buildOutput(iters, results, skipped) {
  return {
    schema: 2,
    surface: "node",
    tool: "laterite-node/bench/perf-matrix.mjs",
    iters,
    results,
    skipped,
  };
}

/** Current swap in use, or null where no instrument exists. Read before and
 * after each child: growth means the child's number includes the pager. */
function swapUsedBytes() {
  if (process.platform === "darwin") {
    const out = spawnSync("sysctl", ["-n", "vm.swapusage"], {
      encoding: "utf8",
    });
    if (out.status !== 0 || typeof out.stdout !== "string") return null;
    return parseSwapUsedDarwin(out.stdout);
  }
  if (process.platform === "linux") {
    try {
      return parseMeminfoSwap(readFileSync("/proc/meminfo", "utf8"));
    } catch {
      return null;
    }
  }
  return null;
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
 * measure the same work by construction: parse + keys-less default `table()`
 * for every group (the arrow-js decode included — it is what a caller pays). */
function typeAllGroups(api, bytes) {
  const f = api.read(bytes);
  for (const code of f.groups) f.table(code);
}

/** The write axis's held input: every group materialised to an arrow-js Table
 * through the read door — built once, outside the timed loop. */
function prepareItems(api, bytes) {
  const f = api.read(bytes);
  return f.groups.map((code) => [code, f.table(code)]);
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

/** The `--mem-worker` child: one operation, once, end-to-end; report this
 * process's own peak RSS on stdout (no library on this path prints, so stdout
 * is a safe channel). A throw exits non-zero and becomes the parent's `failed`
 * refusal cell. */
async function memWorker(op, filePath) {
  const api = await import("../dist/index.mjs");
  let outBytes = null;
  if (op === "validate") {
    api.validate(filePath);
  } else if (op === "parse-to-typed") {
    typeAllGroups(api, readFileSync(filePath));
  } else if (op === "write") {
    // Read + type + emit: you cannot write what you do not hold, so the write
    // cell's peak includes the input materialisation — attribute it against
    // the same rung's parse-to-typed cell.
    const items = prepareItems(api, readFileSync(filePath));
    outBytes = api.buildAgs4(items).bytes.length;
  } else {
    throw new Error(`unknown mem-worker op: ${op}`);
  }
  const report = {
    maxrss_bytes: maxRssToBytes(process.resourceUsage().maxRSS),
    out_bytes: outBytes,
  };
  process.stdout.write(`${JSON.stringify(report)}\n`);
}

/** One (op, rung) memory cell: fresh child (this same script), swap watched
 * across the run. Every veto is a recorded refusal, never a silent skip. */
function measureMem(op, filePath, inputBytes) {
  if (!memRungAllowed(inputBytes)) {
    return refusalCell(
      "beyond-mem-cap",
      `${inputBytes}-byte rung is past the ${MEM_CAP_BYTES}-byte cap ` +
        "(epic #820 decision 7: a swapping run measures the pager)",
    );
  }
  const swapBefore = swapUsedBytes();
  const out = spawnSync(
    process.execPath,
    [SELF, "--mem-worker", op, "--mem-file", filePath],
    { encoding: "utf8" },
  );
  const swapAfter = swapUsedBytes();
  if (out.error) return refusalCell("failed", `spawn: ${out.error.message}`);
  if (out.status !== 0) {
    const tail = (out.stderr ?? "").trim().split("\n").slice(-3).join(" | ");
    return refusalCell("failed", tail || `exit ${out.status}`);
  }
  if (swapBefore !== null && swapAfter !== null) {
    const grew = swapAfter - swapBefore;
    if (grew > SWAP_REFUSAL_BYTES) {
      return refusalCell(
        "swapped",
        `swap grew ${(grew / 1e6).toFixed(1)} MB during the run`,
      );
    }
  }
  try {
    const report = JSON.parse(out.stdout);
    return memCell(
      report.maxrss_bytes,
      Math.max(report.out_bytes ?? inputBytes, 1),
    );
  } catch (e) {
    return refusalCell("failed", `unreadable worker report: ${e.message}`);
  }
}

async function main() {
  let manifestPath = join(REPO, "output", "perf-ladder", "manifest.json");
  let outPath = join(REPO, "output", "perf-results", "node.json");
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

  const api = await import("../dist/index.mjs");
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
  // manifest. The rust bin interleaves and gets away with it because its
  // parent genuinely frees its holds before spawning; a V8 parent that has
  // just run the timed loops keeps gigabytes resident (nulled refs do not
  // return pages), and at the top rung that footprint squeezed the children
  // into swap — the refusal then records the harness, not the library.
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

  const results = [];
  for (const rung of rungs) {
    const bytes = rung.bytes;
    // A Buffer, not a string: V8 caps a string at ~512 MB, the byte door does
    // not — and bytes are the door large callers are told to use.
    const data = readFileSync(rung.path);
    console.error(
      `perf-matrix.mjs: ${rung.label} (${bytes} bytes) × ${iters} iters`,
    );

    const validate = measurement(
      "validate",
      rung.label,
      bytes,
      medianMs(1, iters, () => api.validate(rung.path)),
    );
    const typed = measurement(
      "parse-to-typed",
      rung.label,
      bytes,
      medianMs(2, iters, () => typeAllGroups(api, data)),
    );
    const items = prepareItems(api, data);
    const write = measurement(
      "write",
      rung.label,
      bytes,
      medianMs(1, iters, () => api.buildAgs4(items)),
    );
    const cells = memCells.get(rung.label);
    if (cells) {
      for (const m of [validate, typed, write]) {
        m.mem = cells[m.op];
      }
    }
    results.push(validate, typed, write);
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
if (process.argv[1] && SELF === resolve(process.argv[1])) {
  await main();
}
