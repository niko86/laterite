// The `laterite` package surface — the Node port of laterite-py's `__init__.py`.
// P2 is the engine-free read/validate/emit core (Arrow-direct); the optional
// DuckDB `sql()`/`at()` layer + typed-graph + transport land in P3.
import { type Table, tableFromArrays, tableToIPC } from "apache-arrow";
import { Ags4File } from "./ags4-file";
import { BuildResult } from "./build-result";
import { Ags4Error, fromNativeError, makeError, raiseFor } from "./errors";
import { FixResult } from "./fix-result";
import {
  type GroupIpc,
  emitAgs4FromIpc,
  fixFile,
  listRules as nativeListRules,
  parseArrow,
  runCheck,
} from "./native";
import * as registry from "./registry";
import { Report } from "./report";
import { AgsGroup } from "./typed-graph";

export interface ReadOptions {
  /** Validate/parse in-memory text instead of a file path. */
  text?: string;
  /** Source encoding label (`"utf-8"` default, `"windows-1252"`, …). */
  encoding?: string;
}

/**
 * Parse AGS4 into an `Ags4File`. Where the data→AGS4 door (`buildAgs4`)
 * *constructs* a file, `read` *loads* one that already exists — from a file
 * `source` path, raw `Uint8Array`/`Buffer` bytes, or in-memory `opts.text`.
 *
 * Prefer bytes over a string for large inputs: V8 caps a single string at
 * ~512 MB, but a `Uint8Array` does not, so a web backend can hand a
 * multi-hundred-MB upload straight in without first stringifying it. A string
 * `source` is the file path; non-string bytes are the raw content; an absent
 * `source` means the input came in as `opts.text`.
 *
 * The engine never returns a soft failure for un-parseable input — it throws a
 * typed error (the same exit-code identity as `lat-check`) so callers branch on
 * the class, not a brittle message match.
 *
 * @param source - File path (string), raw `Uint8Array`/`Buffer` bytes, or
 *   omitted when the input is supplied via `opts.text`.
 * @param opts - Read options ({@link ReadOptions}): in-memory `text` and the
 *   source `encoding` label (default `"utf-8"`).
 * @returns An {@link Ags4File} wrapping the parsed groups.
 * @throws {FileNotFoundError} The path could not be opened.
 * @throws {NotAgs4Error} The input has no GROUP rows / is not decodable as AGS4.
 * @throws {UnsupportedEditionError} A recognised but unsupported edition (e.g. AGS3).
 * @throws {Ags4Error} Any other native parse failure (the fallback mapping).
 */
export function read(source?: string | Uint8Array, opts: ReadOptions = {}): Ags4File {
  const path = typeof source === "string" ? source : undefined;
  const data = typeof source === "string" || source == null ? undefined : source;
  try {
    return new Ags4File(parseArrow(path, opts.text, data, opts.encoding));
  } catch (e) {
    throw fromNativeError(e);
  }
}

export interface ValidateOptions extends ReadOptions {
  /** Force an edition (`"4.0.3"`…`"4.2"`); default auto-detects from `TRAN_AGS`. */
  dictVersion?: string;
  /** Include warning-severity findings. */
  warnings?: boolean;
  /** Include FYI-severity findings. */
  fyi?: boolean;
  /** Also run Rule 20's on-disk half (the sibling `FILE/` tree must exist). */
  checkFiles?: boolean;
}

