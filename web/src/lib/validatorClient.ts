// Main-thread handle to the engine workers. Components never touch the
// wasm or a worker directly — they call `validate()` / `validateGzip()`
// here and get a Promise back. Request/response are correlated by a
// monotonic id; Solid's `createResource` already discards the result of a
// superseded run, so this layer just needs faithful id correlation.
//
// There are TWO workers (#354, ags-wiki/design/dec-engine-tiering.md). The
// always-on one is created at module load and serves Validate, Fix, Export and
// most of Tools. The second is created the first time Explore or Tools → Excel
// is opened, and serves those two alone — which is what keeps `ParsedDataset`,
// and in #355 the whole tier-2 engine, off the path of everyone who opens
// neither. Both speak the same protocol, so one channel drives either.
//
// The channel itself — spawn, correlate, retire — is `workerChannel.ts` since
// #357, and the reply protocol — which reply kind resolves which request's
// promise — is `engineProtocol.ts` since #380, where a unit test can reach it.
// What is left here is ownership: the two live workers, and the typed request
// functions the panes call.

import type {
  ValidationReport,
  DictVersionOpt,
  EncodingOpt,
  EmitMode,
  ExportResult,
  Fix,
  RevisionDelta,
  TypeClashMode,
} from "./validator";
import type { GroupMeta } from "./duckTypes";
import type { WorkerReq, WorkerRes } from "./engineDispatch";
import { createChannel } from "./workerChannel";
import {
  settle,
  type Pending,
  type GzipResult,
  type ExcelConversion,
  type MergeConversion,
  type CensorResult,
} from "./engineProtocol";

/** Re-exported so a pane's whole engine surface still imports from here. */
export type {
  GzipResult,
  ExcelConversion,
  MergeWarning,
  MergeRevision,
  MergeConversion,
  CensorResult,
} from "./engineProtocol";

/** A runtime custom AGS4 dictionary (laterite-dev#568) for `validate`/`certify`: raw `.ags` or
 *  JSON `bytes` (the browser's only form — no filesystem), and `replace` to drop the
 *  bundled base so the dict fully replaces the standard rather than overlaying it.
 *  Omit the argument entirely to validate against the bundled edition. */
export interface CustomDict {
  bytes: Uint8Array;
  replace?: boolean;
}

// The always-on worker, created at module load exactly as it always has been:
// Validate is where a visitor lands, and its engine's deadline is the moment a
// file is loaded — which the sample buttons can reach in milliseconds.
const primary = createChannel<WorkerRes, WorkerReq, Pending>(
  () =>
    new Worker(new URL("./validator.worker.ts", import.meta.url), {
      type: "module",
    }),
  settle,
);
primary.start();

// The second worker: Explore's parse + Arrow pulls, and Tools → Excel's two
// conversions. Nothing here creates it — `startTier2Worker()` and the four ops
// that need it do, so it stays uncreated for a visit that opens neither tab.
const tier2 = createChannel<WorkerRes, WorkerReq, Pending>(
  () =>
    new Worker(new URL("./tier2.worker.ts", import.meta.url), {
      type: "module",
    }),
  settle,
);

/** Re-exported so a pane imports its whole engine surface from one module: the
 *  ops, and the one failure they distinguish. An op rejects with this when no
 *  engine is running — it never downloaded (#357) or its worker died (#363) —
 *  which is the case a retry can clear, since the channel has retired that
 *  worker and the next request starts a fresh one. `reason` says which, because
 *  the two are equally retryable and not equally explicable. */
export { EngineUnavailableError } from "./workerChannel";

export function ready(): Promise<void> {
  return primary.ready();
}

/** Bring the second engine worker up. Explore and Tools → Excel call this when
 *  they mount, so its wasm is instantiating while the user is still looking at
 *  the tab rather than starting when they finally click something. Idempotent —
 *  and the only thing besides a request to that worker that creates it, which is
 *  what an e2e in `web/e2e/app.spec.ts` holds us to. */
export function startTier2Worker(): void {
  tier2.start();
}

