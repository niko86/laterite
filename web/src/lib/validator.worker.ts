// The validator runs here, off the main thread, so a pathologically
// dirty file (millions of findings) can churn for tens of seconds without
// ever freezing the UI. The worker owns the single wasm instance; the
// main thread talks to it only through `validatorClient.ts`.
//
// Protocol: every request carries a monotonic `id`; every response echoes
// it so the client can correlate (and discard superseded runs). The wasm
// `validate()` is synchronous and uninterruptible once entered, so
// "cancellation" is necessarily *discard the stale result*, not *abort
// mid-rule* — but because it runs here, a superseded run never blocks the
// next paint.

import init, {
  validate,
  compute_fixes,
  apply_fixes,
  parse,
  diff,
  dictionary,
} from "../wasm/ags4_wasm.js";
import type { ParsedDataset } from "../wasm/ags4_wasm.js";
import wasmUrl from "../wasm/ags4_wasm_bg.wasm?url";
import type {
  ValidationReport,
  Fix,
  RevisionDelta,
  StandardDict,
} from "./validator";
import type { GroupMeta } from "./duckTypes";

export interface ValidateReq {
  id: number;
  kind: "validate";
  /** Transferred from the main thread; the worker views it as bytes. */
  bytes: ArrayBuffer;
  dict: string | null;
  includeFyi: boolean;
  encoding: string;
  /** `null` = uncapped (the full-report download path). */
  maxPerRule: number | null;
  /** When set, gzip the JSON in-worker and return bytes, not the object —
   *  keeps the multi-hundred-MB string off the main thread (Phase: download). */
  gzip?: boolean;
}
/** Compute the safe fixes for a file (the FYI-on engine path). */
export interface ComputeFixesReq {
  id: number;
  kind: "computeFixes";
  bytes: ArrayBuffer;
  dict: string | null;
  encoding: string;
}
/** Apply a selected subset of fixes; the worker returns the new file as
 *  UTF-8 bytes (transferred back, so the buffer never copies twice). */
export interface ApplyFixesReq {
  id: number;
  kind: "applyFixes";
  bytes: ArrayBuffer;
  encoding: string;
  fixes: Fix[];
}
/** Parse a file to a typed dataset, held in the worker. Returns the per-
 *  group schema; the heavy Arrow IPC is pulled lazily per group via
 *  `arrowIpc`, so peak residency stays at one group's batch. */
export interface ParseReq {
  id: number;
  kind: "parse";
  bytes: ArrayBuffer;
  encoding: string;
}
/** Pull one group's typed Arrow IPC stream from the held ParsedDataset.
 *  Carries no bytes — it reads the most recently parsed dataset. */
export interface ArrowReq {
  id: number;
  kind: "arrowIpc";
  code: string;
}
/** Compare two AGS4 files (Tools → Revision diff). Carries both buffers,
 *  transferred; returns the KEY-aware, type-aware structured delta. */
export interface RevisionDiffReq {
  id: number;
  kind: "revisionDiff";
  aBytes: ArrayBuffer;
  bBytes: ArrayBuffer;
  encoding: string;
  /** per-group cap on serialized row deltas (counts stay true totals). */
  maxRowsPerGroup: number | null;
}
/** Load the bundled STANDARD dictionary for an edition (Tools reference).
 *  Carries no file bytes — it reads the engine's own per-edition dict. */
export interface DictionaryReq {
  id: number;
  kind: "dictionary";
  /** "auto"/null → the fallback edition; else 4.0.3|4.0.4|4.1|4.1.1|4.2. */
  edition: string | null;
}
export type WorkerReq =
  | ValidateReq
  | ComputeFixesReq
  | ApplyFixesReq
  | ParseReq
  | ArrowReq
  | RevisionDiffReq
  | DictionaryReq;

export type WorkerRes =
  | { type: "ready" }
  | { type: "initError"; error: string }
  | { id: number; ok: true; kind: "report"; report: ValidationReport }
  | { id: number; ok: true; kind: "gzip"; bytes: ArrayBuffer; report: ReportMeta }
  | { id: number; ok: true; kind: "fixes"; fixes: Fix[] }
  | { id: number; ok: true; kind: "applied"; bytes: ArrayBuffer }
  | { id: number; ok: true; kind: "parsed"; groups: GroupMeta[] }
  | { id: number; ok: true; kind: "arrow"; code: string; bytes: ArrayBuffer }
  | { id: number; ok: true; kind: "revisionDelta"; delta: RevisionDelta }
  | { id: number; ok: true; kind: "dictionary"; dict: StandardDict }
  | { id: number; ok: false; error: string };

/** The header counts of a report, sent alongside gzipped bytes so the UI
 *  can label the download without re-parsing the compressed payload. */
