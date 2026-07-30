// Main-thread handle to the validator worker. Components never touch the
// wasm or the worker directly — they call `validate()` / `validateGzip()`
// here and get a Promise back. Request/response are correlated by a
// monotonic id; Solid's `createResource` already discards the result of a
// superseded run, so this layer just needs faithful id correlation.

import type {
  ValidationReport,
  DictVersionOpt,
  EncodingOpt,
  EmitMode,
  ExportResult,
  Fix,
  RevisionDelta,
  StandardDict,
  TypeClashMode,
} from "./validator";
import type {
  WorkerReq,
  WorkerRes,
  ReportMeta,
  CensorTally,
} from "./validator.worker";
import type { GroupMeta } from "./duckTypes";

type Pending =
  | {
      kind: "report";
      resolve: (r: ValidationReport) => void;
      reject: (e: Error) => void;
    }
  | {
      kind: "cert";
      resolve: (json: string) => void;
      reject: (e: Error) => void;
    }
  | {
      kind: "gzip";
      resolve: (r: GzipResult) => void;
      reject: (e: Error) => void;
    }
  | { kind: "fixes"; resolve: (r: Fix[]) => void; reject: (e: Error) => void }
  | {
      kind: "applied";
      resolve: (r: Uint8Array) => void;
      reject: (e: Error) => void;
    }
  | {
      kind: "parsed";
      resolve: (g: GroupMeta[]) => void;
      reject: (e: Error) => void;
    }
  | {
      kind: "arrow";
      resolve: (b: Uint8Array) => void;
      reject: (e: Error) => void;
    }
  | {
      kind: "revisionDelta";
      resolve: (d: RevisionDelta) => void;
      reject: (e: Error) => void;
    }
  | {
      kind: "mergeResult";
      resolve: (r: MergeConversion) => void;
      reject: (e: Error) => void;
    }
  | {
      kind: "censor";
      resolve: (r: CensorResult) => void;
      reject: (e: Error) => void;
    }
  | {
      kind: "dictionary";
      resolve: (d: StandardDict) => void;
      reject: (e: Error) => void;
    }
  | {
      kind: "toAgs4";
      resolve: (r: ExportResult) => void;
      reject: (e: Error) => void;
    }
  | {
      kind: "excel";
      resolve: (r: ExcelConversion) => void;
      reject: (e: Error) => void;
    };

export interface GzipResult {
  bytes: ArrayBuffer;
  meta: ReportMeta;
}

/** A runtime custom AGS4 dictionary (#568) for `validate`/`certify`: raw `.ags` or
 *  JSON `bytes` (the browser's only form — no filesystem), and `replace` to drop the
 *  bundled base so the dict fully replaces the standard rather than overlaying it.
 *  Omit the argument entirely to validate against the bundled edition. */
export interface CustomDict {
  bytes: Uint8Array;
  replace?: boolean;
}

/** The result of an Excel conversion: the output file bytes (`.xlsx` for
 *  export, `.ags` for import) plus the engine's warnings and sheet/row counts.
 *  `bytes` is an ArrayBuffer (a `BlobPart`, ready for `downloadBlob`), matching
 *  the gzip-download path. */
export interface ExcelConversion {
  bytes: ArrayBuffer;
  warnings: string[];
  sheets: number;
  rows: number;
}

/** One advisory note from a merge (a recency contradiction, a non-X type widen,
 *  a missing merge-TRAN stamp). Mirrors the wire shape the engine serialises. */
export interface MergeWarning {
  kind: string;
  group: string | null;
  heading: string | null;
  message: string;
}

/** One per-row content revision — a later file changed a KEY-matched row. */
export interface MergeRevision {
  group: string;
  key: string[];
  changed: string[];
  winnerFile: number;
}

/** The result of a merge: the reconciled `.ags` `bytes` (an ArrayBuffer, a
 *  `BlobPart` ready for `downloadBlob`) plus the warnings and per-row revisions
 *  audit the Tools UI surfaces. */
export interface MergeConversion {
  bytes: ArrayBuffer;
  warnings: MergeWarning[];
  revisions: MergeRevision[];
}

