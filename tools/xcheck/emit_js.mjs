#!/usr/bin/env node
// The JS legs of the cross-surface OUTPUT-VALUE gate (plan
// output/output-value-gate-plan.md §2): the node binding and the wasm ENGINE.
//
//   * node — the laterite-node addon (`read(...).text`, `buildAgs4(...)`).
//   * wasm-engine — laterite-ags4-wasm, the browser cdylib, driven under node by
//     handing the glue the .wasm BYTES (no fetch) — the IDENTICAL artifact the
//     browser loads, post-wasm-opt. Named `wasm-engine`, never "browser": the
//     ~40 KB of TypeScript above it (validatorClient, fixpreview) is NOT gated
//     here (plan §4). It has no byte-faithful re-emit door, so it joins only the
//     build direction.
//
// For each case's `op` a leg drives ONE public expression — no adapter logic;
// the observation is the three-variant envelope {"ok"|"err"|"absent"}. The
// comparator does zero normalisation, so host-idiom transforms live HERE. Each
// leg self-skips when its artifact is absent (in CI --require-legs all fails it).
//
//   node tools/xcheck/emit_js.mjs --out <dir> [--cases <dir>] [--repo-root <dir>]

import { mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";

function loadCases(casesDir) {
  return readdirSync(casesDir)
    .filter((f) => f.endsWith(".json"))
    .sort()
    .flatMap((f) => JSON.parse(readFileSync(join(casesDir, f), "utf8")).cases);
}

/** Typed inline `build` groups → node `buildAgs4`'s `[[code, rowObjects]]` shape:
 *  each row's positional values zipped back onto the headings. */
function toNodeGroups(build) {
  return build.map((g) => [
    g.code,
    g.rows.map((r) => Object.fromEntries(g.headings.map((h, i) => [h, r[i]]))),
  ]);
}

async function runNode(cases, repoRoot) {
  const mod = await import(new URL("../../rust-packages/laterite-node/dist/index.mjs", import.meta.url));
  const observe = (aCase) => {
    const op = aCase.op;
    if (op === "reemit_canonical") {
      const fixture = aCase.input?.fixture;
      if (fixture == null) return null;
      try {
        return { ok: mod.read(join(repoRoot, fixture)).text };
      } catch (e) {
        return { err: (e?.name ?? "Error").replace(/Error$/, "") };
      }
    }
    if (op === "build_typed") {
      const build = aCase.input?.build;
      if (build == null) return null;
      try {
        return { ok: mod.buildAgs4(toNodeGroups(build), buildOpts(aCase)).text };
      } catch (e) {
        return { err: (e?.name ?? "Error").replace(/Error$/, "") };
      }
    }
    return null;
  };
  return collect("node", mod.engineFingerprint(), cases, observe);
}

/** The two knobs the build legs share, in each surface's own spelling.
 *
 * Node and wasm take the SAME object shape here, which is the point: the case
 * manifest states one transmission and both doors must reproduce the same
 * bytes from it. `null` when the case sets no options, so the default-path
 * cases keep exercising the defaults. */
function buildOpts(aCase) {
  const o = aCase.input?.build_opts;
  if (o == null) return undefined;
  return {
    synthesiseMetadata: Boolean(o.synthesise_metadata),
    ...(o.tran ? { tran: o.tran } : {}),
  };
}

async function runWasm(cases, repoRoot) {
  const glue = await import(new URL("../../web/src/wasm/ags4_wasm.js", import.meta.url));
  const wasmBytes = readFileSync(new URL("../../web/src/wasm/ags4_wasm_bg.wasm", import.meta.url));
  await glue.default({ module_or_path: wasmBytes });
  const observe = (aCase) => {
    if (aCase.op === "build_typed") {
      const build = aCase.input?.build;
      if (build == null) return null;
      try {
        // wasm's build_ags4 takes the groups_json string DIRECTLY — the same
        // {code, headings, units?, types?, rows} shape the manifest carries —
        // then ONE named options object.
        return { ok: glue.build_ags4(JSON.stringify(build), buildOpts(aCase)).text };
      } catch (e) {
        return { err: (e?.message ?? "Error").split("\n")[0] };
      }
    }
    return null;
  };
  return collect("wasm-engine", glue.engine_fingerprint(), cases, observe);
}

// `engine` is the digest of the rules this leg is ACTUALLY running, asked of the
// artifact rather than assumed from the tree. Both JS legs read a built artifact —
// `node` the napi addon, `wasm-engine` a wasm-pack output — either of which can be
// stale while every case still matches, because a stale engine and a current one
// usually agree. Without this the run would report N-way identity across a surface
// compiled some time ago; the comparator holds it to the authority's.
function collect(leg, engine, cases, observe) {
  const observations = {};
  for (const aCase of cases) {
    if (!aCase.legs.includes(leg)) continue;
    const obs = observe(aCase);
    if (obs != null) observations[aCase.id] = obs;
  }
  return { schema: 1, leg, engine, cases: observations };
}

function parseArgs(argv) {
  const opts = { out: "output/xcheck", cases: "rust-packages/laterite-ags4-xcheck/cases", repoRoot: "." };
  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === "--out") opts.out = argv[++i];
    else if (argv[i] === "--cases") opts.cases = argv[++i];
    else if (argv[i] === "--repo-root") opts.repoRoot = argv[++i];
    else throw new Error(`unknown arg: ${argv[i]}`);
  }
  return opts;
}

async function main() {
  const { out, cases: casesDir, repoRoot } = parseArgs(process.argv.slice(2));
  const cases = loadCases(casesDir);
  mkdirSync(out, { recursive: true });
  for (const run of [runNode, runWasm]) {
    try {
      const payload = await run(cases, repoRoot);
      const path = join(out, `${payload.leg}.json`);
      writeFileSync(path, JSON.stringify(payload, null, 2));
      console.error(`${payload.leg}: ${Object.keys(payload.cases).length} cases -> ${path}`);
    } catch (e) {
      console.error(`leg unavailable — skipping (${e.message.split("\n")[0]})`);
    }
  }
}

await main();
