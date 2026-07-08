#!/usr/bin/env node
// `lat` — the Node CLI for laterite (npm package `laterite`). The third launcher
// of the one AGS4 tool, alongside the Rust binary and `uvx --from laterite lat`:
// the same verbs — validate / read / fix / diff / certify / rules / transport /
// excel — over the public API. Scriptable outputs (`--json` / `--ndjson`,
// `read --json` / `--csv`, `rules --json`) are byte-identical to the Rust binary;
// the human views are each surface's own presentation. A bare `lat <file>` is
// shorthand for `lat validate <file>`.
import { existsSync, readFileSync, writeFileSync } from "node:fs";

import { diff, fix, fromExcel, read, toExcel, transport, validate } from "./index";
import { listRules as rulesMetaJson, readGroupsRaw } from "./native";

const SUBCOMMANDS = new Set([
  "validate", "read", "fix", "diff", "certify", "rules",
  "pack", "unpack", "lock", "unlock", "excel",
]);

// Flags that consume the following token as their value (everything else is a
// boolean switch). Mirrors the Rust verbs' valued options.
const VALUED = new Set([
  "dict-version", "encoding", "out", "json-out", "index", "fix-out",
  "level", "log-n", "password-file",
]);

interface Parsed {
  positionals: string[];
  flags: Record<string, string | boolean>;
}

function parseArgs(argv: string[]): Parsed {
  const positionals: string[] = [];
  const flags: Record<string, string | boolean> = {};
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === undefined) continue;
    if (a === "--") {
      positionals.push(...argv.slice(i + 1));
      break;
    }
    if (a.startsWith("--")) {
      const eq = a.indexOf("=");
      if (eq >= 0) {
        flags[a.slice(2, eq)] = a.slice(eq + 1);
        continue;
      }
      const key = a.slice(2);
      const next = argv[i + 1];
      if (VALUED.has(key) && next !== undefined && !next.startsWith("--")) {
        flags[key] = next;
        i++;
      } else {
        flags[key] = true;
      }
    } else {
      positionals.push(a);
    }
  }
  return { positionals, flags };
}

function fail(msg: string, code: number): never {
  process.stderr.write(`error: ${msg}\n`);
  process.exit(code);
}

const note = (msg: string): void => void process.stderr.write(`${msg}\n`);
const str = (v: string | boolean | undefined): string | undefined =>
  typeof v === "string" ? v : undefined;

// Map a thrown engine error to the shared exit code (3 io, 4 not-ags4/bad-input,
// 5 bad-dict, 6 schema) — the Rust binary's scheme.
function exitCodeFor(e: unknown): number {
  const err = e as { kind?: string; name?: string; message?: string };
  if (err?.kind === "not_found" || /ENOENT/.test(err?.message ?? "")) return 3;
  if (err?.name === "BadDictError" || err?.kind === "bad_dict") return 5;
  if (
    err?.name === "NotAgs4Error" ||
    err?.name === "UnsupportedEditionError" ||
    err?.kind === "not_ags4" ||
    err?.kind === "unsupported_edition"
  ) {
    return 4;
  }
  return 6;
}

// Write to `--out <path>` (returning a note) or stdout.
function emit(body: string, out: string | undefined): void {
  if (out) {
    writeFileSync(out, body);
    note(`written to ${out}`);
  } else {
    process.stdout.write(body);
  }
}

// ---- read: byte-parity with the Rust `read` (raw file cells) ----------
function csvRow(cells: string[]): string {
  return `${cells.map((c) => (/[",\r\n]/.test(c) ? `"${c.replace(/"/g, '""')}"` : c)).join(",")}\n`;
}

function runRead(p: Parsed, json: boolean): number {
  const file = p.positionals[0];
  if (!file) fail("read needs a file", 5);
  if (!existsSync(file)) fail(`${file}: not found`, 3);
  let raw: { order: string[]; groups: Record<string, { headings: string[]; rows: string[][] }> };
  try {
    raw = JSON.parse(readGroupsRaw(file));
  } catch (e) {
    fail((e as Error).message, exitCodeFor(e));
  }
  const group = p.positionals[1];
  const out = str(p.flags["out"]);
  if (!group) {
    if (raw.order.length === 0) {
      note("no groups in the file");
      return 0;
    }
    emit(json ? `${JSON.stringify(raw.order, null, 2)}\n` : `${raw.order.join("\n")}\n`, out);
    return 0;
  }
  const g = raw.groups[group];
  if (!g) {
    const present = raw.order.length ? raw.order.join(", ") : "none";
    fail(`group "${group}" not found in ${file} (present: ${present})`, 4);
  }
  let body: string;
  if (json) {
    const objs = g.rows.map((row) => Object.fromEntries(g.headings.map((h, i) => [h, row[i] ?? ""])));
    body = `${JSON.stringify(objs, null, 2)}\n`;
  } else if (p.flags["csv"]) {
    body = csvRow(g.headings) + g.rows.map(csvRow).join("");
  } else {
    body = readTable(g.headings, g.rows);
  }
  emit(body, out);
  return 0;
}