/** Validate AGS4 — a file `source` path, raw `Uint8Array`/`Buffer` bytes, or
 * in-memory `text` (via `opts.text`) — against the numbered AGS4 rules, returning
 * a `Report`. The crucial distinction: rule *violations* are data, not errors —
 * they come back inside the `Report` (`.findings`, `.byRule()`, `.isValid`); only
 * *un-validatable* input (missing file, not AGS4, AGS3, bad dictionary) throws.
 * By default the report is the error-tier findings; opt warnings and FYIs in with
 * `opts.warnings` / `opts.fyi`. The edition auto-detects from `TRAN_AGS` unless
 * pinned with `opts.dictVersion`. Pass bytes (not a string) for large inputs:
 * V8 caps strings at ~512 MB but a `Uint8Array` does not, so a web backend can
 * hand a multi-hundred-MB upload straight in — the same byte door as `read`.
 * Mirrors `laterite.validate()` / the `lat-check` binary.
 *
 * @param source File path (string), or raw bytes (`Uint8Array`/`Buffer`); omit to
 *   validate `opts.text` instead.
 * @param opts Validation knobs (`ValidateOptions`) — the dictionary-version pin
 *   (`dictVersion`), severity gates, in-memory text/encoding, and the on-disk
 *   Rule-20 toggle; see the interface for each field.
 * @returns A `Report` carrying the findings, the resolved `dictVersion`, the
 *   finding `count`, `isValid`, and `lat-check`-faithful `toJson()` / `toNdjson()`.
 * @throws {FileNotFoundError} the path could not be opened.
 * @throws {NotAgs4Error} the input has no GROUP rows / is not decodable AGS4.
 * @throws {UnsupportedEditionError} a recognised-but-unsupported edition (AGS3).
 * @throws {BadDictError} an invalid `opts.dictVersion` / unimplemented dictionary.
 */
export function validate(source?: string | Uint8Array, opts: ValidateOptions = {}): Report {
  const path = typeof source === "string" ? source : undefined;
  const data = typeof source === "string" || source == null ? undefined : source;
  const r = runCheck(
    path,
    opts.text,
    data,
    opts.dictVersion,
    opts.warnings,
    opts.fyi,
    opts.checkFiles,
    opts.encoding,
  );
  return new Report(raiseFor(r));
}

/** Row-oriented group data: an array of `{HEADING: value}` objects. */
export type GroupRows = Array<Record<string, unknown>>;
/** One group's data for `buildAgs4` — an arrow-js `Table` or row objects. */
export type GroupData = Table | GroupRows;

export interface EmitOptions {
  /** `"4.0.3" | "4.0.4" | "4.1" | "4.1.1" | "4.2"` (default `"4.1.1"`). */
  dictVersion?: string;
  /** `"autofix"` (default) | `"report"` | `"strict"`. */
  mode?: "autofix" | "report" | "strict";
}

/** Transpose row objects → an arrow-js Table (column types inferred from the
 * JS values — number→Float64, string→Utf8, …; the native producer then
 * canonicalises each cell to its AGS4 type). */
function rowsToTable(rows: GroupRows): Table {
  const columns = new Map<string, unknown[]>();
  for (const row of rows) {
    for (const key of Object.keys(row)) {
      if (!columns.has(key)) columns.set(key, []);
    }
  }
  for (const row of rows) {
    for (const [key, values] of columns) values.push(row[key] ?? null);
  }
  const obj: Record<string, unknown[]> = {};
  for (const [key, values] of columns) obj[key] = values;
  return tableFromArrays(obj);
}

/** Walk a typed-graph tree (`new PROJ({…, locas:[new LOCA({…})]})`) depth-first
 * into per-group row buckets — every declared heading becomes a column (null if
 * unset); child arrays are recursed via the registry's parent→child links. */
function walkTree(root: AgsGroup): Array<[string, GroupRows]> {
  const buckets = new Map<string, GroupRows>();
  const visit = (node: AgsGroup): void => {
    const code = (node.constructor as { code?: string }).code;
    const desc = code !== undefined ? registry.get(code) : undefined;
    if (code === undefined || desc === undefined) {
      throw new Ags4Error("buildAgs4: not a known typed AGS group instance");
    }
    const record = node as unknown as Record<string, unknown>;
    const row: Record<string, unknown> = {};
    for (const h of desc.headings) row[h.name] = record[h.name] ?? null;
    if (!buckets.has(code)) buckets.set(code, []);
    buckets.get(code)!.push(row);
    for (const child of registry.childGroups(code)) {
      const children = record[`${child.code.toLowerCase()}s`];
      if (Array.isArray(children)) for (const c of children) visit(c as AgsGroup);
    }
  };
  visit(root);
  return [...buckets];
}

