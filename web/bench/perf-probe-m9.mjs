// #892 M9 attribution probe (SPIKE BRANCH ONLY — never lands on main).
//
// The #893 sweep priced the write door's decode slab at −20.7% of the
// linear-memory high-water at 100 MB but 0.0% at 265 MB — the headline
// rung's high-water is owned by something else, and NAMING it is the M9
// mint's first step (the ledger row: "masked rather than mooted"). This
// probe attributes it by stage: wasm linear memory only grows, so a
// reading at each boundary attributes the growth since the previous one,
// and the LAST stage that grows memory owns the high-water.
//
// Variants (fresh instantiation per child, rule 13's instrument):
//   shipped  the shipped `build_ags4_ipc` — the reference: its final
//            byteLength is the door's real high-water, and the staged
//            twin must land within a page of it or the checkpoints
//            changed the shape they measure.
//   staged   `build_ags4_ipc_spike_stages` (this branch): the shipped
//            shape with internal boundaries — entry / after the copy-in +
//            decode loop (the slab) / after the emit (EmitResult live) /
//            after shape_report (the from_utf8_lossy output copy live
//            BESIDE EmitResult.bytes).
//   jit      `build_ags4_ipc_spike_jit_stages`: the #893 fix shape with
//            the same boundaries, so the profiles compare cell for cell.
//
// JS-side boundaries ride along: after init / after read() / after the
// arrow_ipc framing — the pre-door growth the door then does or does not
// climb past.
//
// Output text sha256-checked identical across all three variants per rung.
// Run from web/ (built wasm-full with the spike exports):
//   node bench/perf-probe-m9.mjs [--reps 2]
// Writes ../tools/perf-results/m9-attribution.json (repo-relative).
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
const OUT = join(REPO, "tools", "perf-results", "m9-attribution.json");

const MEM_CAP_BYTES = 300_000_000; // epic #820 decision 7 — ladder policy (rule 13)
const PAGE = 65536;
const VARIANTS = ["shipped", "staged", "jit"];

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
  const jsStage = {};
  jsStage.init_bytes = memory.buffer.byteLength;
  const data = readFileSync(filePath);
  const ds = glue.read(data);
  jsStage.after_read_bytes = memory.buffer.byteLength;
  const items = ds.group_codes().map((code) => ({ code, ipc: ds.arrow_ipc(code) }));
  ds.free();
  jsStage.after_frame_bytes = memory.buffer.byteLength;

  const t = process.hrtime.bigint();
  let text;
  let doorStages = null;
  if (variant === "shipped") {
    text = glue.build_ags4_ipc(items).text;
  } else {
    const r =
      variant === "jit"
        ? glue.build_ags4_ipc_spike_jit_stages(items)
        : glue.build_ags4_ipc_spike_stages(items);
    text = r.text;
    doorStages = {
      entry_bytes: r.entry_pages * PAGE,
      after_decode_bytes: r.after_decode_pages * PAGE,
      after_emit_bytes: r.after_emit_pages * PAGE,
      after_shape_bytes: r.after_shape_pages * PAGE,
    };
  }
  const doorMs = Number(process.hrtime.bigint() - t) / 1e6;
  const out = {
    linear_memory_bytes: memory.buffer.byteLength,
    out_bytes: Buffer.byteLength(text, "utf8"),
    sha256: createHash("sha256").update(text).digest("hex"),
    door_ms: doorMs,
    js_stages: jsStage,
    door_stages: doorStages,
  };
  process.stdout.write(`${JSON.stringify(out)}\n`);
}

function measureCell(variant, rung, reps) {
  if (rung.bytes > MEM_CAP_BYTES) {
    return { refusal: "beyond-mem-cap", detail: `${rung.bytes} > ${MEM_CAP_BYTES}` };
  }
  const runs = [];
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
    runs.push(JSON.parse(out.stdout));
  }
  const highs = runs.map((r) => r.linear_memory_bytes);
  const doorMs = runs.map((r) => r.door_ms).sort((a, b) => a - b);
  const r0 = runs[0];
  return {
    instrument: "wasm-linear-memory",
    peak_linear_memory_bytes: highs[0],
    x_output: Math.round((highs[0] / Math.max(r0.out_bytes ?? rung.bytes, 1)) * 100) / 100,
    deterministic: highs.every((h) => h === highs[0]),
    highs_bytes: highs,
    door_ms_median: Math.round(doorMs[doorMs.length >> 1] * 10) / 10,
    out_bytes: r0.out_bytes,
    out_sha256: r0.sha256,
    js_stages: r0.js_stages,
    door_stages: r0.door_stages,
  };
}

