#!/usr/bin/env node
// `lat` — the Node CLI for laterite (npm package `laterite`). The third launcher
// of the one AGS4 tool, alongside the Rust binary and `uvx --from laterite lat`:
// the same verbs — validate / read / fix / diff / certify / rules / transport /
// excel — over the public API. Scriptable outputs (`--json` / `--ndjson`,
// `read --json` / `--csv`, `rules --json`) are byte-identical to the Rust binary;
// the human views are each surface's own presentation. A bare `lat <file>` is
// shorthand for `lat validate <file>`.
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { basename, dirname, extname, join } from "node:path";

import type { MergeOptions, Report, TranStamp, ValidateOptions } from "./index";
import {
  diff,
  fromExcel,
  merge,
  read,
  toExcel,
  transport,
  validate,
} from "./index";
import { StaleCertError } from "./errors";
import { FixResult } from "./fix-result";
import {
  editions as nativeEditions,
  engineFingerprint,
  fallbackEdition,
  fixFile,
  listRules as rulesMetaJson,
  readGroupsRaw,
  renderReadCsv,
  renderReadJson,
  resolveEncodingLabel,
  typeClashModes,
} from "./native";

// The verb table IS the dispatch table. It used to be a hand-written Set sitting
// beside a `switch` that did the real work — two lists that could disagree, and a
// census reading either one could lie about the other. Now there is one: `census()`
// dumps these keys, and `main` dispatches through them, so the npx launcher cannot
// advertise a verb it does not run, or run one it does not advertise.
//
// `merge` is the verb this table LOST: it shipped in the native binary (laterite-dev#494) and
// never arrived here, and no gate noticed — see tools/gen_census.py, which now
// diffs these keys against clap's and fails if one launcher is missing a door.
//
// Each verb now also declares the FLAGS it accepts, and that declaration is
// load-bearing three ways: it drives the value-vs-boolean parse, it is what rejects
// a flag the verb does not know, and it is what the census reports. One table.
//
// It used to be none. This launcher had a single GLOBAL set of valued flags and no
// per-verb notion of "accepted" at all — so it silently swallowed any flag on any
// verb, where clap rejects them per verb. That is not a tidiness problem, it is how
// two real bugs hid in plain sight:
//
//   * `--encoding` was accepted by every verb and honoured by none, so a legacy
//     cp1252 file was decoded as UTF-8 and the findings blamed the file.
//   * `--dict <custom.ags>` was accepted and IGNORED, so a user's project dictionary
//     was quietly dropped, the file was checked against the bundled one, and the tool
//     said "clean". `--dict` is now a real custom-dictionary overlay (laterite-dev#568), honoured
//     on every surface — an `.ags`/JSON dictionary layered over a base edition, with
//     `--dict-replace` for a full replacement.
interface Spec {
  run: (p: Parsed, json: boolean, ndjson: boolean) => number;
  /** Verb-specific long flags, without the `--`. Globals are added separately. */
  flags?: readonly string[];
  /** Which of `flags` consume the next token as their value. */
  valued?: readonly string[];
  /** The positional arguments this verb reads off `p.positionals`, in order and by
   *  the authority's names. Declared so the census can compare ARITY: a launcher
   *  that reads one path where clap requires two takes a command line the other two
   *  reject, and no amount of flag-name agreement would show it. */
  positionals?: readonly string[];
}

/** A flag's closed value set, where it has one. Keyed by flag long-name (without
 *  `--`), and sourced from the engine — never hand-listed. The census emits these
 *  as each arg's `values` so tools/gen_census.py can diff the modes a launcher
 *  accepts against the other two: the table that catches a surface offering a
 *  *different* set of modes, not just a different flag.
 *
 *  Computed lazily (a function, not a const) so the native addon is not called at
 *  module load — same reason `census()` calls `nativeEditions()` inline rather
 *  than baking it into a top-level table. `--dict-version` is deliberately absent:
 *  its set is the editions table, compared there, and represented differently by
 *  each parser framework (a per-arg copy would be double-reported false drift). */
function flagValueSets(): Record<string, readonly string[]> {
  return { "on-type-clash": typeClashModes() };
}

/** Accepted on every verb (a boolean). Mirrors clap's one remaining global arg —
 *  `--json`/`--ndjson` moved onto the report-producing verbs in laterite-dev#545, so a verb that
 *  can't render JSON rejects the flag (via `rejectUnknownFlags`) instead of ignoring it. */
const GLOBAL_FLAGS = ["quiet"] as const;

/** Shared by the verbs that resolve a dictionary edition + decode bytes. `dict-replace`
 *  is a boolean (no value), so it rides in `flags` but never in the `valued` lists. */
const DICT_FLAGS = [
  "dict",
  "dict-replace",
  "dict-version",
  "encoding",
] as const;