/** The result of an anonymise: the scrubbed file `text` (a `BlobPart` for
 *  `downloadBlob`) plus the per-action `tally` (cells pseudonymised/blanked/
 *  tokenised/bracket-stripped, and custom groups/columns dropped). */
export interface CensorResult {
  text: string;
  tally: CensorTally;
}

const worker = new Worker(new URL("./validator.worker.ts", import.meta.url), {
  type: "module",
});

let nextId = 1;
const pending = new Map<number, Pending>();

// Resolves when the worker has instantiated the wasm; rejects if init
// failed. Panes gate their first render on this (mirrors the old
// main-thread `initValidator()` await).
const readyPromise = new Promise<void>((resolve, reject) => {
  const onInit = (e: MessageEvent<WorkerRes>) => {
    const msg = e.data;
    if ("type" in msg && msg.type === "ready") {
      worker.removeEventListener("message", onInit);
      resolve();
      // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- worker messages are a runtime boundary; keep the explicit check though the type narrows to it
    } else if ("type" in msg && msg.type === "initError") {
      worker.removeEventListener("message", onInit);
      reject(new Error(msg.error));
    }
  };
  worker.addEventListener("message", onInit);
});

worker.addEventListener("message", (e: MessageEvent<WorkerRes>) => {
  const msg = e.data;
  if ("type" in msg) return; // ready / initError handled above
  const p = pending.get(msg.id);
  if (!p) return; // superseded + already dropped
  pending.delete(msg.id);
  if (!msg.ok) {
    p.reject(new Error(msg.error));
  } else if (msg.kind === "report" && p.kind === "report") {
    p.resolve(msg.report);
  } else if (msg.kind === "cert" && p.kind === "cert") {
    p.resolve(msg.json);
  } else if (msg.kind === "gzip" && p.kind === "gzip") {
    p.resolve({ bytes: msg.bytes, meta: msg.report });
  } else if (msg.kind === "fixes" && p.kind === "fixes") {
    p.resolve(msg.fixes);
  } else if (msg.kind === "applied" && p.kind === "applied") {
    p.resolve(new Uint8Array(msg.bytes));
  } else if (msg.kind === "parsed" && p.kind === "parsed") {
    p.resolve(msg.groups);
  } else if (msg.kind === "arrow" && p.kind === "arrow") {
    p.resolve(new Uint8Array(msg.bytes));
  } else if (msg.kind === "revisionDelta" && p.kind === "revisionDelta") {
    p.resolve(msg.delta);
  } else if (msg.kind === "mergeResult" && p.kind === "mergeResult") {
    p.resolve({
      bytes: msg.bytes,
      warnings: JSON.parse(msg.warningsJson) as MergeWarning[],
      revisions: JSON.parse(msg.revisionsJson) as MergeRevision[],
    });
  } else if (msg.kind === "dictionary" && p.kind === "dictionary") {
    p.resolve(msg.dict);
  } else if (msg.kind === "censor" && p.kind === "censor") {
    p.resolve({ text: msg.text, tally: msg.tally });
  } else if (msg.kind === "toAgs4" && p.kind === "toAgs4") {
    p.resolve(msg.result);
  } else if (msg.kind === "excel" && p.kind === "excel") {
    p.resolve({
      bytes: msg.bytes,
      warnings: msg.warnings,
      sheets: msg.sheets,
      rows: msg.rows,
    });
  } else {
    p.reject(
      new Error(`unexpected ${msg.kind} response for ${p.kind} request`),
    );
  }
});

worker.addEventListener("error", (e) => {
  // A hard worker error rejects everything in flight rather than hanging.
  const err = new Error(e.message || "validator worker crashed");
  for (const [, p] of pending) p.reject(err);
  pending.clear();
});

export function ready(): Promise<void> {
  return readyPromise;
}

// Omit over a discriminated union must DISTRIBUTE, else only the keys
// common to every member survive (dropping `dict`/`fixes`/`code`/… from the
// per-kind requests). The built-in Omit doesn't distribute, so spell it out.
type DistributiveOmit<T, K extends keyof never> = T extends unknown
  ? Omit<T, K>
  : never;
type ReqInit = DistributiveOmit<WorkerReq, "id">;