/**
 * Build valid AGS4 from your own data — the data→AGS4 door. Where `read` loads
 * an *existing* file, `buildAgs4` *constructs* a new one: it lays the groups out
 * in order, fills UNIT/TYPE from the chosen `dictVersion`, then runs the output
 * through the validator (the `mode` knob on `opts` decides what happens to the
 * findings — e.g. `"autofix"` applies the safe fixes, `"report"` merely records
 * them). The returned `BuildResult` carries the bytes, the residual `findings`,
 * and a `fixesApplied` count; persist it with `BuildResult.save`. Needs no DuckDB.
 *
 * `groups` accepts two shapes. A **typed-graph root** (`new PROJ({…, locas:[new
 * LOCA({…})]})`) is walked depth-first via the registry's parent→child links,
 * every declared heading becoming a column (null if unset). Or pass a **Map /
 * array of `[code, data]`** entries where `data` is an arrow-js `Table` or row
 * objects whose **keys are the AGS headings** (`LOCA_ID`, …). Either way group
 * order is preserved, so put `PROJ` first.
 *
 * @param groups The data to emit — a typed-graph root (`new PROJ({…})`), or a
 *   `Map`/array of `[groupCode, Table | rowObjects]` entries (headings as keys).
 * @param opts Emit options (`dictVersion`, `mode`); see {@link EmitOptions}.
 * @returns A {@link BuildResult} — `.bytes`/`.text` of the AGS4 document, the
 *   `findings` it could not fix, the `fixesApplied` count, and `.save(path)`.
 * @throws {Ags4Error} If a typed-graph node is not a registered AGS group.
 * @throws If the native emitter rejects the input (e.g. an unknown `dictVersion`).
 */
export function buildAgs4(
  groups: AgsGroup | Map<string, GroupData> | Array<[string, GroupData]>,
  opts: EmitOptions = {},
): BuildResult {
  const items: Array<[string, GroupData]> =
    groups instanceof AgsGroup
      ? walkTree(groups)
      : groups instanceof Map
        ? [...groups]
        : groups;
  const ipcGroups: GroupIpc[] = items.map(([code, data]) => {
    const table = Array.isArray(data) ? rowsToTable(data) : data;
    return { code, ipc: Buffer.from(tableToIPC(table, "stream")) };
  });
  const res = emitAgs4FromIpc(ipcGroups, opts.dictVersion, opts.mode);
  const byRule = JSON.parse(res.findingsJson) as Record<string, Array<Record<string, unknown>>>;
  const findings = Object.entries(byRule).flatMap(([rule, list]) =>
    list.map((f) => ({ rule, ...f })),
  );
  return new BuildResult(res.bytes, findings, res.fixesApplied);
}

/** One cited divergence observation on a rule (`{id, note}`). */
export interface RuleObservation {
  id: string;
  note: string;
}

/** One rule's catalogue entry — the gated `rules_meta.json` shape. */
export interface RuleMeta {
  rule: string;
  title: string;
  checks: string;
  severity: string;
  fixable: boolean;
  observations: RuleObservation[];
}

/**
 * The AGS4 rule catalogue — one `RuleMeta` per numbered rule, surfaced so a
 * caller can show *which* rules exist and how each behaves before (or instead
 * of) running `validate`/`fix` over a file. Each entry carries the rule id and
 * title, a one-line `checks` summary, its `severity`, whether `fix` can
 * mechanically repair it (`fixable`), and any cited `O-N` divergence
 * observations. Mirrors `laterite.list_rules()` / `lat-check --list-rules`;
 * backed by the gated `rules_meta.json` (the catalogue is static, so this takes
 * no input and reads no file).
 *
 * @returns The full rule catalogue as a `RuleMeta[]` — see `RuleMeta` for the
 *   per-entry shape (`rule`, `title`, `checks`, `severity`, `fixable`,
 *   `observations`).
 */
