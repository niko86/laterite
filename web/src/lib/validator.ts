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
 *  signal the SummaryBanner uses to show the info tier rather than the error
 *  one. */
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
// `compute_fixes` now returns `Fix[]` rather than `any`, so these are the
// engine's own published shapes. `FixKind`/`FixRisk` were local unions here;
// they are the members of `Fix`'s fields upstream (there is no named type to
// import), so they stay named here — derived from `Fix` rather than retyped, so
// a new variant in the Rust enum cannot leave this list behind.
export type { Fix, SpanEdit } from "../wasm/ags4_wasm";
import type { Fix } from "../wasm/ags4_wasm";

export type FixKind = Fix["kind"];

/** Safe = bulk-applicable (fix-all-safe); risky = guesses intent (lossy /
 *  surprising), opt-in only. */
export type FixRisk = Fix["risk"];

// `computeFixes` / `applyFixes` live in `validatorClient.ts` — they round-
// trip through the worker (the only wasm owner) and so are async there.

// The selectable dictionary version — single-sourced onto the generated editions
// module (laterite-dev#529). It was a hand-typed union kept in lockstep by hand with three
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
// dictionary (canonical names, descriptions, units, types, status). The VALUE is
// produced locally by `lib/dict.ts::projectEdition` from the canonical union
// `ags_dictionary.json` (the single web dict source), but the TYPE is the
// engine's, so the local projection has to keep conforming to what the crate's
// `dictionary()` export returns — which is the contract that comment used to
// assert by hand. Nothing here CALLS that export: #349 removed the web-side op,
// settling the static JSON as the design. The type is the whole of the tie. ---
export type { DictGroup, DictHeading, StandardDict } from "../wasm/ags4_wasm";

// --- Revision diff (Tools): the `laterite-ags4-wasm` diff(a, b) result. KEY-aware,
// type-aware comparison of two AGS4 files. ---
export type {
  CellDelta,
  GroupDelta,
  RevisionDelta,
  RowDelta,
} from "../wasm/ags4_wasm";

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
