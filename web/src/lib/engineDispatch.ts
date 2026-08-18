// The engine-op dispatch, lifted out of `validator.worker.ts` so it can be
// driven by EITHER engine build (#338, see ags-wiki/design/dec-engine-tiering.md).
//
// The worker used to bind its engine by static import, which meant a second
// worker running a DIFFERENT build could not exist without duplicating every
// op. Here the engine is a parameter, so the always-on worker can run
// the tier-1 engine and a lazily-created one can run the full engine for Explore
// and Excel, off the same code.
//
// Deliberately NOT in here: wasm instantiation, the readiness promise, and the
// try/catch that turns a throw into an `{ ok: false }` reply. Those belong to a
// worker entry point — the dispatch throws freely and lets its caller report.

import type {
  DictVersionOpt,
  EmitMode,
  EncodingOpt,
  ValidationReport,
  Fix,
  RevisionDelta,
  ExportResult,
  TypeClashMode,
} from "./validator";
import type { GroupMeta } from "./duckTypes";
// Types come from the FULL build because it is the superset — the only one of
// the two that declares every op a dispatch can serve. Tier 1 is missing
// `ags4_to_xlsx`, `xlsx_to_ags4` and `ParsedDataset.arrow_ipc` by design (#355),
// so deriving from it would leave four ops undescribable.
import type * as FullEngine from "../wasm-full/ags4_wasm_full.js";
import type {
  ExcelResult,
  ParsedDataset,
} from "../wasm-full/ags4_wasm_full.js";

/** The ops EVERY build serves. Structurally the generated wasm module, narrowed
 *  to the functions actually called — so a build that gates one of them out
 *  fails to typecheck where it is PASSED, naming the op, rather than at a call
 *  site deep in a branch. */
export type CoreEngineApi = Pick<
  typeof FullEngine,
  | "validate"
  | "certify"
  | "compute_fixes"
  | "apply_fixes"
  | "diff"
  | "merge"
  | "censor"
  | "build_ags4"
>;

/** The three names this dispatch can only take from the FULL build. Two of them
 *  — the Excel conversions — simply do not exist in tier 1. `read` is subtler and
 *  worth stating precisely: it is UNGATED in the crate, so tier 1 has it, but the
 *  dataset it returns has no `arrow_ipc` door (that method is the `arrow`
 *  feature) and this dispatch calls exactly that. So tier 1's `read` is not the
 *  one meant here, which is why the two builds' `ParsedDataset` types are
 *  incompatible and passing tier 1's module whole fails to compile.
 *
 *  `arrow` + `excel` are the entire weight difference between the tiers —
 *  839 KiB gzipped against 1771. */
export type HeavyEngineApi = Pick<
  typeof FullEngine,
  "read" | "ags4_to_xlsx" | "xlsx_to_ags4"
>;

/** What a dispatch drives. The heavy half is OPTIONAL because the always-on
 *  worker runs tier 1 and passes only the core ops; `validatorClient.ts` never
 *  routes an Explore or Excel request to it. If one ever arrived anyway, the
 *  guards below name the missing op instead of failing as "undefined is not a
 *  function" inside a wasm shim. */
export type EngineApi = CoreEngineApi & Partial<HeavyEngineApi>;

/** How a dispatch hands a response back. The worker supplies one that posts to
 *  the main thread; a test supplies one that collects. */
export type Reply = (msg: WorkerRes, transfer?: Transferable[]) => void;

export interface ValidateReq {
  id: number;
  kind: "validate";
  /** Transferred from the main thread; the worker views it as bytes. */
  bytes: ArrayBuffer;
  dict: DictVersionOpt | null;
  includeFyi: boolean;
  encoding: EncodingOpt;
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
  dict: DictVersionOpt | null;
  encoding: EncodingOpt;
}
/** Mint a `.ags.idx` certificate for a clean file (Validate → download).
 *  Carries the file bytes + the browser's RFC-3339 timestamp (wasm has no
 *  clock, so the caller supplies `checkedAt`). */
export interface CertifyReq {
  id: number;
  kind: "certify";
  bytes: ArrayBuffer;
  dict: DictVersionOpt | null;
  encoding: EncodingOpt;
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
  encoding: EncodingOpt;
  fixes: Fix[];
}
/** Parse a file to a typed dataset, held in the worker. Returns the per-
 *  group schema; the heavy Arrow IPC is pulled lazily per group via
 *  `arrowIpc`, so peak residency stays at one group's batch. */