/** Per-stage growth deltas — the attribution table. Grow-only memory means
 * the last positive delta names the stage that set the high-water. */
function deltas(cell) {
  if (cell.refusal || !cell.door_stages) return null;
  const j = cell.js_stages;
  const d = cell.door_stages;
  return {
    read_growth: j.after_read_bytes - j.init_bytes,
    frame_growth: j.after_frame_bytes - j.after_read_bytes,
    door_decode_growth: d.after_decode_bytes - d.entry_bytes,
    door_emit_growth: d.after_emit_bytes - d.after_decode_bytes,
    door_shape_growth: d.after_shape_bytes - d.after_emit_bytes,
    return_growth: cell.peak_linear_memory_bytes - d.after_shape_bytes,
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

  const cells = {};
  for (const variant of VARIANTS) {
    cells[variant] = {};
    for (const rung of rungs) {
      const c = measureCell(variant, rung, reps);
      cells[variant][rung.label] = c;
      const dl = deltas(c);
      console.log(
        `${variant.padEnd(7)} ${rung.label.padStart(6)}: ` +
          (c.refusal
            ? `REFUSAL ${c.refusal}`
            : `high=${(c.peak_linear_memory_bytes / 1e6).toFixed(0)}MB ` +
              `${c.x_output}x det=${c.deterministic}` +
              (dl
                ? ` | growth MB: read=${(dl.read_growth / 1e6).toFixed(0)}` +
                  ` frame=${(dl.frame_growth / 1e6).toFixed(0)}` +
                  ` decode=${(dl.door_decode_growth / 1e6).toFixed(0)}` +
                  ` emit=${(dl.door_emit_growth / 1e6).toFixed(0)}` +
                  ` shape=${(dl.door_shape_growth / 1e6).toFixed(0)}` +
                  ` ret=${(dl.return_growth / 1e6).toFixed(0)}`
                : "")),
      );
    }
  }

  // Self-checks: byte-identity across variants; the staged twin must land
  // within one page of the shipped door or the checkpoints changed the shape.
  const identity = {};
  let stagedFaithful = true;
  for (const rung of rungs) {
    const s = cells.shipped[rung.label];
    if (s.refusal) continue;
    for (const v of ["staged", "jit"]) {
      const c = cells[v][rung.label];
      if (!c.refusal) identity[`${v}/${rung.label}`] = c.out_sha256 === s.out_sha256;
    }
    const st = cells.staged[rung.label];
    if (!st.refusal) {
      const drift = Math.abs(st.peak_linear_memory_bytes - s.peak_linear_memory_bytes);
      if (drift > PAGE) {
        stagedFaithful = false;
        console.error(
          `STAGED-SHAPE DRIFT at ${rung.label}: staged high differs from shipped by ${drift} bytes`,
        );
      }
    }
  }

  const out = {
    schema: "m9-attribution/1",
    issue: 892,
    generated: new Date().toISOString(),
    git_sha: gitSha,
    instrument:
      "wasm-linear-memory (rule 13): fresh instantiation; grow-only, so per-boundary readings attribute growth to stages",
    denominator: "x-of-output",
    reps,
    loadavg_start: loadavg(),
    cells,
    attribution: Object.fromEntries(
      rungs.map((r) => [
        r.label,
        {
          staged: deltas(cells.staged[r.label]),
          jit: deltas(cells.jit[r.label]),
        },
      ]),
    ),
    output_byte_identity: identity,
    staged_shape_faithful: stagedFaithful,
  };
  mkdirSync(dirname(OUT), { recursive: true });
  writeFileSync(OUT, `${JSON.stringify(out, null, 2)}\n`);
  console.log(`written: ${OUT}`);
  if (!Object.values(identity).every(Boolean)) {
    console.error("BYTE-IDENTITY FAILURE — a variant changed the output");
    process.exitCode = 1;
  }
  if (!stagedFaithful) process.exitCode = 1;
}

await main();