// Send `bytes` to the worker as a transferable. We transfer a *copy*
// (`slice()`) so the caller's original Uint8Array stays intact — the main
// thread still needs it to decode the editor text + finding snippets.
function post(req: ReqInit, bytes: Uint8Array): number {
  const id = nextId++;
  const copy = bytes.slice().buffer;
  worker.postMessage({ ...req, id, bytes: copy }, [copy]);
  return id;
}

// For requests that carry no bytes (e.g. arrowIpc, which reads the worker-
// held dataset). No transfer list.
function postBare(req: ReqInit): number {
  const id = nextId++;
  worker.postMessage({ ...req, id });
  return id;
}

// For requests carrying TWO byte buffers (the revision diff). Transfer
// copies so the caller's originals stay intact (same rationale as post()).
function postDual(req: ReqInit, a: Uint8Array, b: Uint8Array): number {
  const id = nextId++;
  const aCopy = a.slice().buffer;
  const bCopy = b.slice().buffer;
  worker.postMessage({ ...req, id, aBytes: aCopy, bBytes: bCopy }, [
    aCopy,
    bCopy,
  ]);
  return id;
}

/** Validate, returning the (capped, per `maxPerRule`) report. */
export function validate(
  bytes: Uint8Array,
  dictVersion: DictVersionOpt,
  includeFyi: boolean,
  encoding: EncodingOpt,
  maxPerRule: number | null,
  dict?: CustomDict,
): Promise<ValidationReport> {
  return new Promise((resolve, reject) => {
    const id = post(
      {
        kind: "validate",
        bytes: new ArrayBuffer(0), // replaced inside post()
        dict: dictVersion === "auto" ? null : dictVersion,
        includeFyi,
        encoding,
        maxPerRule,
        dictBytes: dict?.bytes,
        dictReplace: dict?.replace ?? false,
      },
      bytes,
    );
    pending.set(id, { kind: "report", resolve, reject });
  });
}

/** Validate *uncapped* and gzip the JSON in the worker, returning the
 *  compressed bytes (the multi-hundred-MB string never reaches the main
 *  thread). Used by the "Download full report" action. */
export function validateGzip(
  bytes: Uint8Array,
  dictVersion: DictVersionOpt,
  includeFyi: boolean,
  encoding: EncodingOpt,
  dict?: CustomDict,
): Promise<GzipResult> {
  return new Promise((resolve, reject) => {
    const id = post(
      {
        kind: "validate",
        bytes: new ArrayBuffer(0),
        dict: dictVersion === "auto" ? null : dictVersion,
        includeFyi,
        encoding,
        maxPerRule: null,
        gzip: true,
        dictBytes: dict?.bytes,
        dictReplace: dict?.replace ?? false,
      },
      bytes,
    );
    pending.set(id, { kind: "gzip", resolve, reject });
  });
}

/** Mint a `.ags.idx` certificate for a clean file. Round-trips through the
 *  worker (the only wasm owner). Rejects if the file has findings or can't be
 *  parsed. The browser supplies the timestamp — wasm has no clock. */
export function certify(
  bytes: Uint8Array,
  dictVersion: DictVersionOpt,
  encoding: EncodingOpt,
  dict?: CustomDict,
): Promise<string> {
  return new Promise((resolve, reject) => {
    const id = post(
      {
        kind: "certify",
        bytes: new ArrayBuffer(0),
        dict: dictVersion === "auto" ? null : dictVersion,
        encoding,
        checkedAt: new Date().toISOString(),
        dictBytes: dict?.bytes,
        dictReplace: dict?.replace ?? false,
      },
      bytes,
    );
    pending.set(id, { kind: "cert", resolve, reject });
  });
}

/** Compute the safe fixes for a file. Round-trips through the worker (the
 *  only wasm owner), so it's async. Resolves to [] on a parse error (the
 *  engine returns no fixes for an un-parseable file). */
export function computeFixes(
  bytes: Uint8Array,
  dictVersion: DictVersionOpt,
  encoding: EncodingOpt,
): Promise<Fix[]> {
  return new Promise((resolve, reject) => {
    const id = post(
      {
        kind: "computeFixes",
        bytes: new ArrayBuffer(0), // replaced inside post()
        dict: dictVersion === "auto" ? null : dictVersion,
        encoding,
      },
      bytes,
    );
    pending.set(id, { kind: "fixes", resolve, reject });
  });
}