export interface ParseReq {
  id: number;
  kind: "parse";
  bytes: ArrayBuffer;
  encoding: EncodingOpt;
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
  encoding: EncodingOpt;
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
  encoding: EncodingOpt;
  onTypeClash: TypeClashMode;
  /** All five TRAN members or nothing — the engine refuses a partial stamp. */
  tran: {
    issue: string;
    date: string;
    producer: string;
    recipient: string;
    status: string;
    description?: string;
    remarks?: string;
  } | null;
}
/** Per-action cell/structure counts from the shared scrub engine — the leaf's
 *  `Tally`, snake_case as serialised across the wasm boundary. Re-exported from
 *  the crate rather than re-declared: `censor` returns a typed `CensorResult`
 *  now, so this file's copy was a third description of the same eight counters.
 *  (From the full build, like every other type here — `censor` is in both, so
 *  the two declarations are identical; taking them all from one build is what
 *  stops a reader wondering which.) */
export type { CensorTally } from "../wasm-full/ags4_wasm_full";
import type { CensorTally } from "../wasm-full/ags4_wasm_full";
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
/** Build valid AGS4 from per-group data (Export tab). Carries no file bytes
 *  — `groupsJson` is the `[{code, headings, rows}, …]` shape `build_ags4` wants. */
export interface ToAgs4Req {
  id: number;
  kind: "toAgs4";
  groupsJson: string;
  /** `"auto"`/null → the standard edition; else a concrete one. */
  edition: DictVersionOpt | null;
  mode: EmitMode;
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

async function gzipBytes(json: string): Promise<ArrayBuffer> {
  // Stream-compress so peak memory stays bounded (one chunk at a time)
  // rather than holding source + full compressed copy simultaneously.
  const stream = new Blob([json])
    .stream()
    .pipeThrough(new CompressionStream("gzip"));
  return await new Response(stream).arrayBuffer();
}

/** Build a dispatch bound to one engine.
 *
 *  State — the parsed dataset — lives in this closure rather than at module
 *  scope, which is what lets two workers hold their own without seeing each
 *  other's. Explore's dataset belongs to Explore's worker. */
export function createEngineDispatch(engine: EngineApi, reply: Reply) {
  const {
    validate,
    certify,
    compute_fixes,
    apply_fixes,
    read,
    diff,
    merge,
    censor,
    build_ags4,
    ags4_to_xlsx,
    xlsx_to_ags4,
  } = engine;

  // Reachable only by a routing mistake: `validatorClient.ts` sends all four
  // arrow/excel ops to the second worker, which runs the full build. Worth a
  // sentence anyway — the alternative failure is "undefined is not a function"
  // from inside a wasm shim, which names neither the op nor the reason.
  const absent = (op: string) =>
    new Error(
      `this engine build has no ${op}() — it is tier 1, which drops arrow + excel (#355)`,
    );

  // The most-recently parsed dataset (the Explore/Tools typed-data path).
  // Held because `arrow_ipc(code)` builds each group's Arrow batch lazily AND
  // drops it on return, so the dataset must outlive the parse call to serve
  // per-group pulls. Freed before a new parse so wasm memory stays at one
  // dataset. Callers must drain all `arrowIpc` pulls for a parse before issuing
  // the next parse.
  let dataset: ParsedDataset | null = null;

  return async function dispatch(req: WorkerReq): Promise<void> {
    if (req.kind === "computeFixes") {
      const fixes = compute_fixes(
        new Uint8Array(req.bytes),
        req.dict,
        req.encoding,
      );
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
      if (!read) throw absent("read");
      dataset?.free();
      const ds = read(new Uint8Array(req.bytes), req.encoding);
      dataset = ds;
      const groups: GroupMeta[] = ds.group_codes().map((code) => {
        const m = ds.meta(code);
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
        {
          encoding: req.encoding,
          maxRowsPerGroup: req.maxRowsPerGroup ?? undefined,
        },
      );
      reply({ id: req.id, ok: true, kind: "revisionDelta", delta });
      return;
    }

    if (req.kind === "merge") {
      // Throws (→ caught below) on an unsettled TYPE clash, a UNIT clash (fatal
      // in every mode), or unparseable input.
      const out = merge(
        new Uint8Array(req.aBytes),
        new Uint8Array(req.bBytes),
        {
          encoding: req.encoding,
          onTypeClash: req.onTypeClash,
          tran: req.tran ?? undefined,
        },
      );
      // `MergeResult` is a wasm-owned handle: read every getter, then free it.
      // This branch was the ONLY one that never did — the dataset at :342 and
      // the Excel result at :486 both free theirs — so every merge leaked its
      // handle for the life of the worker. A merge tool is used repeatedly by
      // design (that is what merging deliveries IS), so the leak grew with use.
      // `finally`, because a getter throwing must not turn a leak into the
      // second bug of the session.
      let outBytes: Uint8Array;
      let warningsJson: string;
      let revisionsJson: string;
      try {
        outBytes = out.bytes;
        warningsJson = out.warnings_json;
        revisionsJson = out.revisions_json;
      } finally {
        out.free();
      }
      const buf = outBytes.slice().buffer;
      reply(
        {
          id: req.id,
          ok: true,
          kind: "mergeResult",
          bytes: buf,
          warningsJson,
          revisionsJson,
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
      const res = censor(new Uint8Array(req.bytes), req.sensitiveJson, {
        selectedCodes: req.selectedCodes,
        token: req.token,
        dropCustom: req.dropCustom,
        includeFreetext: req.includeFreetext,
      });
      reply({
        id: req.id,
        ok: true,
        kind: "censor",
        text: res.text,
        tally: res.tally,
      });
      return;
    }

    if (req.kind === "toAgs4") {
      // Throws (→ caught below) on invalid JSON or a strict-mode rejection.
      // `synthesiseMetadata` and `tran` are deliberately NOT wired through
      // yet. The export pane has no UI for either, and both are opt-in for the
      // same reason: synthesis adds whole groups the user never entered, and a
      // TRAN asserts a transmission only they can state. Exporting with
      // synthesis silently on would produce a file carrying groups they did not
      // author. When the pane grows the fields, they belong here.
      //
      // (Under the old positional signature these were slots 4 and 5, unpassed
      // and therefore invisible — the options object at least makes their
      // absence something you can see.)
      // No cast: `build_ags4` is typed `BuildReport` by the crate now, and
      // `ExportResult` is an alias onto it. The old `as ExportResult` was
      // asserting a shape nothing checked, over a return typed `any`.
      const result = build_ags4(req.groupsJson, {
        dictVersion: req.edition ?? undefined,
        mode: req.mode,
      });
      reply({ id: req.id, ok: true, kind: "toAgs4", result });
      return;
    }

    if (req.kind === "certify") {
      // `certify` throws (a JsError) on a dirty/unparseable file; the outer
      // catch turns that into an `ok: false` reply the client rejects.
      const json = certify(new Uint8Array(req.bytes), {
        dictVersion: req.dict ?? undefined,
        encoding: req.encoding,
        checkedAt: req.checkedAt,
        dictionary: req.dictBytes ?? undefined,
        dictReplace: req.dictReplace ?? false,
      });
      reply({ id: req.id, ok: true, kind: "cert", json });
      return;
    }

    if (req.kind === "excelExport" || req.kind === "excelImport") {
      // Both directions return a wasm `ExcelResult` (bytes + warnings + counts);
      // the conversion fns throw (JsError) on empty/invalid input → outer catch.
      // Read the fields, free the wasm struct, transfer the bytes back.
      //
      // An if/else rather than the ternary this was, so each direction is
      // guarded by the door it actually needs: a shared guard reported the
      // EXPORT function's name for a failed import, which sends a reader looking
      // at the wrong half of the boundary.
      let res: ExcelResult;
      if (req.kind === "excelExport") {
        if (!ags4_to_xlsx) throw absent("ags4_to_xlsx");
        res = ags4_to_xlsx(new Uint8Array(req.bytes));
      } else {
        if (!xlsx_to_ags4) throw absent("xlsx_to_ags4");
        res = xlsx_to_ags4(new Uint8Array(req.bytes), req.formatNumeric);
      }
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

    const report = validate(new Uint8Array(req.bytes), {
      dictVersion: req.dict ?? undefined,
      // Warnings are produced always (Rule 18 etc.) — the severity FilterBar
      // (error+warning on by default) controls display. This is also the
      // engine default now, but stated rather than assumed: the display
      // decision is this app's, not the engine's to change under it.
      warnings: true,
      fyi: req.includeFyi,
      encoding: req.encoding,
      maxPerRule: req.maxPerRule ?? undefined,
      dictionary: req.dictBytes ?? undefined,
      dictReplace: req.dictReplace ?? false,
    });

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
  };
}