function readTable(headings: string[], rows: string[][]): string {
  const w = headings.map((h, i) => Math.max(h.length, ...rows.map((r) => (r[i] ?? "").length)));
  const line = (cells: string[]) => cells.map((c, i) => c.padEnd(w[i] ?? 0)).join(" | ").replace(/\s+$/, "");
  const out = [line(headings), w.map((n) => "-".repeat(n)).join("-+-")];
  for (const r of rows) out.push(line(r));
  return `${out.join("\n")}\n`;
}

// ---- validate --------------------------------------------------------
function runValidate(p: Parsed, json: boolean, ndjson: boolean): number {
  const file = p.positionals[0];
  if (!file) fail("validate needs a file", 5);
  if (!existsSync(file)) fail(`${file}: not found`, 3);
  let report;
  try {
    report = validate(file, {
      index: str(p.flags["index"]),
      warnings: !p.flags["no-warnings"],
      fyi: !!p.flags["show-fyi"],
      dictVersion: str(p.flags["dict-version"]),
      checkFiles: !!p.flags["check-files"],
    });
  } catch (e) {
    fail((e as Error).message, exitCodeFor(e));
  }
  const out = str(p.flags["out"]);
  const jsonOut = str(p.flags["json-out"]);
  if (jsonOut) writeFileSync(jsonOut, report.toJson());
  if (json) {
    // The report's `toJson()` omits the trailing newline the Rust binary prints;
    // add it so `lat validate --json` is byte-identical across surfaces.
    emit(`${report.toJson()}\n`, out);
  } else if (ndjson) {
    emit(report.toNdjson(), out);
  } else {
    const head = `${file} — ${report.dictVersion} (${report.resolution})`;
    const lines = [head, report.isValid ? "  clean — no findings" : `  ${report.count} finding(s)`];
    if (!report.isValid) {
      for (const line of report.toNdjson().trimEnd().split("\n")) {
        const f = JSON.parse(line);
        lines.push(`    ${f.rule} (line ${f.line}, ${f.group}): ${f.desc}`);
      }
    }
    emit(`${lines.join("\n")}\n`, out);
  }
  return report.exitCode;
}

// ---- fix -------------------------------------------------------------
function runFix(p: Parsed): number {
  const file = p.positionals[0];
  if (!file) fail("fix needs a file", 5);
  if (!existsSync(file)) fail(`${file}: not found`, 3);
  let result;
  try {
    result = fix(file, {
      risky: !!p.flags["risky"],
      dictVersion: str(p.flags["dict-version"]),
    });
  } catch (e) {
    fail((e as Error).message, exitCodeFor(e));
  }
  const dest = p.flags["in-place"]
    ? file
    : (str(p.flags["fix-out"]) ?? file.replace(/(\.ags)?$/i, ".fixed.ags"));
  result.save(dest);
  note(`${result.toString()} → ${dest}`);
  return result.findings.length === 0 ? 0 : 1;
}

// ---- diff ------------------------------------------------------------
function runDiff(p: Parsed, json: boolean): number {
  const [a, b] = p.positionals;
  if (!a || !b) fail("diff needs two files", 5);
  for (const f of [a, b]) if (!existsSync(f)) fail(`${f}: not found`, 3);
  let delta;
  try {
    delta = diff(a, b, { dictVersion: str(p.flags["dict-version"]) });
  } catch (e) {
    fail((e as Error).message, exitCodeFor(e));
  }
  if (json) {
    process.stdout.write(`${JSON.stringify(delta, null, 2)}\n`);
    return 0;
  }
  const changed = delta.groups.filter((g) => g.added || g.removed || g.changed);
  if (changed.length === 0) {
    process.stdout.write("no differences\n");
  } else {
    for (const g of changed) {
      process.stdout.write(`${g.code}: +${g.added} -${g.removed} ~${g.changed}\n`);
    }
  }
  return 0;
}