const SPECS: Record<string, Spec> = {
  validate: {
    run: (p, json, ndjson) => runValidate(p, json, ndjson),
    flags: [
      ...DICT_FLAGS,
      "check-files",
      "index",
      "json",
      "json-out",
      "ndjson",
      "no-warnings",
      "out",
      "show-fyi",
      "warnings-as-errors",
    ],
    valued: ["dict", "dict-version", "encoding", "index", "json-out", "out"],
    positionals: ["<file>"],
  },
  read: {
    run: (p, json) => runRead(p, json),
    flags: ["csv", "json", "out", "recover-duplicate-headings"],
    valued: ["out"],
    positionals: ["<file>", "<group>"],
  },
  fix: {
    run: (p, json) => runFix(p, json),
    flags: [...DICT_FLAGS, "fix-out", "in-place", "json", "risky"],
    valued: ["dict", "dict-version", "encoding", "fix-out"],
    positionals: ["<file>"],
  },
  diff: {
    run: (p, json) => runDiff(p, json),
    flags: [...DICT_FLAGS, "json"],
    valued: ["dict", "dict-version", "encoding"],
    positionals: ["<file>", "<other>"],
  },
  merge: {
    run: (p, json) => runMerge(p, json),
    flags: [
      ...DICT_FLAGS,
      "json",
      "on-type-clash",
      "out",
      "tran-date",
      "tran-description",
      "tran-issue",
      "tran-producer",
      "tran-recipient",
      "tran-remarks",
      "tran-status",
    ],
    valued: [
      "dict",
      "dict-version",
      "encoding",
      "on-type-clash",
      "out",
      "tran-date",
      "tran-description",
      "tran-issue",
      "tran-producer",
      "tran-recipient",
      "tran-remarks",
      "tran-status",
    ],
    positionals: ["<files>"],
  },
  certify: {
    run: (p) => runCertify(p),
    // `--check-files` was here (and on the binary, and on uvx). It recorded, in the
    // certificate, that Rule 20's on-disk half had run — and a later `validate
    // --check-files --index` read that record and skipped the check. Delete the FILE/
    // tree in between and the file still reported clean. A certificate is a statement
    // about bytes; the directory beside them is not one.
    flags: [...DICT_FLAGS, "out"],
    valued: ["dict", "dict-version", "encoding", "out"],
    positionals: ["<file>"],
  },
  rules: { run: (_p, json) => runRules(json), flags: ["json"] },
  pack: {
    run: (p) => runTransport("pack", p),
    flags: ["level"],
    valued: ["level"],
    positionals: ["<input>", "<output>"],
  },
  unpack: {
    run: (p) => runTransport("unpack", p),
    positionals: ["<input>", "<output>"],
  },
  lock: {
    run: (p) => runTransport("lock", p),
    flags: ["level", "log-n", "password-file"],
    valued: ["level", "log-n", "password-file"],
    positionals: ["<input>", "<output>"],
  },
  unlock: {
    run: (p) => runTransport("unlock", p),
    flags: ["password-file"],
    valued: ["password-file"],
    positionals: ["<input>", "<output>"],
  },
  excel: {
    run: (p) => runExcel(p),
    flags: ["export", "import", "no-format-numeric"],
    positionals: ["<input>", "<output>"],
  },
};
const SUBCOMMANDS = new Set(Object.keys(SPECS));

/** Every valued flag across every verb.
 *
 * Needed for ONE job: the pre-scan that finds which verb we are running. Until we
 * know the verb we cannot know its value-taking flags, so `--dict-version 4.2` would
 * otherwise leave `4.2` looking like a positional (and be mistaken for the verb).
 * The real parse then uses the verb's OWN set, and anything it does not declare is
 * rejected — so this union never widens what a verb accepts. */
const ANY_VALUED = new Set<string>(
  Object.values(SPECS).flatMap((s) => s.valued ?? []),
);

/** Encoding labels the surface census resolves on every launcher. Mirrors
 *  `ENCODING_PROBES` in `commands/census.rs`; a test pins the lists equal, so the
 *  launchers cannot end up answering different questions and calling it agreement. */
const ENCODING_PROBES = [
  "utf-8",
  "utf8",
  "cp1252",
  "windows-1252",
  "latin1",
  "latin-1",
  "iso-8859-1",
  "iso-8859-15",
  "latin9",
  "latin-9",
  "l9",
  "shift_jis",
  "cp1252x",
] as const;

/** This launcher's tables, as the shape `lat census` emits — the verb table
 *  reflected from HANDLERS (the dispatch itself), never a second list. Diffed
 *  against the native binary by tools/gen_census.py. */
