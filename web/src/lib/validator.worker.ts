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
  certify,
  compute_fixes,
  apply_fixes,
  read,
  diff,
  merge,
  censor,
  dictionary,
  build_ags4,
  ags4_to_xlsx,
  xlsx_to_ags4,
} from "../wasm/ags4_wasm.js";
import type { ParsedDataset } from "../wasm/ags4_wasm.js";
import wasmUrl from "../wasm/ags4_wasm_bg.wasm?url";
import type {
  ValidationReport,
  Fix,
  RevisionDelta,
  StandardDict,
  ExportResult,
  TypeClashMode,
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
  /** An optional custom AGS4 dictionary as raw bytes (`.ags` or JSON), #568 — the
   *  browser's only form (no filesystem). `dictReplace` drops the bundled base so
   *  the dict fully replaces the standard. Omitted ⇒ the bundled edition. */
  dictBytes?: Uint8Array;
  dictReplace?: boolean;
}
/** Compute the safe fixes for a file (the FYI-on engine path). */
export interface ComputeFixesReq {
  id: number;
  kind: "computeFixes";
  bytes: ArrayBuffer;
  dict: string | null;
  encoding: string;
}
/** Mint a `.ags.idx` certificate for a clean file (Validate → download).
 *  Carries the file bytes + the browser's RFC-3339 timestamp (wasm has no
 *  clock, so the caller supplies `checkedAt`). */
