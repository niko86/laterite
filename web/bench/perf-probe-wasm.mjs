// #893 wasm-door sweep probe (SPIKE BRANCH ONLY — never lands on main).
//
// The write door (`build_ags4_ipc`) carries the same decode-then-materialise
// class as M7's native half: every group's IPC decodes eagerly into a
// whole-file `Vec<ArrowGroup>` in linear memory before the streamed emit
// runs. This probe measures that slab's contribution by variant child:
//
//   base  the shipped `build_ags4_ipc`.
//   jit   `build_ags4_ipc_spike_jit` (this branch): each group copies in
//         and decodes only as the emit consumes it — the slab removed.
//
// The JS-side items array is out of rule 13's claim in both variants (the
// engine's hold only); the read door has NO class member (arrow_ipc frames
// one group transiently — see the #893 record), so it gets no child.
//
// Instrument: rule 13's — linear-memory high-water of a FRESH instantiation
// (`memory.buffer.byteLength` at exit; wasm32 linear memory only grows), the
// wasm lane's own labelled claim, never a peak-RSS column. ×-of-output.
// Deterministic by construction; reps verify byte-identity of the
// instrument itself. A/B/A ordering kept for symmetry with the sibling
// probes. Output text sha256-checked identical across variants per rung.
//
// Run from web/ (built wasm-full: npm run build:wasm-full):
//   node bench/perf-probe-wasm.mjs [--reps 2]
// Writes ../tools/perf-results/wasm-door-sweep.json (repo-relative).
import { execSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { loadavg } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const SELF = fileURLToPath(import.meta.url);
const REPO = resolve(dirname(SELF), "..", "..");
const WASM_DIR = join(REPO, "web", "src", "wasm-full");
const MANIFEST = join(REPO, "output", "perf-ladder", "manifest.json");
const OUT = join(REPO, "tools", "perf-results", "wasm-door-sweep.json");

const MEM_CAP_BYTES = 300_000_000; // epic #820 decision 7 — ladder policy (rule 13)
const VARIANTS = ["base", "jit"];

async function initWasm() {
  const specifier = pathToFileURL(join(WASM_DIR, "ags4_wasm_full.js")).href;
  const glue = await import(specifier);
  const initOut = await glue.default({
    module_or_path: readFileSync(join(WASM_DIR, "ags4_wasm_full_bg.wasm")),
  });
  return { glue, memory: initOut.memory };
}

/** The lane harness's prepareItems verbatim: every group framed to
 * `{code, ipc}` through the read door, the parse handle freed. */
function prepareItems(glue, bytes) {
  const ds = glue.read(bytes);
  const items = ds
    .group_codes()
    .map((code) => ({ code, ipc: ds.arrow_ipc(code) }));
  ds.free();
  return items;
}

async function worker(variant, filePath) {
  const { glue, memory } = await initWasm();
  const data = readFileSync(filePath);
  const items = prepareItems(glue, data);
  const t = process.hrtime.bigint();
  const report =
    variant === "jit"
      ? glue.build_ags4_ipc_spike_jit(items)
      : glue.build_ags4_ipc(items);
  const doorMs = Number(process.hrtime.bigint() - t) / 1e6;
  const out = {
    linear_memory_bytes: memory.buffer.byteLength,
    out_bytes: Buffer.byteLength(report.text, "utf8"),
    sha256: createHash("sha256").update(report.text).digest("hex"),
    door_ms: doorMs,
  };
  process.stdout.write(`${JSON.stringify(out)}\n`);
}

function measureCell(variant, rung, reps) {
  if (rung.bytes > MEM_CAP_BYTES) {
    return { refusal: "beyond-mem-cap", detail: `${rung.bytes} > ${MEM_CAP_BYTES}` };
  }
  const highs = [];
  const doorMs = [];
  let outBytes = null;
  let sha = null;
  for (let i = 0; i < reps; i++) {
    const out = spawnSync(
      process.execPath,
      [SELF, "--worker", variant, "--file", rung.path],
      { encoding: "utf8" },
    );
    if (out.status !== 0) {
      const tail = (out.stderr ?? "").trim().split("\n").slice(-3).join(" | ");
      return { refusal: "failed", detail: tail || `exit ${out.status}` };
    }
    const report = JSON.parse(out.stdout);
    highs.push(report.linear_memory_bytes);
    doorMs.push(report.door_ms);
    outBytes = report.out_bytes;
    sha = report.sha256;
  }
  doorMs.sort((a, b) => a - b);
  const deterministic = highs.every((h) => h === highs[0]);
  return {
    instrument: "wasm-linear-memory",
    peak_linear_memory_bytes: highs[0],
    x_output: Math.round((highs[0] / Math.max(outBytes ?? rung.bytes, 1)) * 100) / 100,
    deterministic,
    highs_bytes: highs,
    door_ms_median: Math.round(doorMs[doorMs.length >> 1] * 10) / 10,
    out_bytes: outBytes,
    out_sha256: sha,
  };
}

async function main() {
  const argv = process.argv.slice(2);
  let workerVariant = null;
  let workerFile = null;
  let reps = 2;
  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === "--worker") workerVariant = argv[++i];
    else if (argv[i] === "--file") workerFile = argv[++i];
    else if (argv[i] === "--reps") reps = Number(argv[++i]);
  }
  if (workerVariant !== null) {
    await worker(workerVariant, workerFile);
    return;
  }

  const manifest = JSON.parse(readFileSync(MANIFEST, "utf8"));
  const rungs = manifest.rungs.filter((r) => r.bytes <= MEM_CAP_BYTES);
  const gitSha = execSync("git rev-parse HEAD", { cwd: REPO, encoding: "utf8" }).trim();
  const loadavgStart = loadavg();

  const passes = [];
  for (const [leg, variant] of [["A1", "base"], ["B", "jit"], ["A2", "base"]]) {
    const cells = {};
    for (const rung of rungs) {
      cells[rung.label] = measureCell(variant, rung, reps);
      const c = cells[rung.label];
      console.log(
        `${leg} ${variant.padEnd(4)} ${rung.label.padStart(6)}: ` +
          (c.refusal
            ? `REFUSAL ${c.refusal}`
            : `${c.x_output}x  det=${c.deterministic}  door=${c.door_ms_median}ms`),
      );
    }
    passes.push({ leg, variant, cells });
  }

  const identity = {};
  const a1 = passes[0].cells;
  for (const p of passes.slice(1)) {
    for (const [label, cell] of Object.entries(p.cells)) {
      if (cell.refusal) continue;
      identity[`${p.leg}-${p.variant}/${label}`] =
        cell.out_sha256 === a1[label].out_sha256;
    }
  }

  const out = {
    schema: "wasm-door-sweep/1",
    issue: 893,
    generated: new Date().toISOString(),
    git_sha: gitSha,
    instrument:
      "wasm-linear-memory (rule 13): fresh instantiation, buffer.byteLength at exit",
    denominator: "x-of-output",
    reps,
    loadavg_start: loadavgStart,
    passes,
    output_byte_identity: identity,
  };
  mkdirSync(dirname(OUT), { recursive: true });
  writeFileSync(OUT, `${JSON.stringify(out, null, 2)}\n`);
  console.log(`written: ${OUT}`);
  if (!Object.values(identity).every(Boolean)) {
    console.error("BYTE-IDENTITY FAILURE — a variant changed the output");
    process.exitCode = 1;
  }
}

await main();