export interface ReportMeta {
  finding_count: number;
  shown_count: number;
}

// Under the DOM lib (tsconfig), the dedicated-worker global's
// transfer-list `postMessage(message, transfer)` overload isn't visible;
// the `Worker` type carries exactly that signature, so route every reply
// through it (also gives a single typed choke point for `WorkerRes`).
const ctx = self as unknown as Worker;
const reply = (msg: WorkerRes, transfer?: Transferable[]) =>
  transfer ? ctx.postMessage(msg, transfer) : ctx.postMessage(msg);

// Instantiate once. Passing the bundled-asset URL explicitly avoids the
// import.meta.url fetch fallback (which breaks under a non-root `base`).
// Every handler awaits this, so requests that arrive before init simply
// queue behind it rather than racing a live-before-ready `validate`.
const ready: Promise<void> = init({ module_or_path: wasmUrl }).then(
  () => undefined,
);

ready.then(
  () => reply({ type: "ready" }),
  (e) => reply({ type: "initError", error: String(e) }),
);

// The most-recently parsed dataset (the Explore/Tools typed-data path).
// Held here because `arrow_ipc(code)` builds each group's Arrow batch
// lazily AND drops it on return, so the dataset must outlive the parse call
// to serve per-group pulls. Freed before a new parse so wasm memory stays
// at one dataset. Callers must drain all `arrowIpc` pulls for a parse
// before issuing the next parse.
let dataset: ParsedDataset | null = null;

async function gzipBytes(json: string): Promise<ArrayBuffer> {
  // Stream-compress so peak memory stays bounded (one chunk at a time)
  // rather than holding source + full compressed copy simultaneously.
  const stream = new Blob([json])
    .stream()
    .pipeThrough(new CompressionStream("gzip"));
  return await new Response(stream).arrayBuffer();
}

self.onmessage = async (e: MessageEvent<WorkerReq>) => {
  const req = e.data;
  try {
    await ready;

    if (req.kind === "computeFixes") {
      const fixes = compute_fixes(
        new Uint8Array(req.bytes),
        req.dict,
        req.encoding,
      ) as Fix[];
      reply({ id: req.id, ok: true, kind: "fixes", fixes });
      return;
    }

    if (req.kind === "applyFixes") {
      const out = apply_fixes(
        new Uint8Array(req.bytes),
        req.encoding,
        req.fixes,
      ) as Uint8Array;
      // Transfer the result bytes back in a standalone buffer (slice()
      // gives one sized exactly to the output, safe to transfer).
      const buf = out.slice().buffer;
      reply({ id: req.id, ok: true, kind: "applied", bytes: buf }, [buf]);
      return;
    }

    if (req.kind === "parse") {
      dataset?.free();
      dataset = parse(new Uint8Array(req.bytes), req.encoding);
      const groups: GroupMeta[] = dataset.group_codes().map((code) => {
        const m = dataset!.meta(code) as Omit<GroupMeta, "code"> | null;
        return m
          ? { code, ...m }
          : { code, headings: [], units: [], types: [], sql_types: [] };
      });
      reply({ id: req.id, ok: true, kind: "parsed", groups });
      return;
    }

    if (req.kind === "arrowIpc") {
      if (!dataset) throw new Error("no parsed dataset — call parse first");
      const out = dataset.arrow_ipc(req.code) as Uint8Array;
      const buf = out.slice().buffer;
      reply({ id: req.id, ok: true, kind: "arrow", code: req.code, bytes: buf }, [
        buf,
      ]);
      return;
    }

    if (req.kind === "revisionDiff") {
      const delta = diff(
        new Uint8Array(req.aBytes),
        new Uint8Array(req.bBytes),
        req.encoding,
        req.maxRowsPerGroup ?? undefined,
      ) as RevisionDelta;
      reply({ id: req.id, ok: true, kind: "revisionDelta", delta });
      return;
    }

    if (req.kind === "dictionary") {
      const dict = dictionary(req.edition ?? undefined) as StandardDict;
      reply({ id: req.id, ok: true, kind: "dictionary", dict });
      return;
    }

    const report = validate(
      new Uint8Array(req.bytes),
      req.dict,
      req.includeFyi,
      req.encoding,
      req.maxPerRule,
    ) as ValidationReport;

    if (req.gzip) {
      const bytes = await gzipBytes(JSON.stringify(report));
      reply(
        {
          id: req.id,
          ok: true,
          kind: "gzip",
          bytes,
          report: {
            finding_count: report.finding_count,
            shown_count: report.shown_count,
          },
        },
        [bytes],
      );
    } else {
      reply({ id: req.id, ok: true, kind: "report", report });
    }
  } catch (err) {
    reply({ id: req.id, ok: false, error: String(err) });
  }
};