export interface CertifyReq {
  id: number;
  kind: "certify";
  bytes: ArrayBuffer;
  dict: string | null;
  encoding: string;
  checkedAt: string;
  /** Mint against a custom dictionary (#568); the cert records its `{name, hash}`
   *  so a later `validate --index` re-validates when the effective dict differs. */
  dictBytes?: Uint8Array;
  dictReplace?: boolean;
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
  /** Include the content-addressed `_id`/`_parent_id` key columns (#303). */
  keys?: boolean;
  /** Include the trailing `_content_hash` value fingerprint (#448). */
  contentHash?: boolean;
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
/** Merge two AGS4 deliveries into one file (Tools → Merge). Carries both
 *  buffers, transferred; `onTypeClash` settles a TYPE disagreement ("error" |
 *  "widen" → X | "promote" → greatest nDP precision, coarser values zero-padded);
 *  the optional `tran*` fields stamp a synthesised merge-TRAN. */
export interface MergeReq {
  id: number;
  kind: "merge";
  aBytes: ArrayBuffer;
  bBytes: ArrayBuffer;
  encoding: string;
  onTypeClash: TypeClashMode;
  tranIssue: string | null;
  tranDate: string | null;
  tranProducer: string | null;
}
/** Per-action cell/structure counts from the shared scrub engine — the leaf's
 *  `Tally`, snake_case as serialised across the wasm boundary. */
export interface CensorTally {
  pseudonym: number;
  blank: number;
  token: number;
  brackets: number;
  keyword: number;
  dropped_cols: number;
  dropped_groups: number;
  dropped_defs: number;
}
/** Anonymise a file with the shared scrub engine (Tools → Anonymiser, #581).
 *  Carries the file bytes, transferred; `sensitiveJson` is the classification
 *  SSOT; `selectedCodes` (null = every classified heading) restricts the policy
 *  to the user's ticked columns; `token` replaces token/brackets hits;
 *  `dropCustom` removes non-dictionary groups/columns; `includeFreetext`
 *  tokenises descriptions instead of stripping their `[units]`. */
export interface CensorReq {
  id: number;
  kind: "censor";
  bytes: ArrayBuffer;
  sensitiveJson: string;
  selectedCodes: string[] | null;
  token: string;
  dropCustom: boolean;
  includeFreetext: boolean;
}
/** Load the bundled STANDARD dictionary for an edition (Tools reference).
 *  Carries no file bytes — it reads the engine's own per-edition dict. */
export interface DictionaryReq {
  id: number;
  kind: "dictionary";
  /** "auto"/null → the fallback edition; else 4.0.3|4.0.4|4.1|4.1.1|4.2. */
  edition: string | null;
}
/** Build valid AGS4 from per-group data (Export tab). Carries no file bytes
 *  — `groupsJson` is the `[{code, headings, rows}, …]` shape `build_ags4` wants. */
export interface ToAgs4Req {
  id: number;
  kind: "toAgs4";
  groupsJson: string;
  /** "auto"/null → 4.1.1; else 4.0.3|4.0.4|4.1|4.1.1|4.2. */
  edition: string | null;
  /** autofix | report | strict. */
  mode: string;
}
/** AGS4 bytes → an `.xlsx` workbook (Tools → Excel, export direction). */
export interface ExcelExportReq {
  id: number;
  kind: "excelExport";
  bytes: ArrayBuffer;
}
/** An `.xlsx` workbook's bytes → AGS4 (Tools → Excel, import direction).
 *  `formatNumeric` re-pads DATA cells to their column's TYPE. */
export interface ExcelImportReq {
  id: number;
  kind: "excelImport";
  bytes: ArrayBuffer;
  formatNumeric: boolean;
}
export type WorkerReq =
  | ValidateReq
  | CertifyReq
  | ComputeFixesReq
  | ApplyFixesReq
  | ParseReq
  | ArrowReq
  | RevisionDiffReq
  | MergeReq
  | CensorReq
  | DictionaryReq
  | ToAgs4Req
  | ExcelExportReq
  | ExcelImportReq;

export type WorkerRes =
  | { type: "ready" }
  | { type: "initError"; error: string }
  | { id: number; ok: true; kind: "report"; report: ValidationReport }
  | { id: number; ok: true; kind: "cert"; json: string }
  | {
      id: number;
      ok: true;
      kind: "gzip";
      bytes: ArrayBuffer;
      report: ReportMeta;
    }
  | { id: number; ok: true; kind: "fixes"; fixes: Fix[] }
  | { id: number; ok: true; kind: "applied"; bytes: ArrayBuffer }
  | { id: number; ok: true; kind: "parsed"; groups: GroupMeta[] }
  | { id: number; ok: true; kind: "arrow"; code: string; bytes: ArrayBuffer }
  | { id: number; ok: true; kind: "revisionDelta"; delta: RevisionDelta }
  | {
      id: number;
      ok: true;
      kind: "mergeResult";
      /** The merged `.ags` bytes, transferred. */
      bytes: ArrayBuffer;
      warningsJson: string;
      revisionsJson: string;
    }
  | { id: number; ok: true; kind: "dictionary"; dict: StandardDict }
  | { id: number; ok: true; kind: "censor"; text: string; tally: CensorTally }
  | { id: number; ok: true; kind: "toAgs4"; result: ExportResult }
  | {
      id: number;
      ok: true;
      kind: "excel";
      /** The `.xlsx` (export) or `.ags` (import) bytes, transferred. */
      bytes: ArrayBuffer;
      warnings: string[];
      sheets: number;
      rows: number;
    }
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
const reply = (msg: WorkerRes, transfer?: Transferable[]) => {
  if (transfer) ctx.postMessage(msg, transfer);
  else ctx.postMessage(msg);
};

// Instantiate once. Passing the bundled-asset URL explicitly avoids the
// import.meta.url fetch fallback (which breaks under a non-root `base`).
// Every handler awaits this, so requests that arrive before init simply
// queue behind it rather than racing a live-before-ready `validate`.
const ready: Promise<void> = init({ module_or_path: wasmUrl }).then(
  () => undefined,
);

ready.then(
  () => {
    reply({ type: "ready" });
  },
  (e: unknown) => {
    reply({ type: "initError", error: String(e) });
  },
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
      );
      // Transfer the result bytes back in a standalone buffer (slice()
      // gives one sized exactly to the output, safe to transfer).
      const buf = out.slice().buffer;
      reply({ id: req.id, ok: true, kind: "applied", bytes: buf }, [buf]);
      return;
    }

    if (req.kind === "parse") {
      dataset?.free();
      const ds = read(new Uint8Array(req.bytes), req.encoding);
      dataset = ds;
      const groups: GroupMeta[] = ds.group_codes().map((code) => {
        const m = ds.meta(code) as Omit<GroupMeta, "code"> | null;
        return m
          ? { code, ...m }
          : { code, headings: [], units: [], types: [], sql_types: [] };
      });
      reply({ id: req.id, ok: true, kind: "parsed", groups });
      return;
    }

    if (req.kind === "arrowIpc") {
      if (!dataset) throw new Error("no parsed dataset — call parse first");
      // keys (default false): include the content-addressed _id/_parent_id
      // columns. The Explore ingest passes true so duckdb-wasm carries them and
      // cross-group joins resolve; the group grid strips them for display. (#303)
      // contentHash (default false): include the trailing _content_hash value
      // fingerprint, same opt-in shape. (#448)
      const out = dataset.arrow_ipc(
        req.code,
        req.keys ?? false,
        req.contentHash ?? false,
      );
      const buf = out.slice().buffer;
      reply(
        { id: req.id, ok: true, kind: "arrow", code: req.code, bytes: buf },
        [buf],
      );
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

    if (req.kind === "merge") {
      // Throws (→ caught below) on an unsettled TYPE clash, a UNIT clash (fatal
      // in every mode), or unparseable input.
      const out = merge(
        new Uint8Array(req.aBytes),
        new Uint8Array(req.bBytes),
        req.encoding,
        req.onTypeClash,
        req.tranIssue ?? undefined,
        req.tranDate ?? undefined,
        req.tranProducer ?? undefined,
      );
      const buf = out.bytes.slice().buffer;
      reply(
        {
          id: req.id,
          ok: true,
          kind: "mergeResult",
          bytes: buf,
          warningsJson: out.warnings_json,
          revisionsJson: out.revisions_json,
        },
        [buf],
      );
      return;
    }

    if (req.kind === "censor") {
      // The shared scrub engine (#581). Hashes the bytes for PROJ_ID's filehash,
      // decodes lossily, applies the (optionally column-restricted) policy, and
      // returns the anonymised text + per-action tally. Never throws for normal
      // input; a bad sensitiveJson → JsError → outer catch → ok:false.
      const res = censor(
        new Uint8Array(req.bytes),
        req.sensitiveJson,
        req.selectedCodes,
        req.token,
        req.dropCustom,
        req.includeFreetext,
      ) as { text: string; tally: CensorTally };
      reply({
        id: req.id,
        ok: true,
        kind: "censor",
        text: res.text,
        tally: res.tally,
      });
      return;
    }

    if (req.kind === "dictionary") {
      const dict = dictionary(req.edition ?? undefined) as StandardDict;
      reply({ id: req.id, ok: true, kind: "dictionary", dict });
      return;
    }

    if (req.kind === "toAgs4") {
      // Throws (→ caught below) on invalid JSON or a strict-mode rejection.
      const result = build_ags4(
        req.groupsJson,
        req.edition ?? undefined,
        req.mode,
      ) as ExportResult;
      reply({ id: req.id, ok: true, kind: "toAgs4", result });
      return;
    }

    if (req.kind === "certify") {
      // `certify` throws (a JsError) on a dirty/unparseable file; the outer
      // catch turns that into an `ok: false` reply the client rejects.
      const json = certify(
        new Uint8Array(req.bytes),
        req.dict,
        req.encoding,
        req.checkedAt,
        req.dictBytes,
        req.dictReplace ?? false,
      );
      reply({ id: req.id, ok: true, kind: "cert", json });
      return;
    }

    if (req.kind === "excelExport" || req.kind === "excelImport") {
      // Both directions return a wasm `ExcelResult` (bytes + warnings + counts);
      // the conversion fns throw (JsError) on empty/invalid input → outer catch.
      // Read the fields, free the wasm struct, transfer the bytes back.
      const res =
        req.kind === "excelExport"
          ? ags4_to_xlsx(new Uint8Array(req.bytes))
          : xlsx_to_ags4(new Uint8Array(req.bytes), req.formatNumeric);
      const outBytes = res.bytes;
      const warnings = res.warnings;
      const sheets = res.sheets;
      const rows = res.rows;
      res.free();
      const buf = outBytes.slice().buffer;
      reply(
        {
          id: req.id,
          ok: true,
          kind: "excel",
          bytes: buf,
          warnings,
          sheets,
          rows,
        },
        [buf],
      );
      return;
    }

    const report = validate(
      new Uint8Array(req.bytes),
      req.dict,
      // include_warnings: warnings are produced always (Rule 18 etc.) — the
      // severity FilterBar (error+warning on by default) controls display.
      true,
      req.includeFyi,
      req.encoding,
      req.maxPerRule,
      req.dictBytes,
      req.dictReplace ?? false,
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
