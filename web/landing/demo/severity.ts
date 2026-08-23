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

const LINE_ERROR = "border-l-err bg-err-quiet text-err";

/** The banded output-pane LINE's variant of the same grammar — left edge plus
 *  tint, the border width staying the pane's own. The last finding surface to
 *  join this module (#548): the pane banded every line error-red regardless
 *  of tier. Unknown tiers fall to error, loud, like every variant here. */
export function severityLineTint(severity: string): string {
  return LINE[severity] ?? LINE_ERROR;
}

const LINE: Record<string, string> = {
  error: LINE_ERROR,
  warning: "border-l-warn bg-warn-quiet text-warn",
  fyi: "border-l-info bg-info-quiet text-info",
};

const ROW_ERROR = "bg-err-quiet";

/** The condemned ROW's variant of the grammar (#590) — wash only, no text
 *  repaint and no weight: a row-level fault ("this row has no parent") is a
 *  different claim from a cell verdict, so it must not wear the cell's
 *  dress. Unknown tiers fall to error, loud, like every variant here. */
export function severityRowTint(severity: string): string {
  return ROW[severity] ?? ROW_ERROR;
}

const ROW: Record<string, string> = {
  error: ROW_ERROR,
  warning: "bg-warn-quiet",
  fyi: "bg-info-quiet",
};

const ROW_EDGE_ERROR = "[box-shadow:inset_3px_0_0_var(--err)]";

/** The condemned row's left-edge marker, first cell only — an inset shadow
 *  rather than a border, so the marked row's text stays aligned with its
 *  column instead of shifting by the marker's width. Unknown tiers fall to
 *  error, loud, like every variant here. */
export function severityRowEdge(severity: string): string {
  return ROW_EDGE[severity] ?? ROW_EDGE_ERROR;
}

const ROW_EDGE: Record<string, string> = {
  error: ROW_EDGE_ERROR,
  warning: "[box-shadow:inset_3px_0_0_var(--warn)]",
  fyi: "[box-shadow:inset_3px_0_0_var(--info)]",
};

const RANK: Record<string, number> = { error: 3, warning: 2, fyi: 1 };

/** One comparator for both worst-of aggregations below — an unknown tier
 *  outranks everything known, the same loud-by-default rule as the tints. */
const outranks = (a: string, b: string): boolean =>
  (RANK[a] ?? 3) > (RANK[b] ?? 3);

/** Worst severity per FILE LINE — the unit seam #548 names, DOM-free.
 *  Findings that report an absence carry no line and band nothing, correctly:
 *  delete the TRAN row and there is no line where "TRAN group not found"
 *  happened. Unknown tiers rank as error, same as everywhere in this file. */
export function worstPerLine(
  findings: readonly Finding[],
): Map<number, string> {
  const out = new Map<number, string>();
  for (const f of findings) {
    if (f.line === null) continue;
    const cur = out.get(f.line);
    if (cur === undefined || outranks(f.severity, cur))
      out.set(f.line, f.severity);
  }
  return out;
}

/** The severity a CELL wears when several findings land on it — the worst
 *  one, by the engine's own tiers. Null when nothing is wrong. */
export function worstSeverity(findings: readonly Finding[]): string | null {
  let worst: string | null = null;
  for (const f of findings) {
    if (worst === null || outranks(f.severity, worst)) worst = f.severity;
  }
  return worst;
}