/** True once the second worker exists — so its engine is already downloading or
 *  downloaded. The idle warm (#356) checks this before priming the same 5.2 MB:
 *  a visitor who reaches Explore or Excel inside the idle window would otherwise
 *  fetch it twice, since CacheFirst has no request coalescing. Asking does NOT
 *  create the worker, which is the whole reason this isn't `ready()`. */
export function isTier2Started(): boolean {
  return tier2.started();
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
    primary.post(
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
      { kind: "report", resolve, reject },
    );
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
    primary.post(
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
      { kind: "gzip", resolve, reject },
    );
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
    primary.post(
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
      { kind: "cert", resolve, reject },
    );
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
    primary.post(
      {
        kind: "computeFixes",
        bytes: new ArrayBuffer(0), // replaced inside post()
        dict: dictVersion === "auto" ? null : dictVersion,
        encoding,
      },
      bytes,
      { kind: "fixes", resolve, reject },
    );
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
    primary.post(
      {
        kind: "applyFixes",
        bytes: new ArrayBuffer(0), // replaced inside post()
        encoding,
        fixes,
      },
      bytes,
      { kind: "applied", resolve, reject },
    );
  });
}

/** Parse the file to a typed dataset held in the SECOND worker; resolves to the
 *  per-group schema. Each group's typed Arrow IPC is pulled separately via
 *  arrowIpc(). Operates on the most-recently-parsed dataset, so a caller
 *  must drain all arrowIpc() pulls for one file before parsing the next — a
 *  contract the move doesn't touch, since both halves of it moved together. */
export function parseDataset(
  bytes: Uint8Array,
  encoding: EncodingOpt,
): Promise<GroupMeta[]> {
  return new Promise((resolve, reject) => {
    tier2.post({ kind: "parse", bytes: new ArrayBuffer(0), encoding }, bytes, {
      kind: "parsed",
      resolve,
      reject,
    });
  });
}

/** Pull one group's typed Arrow IPC stream (Uint8Array) from the second
 *  worker's dataset, set by the last parseDataset(). `keys=true` includes the
 *  content-addressed `_id`/`_parent_id` columns — pass it when ingesting into
 *  duckdb-wasm so cross-group joins resolve (#303). `contentHash=true` appends
 *  the trailing `_content_hash` value fingerprint, same opt-in shape (#448). */
export function arrowIpc(
  code: string,
  keys = false,
  contentHash = false,
): Promise<Uint8Array> {
  return new Promise((resolve, reject) => {
    tier2.postBare(
      { kind: "arrowIpc", code, keys, contentHash },
      { kind: "arrow", resolve, reject },
    );
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
    primary.postDual(
      {
        kind: "revisionDiff",
        aBytes: new ArrayBuffer(0), // replaced inside postDual()
        bBytes: new ArrayBuffer(0), // replaced inside postDual()
        encoding,
        maxRowsPerGroup,
      },
      a,
      b,
      { kind: "revisionDelta", resolve, reject },
    );
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
    primary.postDual(
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
      { kind: "mergeResult", resolve, reject },
    );
  });
}

/** Anonymise a file with the shared scrub engine (Tools → Anonymiser, laterite-dev#581) —
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
    primary.post(
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
      { kind: "censor", resolve, reject },
    );
  });
}

/** AGS4 bytes → an `.xlsx` workbook (Tools → Excel, export). One sheet per
 *  group, python-ags4's layout. Rejects if the file has no valid AGS4 groups. */
export function excelExport(bytes: Uint8Array): Promise<ExcelConversion> {
  return new Promise((resolve, reject) => {
    tier2.post({ kind: "excelExport", bytes: new ArrayBuffer(0) }, bytes, {
      kind: "excel",
      resolve,
      reject,
    });
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
    tier2.post(
      { kind: "excelImport", bytes: new ArrayBuffer(0), formatNumeric },
      bytes,
      { kind: "excel", resolve, reject },
    );
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
    primary.postBare(
      {
        kind: "toAgs4",
        groupsJson,
        edition: edition && edition !== "auto" ? edition : null,
        mode,
      },
      { kind: "toAgs4", resolve, reject },
    );
  });
}
