// The `laterite` package surface — the Node port of laterite-py's `__init__.py`.
// P2 is the engine-free read/validate/emit core (Arrow-direct); the optional
// DuckDB `sql()`/`at()` layer + typed-graph + transport land in P3.
import { type Table, tableFromArrays, tableToIPC } from "apache-arrow";
import { Ags4File } from "./ags4-file";
import { BuildResult } from "./build-result";
import { Ags4Error, fromNativeError, raiseFor } from "./errors";
import { type GroupIpc, emitAgs4FromIpc, parseArrow, runCheck } from "./native";
import * as registry from "./registry";
import { Report } from "./report";
import { AgsGroup } from "./typed-graph";

export interface ReadOptions {
  /** Validate/parse in-memory text instead of a file path. */
  text?: string;
  /** Source encoding label (`"utf-8"` default, `"windows-1252"`, …). */
  encoding?: string;
}

/** Parse AGS4 — a file `source` path, raw `Uint8Array`/`Buffer` bytes, or
 * in-memory `text` — into an `Ags4File`. Pass bytes (not a string) for large
 * inputs: V8 caps strings at ~512 MB, but a `Uint8Array` does not, so a web
 * backend can hand a multi-hundred-MB upload straight in. Throws `NotAgs4Error`
 * / `FileNotFoundError` / `UnsupportedEditionError` for un-parseable input. */
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
 * in-memory `text` — against the AGS4 rules. Throws for un-validatable input;
 * rule *violations* come back in the `Report`. (Bytes avoid V8's ~512 MB string
 * cap, the same as `read`.) */
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
  edition?: string;
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

/** Build valid AGS4 from your own data — the data→AGS4 door. Where `read` loads
 * an *existing* file, `buildAgs4` *constructs* a new one (and autofixes +
 * validates it). `groups` is either a **typed-graph root** (`new PROJ({…})`,
 * walked depth-first) OR a Map/array mapping each AGS group code to an arrow-js
 * `Table` or row objects whose **keys are the AGS headings** (`LOCA_ID`, …).
 * UNIT/TYPE are filled from the chosen `edition`; order is preserved (put `PROJ`
 * first). Needs no DuckDB. Persist the result with `BuildResult.save`. */
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
  const res = emitAgs4FromIpc(ipcGroups, opts.edition, opts.mode);
  const byRule = JSON.parse(res.findingsJson) as Record<string, Array<Record<string, unknown>>>;
  const findings = Object.entries(byRule).flatMap(([rule, list]) =>
    list.map((f) => ({ rule, ...f })),
  );
  return new BuildResult(res.bytes, findings, res.fixesApplied);
}

export { Ags4File } from "./ags4-file";
export { AgsSubset, type Filter } from "./subset";
export type { QueryOptions, Row } from "./duckdb";
export { BuildResult, type BuildFinding } from "./build-result";
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
// The 92 typed-graph classes (`import { PROJ, LOCA } from "laterite"`) + base.
export * from "./typed-graph";
// zstd + age file-envelope helpers, as a namespace (mirrors `laterite.transport`).
export * as transport from "./transport";
