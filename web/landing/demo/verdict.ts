/* The scoreboard's arithmetic (#531), pure and engine-faithful. Named
 * verdict.ts rather than scoreboard.ts because the component beside it is
 * Scoreboard.tsx and this filesystem folds case.
 *
 * The chip states a verdict the panel only implies: how many findings stand,
 * or that none do. Severity comes from the engine (severity.ts's rule holds
 * here too — the UI never decides how bad); this module only counts. FYI
 * findings do not gate validity: the verdict is about errors and warnings,
 * the tiers a delivery gate would fail on.
 */

import type { Finding, Report } from "./engine";

export type Tally = {
  readonly errors: number;
  readonly warnings: number;
};

export function tally(findings: readonly Finding[]): Tally {
  let errors = 0;
  let warnings = 0;
  for (const f of findings) {
    if (f.severity === "error") errors += 1;
    else if (f.severity === "warning") warnings += 1;
  }
  return { errors, warnings };
}

/** The chip's text. Zero of both is a stated verdict, not an empty string. */
export function scoreboardLabel(t: Tally): string {
  if (t.errors === 0 && t.warnings === 0) return "✓ valid AGS4";
  const parts: string[] = [];
  if (t.errors)
    parts.push(`${t.errors} ${t.errors === 1 ? "error" : "errors"}`);
  if (t.warnings)
    parts.push(`${t.warnings} ${t.warnings === 1 ? "warning" : "warnings"}`);
  return parts.join(" · ");
}

export type VerdictState =
  | { readonly kind: "refused"; readonly message: string }
  | { readonly kind: "counted"; readonly tally: Tally };

/** What a run's report SAYS (#638): a refusal carrying the engine's own
 *  message, or the counted findings. An error report rides with an empty
 *  findings list, so tallying it stated the all-clear over a run the engine
 *  refused — the KIND is the claim, the count is only its detail. Error wins
 *  even if findings ever ride along with one: a refused run's list is not
 *  evidence. */
export function verdictState(r: Report): VerdictState {
  if (r.error) return { kind: "refused", message: r.error.message };
  return { kind: "counted", tally: tally(r.findings) };
}

/** The refused chip's text — chip-short; the panel carries the engine's
 *  message, unreworded. */
export const REFUSED_LABEL = "✗ not validatable";
