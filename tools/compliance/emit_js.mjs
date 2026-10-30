#!/usr/bin/env node
// The node and wasm legs of the cross-surface findings gate (#1170).
//
// Runs ONE JS surface over a fixtures dir and writes `<surface>.json` in the
// schema `laterite-ags4-compliance` reads, beside what `emit-rust` and
// `emit_python.py` write. Each surface runs in the ci.yml job that already
// builds it: node in `node`, against the napi addon and tsup dist; wasm in
// `ts-lint`, against the tier-1 build in web/src/wasm the browser app ships.
//
// Ported from the dev satellite's emitter, with one change of polarity: that one
// ran both surfaces and skipped whichever was not built. Here the surface is
// named and an unbuilt one is a FAILURE, because a skipped surface reaches the
// comparator as a missing witness and the job that should have produced it
// stays green.
//
// Harness invariants, identical to the other emitters: warnings and FYI on,
// check_files off, encoding left to its default (utf-8, lossy) so the latin-1
// and cp1252 fixtures fire Rule 1 the same way on every surface. Passing
// "windows-1252" would decode those bytes cleanly and make Rule 1 vanish, a
// false split.
//
//   node tools/compliance/emit_js.mjs --surface node|wasm --fixtures <dir> --out <dir>

import { mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const ROOT = new URL("../../", import.meta.url);
const RULE = /^AGS Format Rule /;
const FYI = /^FYI/;

/** Partition finding labels into the rule floor and the FYI labels, the same
 *  prefix split emit_python.py and emit-rust apply. */
function split(labels) {
  const floor = labels.filter((k) => RULE.test(k)).sort();
  const fyi = labels.filter((k) => FYI.test(k)).sort();
  return [floor, fyi];
}

/** The floor as full tuples (rule, line, group, desc, field_index), parsed from
 *  the report NDJSON: the value the four-surface check compares. FYI lines are
 *  excluded; `field_index` is null on a whole-group finding. */
function floorTuples(ndjson) {
  const out = [];
  for (const line of ndjson.split("\n")) {
    if (!line.trim()) continue;
    const o = JSON.parse(line);
    if (!RULE.test(o.rule ?? "")) continue;
    out.push({
      rule: o.rule,
      line: o.line ?? null,
      group: o.group ?? "",
      desc: o.desc ?? "",
      field_index: o.field_index ?? null,
    });
  }
  return out;
}

/** wasm reports a hard error as a snake_case `kind` (`not_ags4`); the other
 *  surfaces name the same error by its variant (`NotAgs4`). Converting the
 *  spelling, rather than keeping a table of kinds, means a new kind cannot be
 *  missing from the table and read as a split. */
function sentinel(kind) {
  return kind.replace(/(^|_)([a-z0-9])/g, (_, __, c) => c.toUpperCase());
}

function fixturePaths(dir) {
  return readdirSync(dir)
    .filter((f) => f.endsWith(".ags"))
    .sort()
    .map((f) => ({ name: f, path: join(dir, f) }));
}

async function runNode(fixtures) {
  const m = await import(new URL("rust-packages/laterite-node/dist/index.mjs", ROOT));
  const results = fixtures.map(({ name, path }) => {
    try {
      const rep = m.validate(path, { warnings: true, fyi: true });
      const [rules, fyi] = split(Object.keys(rep.byRule()));
      const findings = floorTuples(rep.toNdjson());
      return { fixture: name, rules, findings, fyi, hard_error: null, error: null };
    } catch (e) {
      // node throws typed errors (NotAgs4Error, UnsupportedEditionError, …);
      // stripping `Error` recovers the sentinel. Anything untyped is a harness
      // bug, not a verdict, and must not be recorded as one.
      if (!e?.name || e.name === "Error" || e.name === "TypeError") throw e;
      const hard = e.name.replace(/Error$/, "");
      return { fixture: name, rules: [], findings: [], fyi: [], hard_error: hard, error: null };
    }
  });
  return {
    schema: 1,
    surface: "node",
    version: m.engineVersion(),
    engine_sha: m.engineFingerprint(),
    results,
  };
}

async function runWasm(fixtures) {
  const glue = await import(new URL("web/src/wasm/ags4_wasm.js", ROOT));
  const wasmBytes = readFileSync(new URL("web/src/wasm/ags4_wasm_bg.wasm", ROOT));
  await glue.default({ module_or_path: wasmBytes });
  const results = fixtures.map(({ name, path }) => {
    const data = new Uint8Array(readFileSync(path));
    const rep = glue.validate(data, { warnings: true, fyi: true });
    if (rep.error) {
      if (rep.error.kind === "bad_args") throw new Error(`bad_args: ${rep.error.message}`);
      return {
        fixture: name,
        rules: [],
        findings: [],
        fyi: [],
        hard_error: sentinel(rep.error.kind),
        error: null,
      };
    }
    // `total`, not items.length: items may be clipped by maxPerRule (unset here,
    // so they are complete, but total is the count either way).
    const labels = rep.findings.filter((g) => g.total > 0).map((g) => g.rule);
    const [rules, fyi] = split(labels);
    // Tuples straight from the wasm FindingDto `items`: the one surface whose
    // finding serialisation exists nowhere else, so it is compared field by
    // field against the shared renderer the other three route through.
    // `char_span` is wasm-only and has no peer to compare against.
    const findings = [];
    for (const g of rep.findings) {
      if (!RULE.test(g.rule)) continue;
      for (const it of g.items ?? []) {
        findings.push({
          rule: g.rule,
          line: it.line ?? null,
          group: it.group ?? "",
          desc: it.desc ?? "",
          field_index: it.field_index ?? null,
        });
      }
    }
    return { fixture: name, rules, findings, fyi, hard_error: null, error: null };
  });
  return {
    schema: 1,
    surface: "wasm",
    version: glue.engine_version(),
    engine_sha: glue.engine_fingerprint(),
    results,
  };
}

const RUNNERS = { node: runNode, wasm: runWasm };

function parseArgs(argv) {
  const opts = { surface: null, fixtures: null, out: null };
  for (let i = 0; i < argv.length; i++) {
    const flag = argv[i].replace(/^--/, "");
    if (!(flag in opts) || argv[i + 1] === undefined) throw new Error(`bad arg: ${argv[i]}`);
    opts[flag] = argv[++i];
  }
  for (const [k, v] of Object.entries(opts)) if (!v) throw new Error(`--${k} is required`);
  if (!(opts.surface in RUNNERS)) throw new Error(`--surface must be one of: ${Object.keys(RUNNERS)}`);
  return opts;
}

const { surface, fixtures: dir, out } = parseArgs(process.argv.slice(2));
const fixtures = fixturePaths(dir);
if (fixtures.length === 0) throw new Error(`no *.ags fixtures in ${dir}`);
mkdirSync(out, { recursive: true });
const payload = await RUNNERS[surface](fixtures);
const path = join(out, `${surface}.json`);
writeFileSync(path, JSON.stringify(payload));
console.log(
  `${surface} engine ${payload.version} (${payload.engine_sha}): ${payload.results.length} fixtures -> ${path}`,
);