// ---- certify ---------------------------------------------------------
function runCertify(p: Parsed): number {
  const file = p.positionals[0];
  if (!file) fail("certify needs a file", 5);
  if (!existsSync(file)) fail(`${file}: not found`, 3);
  try {
    const dest = read(file)
      .validate({
        warnings: !p.flags["no-warnings"],
        dictVersion: str(p.flags["dict-version"]),
        checkFiles: !!p.flags["check-files"],
      })
      .certify(str(p.flags["out"]));
    note(`certificate written to ${dest}`);
    return 0;
  } catch (e) {
    const msg = (e as Error).message;
    if (/cannot certify/i.test(msg)) {
      fail(msg, 1);
    }
    fail(msg, exitCodeFor(e));
  }
}

// ---- rules -----------------------------------------------------------
function runRules(json: boolean): number {
  if (json) {
    process.stdout.write(`${rulesMetaJson()}\n`);
    return 0;
  }
  const rules: Array<{ rule: string; title: string; severity: string; fixable: boolean }> =
    JSON.parse(rulesMetaJson());
  for (const r of rules) {
    process.stdout.write(`Rule ${r.rule}\t${r.severity}${r.fixable ? "\tfixable" : ""}\t${r.title}\n`);
  }
  return 0;
}

// ---- transport -------------------------------------------------------
function resolvePassword(p: Parsed, prompt: string): string {
  const file = str(p.flags["password-file"]);
  if (file) return readFileSync(file, "utf8").replace(/[\r\n]+$/, "");
  const env = process.env.LAT_TRANSPORT_PASSWORD;
  if (env) return env;
  // No TTY prompt library in the Node package; require a file/env so we never
  // read a passphrase from argv.
  fail(`${prompt} — set $LAT_TRANSPORT_PASSWORD or pass --password-file <path>`, 5);
}

function runTransport(verb: string, p: Parsed): number {
  const [input, output] = p.positionals;
  if (!input || !output) fail(`${verb} needs <in> <out>`, 5);
  if (!existsSync(input)) fail(`${input}: not found`, 3);
  const level = p.flags["level"] ? Number(p.flags["level"]) : undefined;
  try {
    if (verb === "pack") {
      transport.pack(input, output, level);
    } else if (verb === "unpack") {
      transport.unpack(input, output);
    } else if (verb === "lock") {
      const logN = p.flags["log-n"] ? Number(p.flags["log-n"]) : undefined;
      transport.lock(input, output, resolvePassword(p, "passphrase to lock with"), level, logN);
    } else {
      transport.unlock(input, output, resolvePassword(p, "passphrase to unlock"));
    }
  } catch (e) {
    fail((e as Error).message, 6);
  }
  note(`${verb === "unpack" || verb === "unlock" ? "restored" : verb + "ed"} ${input} → ${output}`);
  return 0;
}

// ---- excel -----------------------------------------------------------
function runExcel(p: Parsed): number {
  const [input, output] = p.positionals;
  if (!input || !output) fail("excel needs <in> <out>", 5);
  if (!existsSync(input)) fail(`${input}: not found`, 3);
  let exp: boolean;
  if (p.flags["export"]) exp = true;
  else if (p.flags["import"]) exp = false;
  else if (output.toLowerCase().endsWith(".xlsx")) exp = true;
  else if (output.toLowerCase().endsWith(".ags")) exp = false;
  else fail(`can't infer direction from output ${output} — pass --export (→ .xlsx) or --import (→ .ags)`, 5);
  try {
    if (exp) toExcel(input, output);
    else fromExcel(input, output, { formatNumericColumns: !p.flags["no-format-numeric"] });
  } catch (e) {
    fail((e as Error).message, 6);
  }
  note(`${exp ? "exported" : "imported"} ${input} → ${output}`);
  return 0;
}

// ---- entrypoint ------------------------------------------------------
export function main(argv: string[] = process.argv.slice(2)): number {
  const p = parseArgs(argv);
  const json = !!p.flags["json"];
  const ndjson = !!p.flags["ndjson"];
  if (json && ndjson) fail("--json and --ndjson are mutually exclusive", 5);

  // Bare `lat <file>` ≡ `lat validate <file>`: the first positional decides.
  let verb = p.positionals[0];
  if (!verb) fail("a subcommand or input file is required", 5);
  if (!SUBCOMMANDS.has(verb)) {
    verb = "validate";
  } else {
    p.positionals.shift();
  }

  switch (verb) {
    case "validate":
      return runValidate(p, json, ndjson);
    case "read":
      return runRead(p, json);
    case "fix":
      return runFix(p);
    case "diff":
      return runDiff(p, json);
    case "certify":
      return runCertify(p);
    case "rules":
      return runRules(json);
    case "pack":
    case "unpack":
    case "lock":
    case "unlock":
      return runTransport(verb, p);
    case "excel":
      return runExcel(p);
    default:
      fail(`unknown command ${verb}`, 5);
  }
}

