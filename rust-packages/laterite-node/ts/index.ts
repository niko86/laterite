// The `laterite` package surface — the Node port of laterite-py's `__init__.py`.
// P2 is the engine-free read/validate/emit core (Arrow-direct); the optional
// DuckDB `sql()`/`at()` layer + typed-graph + transport land in P3.
import { readFileSync, renameSync, unlinkSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { type Table, tableFromArrays, tableToIPC } from "apache-arrow";
import { Ags4File } from "./ags4-file";
import { BuildResult, BuildSaved } from "./build-result";
import { type DuckdbStats, stripSynthKeys } from "./duckdb";
import {
  Ags4Error,
  FileNotFoundError,
  StaleCertError,
  fromNativeError,
  makeError,
  raiseFor,
} from "./errors";
import { FixResult } from "./fix-result";
import {
  type ExcelBytesResult,
  type ExcelStats,
  type GroupIpc,
  Sidecar,
  ags4BytesToXlsx,
  ags4ToExcel,
  emitAgs4FromIpc,
  emitAgs4FromIpcUnchecked,
  excelToAgs4,
  fixFile,
  listRules as nativeListRules,
  nativeDiff,
  nativeMerge,
  parseArrow,
  runCheck,
  xlsxBytesToAgs4,
} from "./native";
import * as registry from "./registry";
import { Report } from "./report";
import { AgsGroup } from "./typed-graph";

/** File-reading knobs shared by every verb that opens a source (`read`,
 *  `validate`, …): in-memory `text`, source `encoding`, and an `.ags.idx`
 *  certificate `index`. Held apart from {@link ReadOptions} so table-shaping
 *  knobs (`contentHash`) do NOT ride onto `validate`/`fix`/`diff`, which build
 *  no tables and would silently ignore them. */
export interface FileOptions {
  /** Validate/parse in-memory text instead of a file path. */
  text?: string;
  /** Source encoding label (`"utf-8"` default, `"windows-1252"`, …). */
  encoding?: string;
  /** Path to this file's `.ags.idx` certificate (minted by `Ags4File.certify()`).
   * Opt-in, no autodiscovery — naming it asserts the cert is for THIS file. A
   * fresh cert is carried so a later errors-only `.validate()` skips the rule
   * engine; a size/SHA mismatch throws {@link StaleCertError}. (#294 Batch E / #14) */
  index?: string;
}

/** Options for {@link read}: the shared file-reading knobs, plus table-shaping
 *  knobs that apply only to the typed relational tables `read` produces. */
export interface ReadOptions extends FileOptions {
  /** Add a `_content_hash` column to each group's relational table: a typed,
   *  blank-insensitive fingerprint of the row's whole VALUE (unit-aware), so
   *  `DISTINCT ON (_content_hash)` collapses genuinely-identical rows while
   *  keeping revisions. Off by default — the plain table is byte-identical
   *  without it. Mirrors Python `read(content_hash=…)`. */
  contentHash?: boolean;
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
export function read(
  source?: string | Uint8Array,
  opts: ReadOptions = {},
): Ags4File {
  const path = typeof source === "string" ? source : undefined;
  const data =
    typeof source === "string" || source == null ? undefined : source;
  let handle: Ags4File;
  try {
    // Retain the source so the chained verbs (`ags.validate()`/`fix()`/`diff()`)
    // re-run against the TRUE bytes + read encoding, not the `.bytes` re-emit.
    handle = new Ags4File(
      parseArrow(path, opts.text, data, opts.encoding),
      {
        path,
        text: opts.text,
        data,
        encoding: opts.encoding,
      },
      opts.contentHash ?? false,
    );
  } catch (e) {
    throw fromNativeError(e);
  }
  if (opts.index !== undefined) {
    // A fresh cert is carried so a later errors-only `.validate()` skips the rule
    // engine. An explicit `index=` asserts the cert is for THIS file, so a size /
    // SHA-256 mismatch fails fast rather than silently re-validating. (#294 E/#14)
    const cert = Sidecar.fromJson(readFileSync(opts.index));
    const srcBytes =
      data ??
      (path !== undefined
        ? readFileSync(path)
        : Buffer.from(opts.text ?? "", "utf8"));
    if (!cert.isFreshFor(srcBytes)) {
      throw new StaleCertError(
        `certificate ${opts.index} is stale for this file — its size / SHA-256 differ; ` +
          "rebuild it with read(...).validate().certify()",
      );
    }
    handle._attachCert(cert);
  }
  return handle;
}

export interface ValidateOptions extends FileOptions {
  /** Path to this file's `.ags.idx` certificate. Strictly opt-in — there is no
   *  autodiscovery, because naming it asserts the cert is for THIS file.
   *
   *  A fresh, same-engine, tier-covering cert answers the verdict **without
   *  parsing**; anything less runs the rules as normal and says why on
   *  `report.revalidateReason`. A cert whose size / SHA-256 do not match throws
   *  `StaleCertError` BEFORE the engine runs — finding that out afterwards would
   *  cost exactly what naming a cert is meant to save.
   *
   *  Distinct from `read(file, {index}).validate()`, which must parse to build the
   *  handle and so skips only the rules. Two doors, two costs, two questions. */
  index?: string;
  /** Force an edition (`"4.0.3"`…`"4.2"`); default auto-detects from `TRAN_AGS`.
   *  With `dictionary`, selects the overlay base. */
  dictVersion?: string;
  /** Include warning-severity findings. */
  warnings?: boolean;
  /** Include FYI-severity findings. */
  fyi?: boolean;
  /** Make warnings decide the verdict (#321). A separate dial from `warnings`,
   *  which only decides what the report SHOWS: by default a warning is reported
   *  and `isValid` stays true. This is the compiler's `-Werror`. FYIs are never
   *  fatal, under this flag or any other. */
  warningsAsErrors?: boolean;
  /** Also run Rule 20's on-disk half (the sibling `FILE/` tree must exist). */
  checkFiles?: boolean;
  /** A custom AGS4 dictionary to overlay (laterite-dev#568) — a file path or the raw `.ags`/JSON
   *  bytes of one — so a bespoke group (or a re-parented/overridden standard heading)
   *  validates as first-class instead of being flagged unknown. The base edition is
   *  detected from the dictionary itself unless `dictVersion` forces it or `dictReplace`
   *  drops it. Overrides of standard definitions are honoured with a warning. */
  dictionary?: string | Uint8Array;
  /** Treat `dictionary` as a FULL REPLACEMENT — drop the bundled base entirely rather
   *  than overlaying it (default `false`). Cannot be combined with `dictVersion`. */
  dictReplace?: boolean;
}

/** Split a `dictionary` custom-dict override (laterite-dev#568) into the `[dictPath, dictBytes]`
 *  the native `runCheck`/`fixFile`/`Sidecar.mint` take. A string is a filesystem path;
 *  raw bytes are the in-memory spelling. */
function splitDict(
  dictionary: string | Uint8Array | undefined,
): [string | undefined, Uint8Array | undefined] {
  if (dictionary === undefined) return [undefined, undefined];
  return typeof dictionary === "string"
    ? [dictionary, undefined]
    : [undefined, dictionary];
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
 *   (`dictVersion`), severity gates, in-memory text/encoding, the on-disk
 *   Rule-20 toggle, and `index` (an `.ags.idx` certificate that can answer the
 *   verdict without parsing); see the interface for each field.
 * @returns A `Report` carrying the findings, the resolved `dictVersion`, the
 *   finding `count`, `isValid`, and `lat-check`-faithful `toJson()` / `toNdjson()`.
 * @throws {FileNotFoundError} the path could not be opened.
 * @throws {NotAgs4Error} the input has no GROUP rows / is not decodable AGS4.
 * @throws {UnsupportedEditionError} a recognised-but-unsupported edition (AGS3).
 * @throws {BadDictError} an invalid `opts.dictVersion` / unimplemented dictionary.
 * @throws {StaleCertError} `opts.index` names a cert whose size / SHA-256 do not
 *   match this file — thrown before the rule engine runs.
 */
export function validate(
  source?: string | Uint8Array,
  opts: ValidateOptions = {},
): Report {
  const path = typeof source === "string" ? source : undefined;
  const data =
    typeof source === "string" || source == null ? undefined : source;
  const [dictPath, dictBytes] = splitDict(opts.dictionary);
  // Loaded here, freshness-checked in the engine. The comparison needs the source
  // BYTES and the native layer is about to read them anyway — doing it here would
  // read a 25 MB delivery twice to save parsing it once, which is most of the
  // saving this option exists to deliver.
  const cert =
    opts.index !== undefined
      ? Sidecar.fromJson(readFileSync(opts.index))
      : undefined;
  const r = runCheck(
    path,
    opts.text,
    data,
    opts.dictVersion,
    opts.warnings,
    opts.fyi,
    opts.warningsAsErrors,
    opts.checkFiles,
    opts.encoding,
    dictPath,
    dictBytes,
    opts.dictReplace,
    cert,
    // This door NAMED the cert, so a stale one throws. `Ags4File.validate()`
    // passes no such flag: a cert it inherited from `read(file, {index})` was
    // asserted THERE, and quietly re-validating is right if the file has since
    // moved under it.
    cert !== undefined,
  );
  return new Report(raiseFor(r));
}

/** Row-oriented group data: an array of `{HEADING: value}` objects. */
export type GroupRows = Array<Record<string, unknown>>;
/** One group's data for `buildAgs4` — an arrow-js `Table` or row objects. */
export type GroupData = Table | GroupRows;

/** The transmission a file represents — the caller's half of a synthesised `TRAN`.
 *
 * The five members are **required together**, because all five are REQUIRED
 * headings in the dictionary: a partial stamp emits a `TRAN` that fails Rule
 * 10b on every cell it leaves blank. Supply the object or omit it; there is no
 * meaningful half.
 *
 * `TRAN_AGS`, `TRAN_DLIM` and `TRAN_RCON` are absent on purpose — they describe
 * the syntax of the file the emitter is writing, so it fills them. A value you
 * passed could only contradict the bytes. */
export interface TranStamp {
  /** `TRAN_ISNO` — the issue sequence reference. */
  issue: string;
  /** `TRAN_DATE` — `yyyy-mm-dd`. */
  date: string;
  /** `TRAN_PROD` — who produced the file. */
  producer: string;
  /** `TRAN_RECV` — who it is for. */
  recipient: string;
  /** `TRAN_STAT` — e.g. `"FINAL"`. */
  status: string;
  /** `TRAN_DESC` — what was transferred. Optional. */
  description?: string;
  /** `TRAN_REM` — free remarks. Optional. On a MERGE these are appended to the
   *  provenance note, not substituted for it. On a BUILD there is no provenance
   *  note to append to, so this is `TRAN_REM` verbatim. */
  remarks?: string;
}

export interface EmitOptions {
  /** `"4.0.3" | "4.0.4" | "4.1" | "4.1.1" | "4.2"` (default `"4.1.1"`). */
  dictVersion?: string;
  /** `"autofix"` (default) | `"report"` | `"strict"`. */
  mode?: "autofix" | "report" | "strict";
  /** Per-heading UNIT overrides, keyed `{ code: { heading: unit } }` (#294 F#9)
   *  — e.g. `{ LOCA: { LOCA_XTRA: "kPa" } }`. Name only the headings you want to
   *  set; the rest fill from the dictionary. Throws on an unknown code/heading. */
  units?: Record<string, Record<string, string>>;
  /** Per-heading AGS data-TYPE overrides, same `{ code: { heading: type } }` shape. */
  types?: Record<string, Record<string, string>>;
  /** Mint the mandatory metadata catalogs your data doesn't carry — UNIT and
   *  TYPE (derived from the data), and ABBR when PA picklist codes are used.
   *  `"autofix"` mode only. Default **false**. `TRAN` is NOT among them: pass
   *  `tran` to state one.
   *
   *  Off by default on purpose: synthesis adds whole *groups* you never wrote,
   *  which should be asked for rather than discovered. Left off, a data-only
   *  build reports Rule 14/15/17 so you can see what is missing. `PROJ` and
   *  `DICT` are never synthesised at all — those are authorial facts. */
  synthesiseMetadata?: boolean;
  /** The transmission this file represents. Omit it and no `TRAN` is minted —
   *  Rule 14 reports the gap, rather than a placeholder that would *satisfy*
   *  Rule 14 while asserting a transmission that never happened. */
  tran?: TranStamp;
  /** Destination path — the to-disk rider (mirrors laterite-py's
   *  `build_ags4(out=)`). Given, the judged document is written there and the
   *  result is a {@link BuildSaved} carrying `path` and the verdict but **no**
   *  `bytes`. The write stages to a temporary file in the destination's
   *  directory and renames it into place only after the verdict allows, so
   *  `out` never holds unjudged output: a `"strict"` failure throws with
   *  nothing written, and any autofix rewrite happens before the path
   *  exists. */
  out?: string;
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
 * into per-group row buckets, then emit only the headings you actually SET —
 * columns entirely unset (null) across a group's rows are dropped (except KEY
 * headings), so the typed-graph door matches the frames door (emit your data,
 * not the full union schema). Child arrays are recursed via the registry's
 * parent→child links. A heading set to `""` survives — that's a real value. */
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
    let bucket = buckets.get(code);
    if (!bucket) {
      bucket = [];
      buckets.set(code, bucket);
    }
    bucket.push(row);
    for (const child of registry.childGroups(code)) {
      const children = record[`${child.code.toLowerCase()}s`];
      if (Array.isArray(children))
        for (const c of children) visit(c as AgsGroup);
    }
  };
  visit(root);
  // Prune entirely-unset columns (keep KEY headings — a missing key must be
  // flagged, not silently dropped). Otherwise a sparse node emits ~45 blank
  // columns whose unset edition-specific / PA headings trip Rule 9 / 16.
  for (const [code, rows] of buckets) {
    const desc = registry.get(code);
    if (!desc) continue;
    for (const h of desc.headings) {
      if (registry.isKeyStatus(h.status)) continue;
      if (rows.every((r) => r[h.name] == null)) {
        for (const r of rows) Reflect.deleteProperty(r, h.name);
      }
    }
  }
  return [...buckets];
}

/** Validate a `{code: {heading: value}}` UNIT/TYPE override map (#294 F#9)
 * against the groups actually being built — an unknown group code or a heading
 * not in that group is a caller typo we surface, not a silent no-op. The typo
 * is reported in the caller's name (`buildAgs4` / `buildAgs4Unchecked`). */
function checkMeta(
  meta: Record<string, Record<string, string>> | undefined,
  name: string,
  columns: Map<string, Set<string>>,
  caller: string,
): void {
  if (meta === undefined) return;
  for (const [code, hmap] of Object.entries(meta)) {
    const known = columns.get(code);
    if (known === undefined) {
      throw new Ags4Error(
        `${caller} ${name}: unknown group ${JSON.stringify(code)}`,
      );
    }
    for (const heading of Object.keys(hmap)) {
      if (!known.has(heading)) {
        throw new Ags4Error(
          `${caller} ${name}: group ${JSON.stringify(code)} has no heading ${JSON.stringify(heading)}`,
        );
      }
    }
  }
}

/** The build doors' shared marshalling (#881): a typed-graph root, Map or
 * `[code, data]` array → per-group Arrow IPC buffers plus the column sets the
 * `units=`/`types=` typo check reads. One walk for both doors, so they cannot
 * drift at the input — the same anti-drift split laterite-py's
 * `_frames_to_tables` makes. */
function marshalGroups(
  groups: AgsGroup | Map<string, GroupData> | Array<[string, GroupData]>,
): { ipcGroups: GroupIpc[]; columns: Map<string, Set<string>> } {
  const items: Array<[string, GroupData]> =
    groups instanceof AgsGroup
      ? walkTree(groups)
      : groups instanceof Map
        ? [...groups]
        : groups;
  const ipcGroups: GroupIpc[] = [];
  const columns = new Map<string, Set<string>>();
  for (const [code, data] of items) {
    // Never emit a read(...).table(code, { keys: true }) _id/_parent_id.
    const table = stripSynthKeys(
      Array.isArray(data) ? rowsToTable(data) : data,
    );
    columns.set(code, new Set(table.schema.fields.map((f) => f.name)));
    ipcGroups.push({ code, ipc: Buffer.from(tableToIPC(table, "stream")) });
  }
  return { ipcGroups, columns };
}

/** Write `bytes` to `out` via a temp file in the destination's own directory +
 * `renameSync` — atomic on one filesystem, so `out` never holds a partial
 * write. Shared by the two build doors' `out` riders; what each door lets
 * REACH this write is the doors' difference, not the write's. */
function stagedWrite(bytes: Buffer, out: string): void {
  const tmp = join(
    dirname(out),
    `.laterite-build-${process.pid}-${process.hrtime.bigint()}.tmp`,
  );
  try {
    // `wx` = exclusive create, matching the native twins' `create_new`: a name
    // collision is an error, never a silent overwrite of whatever was
    // squatting on it. hrtime (not Date.now) so two writes in the same
    // millisecond cannot even share a name.
    writeFileSync(tmp, bytes, { flag: "wx" });
    renameSync(tmp, out);
  } catch (err) {
    try {
      unlinkSync(tmp);
    } catch {
      // best-effort cleanup; the original error is the one to surface
    }
    throw err;
  }
}

/**
 * Build valid AGS4 from your own data — the data→AGS4 door. Where `read` loads
 * an *existing* file, `buildAgs4` *constructs* a new one: it lays the groups out
 * in order, fills UNIT/TYPE from the chosen `dictVersion`, then runs the output
 * through the validator (the `mode` knob on `opts` decides what happens to the
 * findings — `"autofix"` applies the safe fixes to what you wrote; `"report"`
 * merely records them). Minting the missing UNIT/TYPE/TRAN/ABBR catalogs is a
 * separate opt-in, `synthesiseMetadata`. The returned `BuildResult` carries the bytes, the residual `findings`,
 * and a `fixesApplied` count; persist it with `BuildResult.save`. Needs no DuckDB.
 *
 * `groups` accepts two shapes. A **typed-graph root** (`new PROJ({…, locas:[new
 * LOCA({…})]})`) is walked depth-first via the registry's parent→child links,
 * only the headings you set becoming columns (entirely-unset ones are dropped,
 * except KEY). The walk covers only
 * PROJ's subtree (the root-metadata groups have no parent), so those groups are
 * absent unless you pass `synthesiseMetadata: true` — without it they surface as
 * Rule 14/15/17 findings rather than being filled in silently. Or pass a
 * **Map / array of `[code,
 * data]`** entries where `data` is
 * an arrow-js `Table` or row objects whose **keys are the AGS headings**
 * (`LOCA_ID`, …). Either way group order is preserved, so put `PROJ` first.
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
  opts: EmitOptions & { out: string },
): BuildSaved;
export function buildAgs4(
  groups: AgsGroup | Map<string, GroupData> | Array<[string, GroupData]>,
  opts?: EmitOptions & { out?: undefined },
): BuildResult;
export function buildAgs4(
  groups: AgsGroup | Map<string, GroupData> | Array<[string, GroupData]>,
  opts: EmitOptions = {},
): BuildResult | BuildSaved {
  const { ipcGroups, columns } = marshalGroups(groups);
  // Per-heading UNIT/TYPE overrides (#294 F#9): an unknown group code or a
  // heading not in that group is a typo to surface, not a silent no-op.
  checkMeta(opts.units, "units", columns, "buildAgs4");
  checkMeta(opts.types, "types", columns, "buildAgs4");
  const res = emitAgs4FromIpc(
    ipcGroups,
    opts.dictVersion,
    opts.mode,
    opts.units,
    opts.types,
    opts.synthesiseMetadata,
    opts.tran,
  );
  const byRule = JSON.parse(res.findingsJson) as Record<
    string,
    Array<Record<string, unknown>>
  >;
  const findings = Object.entries(byRule).flatMap(([rule, list]) =>
    list.map((f) => ({ rule, ...f })),
  );
  if (opts.out === undefined) {
    return new BuildResult(res.bytes, findings, res.applied, res.fixesApplied);
  }
  // The to-disk rider: the engine has already judged `res.bytes` (a strict
  // failure threw above, autofix rewrote in memory), so the destination
  // path never holds unjudged bytes.
  stagedWrite(res.bytes, opts.out);
  return new BuildSaved(opts.out, findings, res.applied, res.fixesApplied);
}

/** Options for {@link buildAgs4Unchecked} — the data-shaping knobs only.
 * `mode` / `synthesiseMetadata` / `tran` are gone, not defaulted: there is no
 * verdict for a mode to act on, and synthesis fills gaps only a report would
 * surface. Passing one is refused at runtime, never silently ignored. */
export interface UncheckedEmitOptions {
  /** `"4.0.3" | "4.0.4" | "4.1" | "4.1.1" | "4.2"` (default `"4.1.1"`). */
  dictVersion?: string;
  /** Per-heading UNIT overrides — as {@link EmitOptions.units}. */
  units?: Record<string, Record<string, string>>;
  /** Per-heading AGS data-TYPE overrides — as {@link EmitOptions.types}. */
  types?: Record<string, Record<string, string>>;
  /** Destination path — the same staged temp + rename as `buildAgs4({ out })`,
   *  minus the verdict gate in front of it. Returns the path written. */
  out?: string;
}

/** The judged door's knobs, refused here by name — absence from the TS type
 * alone would let a JS caller's `mode: "strict"` be silently ignored, which is
 * worse than either honouring or refusing it. */
const JUDGE_COUPLED_KEYS = ["mode", "synthesiseMetadata", "tran"] as const;

/**
 * {@link buildAgs4} without the verdict — you are choosing to ship unchecked
 * bytes (#858/#881).
 *
 * Builds exactly what `buildAgs4(groups, { mode: "report" })` builds — the
 * same dictionary UNIT/TYPE fills, the same canonical cell formatting, the
 * same section order, byte for byte (a test pins the identity) — and skips
 * the validation that follows. **Nothing here confirms the output satisfies
 * any AGS4 rule, and nothing downstream will**: no findings, no fixes, no
 * strict gate. The rule engine is most of what the judged call spends its
 * time on (the decomposition is recorded on #858), so this door exists for
 * the caller who validates elsewhere or has decided not to.
 *
 * Returns the AGS4 bytes as a plain `Buffer` — deliberately **not** a
 * {@link BuildResult}, whose empty `findings` would read as "judged clean"
 * when nothing judged anything. With `out` given, writes them there via the
 * same staged atomic rename as `buildAgs4({ out })` (minus the verdict gate)
 * and returns the path written.
 *
 * @param groups The data to emit — the same shapes {@link buildAgs4} takes.
 * @param opts `dictVersion` / `units` / `types` / `out` only; see
 *   {@link UncheckedEmitOptions}.
 * @returns The AGS4 `Buffer`, or with `out` the destination path.
 * @throws {Ags4Error} On a judged-door knob, an unknown `units`/`types`
 *   group or heading, or a typed-graph node that is not a registered group.
 */
export function buildAgs4Unchecked(
  groups: AgsGroup | Map<string, GroupData> | Array<[string, GroupData]>,
  opts: UncheckedEmitOptions & { out: string },
): string;
export function buildAgs4Unchecked(
  groups: AgsGroup | Map<string, GroupData> | Array<[string, GroupData]>,
  opts?: UncheckedEmitOptions & { out?: undefined },
): Buffer;
export function buildAgs4Unchecked(
  groups: AgsGroup | Map<string, GroupData> | Array<[string, GroupData]>,
  opts: UncheckedEmitOptions = {},
): Buffer | string {
  for (const key of JUDGE_COUPLED_KEYS) {
    if (key in opts) {
      throw new Ags4Error(
        `buildAgs4Unchecked: ${JSON.stringify(key)} is not an option here — ` +
          "there is no verdict for it to act on; use buildAgs4 for a judged build",
      );
    }
  }
  const { ipcGroups, columns } = marshalGroups(groups);
  checkMeta(opts.units, "units", columns, "buildAgs4Unchecked");
  checkMeta(opts.types, "types", columns, "buildAgs4Unchecked");
  const bytes = emitAgs4FromIpcUnchecked(
    ipcGroups,
    opts.dictVersion,
    opts.units,
    opts.types,
  );
  if (opts.out === undefined) return bytes;
  stagedWrite(bytes, opts.out);
  return opts.out;
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

/** The AGS Format Rule labels whose fixes {@link fix} can apply — the values for
 * `only` / `exclude`. Mirrors laterite-py's `FixableRule`; kept in lockstep with
 * the engine's `fixable_rules()` by a cross-surface drift gate
 * (`test_typed_choices.py`). Use {@link listRules} (`fixable: true`) at runtime. */
export type FixableRule =
  "1" | "2a" | "4" | "5" | "6" | "7" | "8" | "11a" | "11b";

export interface FixOptions {
  /** Repair in-memory `text` instead of a file path. */
  text?: string;
  /** Force an edition (`"4.0.3"`…`"4.2"`); default auto-detects from `TRAN_AGS`. */
  dictVersion?: string;
  /** Source encoding label (`"utf-8"` default, `"windows-1252"`, …). */
  encoding?: string;
  /** Also apply the intent-guessing (risky) fixes, not just the safe set. */
  risky?: boolean;
  /** Apply *only* these rules' fixes (by {@link FixableRule} label); others are
   * left in place. The risk gate still applies, so a rule whose only fix is risky
   * needs `risky: true` even when named here. */
  only?: FixableRule[];
  /** Skip these rules' fixes. Combines with `only` (`only` narrows the set, then
   * `exclude` removes from it). */
  exclude?: FixableRule[];
  /** Write the repaired bytes back over the source file. Requires a path
   * `source`; mutually exclusive with `out`. Non-destructive by default. */
  inPlace?: boolean;
  /** Write the repaired bytes to this path. Mutually exclusive with `inPlace`. */
  out?: string;
}

/** Mechanically repair AGS4 — the headless twin of the browser's Fix engine.
 * `source` is a file path, raw `Uint8Array`/`Buffer` bytes, or (via `opts.text`)
 * in-memory text. The *safe* fixes — CRLF / BOM / embedded-CR normalisation,
 * short-row padding, numeric reformatting, and the TRAN delimiter+concatenator
 * rows — are always applied; pass `risky` to also run the intent-guessing set
 * (duplicate-heading rename, ambiguous `dd/mm` datetime canonicalisation, smart-quote→ASCII
 * typography). The repaired bytes are re-validated, so `FixResult.findings` is
 * what could NOT be mechanically fixed.
 *
 * Non-destructive by default — the repaired bytes come back on the result
 * (`.bytes` / `.text` / `.save(path)`), already UTF-8 with no BOM, so fixing a
 * non-UTF-8 file also normalises its encoding. Pass `inPlace` to overwrite the
 * source or `out` to write elsewhere (the two are mutually exclusive). `only` /
 * `exclude` restrict which rules' fixes are applied. Mirrors `laterite.fix()` /
 * `lat-check --fix`.
 *
 * @param source - The AGS4 input: a filesystem path (`string`) or raw bytes
 *   (`Uint8Array`/`Buffer`). Omit to repair `opts.text` instead.
 * @param opts - {@link FixOptions} — `text` source, `risky` fixes, `only` /
 *   `exclude` rule selection, `inPlace` / `out` write-back, `dictVersion`
 *   override, and source `encoding`.
 * @throws {TypeError} If both `inPlace` and `out` are given.
 * @returns A {@link FixResult} carrying the repaired `bytes` (and `.text` /
 *   `.save`), the `applied` fixes (with `fixesApplied` count), the residual
 *   `findings` left after re-validation, and the resolved `dictVersion`.
 * @throws {Ags4Error} (or a subclass — {@link FileNotFoundError},
 *   {@link NotAgs4Error}, {@link UnsupportedEditionError}, {@link BadDictError})
 *   for un-fixable input, carrying the matching `lat-check` exit code.
 */
export function fix(
  source?: string | Uint8Array,
  opts: FixOptions = {},
): FixResult {
  if (opts.inPlace && opts.out !== undefined) {
    throw new TypeError("fix(): `inPlace` and `out` are mutually exclusive");
  }
  // Reject non-fixable rule labels up front (mirrors laterite-py's
  // `_validate_fixable`) — the native selector silently ignores an unknown label,
  // so without this a typo like only:["9"] would quietly repair nothing. The
  // fixable set comes from the engine (`listRules`), never a hand-list.
  if (opts.only !== undefined || opts.exclude !== undefined) {
    const fixable = new Set(
      listRules()
        .filter((r) => r.fixable)
        .map((r) => r.rule),
    );
    for (const [kw, labels] of [
      ["only", opts.only],
      ["exclude", opts.exclude],
    ] as const) {
      for (const label of labels ?? []) {
        if (!fixable.has(label)) {
          throw new TypeError(
            `fix(): ${kw} names rule "${label}", which is not fixable — see listRules() (fixable: true)`,
          );
        }
      }
    }
  }
  const path = typeof source === "string" ? source : undefined;
  const data =
    typeof source === "string" || source == null ? undefined : source;
  if (opts.inPlace && path === undefined) {
    // Nothing on disk to overwrite — mirror laterite-py's Ags4Error guard.
    throw makeError(
      "bad_args",
      5,
      "fix(): `inPlace` needs a path source; use `out` or `.save(path)`",
    );
  }
  const r = fixFile(
    path,
    opts.text,
    data,
    opts.dictVersion,
    opts.encoding,
    opts.risky,
    opts.only,
    opts.exclude,
  );
  if (!r.ok)
    throw makeError(r.errorKind ?? "", r.exitCode, r.error ?? "unknown error");
  const result = new FixResult(r.fixed, r.residual, r.applied, r.dictVersion);
  // Write-back (opt-in, non-destructive by default): `inPlace` overwrites the
  // source path, `out` writes elsewhere — the repaired bytes are always UTF-8
  // with no BOM. Mirrors laterite-py's free `fix(in_place=, out=)`.
  const dest = opts.inPlace ? path : opts.out;
  if (dest !== undefined) writeFileSync(dest, result.bytes);
  return result;
}

// --- diff (revision comparison) -----------------------------------------

/** One changed cell of a matched row (`kind === "changed"`). `type` is the AGS
 * data type; `a`/`b` are the raw values on each side (`null` if that side's row
 * is shorter than the heading list). Snake_case fields mirror the wire shape the
 * shared `laterite-ags4-diff` leaf serialises — identical to Python's dict. */
export interface CellDelta {
  heading: string;
  type: string;
  a: string | null;
  b: string | null;
}

/** One row's verdict: `added` (only in `b`), `removed` (only in `a`), or
 * `changed` (matched by KEY, ≥1 typed cell differs). `key` is the KEY values (or
 * whole-row tuple when unkeyed); `cells` is populated only for `changed`. */
export interface RowDelta {
  kind: "added" | "removed" | "changed";
  key: string[];
  line_a: number | null;
  line_b: number | null;
  cells: CellDelta[];
}

/** A group's deltas. `added`/`removed`/`changed` are true totals; `keyed` is
 * false when matched on the whole-row tuple (no dictionary KEY headings). */
export interface GroupDelta {
  code: string;
  added: number;
  removed: number;
  changed: number;
  headings_added: string[];
  headings_removed: string[];
  keyed: boolean;
  key_headings: string[];
  rows: RowDelta[];
}

/** The revision diff — the shape `diff()` returns (parsed from the shared
 * `laterite-ags4-diff` leaf's JSON; byte-identical to Python / wasm / `lat-check
 * --diff`). */
export interface RevisionDelta {
  groups: GroupDelta[];
  groups_added: string[];
  groups_removed: string[];
  total_added: number;
  total_removed: number;
  total_changed: number;
  /** The delta's `--json` document — the engine's own render kept verbatim, so
   * the npx launcher prints the same bytes `lat diff --json` and the uvx
   * launcher do (#542). Mirrors `Report.toJson()`: the machine render lives on
   * the result. Non-enumerable — `JSON.stringify(delta)` sees only the data. */
  toJson(): string;
}

export interface DiffOptions {
  /** Force the edition used to resolve each group's KEY headings (`"4.0.3"`…
   * `"4.2"`); default takes it from the revision's `TRAN_AGS`. */
  dictVersion?: string;
  /** Source encoding label for path / bytes inputs (default `"utf-8"`). */
  encoding?: string;
}

/** A diff input: a file path (`string`), raw bytes, or an already-read `Ags4File`. */
export type DiffSource = string | Uint8Array | Ags4File;

/** Resolve a diff input to bytes — the Node analog of Python's `_source_bytes`.
 * A string is a filesystem path (read here so a missing one throws the mapped
 * `FileNotFoundError`, not a raw `ENOENT`); an `Ags4File` contributes its
 * byte-faithful re-emit. */
function diffBytes(x: DiffSource): Uint8Array {
  if (x instanceof Ags4File) return x.bytes;
  if (typeof x !== "string") return x;
  try {
    return readFileSync(x);
  } catch (e) {
    if (
      e &&
      typeof e === "object" &&
      (e as { code?: string }).code === "ENOENT"
    ) {
      throw new FileNotFoundError(`No such file or directory: ${x}`, 3);
    }
    throw e;
  }
}

/** Compare two AGS4 documents and return their **revision diff** — the Node port
 * of `laterite.diff()` and the browser's revision-diff tool, over the SAME shared
 * `laterite-ags4-diff` engine `lat-check --diff` uses.
 *
 * `a` (baseline) and `b` (revision) are each a path, raw `Uint8Array`/`Buffer`
 * bytes, or an already-read `Ags4File`. Two choices make the diff meaningful
 * rather than noisy: rows are matched by the group's dictionary **KEY** headings
 * (not line order — re-sorted boreholes still pair up), and cells are compared
 * through the **typed** value (a formatting-only edit like `"1.0"` → `"1.00"` is
 * not a diff). The KEY-heading edition is the revision's `TRAN_AGS` unless pinned
 * with `opts.dictVersion`.
 *
 * @param a - The baseline document (path / bytes / `Ags4File`).
 * @param b - The revision document, in any of the same forms.
 * @param opts - {@link DiffOptions} — the `dictVersion` pin and source `encoding`.
 * @returns A {@link RevisionDelta}: per-group row/heading deltas, `groups_added`/
 *   `groups_removed`, and the `total_added`/`total_removed`/`total_changed` counts.
 * @throws {FileNotFoundError} a path input could not be opened.
 * @throws {NotAgs4Error} either side is not decodable AGS4.
 * @throws {BadDictError} an invalid `opts.dictVersion`.
 */
export function diff(
  a: DiffSource,
  b: DiffSource,
  opts: DiffOptions = {},
): RevisionDelta {
  const aBytes = diffBytes(a);
  const bBytes = diffBytes(b);
  try {
    const raw = nativeDiff(aBytes, bBytes, opts.dictVersion, opts.encoding);
    const delta = JSON.parse(raw) as RevisionDelta;
    Object.defineProperty(delta, "toJson", {
      value: () => raw,
      enumerable: false,
    });
    return delta;
  } catch (e) {
    throw fromNativeError(e);
  }
}

// --- merge (N-way reconciliation) ---------------------------------------

/** One advisory note from a merge — something reconciled without failing (a
 * recency contradiction, a non-`X` type widen, a missing merge-TRAN stamp).
 * Snake_case fields mirror the wire shape, identical to Python's dict. */
export interface MergeWarning {
  kind: string;
  group: string | null;
  heading: string | null;
  message: string;
}

/** One per-row content revision — a later file changed a KEY-matched row's
 * values. `key` is the KEY tuple; `changed` names the headings whose typed value
 * differed; `winnerFile` is the argument index that supplied the winning row. */
export interface RevisionNote {
  group: string;
  key: string[];
  changed: string[];
  winnerFile: number;
}

/** What {@link merge} returns — the reconciled `bytes` plus the `warnings` and
 * per-row `revisions` audit. `text` decodes the bytes as UTF-8. */
export interface MergeResult {
  bytes: Uint8Array;
  warnings: MergeWarning[];
  revisions: RevisionNote[];
  text: string;
}

export interface MergeOptions {
  /** How to settle a heading two deliveries typed differently. Default `"error"`.
   *
   * - `"error"`   — refuse, throwing {@link MergeConflictError}.
   * - `"widen"`   — fall back to `X` (free text). Raw values kept byte-for-byte,
   *                 but the column's TYPE is thrown away.
   * - `"promote"` — keep the column numeric when every clashing code is `nDP`: take
   *                 the greatest precision (`2DP` + `5DP` → `5DP`) and zero-pad the
   *                 coarser values (`10.00` → `10.00000`), so the merged file still
   *                 satisfies Rule 8. Never rounds, never demotes. `nSF`/`nSCI` and
   *                 cross-family clashes fall back to `"widen"` — padding
   *                 significant figures would overstate measured precision.
   *
   * `"promote"` is what keeps a merged file value-comparable with its own inputs:
   * `_content_hash` reads `10.00` as a number under `2DP` but as a string under `X`.
   *
   * A conflicting **UNIT** is fatal in every mode — `TYPE` has an absorber (`X`),
   * `UNIT` has none. */
  onTypeClash?: "error" | "widen" | "promote";
  /** Force the edition used to resolve KEY headings; default takes it from the
   * newest file's `TRAN_AGS`. */
  dictVersion?: string;
  /** Source encoding label for path / bytes inputs (default `"utf-8"`). */
  encoding?: string;
  /** The transmission the merged file represents. Omit it and `TRAN` is
   *  reconciled like any other group, with a warning noting no
   *  merge-transmission stamp was supplied — and because `TRAN_ISNO` is a KEY
   *  heading, each input's transmission normally survives, leaving more TRAN
   *  rows than Rule 14 permits. */
  tran?: TranStamp;
  /** What to do when no `tran` is supplied and the sources carry `TRAN` rows of
   *  their own:
   *
   * - `"reconcile"` — fold `TRAN` like any other group and warn (the default,
   *                   so this option changed nothing for anyone). The merged
   *                   file keeps every input's transmission, which is more rows
   *                   than Rule 14 permits.
   * - `"error"`     — throw {@link MergeConflictError} before any bytes are
   *                   produced.
   *
   * Irrelevant when `tran` is supplied: a stamp synthesises the one `TRAN` row
   * the merged file needs, and this is never read. */
  onMissingTran?: "reconcile" | "error";
}

/** A merge input: a file path (`string`), raw bytes, or an already-read `Ags4File`. */
export type MergeSource = string | Uint8Array | Ags4File;

/** Reconcile two or more AGS4 deliveries of one project into a single file — the
 * Node port of `laterite.merge()`, over the SAME shared `laterite-ags4-merge`
 * engine `lat merge` uses.
 *
 * Each source is a path, raw `Uint8Array`/`Buffer` bytes, or an already-read
 * `Ags4File`. Files merge **in argument order** — a later argument wins a KEY
 * conflict. Rows are identified by their dictionary **KEY** headings (not line
 * order), so a re-sorted borehole list still merges each `LOCA` onto its prior
 * self; the merge is a **union** (a row in one file and absent in another is
 * kept, since silence is not deletion). A heading two files typed differently
 * throws {@link MergeConflictError} unless `opts.onTypeClash` settles it — `"widen"`
 * falls back to `X` (raw values kept), `"promote"` keeps the greatest `nDP` precision
 * (zero-padding the coarser values). Pass a complete `opts.tran` to
 * stamp a synthesised merge-TRAN recording the inputs' issues/dates in `TRAN_REM`.
 *
 * @param sources - Two or more documents to merge (path / bytes / `Ags4File`).
 * @param opts - {@link MergeOptions}.
 * @returns A {@link MergeResult}: the merged `bytes` + `warnings` / `revisions` audit.
 * @throws {MergeConflictError} an unsettled TYPE clash, a UNIT clash (fatal in every
 *   mode), or the output failed to emit.
 * @throws {NotAgs4Error} a source is not decodable AGS4.
 * @throws {BadDictError} an invalid `opts.dictVersion`.
 * @throws {RangeError} fewer than two sources.
 */
export function merge(
  sources: MergeSource[],
  opts: MergeOptions = {},
): MergeResult {
  if (sources.length < 2)
    throw new RangeError("merge needs at least two source documents");
  const files = sources.map(diffBytes);
  try {
    const out = nativeMerge(
      files,
      opts.onTypeClash,
      opts.onMissingTran,
      opts.dictVersion,
      opts.encoding,
      opts.tran,
    );
    const bytes = new Uint8Array(out.bytes);
    return {
      bytes,
      warnings: JSON.parse(out.warningsJson) as MergeWarning[],
      // The wire is the engine structs' canonical snake_case (`winner_file`,
      // identical to the Python fragments and the binary's `--json`, #542);
      // camelCase is this TS API's own convention, so the rename happens here,
      // at the boundary — never on the wire.
      revisions: (
        JSON.parse(out.revisionsJson) as {
          group: string;
          key: string[];
          changed: string[];
          winner_file: number;
        }[]
      ).map((r) => ({
        group: r.group,
        key: r.key,
        changed: r.changed,
        winnerFile: r.winner_file,
      })),
      text: new TextDecoder().decode(bytes),
    };
  } catch (e) {
    throw fromNativeError(e);
  }
}

/** Pick the three stat fields out of an in-memory conversion result. */
function excelStatsOf(r: ExcelBytesResult): ExcelStats {
  return {
    sheetsWritten: r.sheetsWritten,
    rowsWritten: r.rowsWritten,
    warnings: r.warnings,
  };
}

/**
 * Convert AGS4 to an `.xlsx` workbook — one worksheet per group (the Node analog
 * of Python's `to_excel`). `source` is an AGS4 file path or raw `Uint8Array`
 * bytes; `groups` forces the worksheet order (else AGS4 source order).
 *
 * With `xlsxPath` given the workbook is written there and the conversion stats
 * are returned; omit `xlsxPath` to get the `.xlsx` **bytes** back (the FS-free
 * form — an uploaded/in-memory AGS4 needn't hit disk). Bytes both ways drive the
 * same core the browser Excel tools use.
 */
export function toExcel(
  source: string | Uint8Array,
  xlsxPath: string,
  opts?: { groups?: string[] },
): ExcelStats;
export function toExcel(
  source: string | Uint8Array,
  xlsxPath?: undefined,
  opts?: { groups?: string[] },
): Buffer;
export function toExcel(
  source: string | Uint8Array,
  xlsxPath?: string,
  opts: { groups?: string[] } = {},
): ExcelStats | Buffer {
  // Fast path: a real AGS4 file → an `.xlsx` file, straight through the path core.
  if (typeof source === "string" && xlsxPath !== undefined) {
    return ags4ToExcel(source, xlsxPath, opts.groups);
  }
  const agsBytes = typeof source === "string" ? readFileSync(source) : source;
  const r = ags4BytesToXlsx(agsBytes, opts.groups);
  if (xlsxPath === undefined) return r.bytes; // bytes-out
  writeFileSync(xlsxPath, r.bytes); // bytes-in → file
  return excelStatsOf(r);
}

/**
 * Persist AGS4 to a DuckDB database — one born-typed, keyed table per group. The
 * functional twin of the fluent {@link Ags4File.toDuckdb} (`read(src).toDuckdb(out)`).
 *
 * `source` is an AGS4 file path, raw `Uint8Array` bytes, or an already-{@link read}
 * {@link Ags4File}; `output` is the `.duckdb` path to write (required — a database
 * is a file, with no bytes-in-memory form). Returns
 * `{ path, tables_written, rows_written }` and refuses to overwrite an existing
 * `output`. `groups` optionally restricts/re-orders the tables. Each table carries
 * the content-addressed `_id`/`_parent_id` keys, so the store joins and
 * version-diffs by `_id`. Needs the optional `@duckdb/node-api` peer.
 */
export async function toDuckdb(
  source: string | Uint8Array | Ags4File,
  output: string,
  opts: { groups?: string[] } & ReadOptions = {},
): Promise<DuckdbStats> {
  const owned = !(source instanceof Ags4File);
  const file = owned ? read(source, opts) : source;
  try {
    return await file.toDuckdb(output, { groups: opts.groups });
  } finally {
    if (owned) file.close(); // free the intermediate handle's DuckDB engine
  }
}

/**
 * Convert an AGS4-shaped `.xlsx` workbook to AGS4 (the Node analog of
 * `from_excel`). `source` is an `.xlsx` file path or raw `Uint8Array` bytes;
 * `formatNumericColumns` (default true) re-applies AGS4 numeric formatting.
 *
 * With `agsPath` given the AGS4 is written there and the conversion stats are
 * returned; omit `agsPath` to get the AGS4 **bytes** back — so an uploaded `.xlsx`
 * never has to touch disk.
 */
export function fromExcel(
  source: string | Uint8Array,
  agsPath: string,
  opts?: { formatNumericColumns?: boolean },
): ExcelStats;
export function fromExcel(
  source: string | Uint8Array,
  agsPath?: undefined,
  opts?: { formatNumericColumns?: boolean },
): Buffer;
export function fromExcel(
  source: string | Uint8Array,
  agsPath?: string,
  opts: { formatNumericColumns?: boolean } = {},
): ExcelStats | Buffer {
  if (typeof source === "string" && agsPath !== undefined) {
    return excelToAgs4(source, agsPath, opts.formatNumericColumns);
  }
  const xlsxBytes = typeof source === "string" ? readFileSync(source) : source;
  const r = xlsxBytesToAgs4(xlsxBytes, opts.formatNumericColumns);
  if (agsPath === undefined) return r.bytes; // bytes-out
  writeFileSync(agsPath, r.bytes); // bytes-in → file
  return excelStatsOf(r);
}

export { Ags4File } from "./ags4-file";
export { AgsSubset, type Filter } from "./subset";
export type { DuckdbStats, QueryOptions, Row } from "./duckdb";
export { BuildResult, BuildSaved, type BuildFinding } from "./build-result";
export { FixResult, type AppliedFix } from "./fix-result";
export {
  Ags4Error,
  BadDictError,
  FileNotFoundError,
  MergeConflictError,
  NotAgs4Error,
  StaleCertError,
  UnsupportedEditionError,
  WorldCheckRequiresSourceError,
} from "./errors";
export { Report, type RuleFinding } from "./report";
export { engineFingerprint, engineVersion, version } from "./native";
export type { ExcelStats, Finding, GroupMeta, Sidecar } from "./native";
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
