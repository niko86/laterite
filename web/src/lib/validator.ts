// Shared types + data for the Rust `laterite-ags4-wasm` validator. This module is
// deliberately wasm-free so it's safe to import from the main thread: ALL
// wasm calls (validate + the Apply-Fixes engine) live in
// `validator.worker.ts`, driven through `validatorClient.ts`.
//
// The report shapes used to be re-declared here by hand, because wasm-bindgen
// typed the returns as `any`. The crate now publishes them in its own `.d.ts`,
// so this re-exports them instead of maintaining a second description of the
// same bytes — a mirror can only ever be right by accident, and this one was
// wrong about `severity` (see `reportIsOnlyFyi`).
//
// `import type` is erased at compile time, so pulling these from the wasm
// package adds NO runtime import and the module stays main-thread-safe.
export type {
  FindingDto,
  RuleGroup,
  ValErr,
  ValidationReport,
} from "../wasm/ags4_wasm";

import type { FindingDto, ValidationReport } from "../wasm/ags4_wasm";

/** The three severities the UI displays.
 *
 *  Deliberately NOT `NonNullable<FindingDto["severity"]>`: the WIRE type has two
 *  members because the engine omits the field entirely for errors. This is the
 *  RESOLVED union — what a finding means once `severityOf` has read it. */
export type Severity = "error" | "warning" | "fyi";

/** Resolve a finding's severity. **Absent means `"error"`.**
 *
 *  Every caller must come through here rather than writing `?? …` at the point
 *  of use. The app used to default to `"warning"` at five separate sites, which
 *  silently reclassified every error in the browser: the summary banner's split
 *  counted errors as warnings, and the severity filter hid them from the "error"
 *  selection while showing them under "warning". One resolver, one place to be
 *  right. */
export function severityOf(f: Pick<FindingDto, "severity">): Severity {
  return f.severity ?? "error";
}

/** True when a report has findings but EVERY one is FYI (informational) — the
 *  signal the SummaryBanner uses to show amber rather than red. */
export function reportIsOnlyFyi(report: ValidationReport): boolean {
  return (
    report.finding_count > 0 &&
    !report.findings.some((g) => g.items.some((it) => severityOf(it) !== "fyi"))
  );
}

export interface SeverityCounts {
  error: number;
  warning: number;
  fyi: number;
}

/** Per-severity counts summed from the SERIALIZED findings. Exact when the
 *  report is uncapped (`shown_count === finding_count`); on a per-rule-capped
 *  report the items are clipped, so the sum undercounts — see
 *  {@link reportSeverity}, which falls back to the true grand total in that
 *  case. */
export function severityCounts(report: ValidationReport): SeverityCounts {
  const c: SeverityCounts = { error: 0, warning: 0, fyi: 0 };
  for (const g of report.findings)
    for (const it of g.items) c[severityOf(it)]++;
  return c;
}

/** The severity breakdown the banner shows, plus whether it is exact. On a
 *  capped report the per-severity split would undercount, so `exact` is false
 *  and the UI shows the true grand `finding_count` instead of the split. */
export function reportSeverity(report: ValidationReport): {
  counts: SeverityCounts;
  exact: boolean;
} {
  return {
    counts: severityCounts(report),
    exact: report.shown_count >= report.finding_count,
  };
}

// ---- Apply-Fixes (a separate engine surface from validate) ----
// Mirrors the Rust `laterite_ags4_validator::fixes` serde shape (snake_case).
export type FixKind =
  | "normalize_crlf"
  | "strip_bom"
  | "strip_embedded_cr"
  | "rename_duplicate_heading"
  | "insert_tran_dlim"
  | "insert_tran_rcon"
  | "reformat_numeric"
  | "canonicalize_datetime"
  | "normalize_typography"
  | "pad_short_row";

/** Safe = bulk-applicable (fix-all-safe); risky = guesses intent (lossy /
 *  surprising), opt-in only. Mirrors the Rust `FixRisk`. */
export type FixRisk = "safe" | "risky";

/** One in-line text edit: replace char range [start, end) on a 1-based
 *  line with `replacement`. `expected` is what the span should currently
 *  hold; the engine skips the edit if it doesn't (stale-span guard). */
export interface SpanEdit {
  line: number;
  start: number;
  end: number;
  replacement: string;
  expected: string;
}
export interface Fix {
  kind: FixKind;
  label: string;
  /** exact rule label ("AGS Format Rule 8", …) for cross-linking. */
  rule: string;
  /** anchor line for ordering/preview; null for whole-file kinds. */
  line: number | null;
  /** safe (bulk) vs risky (opt-in). */
  risk: FixRisk;
  /** empty for the byte-level kinds (normalize_crlf / strip_bom). */
  edits: SpanEdit[];
}

// `computeFixes` / `applyFixes` live in `validatorClient.ts` — they round-
// trip through the worker (the only wasm owner) and so are async there.

