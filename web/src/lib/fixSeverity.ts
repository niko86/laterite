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

/** `undefined` when there is NO report to join against — still a different fact
 *  from a report that simply doesn't mention a given rule, even though #430 made
 *  both answer "no label known". This one means nobody looked; the other means
 *  the validator looked and didn't raise it, which is the state `fixSeverity`
 *  below argues cannot arise. Answering them from one empty map would merge a
 *  missing validator into a quiet one, which is the whole of #412. */
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

/** The severity of the finding this fix resolves, or `undefined` when no label
 *  is known — the fix is real either way.
 *
 *  Two absences reach that `undefined`, and only one of them can happen. There
 *  may be no report (#412: the labelling `validate` never answered). Or the
 *  report ran and never raised the fix's rule — which #430 settled as
 *  unreachable rather than benign: every fixer in `compute_fixes` is gated on
 *  its numbered rule being a key of the findings it was handed and stamps that
 *  same key as the fix's `rule`, and those findings come from the same door,
 *  bytes, dictionary and encoding as this report, which is FYI-inclusive and
 *  uncapped and carries the warning tier the fix pass leaves off. So the rule
 *  is always here. If a badge ever says "unlabelled" beside a real fix, the two
 *  engine paths have diverged — that deserves a loud look, and the `"warning"`
 *  this used to return would have hidden it behind an ordinary-looking tier. */
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
  return idx.byRule.get(f.rule);
}
