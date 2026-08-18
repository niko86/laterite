import type { Fix, Severity, ValidationReport } from "./validator";
import { severityOf } from "./validator";

// Severity is a FINDING property, not a FIX one — by design the Rust `Fix`
// model omits it so the parity oracle's byte-identical JSON can't regress (see
// wiki design/validator-finding-ux). So to tell whether a fix touches an
// FYI-classified finding (the surprise: Validate hides FYI by default, yet a
// fix for it — e.g. a Rule 1 BOM/extended-char — still showed up and got
// applied there), we map each fix to the severity of the finding it resolves,
// joining on rule + line against a fresh validation report. Most-severe wins on
// a tie, so a fix is treated as FYI only when its finding is unambiguously FYI.
//
// Lifted out of FixPane so the join is reachable without a DOM (#412): the only
// way to observe the no-report case is to construct it directly, and `web/` has
// no component-test stack — the focusTrap/toastQueue/tooltipDelay shape.

const RANK: Record<Severity, number> = { error: 0, warning: 1, fyi: 2 };
const moreSevere = (a: Severity, b: Severity): Severity =>
  RANK[a] <= RANK[b] ? a : b;

export interface SevIndex {
  byRuleLine: Map<string, Severity>;
  byRule: Map<string, Severity>;
}

/** `undefined` when there is NO report to join against — distinct from a report
 *  that simply doesn't mention a given rule. Keeping the two apart is the whole
 *  of #412: an empty index answers every lookup with the `"warning"` default
 *  below, which is a defensible guess about a rule the validator didn't flag
 *  and a fabrication about a validator that never ran. */
export function buildSevIndex(
  report: ValidationReport | undefined,
): SevIndex | undefined {
  if (!report) return undefined;
  const byRuleLine = new Map<string, Severity>();
  const byRule = new Map<string, Severity>();
  for (const g of report.findings)
    for (const it of g.items) {
      const s = severityOf(it);
      if (it.line != null) {
        const k = `${g.rule}|${it.line}`;
        const prev = byRuleLine.get(k);
        byRuleLine.set(k, prev ? moreSevere(prev, s) : s);
      }
      const pr = byRule.get(g.rule);
      byRule.set(g.rule, pr ? moreSevere(pr, s) : s);
    }
  return { byRuleLine, byRule };
}

/** The severity of the finding this fix resolves, or `undefined` when there is
 *  no report to resolve it against — the fix is real, its label is not known.
 *
 *  The `?? "warning"` below is NOT that case: it answers for a rule the report
 *  ran and didn't raise. It survives #412 deliberately, but it is the same
 *  shape `severityOf` warns about, so if it ever turns out that a fix whose
 *  rule the validator never flagged is an ENGINE disagreement rather than a
 *  benign gap, this is the one line to change — the two absences no longer
 *  share a return value. */
export function fixSeverity(
  idx: SevIndex | undefined,
  f: Fix,
): Severity | undefined {
  if (!idx) return undefined;
  const lines = f.line != null ? [f.line] : f.edits.map((e) => e.line);
  for (const ln of lines) {
    const s = idx.byRuleLine.get(`${f.rule}|${ln}`);
    if (s) return s;
  }
  return idx.byRule.get(f.rule) ?? "warning";
}