// The selectable dictionary version — single-sourced onto the generated editions
// module (#529). It was a hand-typed union kept in lockstep by hand with three
// other web copies + the dictionary; re-exported here so its many importers
// (Controls, ExportPane, validatorClient, settings, …) are unchanged.
export type { DictVersionOpt } from "./editions";
export type EncodingOpt = "utf-8" | "windows-1252";

// --- merge (Tools → Merge): how to settle a heading two deliveries typed
// differently. `error` refuses; `widen` falls back to X (raw values kept, but the
// column's TYPE is thrown away); `promote` keeps the column numeric when every
// clashing code is nDP — greatest precision wins (2DP + 5DP -> 5DP) and the coarser
// values are zero-padded, so no digit changes and Rule 8 still holds. nSF/nSCI and
// cross-family clashes fall back to `widen`. Mirrors the engine's TypeClashMode. ---
export type TypeClashMode = "error" | "widen" | "promote";

// --- AGS4 producer (Export tab): the `laterite-ags4-wasm` build_ags4(groups, edition, mode)
// result — build valid AGS4 from data. `mode`: autofix (default) | report |
// strict. ---
export type EmitMode = "autofix" | "report" | "strict";

// The second hand-written mirror this file used to carry: `build_ags4` returned
// `any`, so its result shape was re-described here and the worker cast to it.
// The crate publishes it now, so these are aliases onto the generated types —
// the local names stay so the Export tab and the worker are unchanged, but
// there is one description of the shape instead of two. The old copies were
// also looser than the engine (`severity?: string`, `kind: string`); the
// published unions are exact.
export type { EmitFinding as ExportFinding } from "../wasm/ags4_wasm";
export type { AppliedFix } from "../wasm/ags4_wasm";
export type { BuildReport as ExportResult } from "../wasm/ags4_wasm";

// --- Standard dictionary (Tools reference): one edition of the AGS4 standard
// dictionary (canonical names, descriptions, units, types, status). Now produced
// by `lib/dict.ts::projectEdition` from the canonical union `ags_dictionary.json`
// (the single web dict source); shape kept identical to the prior wasm
// `dictionary(edition)` result so the Tools UIs render unchanged. ---
export interface DictHeading {
  name: string;
  status: string;
  /** AGS TYPE code (ID, X, 2DP, DT, …). */
  type: string;
  unit?: string;
  description: string;
}
export interface DictGroup {
  code: string;
  /** the group's standard description / "contents". */
  contents: string;
  parent?: string;
  headings: DictHeading[];
}
export interface StandardDict {
  /** the edition this dictionary is for ("4.1.1", …). */
  ags_edition: string;
  groups: DictGroup[];
}

// --- Revision diff (Tools): the `laterite-ags4-wasm` diff(a, b) result. KEY-aware,
// type-aware comparison of two AGS4 files. Mirrors the Rust serde shapes.
export interface CellDelta {
  heading: string;
  /** AGS TYPE code the cells were compared as. */
  type: string;
  /** raw value in the baseline / revision (null if the row is short). */
  a: string | null;
  b: string | null;
}
export interface RowDelta {
  kind: "added" | "removed" | "changed";
  /** the KEY values (or whole-row tuple, when unkeyed) identifying the row. */
  key: string[];
  line_a: number | null;
  line_b: number | null;
  /** changed cells — populated only for kind === "changed". */
  cells: CellDelta[];
}
export interface GroupDelta {
  code: string;
  /** true totals, independent of any rows cap. */
  added: number;
  removed: number;
  changed: number;
  headings_added: string[];
  headings_removed: string[];
  /** false ⇒ matched on whole-row tuple (no dictionary KEY headings). */
  keyed: boolean;
  key_headings: string[];
  rows: RowDelta[];
}
export interface RevisionDelta {
  groups: GroupDelta[];
  groups_added: string[];
  groups_removed: string[];
  total_added: number;
  total_removed: number;
  total_changed: number;
}

/** Per-rule serialization cap for the interactive UI. The engine still
 *  finds every violation; this only bounds how many rows per rule cross
 *  the wasm→JS boundary, so a pathologically dirty file stays in the tens
 *  of thousands of serialized rows, not the full millions. The download
 *  path passes `null` (uncapped). Tunable here without a wasm rebuild. */
export const DEFAULT_MAX_PER_RULE = 10_000;

/** Bundled sample files served from public/samples/ (copied from the
 *  validator's test fixtures). Loaded via fetch under the deploy base. */
export const SAMPLES: { name: string; file: string; blurb: string }[] = [
  {
    name: "Clean (minimal)",
    file: "clean_minimal.ags",
    blurb: "valid — 0 findings",
  },
  {
    name: "Rule 8 — bad DATETIME",
    file: "rule8_dt_bad.ags",
    blurb: "typed-value error",
  },
  {
    name: "Rule 9 — unknown heading",
    file: "rule9_unknown_heading.ags",
    blurb: "heading not in dictionary",
  },
  {
    name: "Rule 10a — duplicate key",
    file: "rule10a_dup_key.ags",
    blurb: "duplicate KEY row",
  },
];
