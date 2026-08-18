// The engine reply protocol: which successful reply settles which pending
// request, and the result shapes those settlements construct. Pure AGS4
// knowledge, no I/O — and a module of its own since #380 for the same reason
// `workerChannel` became one in #357: `validatorClient` spawns its always-on
// worker at module load, so nothing in it is importable by a unit test
// (`ReferenceError: Worker is not defined`), and the kind-pairs below plus the
// mismatch branch — the one that reports a protocol bug — had no test surface
// at all.
//
// `validatorClient` re-exports the result interfaces, so panes keep importing
// their whole engine surface from one module.

import type {
  ValidationReport,
  ExportResult,
  Fix,
  RevisionDelta,
} from "./validator";
import type { ReportMeta, CensorTally } from "./validator.worker";
import type { GroupMeta } from "./duckTypes";
import type { WorkerRes } from "./engineDispatch";
import type { OkReply } from "./workerChannel";

/** The engine protocol's successful replies — what `settle` maps to pending
 *  request kinds. The channel is protocol-generic (#379); this pins its reply
 *  side to the engine workers' wire type. */
export type WorkerReply = OkReply<WorkerRes>;

export interface GzipResult {
  bytes: ArrayBuffer;
  meta: ReportMeta;
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

/** One pending request: its kind, and how to settle the promise waiting on it. */
export type Pending =
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
      kind: "toAgs4";
      resolve: (r: ExportResult) => void;
      reject: (e: Error) => void;
    }
  | {
      kind: "excel";
      resolve: (r: ExcelConversion) => void;
      reject: (e: Error) => void;
    };

/** Hand a successful reply to the request waiting on it. Both workers speak
 *  this one protocol, so both channels settle through here — the kind pair is
 *  the whole of it, and a mismatch is a protocol bug, not a user-visible one. */
export function settle(msg: WorkerReply, p: Pending): void {
  if (msg.kind === "report" && p.kind === "report") {
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
}
