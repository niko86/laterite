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

import type { Finding } from "./engine";

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
