#!/usr/bin/env node
// SPIKE (#871) — peak-RSS probe for the `arrowIpc()` pass-through, answering
// "might this cause more memory issues?" with the lane instrument rather than
// reasoning: one FRESH node child per cell, the cell being the child's own
// `ru_maxrss` at exit (`process.resourceUsage().maxRSS`, KB via libuv on every
// platform). Same fairness argument as tools/bench-vs-python-ags4.py.
//
// Cells, each sweeping ALL groups of the rung once and dropping each result
// (a streaming consumer sends the bytes away; it does not accumulate them):
//
//   read-only     read(path), touch the group list — the native-parse floor
//                 every other cell sits on
//   passthrough   buf = file.arrowIpc(code)             (the spike door)
//   table-only    t   = file.table(code)                (decode, cached)
//   table-encode  buf = tableToIPC(file.table(code))    (today's IPC path)
//
// table-only minus passthrough is the CACHE RETENTION: table() memoises every
// decoded group on the handle and arrow-js tables are zero-copy views over the
// IPC buffer, so the cache pins ~the whole file's typed columns for the life
// of the handle. arrowIpc caches nothing.
//
// The `identity` mode pins the caching CONTRACT itself, empirically:
// table(code) twice is the same object; arrowIpc(code) twice is two buffers.
//
// Run:  node rust-packages/laterite-node/spike/mem-probe.mjs [rung]

import { spawnSync } from "node:child_process";
import { existsSync, statSync } from "node:fs";
import { fileURLToPath } from "node:url";

const REPO = fileURLToPath(new URL("../../..", import.meta.url));
const MODES = ["read-only", "passthrough", "table-only", "table-encode"];

const mb = (b) => `${(b / 1e6).toFixed(0)} MB`;

// --- child ------------------------------------------------------------------

if (process.argv[2] === "--child") {
  const [, , , mode, path] = process.argv;
  const { tableFromIPC, tableToIPC } = await import("apache-arrow");
  const { read } = await import(
    `${REPO}rust-packages/laterite-node/dist/index.mjs`
  );

  const file = read(path);
  let total = 0;

  if (mode === "identity") {
    // The caching contract, observed rather than read from the source.
    const code = file.groups[0];
    const sameTable = file.table(code) === file.table(code);
    const keyedSeparate = file.table(code) !== file.table(code, { keys: true });
    const a = file.arrowIpc(code);
    const b = file.arrowIpc(code);
    const freshBuffers = a !== b;
    // Zero-copy check: the decoded table's first column's data buffer lives
    // inside the very Buffer tableFromIPC was handed (same ArrayBuffer), so
    // caching the Table pins the IPC bytes.
    const t = tableFromIPC(a);
    const view = t.getChildAt(0)?.data[0]?.values;
    const sharesMemory = view instanceof Object && view.buffer === a.buffer;
    console.log(
      JSON.stringify({ sameTable, keyedSeparate, freshBuffers, sharesMemory }),
    );
    process.exit(0);
  }

  // Settled RSS before/after the sweep (forced GC around each reading) is the
  // RETENTION axis peak RSS cannot see: peak mixes transient churn with what
  // stays pinned; the settled delta is what the sweep left resident. darwin
  // caveat (the campaign's #831 lesson): natively freed pages are MADV_FREE'd
  // and can stay resident without memory pressure, so a non-zero passthrough
  // delta here is an UPPER bound on genuine retention, not proof of a hold.
  globalThis.gc();
  const rssBefore = process.memoryUsage().rss;

  for (const code of file.groups) {
    if (mode === "passthrough") total += file.arrowIpc(code).length;
    else if (mode === "table-only") total += file.table(code).numRows;
    else if (mode === "table-encode")
      total += tableToIPC(file.table(code)).length;
    else total += code.length; // read-only: touch the list, decode nothing
  }

  globalThis.gc();
  const after = process.memoryUsage();

  // `arrayBuffers` is V8's own ledger of ArrayBuffer bytes still REFERENCED
  // from JS — the retention fact itself, independent of how darwin accounts
  // resident pages (compressor, MADV_FREE). RSS says what the OS holds;
  // arrayBuffers says what the JS side still pins.
  console.log(
    JSON.stringify({
      mode,
      total,
      peak_rss_bytes: process.resourceUsage().maxRSS * 1024,
      settled_before: rssBefore,
      settled_after: after.rss,
      pinned_array_buffers: after.arrayBuffers,
    }),
  );
  process.exit(0);
}

// --- parent -----------------------------------------------------------------

const rung = process.argv[2] ?? "100MB";
const path = `${REPO}output/readme-bench/readme-${rung}.ags`;
if (!existsSync(path)) {
  console.error(`missing fixture ${path} — run the python bench first`);
  process.exit(1);
}

function cell(mode) {
  const r = spawnSync(
    process.execPath,
    ["--expose-gc", fileURLToPath(import.meta.url), "--child", mode, path],
    { encoding: "utf8" },
  );
  if (r.status !== 0) {
    console.error(`child ${mode} failed:\n${r.stderr}`);
    process.exit(1);
  }
  return JSON.parse(r.stdout.trim().split("\n").at(-1));
}

const id = cell("identity");
console.log(
  `caching contract (observed): table() memoised=${id.sameTable}, ` +
    `keyed cache separate=${id.keyedSeparate}, ` +
    `arrowIpc fresh buffer per call=${id.freshBuffers}, ` +
    `decoded table shares the IPC buffer's memory=${id.sharesMemory}\n`,
);

const size = statSync(path).size;
console.log(
  `peak RSS, one fresh child per cell, ${mb(size)} rung — all groups swept once, results dropped\n`,
);
console.log(
  "| cell | peak RSS | vs read-only floor | settled Δ (RSS) | JS-pinned ArrayBuffers at exit |",
);
console.log("|---|---:|---:|---:|---:|");
const f = cell("read-only");
const floor = f.peak_rss_bytes;
console.log(
  `| read-only (floor) | ${mb(floor)} | — | ${mb(f.settled_after - f.settled_before)} | ${mb(f.pinned_array_buffers)} |`,
);
for (const mode of MODES.slice(1)) {
  const c = cell(mode);
  const d = c.peak_rss_bytes - floor;
  const kept = c.settled_after - c.settled_before;
  console.log(
    `| ${mode} | ${mb(c.peak_rss_bytes)} | ${d >= 0 ? "+" : ""}${mb(d)} | ${mb(kept)} | ${mb(c.pinned_array_buffers)} |`,
  );
}