/** Apply the selected fixes, resolving to the new file as UTF-8 bytes.
 *  Callers should reset their encoding select to "utf-8" after, since the
 *  engine always re-encodes the result as UTF-8. */
export function applyFixes(
  bytes: Uint8Array,
  encoding: EncodingOpt,
  fixes: Fix[],
): Promise<Uint8Array> {
  return new Promise((resolve, reject) => {
    const id = post(
      {
        kind: "applyFixes",
        bytes: new ArrayBuffer(0), // replaced inside post()
        encoding,
        fixes,
      },
      bytes,
    );
    pending.set(id, { kind: "applied", resolve, reject });
  });
}

/** Parse the file to a typed dataset held in the worker; resolves to the
 *  per-group schema. Each group's typed Arrow IPC is pulled separately via
 *  arrowIpc(). Operates on the most-recently-parsed dataset, so a caller
 *  must drain all arrowIpc() pulls for one file before parsing the next. */
export function parseDataset(
  bytes: Uint8Array,
  encoding: EncodingOpt,
): Promise<GroupMeta[]> {
  return new Promise((resolve, reject) => {
    const id = post(
      { kind: "parse", bytes: new ArrayBuffer(0), encoding },
      bytes,
    );
    pending.set(id, { kind: "parsed", resolve, reject });
  });
}

/** Pull one group's typed Arrow IPC stream (Uint8Array) from the worker-
 *  held dataset set by the last parseDataset(). `keys=true` includes the
 *  content-addressed `_id`/`_parent_id` columns — pass it when ingesting into
 *  duckdb-wasm so cross-group joins resolve (#303). `contentHash=true` appends
 *  the trailing `_content_hash` value fingerprint, same opt-in shape (#448). */
export function arrowIpc(
  code: string,
  keys = false,
  contentHash = false,
): Promise<Uint8Array> {
  return new Promise((resolve, reject) => {
    const id = postBare({ kind: "arrowIpc", code, keys, contentHash });
    pending.set(id, { kind: "arrow", resolve, reject });
  });
}

/** Compare two AGS4 files (baseline `a`, revision `b`) — the engine-
 *  consistent, KEY-aware, type-aware revision diff. `maxRowsPerGroup` caps
 *  serialized per-row deltas (counts stay true totals); null = uncapped. */
export function revisionDiff(
  a: Uint8Array,
  b: Uint8Array,
  encoding: EncodingOpt,
  maxRowsPerGroup: number | null,
): Promise<RevisionDelta> {
  return new Promise((resolve, reject) => {
    const id = postDual(
      {
        kind: "revisionDiff",
        aBytes: new ArrayBuffer(0), // replaced inside postDual()
        bBytes: new ArrayBuffer(0), // replaced inside postDual()
        encoding,
        maxRowsPerGroup,
      },
      a,
      b,
    );
    pending.set(id, { kind: "revisionDelta", resolve, reject });
  });
}

/** Merge two AGS4 deliveries (`a` then `b`, `b` wins a KEY conflict) into one
 *  file — the engine-consistent, KEY-aware reconciliation. `onTypeClash` settles a
 *  heading the two files typed differently: `"error"` rejects, `"widen"` falls back
 *  to `X` (raw values kept, TYPE thrown away), `"promote"` keeps the greatest `nDP`
 *  precision and zero-pads the coarser values. A conflicting UNIT is fatal in every
 *  mode. The optional `tran*` stamp a synthesised merge-TRAN. Rejects (an unsettled
 *  clash / parse error) with the engine message. */
/** The transmission a merged file represents. All five members are required
 *  together — they are REQUIRED TRAN headings, so the engine rejects a partial
 *  stamp rather than writing blank cells that then fail Rule 10b. */
export interface TranStamp {
  issue: string;
  date: string;
  producer: string;
  recipient: string;
  status: string;
  description?: string;
  remarks?: string;
}

