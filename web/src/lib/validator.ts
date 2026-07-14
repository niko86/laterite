// Shared types + data for the Rust `laterite-ags4-wasm` validator. This module is
// deliberately wasm-free so it's safe to import from the main thread: ALL
// wasm calls (validate + the Apply-Fixes engine) live in
// `validator.worker.ts`, driven through `validatorClient.ts`. The wasm
// `.d.ts` types the returns as `any`, so the json-compatible shapes
// (`ValidationReport`, `Fix`) are mirrored here in one place.

export interface FindingDto {
  line: number | null;
  group: string;
  desc: string;
  // Rule-aware location + severity (additive; serde emits these
  // snake_case keys only when set, so older line-only findings omit
  // them). `field_index` is the tag-stripped column index — the raw
  // on-line field is `field_index + 1` (field 0 is the HEADING tag).
  target?: "line" | "heading" | "cell" | "group";
  field_index?: number;
  heading?: string;
  data_row?: number;
  char_span?: [number, number];
  severity?: "error" | "warning" | "fyi";
}
export interface RuleGroup {
  rule: string;
  /** True per-rule count, before any cap; `items.length` may be smaller. */
  total: number;
  items: FindingDto[];
}
export interface ValErr {
  /** stable machine token: not_ags4 | unsupported_edition | bad_args | … */
  kind: string;
  message: string;
}
export interface ValidationReport {
  ok: boolean;
  /** bundled edition judged against ("4.1.1", …); "" on error */
  dict_version: string;
  /** how it was chosen: forced | exact | guessed | fallback; "" on error */
  resolution: string;
  /** true total across all rules, independent of any cap */
  finding_count: number;
  /** how many findings were actually serialized (≤ finding_count when capped) */
  shown_count: number;
  findings: RuleGroup[];
  error: ValErr | null;
}

/** True when a report has findings but EVERY one is FYI (informational) — the
 *  signal the SummaryBanner uses to show amber rather than red. `severity`
 *  defaults to "warning" when absent, so an un-tagged finding still counts as
 *  non-FYI (one real error among many FYI ⇒ false ⇒ red). */
export function reportIsOnlyFyi(report: ValidationReport): boolean {
  return (
    report.finding_count > 0 &&
    !report.findings.some((g) =>
      g.items.some((it) => (it.severity ?? "warning") !== "fyi"),
    )
  );
}

export interface SeverityCounts {
  error: number;
  warning: number;
  fyi: number;
}

/** Per-severity counts summed from the SERIALIZED findings (severity defaults
 *  to "warning" when a finding predates the field). Exact when the report is
 *  uncapped (`shown_count === finding_count`); on a per-rule-capped report the
 *  items are clipped, so the sum undercounts — see {@link reportSeverity},
 *  which falls back to the true grand total in that case. */
export function severityCounts(report: ValidationReport): SeverityCounts {
  const c: SeverityCounts = { error: 0, warning: 0, fyi: 0 };
  for (const g of report.findings)
    for (const it of g.items) c[it.severity ?? "warning"]++;
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

export type DictVersionOpt =
  | "auto"
  | "4.0.3"
  | "4.0.4"
  | "4.1"
  | "4.1.1"
  | "4.2";
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

/** One finding on the *emitted* output (post-fix in AutoFix). `severity`
 *  omitted ⇒ error, matching the engine. */
export interface ExportFinding {
  rule: string;
  line?: number | null;
  group: string;
  desc: string;
  severity?: string;
}

export interface ExportResult {
  /** The AGS4 document text (UTF-8, CRLF) — wrap in a Blob to download. */
  text: string;
  findings: ExportFinding[];
  /** Count of safe mechanical fixes AutoFix applied (0 for report/strict). */
  fixes_applied: number;
}

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
  { name: "Clean (minimal)", file: "clean_minimal.ags", blurb: "valid — 0 findings" },
  { name: "Rule 8 — bad DATETIME", file: "rule8_dt_bad.ags", blurb: "typed-value error" },
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
