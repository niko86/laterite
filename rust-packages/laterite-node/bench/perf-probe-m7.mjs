// M7 fix-design probe (SPIKE BRANCH ONLY — never lands on main).
//
// The #893 round measured the per-group pair child (`stream`) at only
// −7.7% (100MB) / −8…−14% (265MB) against a ~−19% perfect-reclamation
// bound, and the owner's waiver (PR #898) kept the row live on ONE
// condition: the fix design must force boundary-byte reclamation. This
// probe prices that condition before anything mints. The gap's suspect is
// per-group marshalling garbage the GC'd host never collects mid-run —
// each group abandons TWO allocations of ~its IPC size (`tableToIPC`'s
// Uint8Array, then the `Buffer.from` copy of it) — so the children remove
// that garbage in two steps:
//
//   base      the shipped door: marshal ALL groups' IPC, one native call.
//   stream    per-group `EmitSessionSpike("stream")`, fresh
//             `Buffer.from(tableToIPC(...))` per group — the #893 pair
//             child verbatim: both whole-file holds removed, marshalling
//             garbage left to the GC.
//   reuse     stream + a REUSED non-pooled staging buffer for the copy
//             (`allocUnsafeSlow`, grown geometrically, `subarray` view
//             across the boundary) — the copy half of the garbage never
//             forms. THE SHIPPABLE SHAPE: native `addGroup` decodes
//             within the call and holds nothing, so overwriting the
//             staging next group is sound. `tableToIPC`'s own allocation
//             remains per-group garbage.
//   gcstream  stream + `global.gc()` after every group (child runs with
//             --expose-gc) — ALL marshalling garbage collected promptly.
//             NOT a shippable shape; it MEASURES the perfect-reclamation
//             bound on the real host, replacing the #893 arithmetic
//             bound with an observed one.
//
// Every child holds the caller's own input (`items`, the arrow-js
// Tables) across the door exactly as the lane's write child does.
//
// Instrument: fresh-child peak RSS (this process's own maxRSS at exit) —
// the diagnosis family (epic #820 rule 8); one machine, never a
// cross-library table. Denominator: ×-of-output (the write row's). A/B/A:
// base passes bracket the variants in one sitting. Output bytes
// sha256-checked identical across all variants per rung. door_ms rides
// along: reuse pays an extra memcpy and gcstream pays forced collections,
// and a fix that trades wall time for peak needs that priced.
//
// Run from rust-packages/laterite-node (quiet machine, built package):
//   node bench/perf-probe-m7.mjs [--reps 3]
// Writes ../../tools/perf-results/m7-reuse.json (repo-relative).
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
const OUT = join(REPO, "tools", "perf-results", "m7-reuse.json");

const MEM_CAP_BYTES = 300_000_000; // epic #820 decision 7, as every lane pins it
const VARIANTS = ["base", "stream", "reuse", "gcstream"];

/** The write axis's held input — the lane harness's prepareItems verbatim. */
function prepareItems(api, bytes) {
  const f = api.read(bytes);
  return f.groups.map((code) => [code, f.table(code)]);
}

async function worker(variant, filePath) {
  const api = await import("../dist/index.mjs");
  const bytes = readFileSync(filePath);
  const items = prepareItems(api, bytes);
  const t = process.hrtime.bigint();
  let out;
  if (variant === "base") {
    out = api.buildAgs4(items).bytes;
  } else {
    const require = createRequire(import.meta.url);
    const native = require("../index.js");
    const { tableToIPC } = await import("apache-arrow");
    if (variant === "gcstream" && typeof global.gc !== "function") {
      throw new Error("gcstream needs --expose-gc on the child");
    }
    const session = new native.EmitSessionSpike("stream");
    // Non-pooled so a view of it can cross the boundary without dragging
    // Buffer's shared pool into the measurement.
    let staging = variant === "reuse" ? Buffer.allocUnsafeSlow(1 << 20) : null;
    for (const [code, table] of items) {
      const u8 = tableToIPC(table, "stream");
      if (variant === "reuse") {
        if (u8.length > staging.length) {
          let n = staging.length;
          while (n < u8.length) n *= 2;
          staging = Buffer.allocUnsafeSlow(n);
        }
        staging.set(u8);
        session.addGroup(code, staging.subarray(0, u8.length));
      } else {
        // Fresh copy per group, abandoned to the GC — the honest shipped
        // marshalling (and the #893 pair child verbatim).
        session.addGroup(code, Buffer.from(u8));
      }
      if (variant === "gcstream") global.gc();
    }
    out = session.finish().bytes;
  }
  const doorMs = Number(process.hrtime.bigint() - t) / 1e6;
  const report = {
    maxrss_bytes: process.resourceUsage().maxRSS * 1024, // libuv: kb everywhere
    out_bytes: out.length,
    sha256: createHash("sha256").update(out).digest("hex"),
    door_ms: doorMs,
  };
  process.stdout.write(`${JSON.stringify(report)}\n`);
}

function measureCell(variant, rung, reps) {
  if (rung.bytes > MEM_CAP_BYTES) {
    return { refusal: "beyond-mem-cap", detail: `${rung.bytes} > ${MEM_CAP_BYTES}` };
  }
  const nodeArgs = variant === "gcstream" ? ["--expose-gc"] : [];
  const peaks = [];
  const doorMs = [];
  let outBytes = null;
  let sha = null;
  for (let i = 0; i < reps; i++) {
    const out = spawnSync(
      process.execPath,
      [...nodeArgs, SELF, "--worker", variant, "--file", rung.path],
      { encoding: "utf8" },
    );
    if (out.status !== 0) {
      const tail = (out.stderr ?? "").trim().split("\n").slice(-3).join(" | ");
      return { refusal: "failed", detail: tail || `exit ${out.status}` };
    }
    const report = JSON.parse(out.stdout);
    peaks.push(report.maxrss_bytes);
    doorMs.push(report.door_ms);
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
    door_ms_median: Math.round(doorMs[doorMs.length >> 1] * 10) / 10,
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
        `${leg} ${variant.padEnd(8)} ${rung.label.padStart(6)}: ` +
          (c.refusal ? `REFUSAL ${c.refusal}` : `${c.x_output}x  door=${c.door_ms_median}ms`),
      );
    }
    passes.push({ leg, variant, cells });
  }

  // Byte-identity across every variant, per rung, vs the A1 leg.
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
    schema: "m7-reuse/1",
    issue: 893,
    follow_up: "M7 fix-design spike: does forced boundary-byte reclamation close the stream->bound gap?",
    generated: new Date().toISOString(),
    git_sha: gitSha,
    instrument:
      "fresh-child peak RSS (process.resourceUsage().maxRSS) — diagnosis family",
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
