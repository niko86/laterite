#!/usr/bin/env node
// SPIKE (#871) — price the `arrowIpc()` pass-through against `table()`'s
// decode. The question this answers: when a consumer's destination speaks
// Arrow IPC itself (duckdb, a worker, a socket), how much of the `table()`
// path is the arrow-js materialisation the pass-through skips?
//
// Two legs per group, timed separately over the same corpus rungs the README
// bench pins:
//
//   ipc     file.arrowIpc(code)      — native table build + IPC framing
//   decode  tableFromIPC(buf)        — the arrow-js materialisation
//
// `table(code)` pays ipc + decode (same native call, then the decode, then a
// cache write); a pass-through consumer pays ipc alone. The report is the sum
// over all groups per rung, plus decode's share of the summed table() path.
//
// Run (fixtures unpacked, addon built release):
//   node rust-packages/laterite-node/spike/arrow-ipc-bench.mjs

import { existsSync, statSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { tableFromIPC, tableToIPC } from "apache-arrow";

const REPO = fileURLToPath(new URL("../../..", import.meta.url));
const FIXTURES = `${REPO}output/readme-bench`;
const RUNGS = ["5MB", "25MB", "100MB"];
const RUNS = 5;

const { read } = await import(
  `${REPO}rust-packages/laterite-node/dist/index.mjs`
);

const fmt = (ms) =>
  ms >= 1000 ? `${(ms / 1000).toFixed(2)} s` : `${ms.toFixed(1)} ms`;
const mb = (b) => `${(b / 1e6).toFixed(1)} MB`;

function timed(fn, runs) {
  fn(); // warm-up: JIT + lazy native init, uncharged
  const t0 = performance.now();
  for (let i = 0; i < runs; i++) fn();
  return (performance.now() - t0) / runs;
}

// The last column is the counterfactual an IPC-wanting consumer pays TODAY
// without the pass-through: table() decode + `tableToIPC` re-encode. The
// pass-through door replaces (decode + encode) with nothing.
console.log(
  `arrowIpc pass-through vs table() decode — mean of ${RUNS} warm runs\n`,
);
console.log(
  "| File | groups | IPC bytes | `arrowIpc` (all groups) | decode leg | re-encode leg | today's IPC path (sum all) | saved |",
);
console.log("|---:|---:|---:|---:|---:|---:|---:|---:|");

for (const rung of RUNGS) {
  const path = `${FIXTURES}/readme-${rung}.ags`;
  if (!existsSync(path)) {
    console.error(`missing fixture ${path} — run the python bench first`);
    process.exit(1);
  }
  const file = read(path);
  let ipcMs = 0;
  let decodeMs = 0;
  let encodeMs = 0;
  let ipcBytes = 0;
  for (const code of file.groups) {
    ipcMs += timed(() => file.arrowIpc(code), RUNS);
    const buf = file.arrowIpc(code);
    ipcBytes += buf.length;
    decodeMs += timed(() => tableFromIPC(buf), RUNS);
    const table = tableFromIPC(buf);
    encodeMs += timed(() => tableToIPC(table), RUNS);
  }
  const today = ipcMs + decodeMs + encodeMs;
  const savedPct = (((decodeMs + encodeMs) / today) * 100).toFixed(0);
  console.log(
    `| ${mb(statSync(path).size)} | ${file.groups.length} | ${mb(ipcBytes)} | ${fmt(ipcMs)} | ${fmt(decodeMs)} | ${fmt(encodeMs)} | ${fmt(today)} | ${savedPct}% |`,
  );
}
