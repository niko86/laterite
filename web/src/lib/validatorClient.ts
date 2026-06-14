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
} from "./validator";
import type { WorkerReq, WorkerRes, ReportMeta } from "./validator.worker";
import type { GroupMeta } from "./duckTypes";

type Pending =
  | { kind: "report"; resolve: (r: ValidationReport) => void; reject: (e: Error) => void }
  | { kind: "gzip"; resolve: (r: GzipResult) => void; reject: (e: Error) => void }
  | { kind: "fixes"; resolve: (r: Fix[]) => void; reject: (e: Error) => void }
  | { kind: "applied"; resolve: (r: Uint8Array) => void; reject: (e: Error) => void }
  | { kind: "parsed"; resolve: (g: GroupMeta[]) => void; reject: (e: Error) => void }
  | { kind: "arrow"; resolve: (b: Uint8Array) => void; reject: (e: Error) => void }
  | { kind: "revisionDelta"; resolve: (d: RevisionDelta) => void; reject: (e: Error) => void }
  | { kind: "dictionary"; resolve: (d: StandardDict) => void; reject: (e: Error) => void }
  | { kind: "toAgs4"; resolve: (r: ExportResult) => void; reject: (e: Error) => void };

export interface GzipResult {
  bytes: ArrayBuffer;
  meta: ReportMeta;
}

const worker = new Worker(
  new URL("./validator.worker.ts", import.meta.url),
  { type: "module" },
);

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
  } else if (msg.kind === "dictionary" && p.kind === "dictionary") {
    p.resolve(msg.dict);
  } else if (msg.kind === "toAgs4" && p.kind === "toAgs4") {
    p.resolve(msg.result);
  } else {
    p.reject(new Error(`unexpected ${msg.kind} response for ${p.kind} request`));
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
  worker.postMessage({ ...req, id, bytes: copy } as WorkerReq, [copy]);
  return id;
}

// For requests that carry no bytes (e.g. arrowIpc, which reads the worker-
// held dataset). No transfer list.
function postBare(req: ReqInit): number {
  const id = nextId++;
  worker.postMessage({ ...req, id } as WorkerReq);
  return id;
}

// For requests carrying TWO byte buffers (the revision diff). Transfer
// copies so the caller's originals stay intact (same rationale as post()).
function postDual(req: ReqInit, a: Uint8Array, b: Uint8Array): number {
  const id = nextId++;
  const aCopy = a.slice().buffer;
  const bCopy = b.slice().buffer;
  worker.postMessage({ ...req, id, aBytes: aCopy, bBytes: bCopy } as WorkerReq, [
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
      },
      bytes,
    );
    pending.set(id, { kind: "gzip", resolve, reject });
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
 *  held dataset set by the last parseDataset(). */
export function arrowIpc(code: string): Promise<Uint8Array> {
  return new Promise((resolve, reject) => {
    const id = postBare({ kind: "arrowIpc", code });
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