export function census(): unknown {
  return {
    // See CENSUS_VERSION in the Rust census — bumped when a TABLE is added, so a
    // launcher built before a table existed fails loudly rather than reporting it
    // empty (which would read as "no drift").
    census_version: 6,
    surface: "cli-npx",
    authority: false,
    // The ENGINE this launcher carries, asked of the addon rather than restated
    // here — so it reports what it is actually running, not what this file was
    // written believing. Not a table and not in the census snapshot; see the note
    // at the same key in the Rust census for why.
    engine: engineFingerprint(),
    // Reflected from SPECS — the same declaration that parses argv and rejects an
    // unknown flag. Before, this reported `args: []` for every verb, because there
    // WAS no per-verb flag table: the census could only say "this launcher has no
    // opinion", which is exactly how a swallowed flag stays invisible.
    verbs: [...SUBCOMMANDS].sort().map((verb) => {
      const valueSets = flagValueSets();
      return {
        verb,
        args: [
          ...[...(SPECS[verb]?.flags ?? [])].sort().map((f) => ({
            name: `--${f}`,
            takes_value: (SPECS[verb]?.valued ?? []).includes(f),
            // The closed value set this flag accepts, where it has one. Empty
            // otherwise — matching the other launchers, which report `[]` for a
            // free-form or boolean flag. This is what laterite-dev#555 part 3b added: before,
            // the census had no per-arg value column, so a launcher accepting a
            // DIFFERENT set of `--on-type-clash` modes was invisible to the diff.
            values: [...(valueSets[f] ?? [])].sort(),
          })),
          // Positionals report as `<name>`, as clap does — so the census compares the
          // ARITY of each verb, not just its flags.
          ...(SPECS[verb]?.positionals ?? []).map((n) => ({
            name: n,
            takes_value: true,
            values: [] as string[],
          })),
        ].sort((a, b) => a.name.localeCompare(b.name)),
      };
    }),
    global_args: [...GLOBAL_FLAGS].sort().map((f) => ({
      name: `--${f}`,
      takes_value: false,
      values: [] as string[],
    })),
    documented_verbs: [...SUBCOMMANDS].sort(),
    // This launcher keeps NO edition table: `--dict-version` goes straight to the
    // engine, which validates it against the generated `DictVersion::ALL`. That is
    // the ideal state — nothing to drift — so the census reports the engine's list,
    // which is what this launcher actually accepts.
    editions: nativeEditions(),
    fallback_edition: fallbackEdition(),
    // What THIS surface makes of each label — via napi's `resolveEncodingLabel`,
    // which goes through the Rust crate's OWN wrapper, not the parse leaf. That is
    // the point: the leaf was always right, and the bug lived in the wrapper, which
    // turned every unknown label into a silent UTF-8 decode. `cp1252x` MUST be null.
    encodings: Object.fromEntries(
      ENCODING_PROBES.map((l) => [l, resolveEncodingLabel(l) ?? null]),
    ),
  };
}

interface Parsed {
  positionals: string[];
  flags: Record<string, string | boolean>;
}

function parseArgs(argv: string[], valued: Set<string>): Parsed {
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
      if (valued.has(key) && next !== undefined && !next.startsWith("--")) {
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

/** Which verb is this argv running? Decided BEFORE the real parse, because the
 *  verb is what tells us its value-taking flags. Uses the union set (`ANY_VALUED`)
 *  purely to skip over `--flag <value>` pairs while looking for the first bare
 *  token. An unrecognised first token means `lat <file>` ≡ `lat validate <file>`. */
function pickVerb(argv: string[]): string {
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === undefined) continue;
    if (a === "--") break;
    if (a.startsWith("--")) {
      const key = a.slice(2);
      const next = argv[i + 1];
      // `--flag=value` carries its value; `--flag value` eats the next token.
      if (
        !key.includes("=") &&
        ANY_VALUED.has(key) &&
        next !== undefined &&
        !next.startsWith("--")
      ) {
        i++;
      }
      continue;
    }
    return SUBCOMMANDS.has(a) || a === "census" ? a : "validate";
  }
  return "validate";
}

/** Refuse a flag the verb does not declare — what clap does, and what this launcher
 *  never did. A swallowed flag is worse than a rejected one: the user believes it took
 *  effect. `--encoding`, `--dict` and `--index` were all silently ignored this way,
 *  and so was any typo — `--no-warnigs` left the caller sure warnings were off. */
function rejectUnknownFlags(p: Parsed, verb: string, spec: Spec): void {
  const known = new Set<string>([...GLOBAL_FLAGS, ...(spec.flags ?? [])]);
  for (const key of Object.keys(p.flags)) {
    if (!known.has(key)) {
      fail(`\`${verb}\` does not accept --${key}`, 5);
    }
  }
}

function fail(msg: string, code: number): never {
  process.stderr.write(`error: ${msg}\n`);
  process.exit(code);
}

const note = (msg: string): void => void process.stderr.write(`${msg}\n`);
const str = (v: string | boolean | undefined): string | undefined =>
  typeof v === "string" ? v : undefined;

/** The edition pin as the LIBRARY spells it: an edition, or `undefined` for none.
 *
 * `--dict-version auto` is the CLI's sentinel for "no pin" (the Rust `parse_dv` maps
 * it to `None`), and the library has no such value. Handing `"auto"` through does not
 * fail loudly — it makes the request look like a FORCED edition, so a certificate
 * minted without one stops covering it and the `--index` skip quietly disarms. The
 * uvx launcher had exactly that bug the first time its `--index` ran. */
const pin = (p: Parsed): string | undefined => {
  const dv = str(p.flags["dict-version"]);
  return dv === "auto" ? undefined : dv;
};

