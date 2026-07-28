#!/usr/bin/env node
// Reproduce the npm README's performance table.
//
// The npm README quotes ABSOLUTE throughput, not a speedup: there is no
// python-ags4 on Node, so a comparison table would have nothing honest to
// compare against. What a Node consumer wants to know is "how long does this
// take on a file my size", which is a self-contained number.
//
// It reuses the SAME rungs and the SAME pinned manifest as
// `tools/bench-vs-python-ags4.py`, so the Node column sits on the same axis as
// the Python one — a reader can put the two READMEs side by side and the
// comparison is meaningful. Fixtures are `forge scale --scaffold wide --seed 0`,
// byte-identical across machines, and each rung's SHA-256 is verified before
// timing: comparing against different data is worse than not comparing.
//
// Generate the fixtures first (they are shared):
//     uv run python tools/bench-vs-python-ags4.py --rungs 5MB,25MB,100MB
//
// Then:
//     node tools/bench-node.mjs                  # default rungs
//     node tools/bench-node.mjs --rungs 5MB,25MB
//     node tools/bench-node.mjs --runs 10
//
// Needs the native addon built: (cd rust-packages/laterite-node && npm run build)

import { createHash } from "node:crypto";
import { existsSync, readFileSync, statSync } from "node:fs";
import { fileURLToPath } from "node:url";

const REPO = fileURLToPath(new URL("..", import.meta.url));
const FIXTURES = `${REPO}output/readme-bench`;
const MANIFEST = `${REPO}tools/readme-bench-fixtures.json`;
const DEFAULT_RUNGS = ["5MB", "25MB", "100MB"];

function parseArgs(argv) {
  const opts = { rungs: DEFAULT_RUNGS, runs: 5 };
  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === "--rungs") opts.rungs = argv[++i].split(",");
    else if (argv[i] === "--runs") opts.runs = Number(argv[++i]);
    else die(`unknown arg: ${argv[i]}`);
  }
  return opts;
}

function die(msg) {
  console.error(`bench-node: ${msg}`);
  process.exit(1);
}

/** Mean wall-clock of `fn` over `runs` warm iterations, in milliseconds. One
 *  untimed warm-up first so the JIT and the addon's lazy init aren't charged. */
function timed(fn, runs) {
  fn();
  const t0 = performance.now();
  for (let i = 0; i < runs; i++) fn();
  return (performance.now() - t0) / runs;
}

const fmt = (ms) => (ms >= 1000 ? `${(ms / 1000).toFixed(1)} s` : `${Math.round(ms)} ms`);

async function main() {
  const { rungs, runs } = parseArgs(process.argv.slice(2));

  let mod;
  try {
    mod = await import(`${REPO}rust-packages/laterite-node/dist/index.mjs`);
  } catch (e) {
    die(
      `the laterite-node addon is not built (${e.message.split("\n")[0]}).\n` +
        `       Build it first:  (cd rust-packages/laterite-node && npm run build)`,
    );
  }

  const manifest = JSON.parse(readFileSync(MANIFEST, "utf8"));
  const rows = [];

  for (const rung of rungs) {
    const pinned = manifest[rung];
    if (!pinned) die(`no manifest entry for rung ${rung}`);
    const path = `${FIXTURES}/readme-${rung}.ags`;
    if (!existsSync(path)) {
      // The Python bench leaves the rungs zstd-packed between runs (AGS4 is
      // extremely compressible, so the resting footprint is a fraction of the
      // ~900 MB plain). Unpack on demand through laterite's OWN transport, the
      // same dogfooding the Python side does — the SHA-256 check below then
      // doubles as a byte-exact pack/unpack round-trip test.
      const packed = `${path}.zst`;
      if (!existsSync(packed)) {
        die(
          `fixture ${path} is absent — generate the shared rungs first:\n` +
            `       uv run python tools/bench-vs-python-ags4.py --rungs ${rungs.join(",")}`,
        );
      }
      process.stderr.write(`  unpacking readme-${rung}.ags.zst …\n`);
      mod.transport.unpack(packed, path);
    }
    const bytes = readFileSync(path);
    // Same discipline as the Python bench: a drifted fixture is a HARD error,
    // never a warning. Numbers measured against different data are not numbers.
    const sha = createHash("sha256").update(bytes).digest("hex");
    if (sha !== pinned.sha256) {
      die(
        `fixture ${rung} drifted (sha256 ${sha.slice(0, 12)}… != pinned ` +
          `${pinned.sha256.slice(0, 12)}…) — regenerate with the Python bench`,
      );
    }

    const mb = statSync(path).size / 1e6;
    process.stderr.write(`[${mb.toFixed(1)} MB] timing …\n`);

    const readMs = timed(() => mod.read(path), runs);
    const validateMs = timed(() => mod.validate(path), runs);
    // The typed materialization a Node consumer actually consumes: every group
    // to an Arrow table. Keyless (the default) — the content-addressed keychain
    // is opt-in and is timed separately below.
    const tableMs = timed(() => {
      const f = mod.read(path);
      for (const code of f.groups) f.table(code);
    }, runs);

    rows.push({ mb, readMs, validateMs, tableMs });
  }

  const cell = (mb, ms) => `${fmt(ms)} · ${Math.round(mb / (ms / 1000))} MB/s`;
  console.log(`\nlaterite-node — mean of ${runs} warm runs\n`);
  console.log("| File (123 groups) | `read` | `validate` | `read` + typed tables |");
  console.log("|---:|---:|---:|---:|");
  for (const r of rows) {
    console.log(
      `| ${r.mb.toFixed(1)} MB | ${cell(r.mb, r.readMs)} | ` +
        `${cell(r.mb, r.validateMs)} | ${cell(r.mb, r.tableMs)} |`,
    );
  }
  console.log();
}

await main();