export function listRules(): RuleMeta[] {
  return (JSON.parse(nativeListRules()) as { rules: RuleMeta[] }).rules;
}

export interface FixOptions {
  /** Repair in-memory `text` instead of a file path. */
  text?: string;
  /** Force an edition (`"4.0.3"`…`"4.2"`); default auto-detects from `TRAN_AGS`. */
  dictVersion?: string;
  /** Source encoding label (`"utf-8"` default, `"windows-1252"`, …). */
  encoding?: string;
  /** Also apply the intent-guessing (risky) fixes, not just the safe set. */
  risky?: boolean;
}

/** Mechanically repair AGS4 — the headless twin of the browser's Fix engine.
 * `source` is a file path, raw `Uint8Array`/`Buffer` bytes, or (via `opts.text`)
 * in-memory text. The *safe* fixes — CRLF / BOM / embedded-CR normalisation,
 * short-row padding, numeric reformatting, and the TRAN delimiter+concatenator
 * rows — are always applied; pass `risky` to also run the intent-guessing set
 * (duplicate-heading rename, `dd/mm` datetime canonicalisation, smart-quote→ASCII
 * typography). The repaired bytes are re-validated, so `FixResult.findings` is
 * what could NOT be mechanically fixed.
 *
 * Non-destructive: nothing is written here — the repaired bytes come back on the
 * result (`.bytes` / `.text` / `.save(path)`), already UTF-8 with no BOM, so
 * fixing a non-UTF-8 file also normalises its encoding. Mirrors `laterite.fix()`
 * / `lat-check --fix`.
 *
 * @param source - The AGS4 input: a filesystem path (`string`) or raw bytes
 *   (`Uint8Array`/`Buffer`). Omit to repair `opts.text` instead.
 * @param opts - {@link FixOptions} — `text` source, `risky` fixes, `dictVersion`
 *   override, and source `encoding`.
 * @returns A {@link FixResult} carrying the repaired `bytes` (and `.text` /
 *   `.save`), the `applied` fixes (with `fixesApplied` count), the residual
 *   `findings` left after re-validation, and the resolved `dictVersion`.
 * @throws {Ags4Error} (or a subclass — {@link FileNotFoundError},
 *   {@link NotAgs4Error}, {@link UnsupportedEditionError}, {@link BadDictError})
 *   for un-fixable input, carrying the matching `lat-check` exit code.
 */
export function fix(source?: string | Uint8Array, opts: FixOptions = {}): FixResult {
  const path = typeof source === "string" ? source : undefined;
  const data = typeof source === "string" || source == null ? undefined : source;
  const r = fixFile(path, opts.text, data, opts.dictVersion, opts.encoding, opts.risky);
  if (!r.ok) throw makeError(r.errorKind ?? "", r.exitCode, r.error ?? "unknown error");
  return new FixResult(r.fixed, r.residual, r.applied, r.dictVersion);
}

export { Ags4File } from "./ags4-file";
export { AgsSubset, type Filter } from "./subset";
export type { QueryOptions, Row } from "./duckdb";
export { BuildResult, type BuildFinding } from "./build-result";
export { FixResult, type AppliedFix } from "./fix-result";
export {
  Ags4Error,
  BadDictError,
  FileNotFoundError,
  NotAgs4Error,
  UnsupportedEditionError,
} from "./errors";
export { Report, type RuleFinding } from "./report";
export { version } from "./native";
export type { Finding, GroupMeta } from "./native";
// AGS type-system helpers, as a namespace (mirrors Python's `laterite.ags_types`).
export * as agsTypes from "./ags-types";
export type { AgsValue, CanonicalType } from "./ags-types";
// The read-only group registry, as a namespace (mirrors `laterite.registry`).
export * as registry from "./registry";
export { GroupDescriptor, type Heading, type HeadingStatus } from "./registry";
// The 174 typed-graph classes (`import { PROJ, LOCA } from "laterite"`) + base.
export * from "./typed-graph";
// zstd + age file-envelope helpers, as a namespace (mirrors `laterite.transport`).
export * as transport from "./transport";
