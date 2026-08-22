/* The severity grammar (#526): one mapping from what the ENGINE says to how
 * it looks, shared by every place a finding renders — cells, table strips,
 * carousel cards, the panel. Severity is never decided in the UI (the
 * recorded principle from #397); this module only translates it.
 */

import type { Finding } from "./engine";

/** Callout container classes per severity — plus "note", the neutral tone for
 *  explanatory boxes that carry no verdict (the fix note). */
const ERROR_TINT = "border-err/40 bg-err-quiet text-err";

const TINT: Record<string, string> = {
  error: ERROR_TINT,
  warning: "border-warn/40 bg-warn-quiet text-warn",
  fyi: "border-info/40 bg-info-quiet text-info",
  note: "border-line bg-surface text-fg-soft dark:bg-surface-raised",
};

/** An unknown severity renders as an ERROR, deliberately: a new engine tier
 *  must fail loud and red, not as plain text nobody notices. */
export function severityTint(severity: string): string {
  return TINT[severity] ?? ERROR_TINT;
}

/** The scoreboard's verdict pair (#531): the same grammar one step outside
 *  severity — "clean" is not an engine tier, it is the absence of any.
 *  Failing reuses ERROR_TINT so the chip and the callouts cannot drift. */
export function verdictTint(clean: boolean): string {
  return clean ? "border-ok/40 bg-ok-quiet text-ok" : ERROR_TINT;
}

const CELL_ERROR = "bg-err-quiet text-err";

/** The failing CELL's variant of the same grammar — tint + text, no border
 *  (the table draws its own). Here rather than inline in GroupTable so a new
 *  engine tier is ONE edit, and so the unknown-tier fallback cannot diverge:
 *  an inline switch would let an unknown severity win the cell via
 *  worstSeverity and then match no arm — untinted, the quiet failure this
 *  module exists to forbid. */
export function severityCellTint(severity: string): string {
  return CELL[severity] ?? CELL_ERROR;
}

const CELL: Record<string, string> = {
  error: CELL_ERROR,
  warning: "bg-warn-quiet text-warn",
  fyi: "bg-info-quiet text-info",
};

const RANK: Record<string, number> = { error: 3, warning: 2, fyi: 1 };

/** The severity a CELL wears when several findings land on it — the worst
 *  one, by the engine's own tiers. Null when nothing is wrong. */
export function worstSeverity(findings: readonly Finding[]): string | null {
  let worst: string | null = null;
  for (const f of findings) {
    if (worst === null || (RANK[f.severity] ?? 3) > (RANK[worst] ?? 3))
      worst = f.severity;
  }
  return worst;
}
