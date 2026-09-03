// #893 M7 diagnosis probe (SPIKE BRANCH ONLY — never lands on main).
//
// Variant children of the node write door (`buildAgs4`), alone and paired,
// per the #848 co-peak rule: the door's two whole-file holds — the JS-side
// `ipcGroups` accumulate-before-call slab and the native decode-then-
// materialise `Vec<ArrowGroup>` — are removed one at a time and then both,
// and only the measured deltas price anything.
//
//   base     the shipped door: marshal ALL groups' IPC, one native call
//            decoding ALL groups up front, then the streamed emit.
//   jit      LATERITE_M7_SPIKE=jit (native env gate): same one call + JS
//            slab, but each group's IPC decodes only as the emit consumes
//            it — the decoded slab removed ALONE.
//   handoff  per-group `EmitSessionSpike("eager")`: each group marshals,
//            crosses, decodes, and its JS Buffer goes out of scope — the JS
//            slab removed ALONE (the decoded slab still accumulates).
//   stream   per-group `EmitSessionSpike("stream")`: each group marshals,
//            crosses, decodes AND writes through the emit stream at once —
//            both holds removed (the pair).
//
// Every child holds the caller's own input (`items`, the arrow-js Tables)
// across the door exactly as the lane's write child does — the deltas
// isolate the door's slabs, not the caller's hold. Two reference children
// ride along for the increment arithmetic: `typed` (the lane's
// parse-to-typed op) and `held` (prepareItems only — what the write child
// holds before the door runs).
//
// Instrument: fresh-child peak RSS (this process's own maxRSS at exit) —
// the diagnosis family (epic #820 rule 8); one machine, never a
// cross-library table. Denominator: ×-of-output (the write row's), input
// bytes for typed/held. A/B/A: base passes bracket the variants in one
// sitting. Output bytes sha256-checked identical across all four door
// variants per rung.
//
// Run from rust-packages/laterite-node (quiet machine, built package):
//   node bench/perf-probe-m7.mjs [--reps 3]
// Writes ../../tools/perf-results/m7-diagnosis.json (repo-relative).
import { spawnSync, execSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { createRequire } from "node:module";
import { loadavg } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SELF = fileURLToPath(import.meta.url);
const REPO = resolve(dirname(SELF), "..", "..", "..");
const MANIFEST = join(REPO, "output", "perf-ladder", "manifest.json");
const OUT = join(REPO, "tools", "perf-results", "m7-diagnosis.json");

const MEM_CAP_BYTES = 300_000_000; // epic #820 decision 7, as every lane pins it
const VARIANTS = ["base", "jit", "handoff", "stream", "typed", "held"];

/** The write axis's held input — the lane harness's prepareItems verbatim. */
function prepareItems(api, bytes) {
  const f = api.read(bytes);
  return f.groups.map((code) => [code, f.table(code)]);
}

/** The lane's parse-to-typed op — typeAllGroups verbatim. */
function typeAllGroups(api, bytes) {
  const f = api.read(bytes);
  for (const code of f.groups) f.table(code);
}

async function worker(variant, filePath) {
  const api = await import("../dist/index.mjs");
  const bytes = readFileSync(filePath);
  let outBytes = null;
  let sha = null;
  let doorMs = null;
  if (variant === "typed") {
    typeAllGroups(api, bytes);
  } else {
    const items = prepareItems(api, bytes);
    if (variant !== "held") {
      const t = process.hrtime.bigint();
      let out;
      if (variant === "base" || variant === "jit") {
        // jit differs only by the env var the parent set on this child.
        out = api.buildAgs4(items).bytes;
      } else {
        const require = createRequire(import.meta.url);
        const native = require("../index.js");
        const { tableToIPC } = await import("apache-arrow");
        const mode = variant === "handoff" ? "eager" : "stream";
        const session = new native.EmitSessionSpike(mode);
        for (const [code, table] of items) {
          // Marshal ONE group, hand it across, let its Buffer go out of
          // scope — the honest GC'd-host behaviour a real per-group
          // handoff would show.
          session.addGroup(code, Buffer.from(tableToIPC(table, "stream")));
        }
        out = session.finish().bytes;
      }
      doorMs = Number(process.hrtime.bigint() - t) / 1e6;
      outBytes = out.length;
      sha = createHash("sha256").update(out).digest("hex");
    }
  }
  const report = {
    maxrss_bytes: process.resourceUsage().maxRSS * 1024, // libuv: kb everywhere
    out_bytes: outBytes,
    sha256: sha,
    door_ms: doorMs,
  };
  process.stdout.write(`${JSON.stringify(report)}\n`);
}

function measureCell(variant, rung, reps) {
  if (rung.bytes > MEM_CAP_BYTES) {
    return { refusal: "beyond-mem-cap", detail: `${rung.bytes} > ${MEM_CAP_BYTES}` };
  }
  const env = { ...process.env };
  delete env.LATERITE_M7_SPIKE;
  if (variant === "jit") env.LATERITE_M7_SPIKE = "jit";
  const peaks = [];
  const doorMs = [];
  let outBytes = null;
  let sha = null;
  for (let i = 0; i < reps; i++) {
    const out = spawnSync(
      process.execPath,
      [SELF, "--worker", variant, "--file", rung.path],
      { encoding: "utf8", env },
    );
    if (out.status !== 0) {
      const tail = (out.stderr ?? "").trim().split("\n").slice(-3).join(" | ");
      return { refusal: "failed", detail: tail || `exit ${out.status}` };
    }
    const report = JSON.parse(out.stdout);
    peaks.push(report.maxrss_bytes);
    if (report.door_ms !== null) doorMs.push(report.door_ms);
    outBytes = report.out_bytes;
    sha = report.sha256;
  }
  peaks.sort((a, b) => a - b);
  doorMs.sort((a, b) => a - b);
  const peak = peaks[peaks.length >> 1];
  const denom = Math.max(outBytes ?? rung.bytes, 1);
  return {
    peak_rss_bytes: peak,
    x_output: Math.round((peak / denom) * 100) / 100,
    peaks_bytes: peaks,
    door_ms_median:
      doorMs.length > 0 ? Math.round(doorMs[doorMs.length >> 1] * 10) / 10 : null,
    out_bytes: outBytes,
    out_sha256: sha,
  };
}

async function main() {
  const argv = process.argv.slice(2);
  let workerVariant = null;
  let workerFile = null;
  let reps = 3;
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
  const legs = [
    ["A1", "base"],
    ...VARIANTS.slice(1).map((v) => ["B", v]),
    ["A2", "base"],
  ];
  for (const [leg, variant] of legs) {
    const cells = {};
    for (const rung of rungs) {
      cells[rung.label] = measureCell(variant, rung, reps);
      const c = cells[rung.label];
      console.log(
        `${leg} ${variant.padEnd(7)} ${rung.label.padStart(6)}: ` +
          (c.refusal ? `REFUSAL ${c.refusal}` : `${c.x_output}x  door=${c.door_ms_median}ms`),
      );
    }
    passes.push({ leg, variant, cells });
  }

  // Byte-identity across the four door variants, per rung, vs the A1 leg.
  const identity = {};
  const a1 = passes[0].cells;
  for (const p of passes.slice(1)) {
    if (["typed", "held"].includes(p.variant)) continue;
    for (const [label, cell] of Object.entries(p.cells)) {
      if (cell.refusal) continue;
      identity[`${p.leg}-${p.variant}/${label}`] =
        cell.out_sha256 === a1[label].out_sha256;
    }
  }

  const out = {
    schema: "m7-diagnosis/1",
    issue: 893,
    generated: new Date().toISOString(),
    git_sha: gitSha,
    instrument:
      "fresh-child peak RSS (process.resourceUsage().maxRSS) — diagnosis family",
    denominator: "x-of-output for the door variants; x-of-input for typed/held",
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