// Map a thrown engine error to the shared exit code (3 io, 4 not-ags4/bad-input,
// 5 bad-dict, 6 schema) — the Rust binary's scheme.
function exitCodeFor(e: unknown): number {
  // A thrown value really can be null/undefined, so keep the type nullable —
  // that's what justifies the `?.` guards below (a bare cast would assert
  // non-null and make them look redundant).
  const err = e as
    { kind?: string; name?: string; message?: string } | null | undefined;
  if (err?.kind === "not_found" || /ENOENT/.test(err?.message ?? "")) return 3;
  if (
    err?.name === "BadDictError" ||
    err?.kind === "bad_dict" ||
    // An on-disk check with nothing on disk to check: an incoherent request, the
    // same class of mistake as a bogus --dict-version, so the same exit code.
    err?.name === "WorldCheckRequiresSourceError" ||
    err?.kind === "world_check_requires_source"
  ) {
    return 5;
  }
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
// The CSV/JSON bodies are rendered by the ENGINE (`renderReadCsv`/`renderReadJson`
// → core's single writers), not here. This file used to hand-port RFC-4180
// quoting and build the JSON with `JSON.stringify(x, null, 2)` while the binary
// used serde_json and Python used json.dumps — three libraries held
// byte-identical by discipline, with no gate on `read` output (laterite-dev#530).

function runRead(p: Parsed, json: boolean): number {
  const file = p.positionals[0];
  if (!file) fail("read needs a file", 5);
  if (!existsSync(file)) fail(`${file}: not found`, 3);
  // `read` does not declare `encoding` in SPECS (readGroupsRaw takes none on any
  // surface, and clap rejects the flag on the native `read` for the same reason), so
  // `rejectUnknownFlags` now refuses it before we ever get here. It used to be
  // silently swallowed, leaving the user believing their file was read as cp1252.
  let raw: {
    order: string[];
    groups: Record<string, { headings: string[]; rows: string[][] }>;
  };
  try {
    raw = JSON.parse(
      readGroupsRaw(file, Boolean(p.flags["recover-duplicate-headings"])),
    ) as typeof raw;
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
    emit(
      json
        ? `${JSON.stringify(raw.order, null, 2)}\n`
        : `${raw.order.join("\n")}\n`,
      out,
    );
    return 0;
  }
  const g = raw.groups[group];
  if (!g) {
    const present = raw.order.length ? raw.order.join(", ") : "none";
    fail(`group "${group}" not found in ${file} (present: ${present})`, 4);
  }
  let body: string;
  if (json) {
    body = renderReadJson(g.headings, g.rows);
  } else if (p.flags["csv"]) {
    body = renderReadCsv(g.headings, g.rows);
  } else {
    // The table stays local — a presentation choice, not a data format.
    body = readTable(g.headings, g.rows);
  }
  emit(body, out);
  return 0;
}

function readTable(headings: string[], rows: string[][]): string {
  const w = headings.map((h, i) =>
    Math.max(h.length, ...rows.map((r) => (r[i] ?? "").length)),
  );
  const line = (cells: string[]) =>
    cells
      .map((c, i) => c.padEnd(w[i] ?? 0))
      .join(" | ")
      .replace(/\s+$/, "");
  const out = [line(headings), w.map((n) => "-".repeat(n)).join("-+-")];
  for (const r of rows) out.push(line(r));
  return `${out.join("\n")}\n`;
}

// ---- validate --------------------------------------------------------
/** `--index <cert>`: consume the `.ags.idx` certificate, then validate.
 *
 * This flag was accepted and silently DROPPED before the flags census: the free
 * `validate()` takes `index` only because `ValidateOptions extends ReadOptions`,
 * and it never reads it — so `tsc` was happy while the cert went nowhere. Passing
 * a cert for an entirely different file changed nothing at all.
 *
 * The cert policy is the LIBRARY's (`read({index})` freshness-checks it, and
 * `Ags4File.validate()` decides whether it may stand in for an engine run) — not a
 * second hand-written copy here. The CLI adds only the binary's own recovery
 * posture: a cert that cannot be trusted is a NOTE, not an error. Re-validating is
 * always safe; refusing to run because the cert went stale would not be.
 */
function validateWithCert(
  file: string,
  index: string,
  opts: Omit<ValidateOptions, "text">,
): Report {
  try {
    // `.report` is optional only on a handle nobody validated yet; we just did.
    const report = read(file, { index, encoding: opts.encoding }).validate(
      opts,
    ).report;
    if (!report) throw new Error("validate() produced no report");
    return report;
  } catch (e) {
    if (!(e instanceof StaleCertError)) throw e;
    note(`note: --index not used (${e.message}); running the full check`);
    return validate(file, opts);
  }
}

function runValidate(p: Parsed, json: boolean, ndjson: boolean): number {
  const file = p.positionals[0];
  if (!file) fail("validate needs a file", 5);
  if (!existsSync(file)) fail(`${file}: not found`, 3);
  // The two dials contradict each other — one hides the warning tier, the other
  // makes it fatal — so refuse rather than silently pick a winner. `lat`'s clap
  // spells this `conflicts_with`; argparse's is `add_mutually_exclusive_group`.
  if (p.flags["no-warnings"] && p.flags["warnings-as-errors"])
    fail("--no-warnings and --warnings-as-errors cannot be used together", 5);
  const opts: ValidateOptions = {
    warnings: !p.flags["no-warnings"],
    fyi: !!p.flags["show-fyi"],
    warningsAsErrors: !!p.flags["warnings-as-errors"],
    dictVersion: pin(p),
    encoding: str(p.flags["encoding"]),
    checkFiles: !!p.flags["check-files"],
    dictionary: str(p.flags["dict"]),
    dictReplace: !!p.flags["dict-replace"],
  };
  const index = str(p.flags["index"]);
  let report;
  try {
    report =
      index === undefined
        ? validate(file, opts)
        : validateWithCert(file, index, opts);
  } catch (e) {
    fail((e as Error).message, exitCodeFor(e));
  }
  if (report.certified) {
    // The rule ENGINE was skipped — not "the file was not checked". If `--check-files`
    // was on, its on-disk half still ran (a certificate can never vouch for a directory),
    // so the wording must not imply otherwise. The same note, on the same stream, as the
    // binary and uvx: this is one tool behind three launchers.
    note(
      "note: certified clean by the .ags.idx certificate — rule engine skipped",
    );
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
    // What the report SHOWS is a question about findings, not about the verdict
    // — since #321 a file can pass with warnings on it, and printing
    // "clean — no findings" over a listed warning would be a lie. The verdict
    // leaves by `report.exitCode` alone.
    const lines = [
      head,
      report.count === 0
        ? "  clean — no findings"
        : `  ${report.count} finding(s)`,
    ];
    if (report.count !== 0) {
      for (const line of report.toNdjson().trimEnd().split("\n")) {
        // NDJSON is our own `lat-check --ndjson` output (one flat finding per
        // line), so the shape is known — assert it rather than reading `any`.
        const f = JSON.parse(line) as {
          rule: string;
          line: number;
          group: string;
          desc: string;
        };
        lines.push(`    ${f.rule} (line ${f.line}, ${f.group}): ${f.desc}`);
      }
    }
    emit(`${lines.join("\n")}\n`, out);
  }
  return report.exitCode;
}

// `delivery.ags` → `delivery.fixed.ags`; `data.txt` → `data.fixed.txt`; an
// extension-less `foo` → `foo.fixed`. The default fix destination — MUST match
// the Rust binary's `sibling_fixed_path` (common.rs) and uvx's
// `src.with_name(f"{src.stem}.fixed{src.suffix}")` (_cli.py), so the three
// launchers write the SAME filename. The old `file.replace(/(\.ags)?$/i, ".fixed.ags")`
// matched `(\.ags)?` ZERO-WIDTH at end-of-string, so `data.txt` became
// `data.txt.fixed.ags` and `data` became `data.fixed.ags` — npx alone.
function siblingFixedPath(file: string): string {
  const ext = extname(file); // ".txt" / ".ags" / "" — case preserved
  const stem = basename(file, ext);
  return join(dirname(file), ext ? `${stem}.fixed${ext}` : `${stem}.fixed`);
}

// ---- fix -------------------------------------------------------------
function runFix(p: Parsed, json: boolean): number {
  const file = p.positionals[0];
  if (!file) fail("fix needs a file", 5);
  if (!existsSync(file)) fail(`${file}: not found`, 3);
  // Native `fixFile` directly (not the library `fix()`): the `--dict` custom overlay
  // (laterite-dev#568) is a CLI flag, not a public `FixOptions` knob — mirrors the uvx launcher,
  // which likewise reaches `_native.fix_file(dict_path=…)` past the library `fix()`.
  const r = fixFile(
    file,
    undefined,
    undefined,
    str(p.flags["dict-version"]),
    str(p.flags["encoding"]),
    !!p.flags["risky"],
    undefined,
    undefined,
    str(p.flags["dict"]),
    undefined,
    !!p.flags["dict-replace"],
  );
  if (!r.ok) fail(r.error ?? "unknown error", r.exitCode);
  // Reuse `FixResult` so the one-line note is byte-identical to the library path.
  const result = new FixResult(r.fixed, r.residual, r.applied, r.dictVersion);
  const dest = p.flags["in-place"]
    ? file
    : (str(p.flags["fix-out"]) ?? siblingFixedPath(file));
  result.save(dest);
  const residual = result.findings.length;
  // --json: the machine-readable report replaces the human note (laterite-dev#545). Same shape
  // and key order as the native `lat fix --json` / uvx — `applied` is the native
  // `fixFile` `{kind, label, rule, line, risk}` ledger; `residual` is the count.
  // (`risky_available` is human-only: `FixReport` has no risky-count to mirror.)
  if (json) {
    // Rebuild each entry explicitly: a whole-file fix has no `line`, and napi maps
    // that `Option<u32>::None` to `undefined`, which `JSON.stringify` DROPS — so the
    // key would vanish here while the Rust/Python `null` stays. `?? null` pins the
    // field present, keeping the three launchers' bytes identical.
    const applied = result.applied.map((f) => ({
      kind: f.kind,
      label: f.label,
      rule: f.rule,
      line: f.line ?? null,
      risk: f.risk,
    }));
    const report = { file, dest, applied, residual };
    process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
    return residual === 0 ? 0 : 1;
  }
  // STDOUT, not note(): the applied/residual line is the RESULT, and the
  // agent-first contract routes resolved-mode results to stdout — the other
  // two launchers already print theirs there, so this was a content-reaching
  // stream divergence, not layout (#542). The distinct fix KINDS ride along
  // because they are a fact the other two state (the content gate found this
  // launcher omitting them) — sorted, so the set can't reorder per run.
  const kinds = [...new Set(result.applied.map((a) => a.kind))].sort();
  const kindNote = kinds.length ? ` [${kinds.join(", ")}]` : "";
  process.stdout.write(`${result.toString()}${kindNote} → ${dest}\n`);
  return residual === 0 ? 0 : 1;
}

// ---- diff ------------------------------------------------------------
function runDiff(p: Parsed, json: boolean): number {
  const [a, b] = p.positionals;
  if (!a || !b) fail("diff needs two files", 5);
  for (const f of [a, b]) if (!existsSync(f)) fail(`${f}: not found`, 3);
  let delta;
  try {
    delta = diff(a, b, {
      dictVersion: str(p.flags["dict-version"]),
      encoding: str(p.flags["encoding"]),
    });
  } catch (e) {
    fail((e as Error).message, exitCodeFor(e));
  }
  if (json) {
    // The engine's own render, verbatim — the same bytes the other two
    // launchers print (#542) — not a re-stringify of the parsed object.
    process.stdout.write(`${delta.toJson()}\n`);
    return 0;
  }
  // Layout is this launcher's own; the FACTS are the contract's (#542). The
  // other two state the a → b header, every delta group (a heading-only change
  // is a delta — the old `added || removed || changed` filter dropped those),
  // the group add/remove lines and the totals, so this launcher must too.
  const lines = [`${a} → ${b}`];
  for (const g of delta.groups) {
    lines.push(`${g.code}: +${g.added} -${g.removed} ~${g.changed}`);
  }
  if (delta.groups_added.length) {
    lines.push(`groups added: ${delta.groups_added.join(", ")}`);
  }
  if (delta.groups_removed.length) {
    lines.push(`groups removed: ${delta.groups_removed.join(", ")}`);
  }
  lines.push(
    `total: +${delta.total_added} -${delta.total_removed} ~${delta.total_changed}`,
  );
  process.stdout.write(`${lines.join("\n")}\n`);
  return 0;
}

// ---- merge -----------------------------------------------------------
// `lat merge <files...> --out <merged.ags>` — reconcile N deliveries of one project
// into a single file. Argument order IS the authority (the last file wins a KEY
// conflict), and `--out` is required rather than defaulted: writing over one of the
// inputs by default would make a destructive merge the easy path.

/** Fold the five `--tran-*` flags into the one object the library takes.
 *
 * CLI flags are irreducibly five independent optionals, so the flattening has to
 * happen somewhere; doing it here keeps `TranStamp`'s "all five or none" rule as
 * the single arbiter — the native side rejects a partial object by name. A flag
 * set to the empty string counts as unset, matching `from_parts`. */
function tranFromFlags(
  flags: Record<string, string | boolean>,
): TranStamp | undefined {
  const issue = str(flags["tran-issue"]);
  const date = str(flags["tran-date"]);
  const producer = str(flags["tran-producer"]);
  const recipient = str(flags["tran-recipient"]);
  const status = str(flags["tran-status"]);
  // OTHER headings, so outside the all-five-or-none rule: stating one alone is
  // not a partial stamp, it is a stamp with no ISNO — which the library still
  // refuses, in its own words.
  const description = str(flags["tran-description"]);
  const remarks = str(flags["tran-remarks"]);
  if (
    !issue &&
    !date &&
    !producer &&
    !recipient &&
    !status &&
    !description &&
    !remarks
  )
    return undefined;
  // Deliberately NOT validated here: passing the partial object through means
  // the error text comes from the one place that owns the rule, so the CLI can
  // never disagree with the library about what a complete stamp is.
  return {
    issue,
    date,
    producer,
    recipient,
    status,
    description,
    remarks,
  } as TranStamp;
}

function runMerge(p: Parsed, json: boolean): number {
  const files = p.positionals;
  if (files.length < 2) fail("merge needs at least two files", 5);
  for (const f of files) if (!existsSync(f)) fail(`${f}: not found`, 3);
  const out = str(p.flags["out"]);
  if (!out) fail("merge needs --out <path>", 5);

  const clash = str(p.flags["on-type-clash"]) ?? "error";
  // Reject an unknown mode instead of letting it fall through as `undefined` (which
  // would silently mean "error" — a typo'd `--on-type-clash promot` would then refuse
  // the merge and look like a real type clash). The accepted set and the message are
  // BOTH the engine's `TypeClashMode::ALL` (laterite-dev#555) — this was two hand-typed copies of
  // the modes, which a fourth mode would have reached through neither.
  const modes = typeClashModes();
  if (!modes.includes(clash)) {
    fail(`--on-type-clash: unknown mode '${clash}' (${modes.join(", ")})`, 5);
  }
  // `modes` is `string[]` (from the native binding), so `.includes()` cannot
  // narrow `clash` to the option's literal union the way the old literal `!==`
  // chain did. The `fail` above returns `never` for anything outside the set, so
  // by here `clash` IS one of the modes. Cast via the EXISTING option type rather
  // than re-typing the literal triple here — that would just re-add the hand-copy
  // this change removes.
  const onTypeClash = clash as NonNullable<MergeOptions["onTypeClash"]>;

  let res;
  try {
    res = merge(files, {
      onTypeClash,
      dictVersion: str(p.flags["dict-version"]),
      encoding: str(p.flags["encoding"]),
      tran: tranFromFlags(p.flags),
    });
  } catch (e) {
    fail((e as Error).message, exitCodeFor(e));
  }

  try {
    writeFileSync(out, res.bytes);
  } catch (e) {
    fail(`writing ${out}: ${(e as Error).message}`, 3);
  }

  if (json) {
    // The TS library exposes `winnerFile` (camelCase is right for a TS API), but
    // `--json` is a WIRE contract shared with the Rust binary and the uvx launcher,
    // and there it is `winner_file`. Translate at the CLI boundary rather than
    // letting the language's naming convention leak into the format — a script
    // reading `.revisions[].winner_file` must not care which launcher ran.
    const revisions = res.revisions.map((r) => ({
      group: r.group,
      key: r.key,
      changed: r.changed,
      winner_file: r.winnerFile,
    }));
    process.stdout.write(
      `${JSON.stringify({ out, bytes: res.bytes.length, warnings: res.warnings, revisions }, null, 2)}\n`,
    );
    return 0;
  }
  process.stdout.write(
    `merged ${files.length} files → ${out} (${res.bytes.length} bytes)\n`,
  );
  for (const w of res.warnings)
    process.stdout.write(`  warning [${w.kind}]: ${w.message}\n`);
  if (res.revisions.length > 0) {
    process.stdout.write(`  ${res.revisions.length} row revision(s):\n`);
    for (const r of res.revisions) {
      process.stdout.write(
        `    ${r.group} ${JSON.stringify(r.key)}: changed ${JSON.stringify(r.changed)} (from file[${r.winnerFile}])\n`,
      );
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
    // The encoding belongs on `read` — that is where the bytes are decoded. It used
    // to be dropped here, so `lat certify --encoding cp1252` silently decoded the
    // file as UTF-8 and refused to certify it over findings that were artefacts of
    // the wrong decoder.
    // No pre-validation here. `certify()` runs the rules itself, with every tier on, and
    // records what they actually returned — this launcher used to validate first and
    // hand the verdict over, which made the certificate's contents an assertion by the
    // caller rather than a measurement by the engine.
    const dest = read(file, { encoding: str(p.flags["encoding"]) }).certify(
      str(p.flags["out"]),
      {
        dictVersion: str(p.flags["dict-version"]),
        dictionary: str(p.flags["dict"]),
        dictReplace: !!p.flags["dict-replace"],
      },
    );
    // STDOUT, like the binary and uvx. This line is the verb's RESULT, not a progress
    // note — `CERT=$(lat certify f.ags)` is the obvious way to use it, and this
    // launcher alone wrote it to stderr, handing every such script an empty string.
    // A stream is not something a knob-name gate can compare, and none of ours did.
    process.stdout.write(`certificate written to ${dest}\n`);
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
  // `rules_meta.json` is an OBJECT — `{schema_version, rules: [...]}` — not a bare
  // array. This used to annotate the parse as `Array<...>` and iterate it directly,
  // which TypeScript accepted (JSON.parse returns `any`, so the annotation was an
  // unchecked assertion) and which threw `rules is not iterable` at runtime: the
  // human `lat rules` crashed on this launcher alone, while `--json` — the only path
  // the tests covered — was fine.
  const { rules } = JSON.parse(rulesMetaJson()) as {
    rules: Array<{
      rule: string;
      title: string;
      severity: string;
      fixable: boolean;
    }>;
  };
  for (const r of rules) {
    process.stdout.write(
      `Rule ${r.rule}\t${r.severity}${r.fixable ? "\tfixable" : ""}\t${r.title}\n`,
    );
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
  fail(
    `${prompt} — set $LAT_TRANSPORT_PASSWORD or pass --password-file <path>`,
    5,
  );
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
      transport.lock(
        input,
        output,
        resolvePassword(p, "passphrase to lock with"),
        level,
        logN,
      );
    } else {
      transport.unlock(
        input,
        output,
        resolvePassword(p, "passphrase to unlock"),
      );
    }
  } catch (e) {
    fail((e as Error).message, 6);
  }
  note(
    `${verb === "unpack" || verb === "unlock" ? "restored" : verb + "ed"} ${input} → ${output}`,
  );
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
  else
    fail(
      `can't infer direction from output ${output} — pass --export (→ .xlsx) or --import (→ .ags)`,
      5,
    );
  try {
    if (exp) toExcel(input, output);
    else
      fromExcel(input, output, {
        formatNumericColumns: !p.flags["no-format-numeric"],
      });
  } catch (e) {
    fail((e as Error).message, 6);
  }
  note(`${exp ? "exported" : "imported"} ${input} → ${output}`);
  return 0;
}

// ---- the shipped guide ----------------------------------------------
//
// `lat --readme` and `lat <verb> --help` (#509). This launcher had neither: the
// flag fell through to `rejectUnknownFlags` and exited 5, so there was no working
// help path at all, while `README-cli.md`'s own Usage block promises one.
//
// The text is the guide the OTHER two launchers print — `gen_cli_readme.py`
// mirrors the one authority into this package and `tests/test_cli_readme_mirrors.py`
// holds them byte-identical. Writing Node its own help would have been a fourth
// description of the same flags, which is the class of defect #509 is about.
//
// `../README-cli.md` resolves the same in all three places this module runs from:
// the published package (`dist/cli.mjs` → package root), a repo build (same), and
// vitest importing `ts/cli.ts` directly (→ the package dir). One path, no probing.
const README_URL = new URL("../README-cli.md", import.meta.url);

/** A `## ` heading and the block under it, keyed by heading. */
function readmeSections(): Map<string, string> {
  const text = readFileSync(README_URL, "utf8");
  const out = new Map<string, string>();
  const re = /^## (.+?)\n([\s\S]*?)(?=^## |$(?![\s\S]))/gm;
  for (const m of text.matchAll(re)) {
    if (m[1] !== undefined) out.set(m[1], m[0].replace(/\s+$/, ""));
  }
  return out;
}

/** The section documenting one verb, or undefined.
 *
 *  Matched against the WORDS of a heading, not its first token: `## transport —
 *  pack / unpack / lock / unlock` documents four verbs at once, and a first-token
 *  rule drops all four back to the full guide — silently, which is the shape of
 *  the defect being fixed. `_readme_section` in `_cli.py` is the same rule. */
function verbHelp(verb: string): string | undefined {
  const sections = readmeSections();
  for (const [heading, block] of sections) {
    const words: string[] =
      heading.toLowerCase().match(/[a-z][a-z0-9-]*/g) ?? [];
    if (words.includes(verb)) {
      // Appended for the same reason clap lists them under every verb: `--quiet`
      // and the dictionary flags belong to no single verb, and a scoped help that
      // hid them sends the reader back to the document they were avoiding.
      const globals = sections.get("Global options");
      return globals ? `${block}\n\n${globals}`.replace(/\s+$/, "") : block;
    }
  }
  return undefined;
}

/** Which verb is this argv asking for help about, or undefined for the whole guide.
 *
 *  Deliberately NOT plain `pickVerb`, which answers `validate` for an argv with no
 *  verb AND no file — that would scope a bare `lat --help` to validate and hide
 *  the rest of the guide.
 *
 *  But `lat <file> --help` DOES scope, to validate: that is the same shorthand the
 *  dispatch applies, and what the binary answers, since its argv pre-scan splices
 *  the default verb in before clap sees the flag. So the rule is pickVerb's, with
 *  "there is nothing here to work on" carved out. */
function helpVerb(argv: string[]): string | undefined {
  const explicit = argv.find((a) => SUBCOMMANDS.has(a));
  if (explicit !== undefined) return explicit;
  // The help flags come out first: `parseArgs` only knows `--`-prefixed flags, so
  // a bare `-h` reads as a POSITIONAL and `lat -h` would scope itself to validate
  // — which it did, until the whole-guide case caught it.
  const rest = argv.filter((a) => a !== "--help" && a !== "-h");
  return parseArgs(rest, ANY_VALUED).positionals.length > 0
    ? "validate"
    : undefined;
}

// ---- entrypoint ------------------------------------------------------
export function main(argv: string[] = process.argv.slice(2)): number {
  // Before the verb dispatch and before any parsing, so `--help` beats a missing
  // required argument the way clap orders it: `lat certify --help` must print help
  // rather than "a subcommand or input file is required".
  if (argv.includes("--readme")) {
    process.stdout.write(`${readFileSync(README_URL, "utf8")}\n`);
    return 0;
  }
  if (argv.includes("--help") || argv.includes("-h")) {
    const verb = helpVerb(argv);
    const section = verb === undefined ? undefined : verbHelp(verb);
    // A verb with no section is a documentation gap, not a reason to print an
    // empty page. `cli-help.test.ts` fails on it; a reader meanwhile gets the
    // document that does answer them.
    process.stdout.write(`${section ?? readFileSync(README_URL, "utf8")}\n`);
    return 0;
  }

  // The verb first — it is what determines which flags take a value, and which are
  // accepted at all. (A single global flag table is what let `--encoding` and
  // `--dict` be swallowed by verbs that never used them.)
  const verb = pickVerb(argv);

  // The census is a hidden machine door: not in SPECS, so it is not a user verb and
  // never gets `validate` spliced in front of it.
  if (verb === "census") {
    process.stdout.write(`${JSON.stringify(census(), null, 2)}\n`);
    return 0;
  }

  const spec = SPECS[verb];
  if (!spec) fail(`unknown command ${verb}`, 5);

  const p = parseArgs(argv, new Set(spec.valued ?? []));
  // Drop the verb itself from the positionals when it was written out explicitly;
  // a bare `lat <file>` never put one there.
  if (p.positionals[0] === verb) p.positionals.shift();

  const json = !!p.flags["json"];
  const ndjson = !!p.flags["ndjson"];
  if (json && ndjson) fail("--json and --ndjson are mutually exclusive", 5);
  rejectUnknownFlags(p, verb, spec);

  if (p.positionals.length === 0 && verb === "validate" && argv.length === 0) {
    fail("a subcommand or input file is required", 5);
  }
  return spec.run(p, json, ndjson);
}