export function mergeFiles(
  a: Uint8Array,
  b: Uint8Array,
  opts: {
    encoding: EncodingOpt;
    onTypeClash: TypeClashMode;
    tran?: TranStamp | null;
  },
): Promise<MergeConversion> {
  return new Promise((resolve, reject) => {
    const id = postDual(
      {
        kind: "merge",
        aBytes: new ArrayBuffer(0), // replaced inside postDual()
        bBytes: new ArrayBuffer(0), // replaced inside postDual()
        encoding: opts.encoding,
        onTypeClash: opts.onTypeClash,
        tran: opts.tran ?? null,
      },
      a,
      b,
    );
    pending.set(id, { kind: "mergeResult", resolve, reject });
  });
}

/** Anonymise a file with the shared scrub engine (Tools → Anonymiser, #581) —
 *  the same `laterite-ags4-censor` engine the corpus tool drives. `sensitiveJson`
 *  is the classification SSOT text; `selectedCodes` (null = every classified
 *  heading) restricts scrubbing to the user's ticked columns; `token` is the
 *  replacement; `dropCustom` removes non-dictionary groups/columns;
 *  `includeFreetext` tokenises descriptions instead of stripping `[units]`.
 *  Resolves to the scrubbed text + the per-action tally. */
export function censorFile(
  bytes: Uint8Array,
  opts: {
    sensitiveJson: string;
    selectedCodes: string[] | null;
    token: string;
    dropCustom: boolean;
    includeFreetext: boolean;
  },
): Promise<CensorResult> {
  return new Promise((resolve, reject) => {
    const id = post(
      {
        kind: "censor",
        bytes: new ArrayBuffer(0), // replaced inside post()
        sensitiveJson: opts.sensitiveJson,
        selectedCodes: opts.selectedCodes,
        token: opts.token,
        dropCustom: opts.dropCustom,
        includeFreetext: opts.includeFreetext,
      },
      bytes,
    );
    pending.set(id, { kind: "censor", resolve, reject });
  });
}

/** The bundled STANDARD dictionary for an edition (Tools reference) — the real
 *  per-edition AGS4 dictionary (canonical names + descriptions + units + types
 *  + status). `edition`: "auto"/null → the fallback edition; else 4.0.3…4.2. */
export function dictionary(
  edition: DictVersionOpt | null,
): Promise<StandardDict> {
  return new Promise((resolve, reject) => {
    const id = postBare({
      kind: "dictionary",
      edition: edition && edition !== "auto" ? edition : null,
    });
    pending.set(id, { kind: "dictionary", resolve, reject });
  });
}

/** AGS4 bytes → an `.xlsx` workbook (Tools → Excel, export). One sheet per
 *  group, python-ags4's layout. Rejects if the file has no valid AGS4 groups. */
export function excelExport(bytes: Uint8Array): Promise<ExcelConversion> {
  return new Promise((resolve, reject) => {
    const id = post({ kind: "excelExport", bytes: new ArrayBuffer(0) }, bytes);
    pending.set(id, { kind: "excel", resolve, reject });
  });
}

/** An `.xlsx` workbook's bytes → AGS4 (Tools → Excel, import). Non-Rule-19
 *  columns and non-UNIT/TYPE/DATA rows are dropped (surfaced in `warnings`).
 *  `formatNumeric` re-pads DATA cells to their column's TYPE. Rejects if no
 *  sheet yields a valid group. */
export function excelImport(
  bytes: Uint8Array,
  formatNumeric: boolean,
): Promise<ExcelConversion> {
  return new Promise((resolve, reject) => {
    const id = post(
      { kind: "excelImport", bytes: new ArrayBuffer(0), formatNumeric },
      bytes,
    );
    pending.set(id, { kind: "excel", resolve, reject });
  });
}

/** Build valid AGS4 from per-group data (Export tab). `groupsJson` is the
 *  `[{code, headings, units?, types?, rows}, …]` shape; `mode` is autofix /
 *  report / strict. Rejects on invalid JSON or a strict-mode rejection. */
export function toAgs4(
  groupsJson: string,
  edition: DictVersionOpt | null,
  mode: EmitMode,
): Promise<ExportResult> {
  return new Promise((resolve, reject) => {
    const id = postBare({
      kind: "toAgs4",
      groupsJson,
      edition: edition && edition !== "auto" ? edition : null,
      mode,
    });
    pending.set(id, { kind: "toAgs4", resolve, reject });
  });
}
