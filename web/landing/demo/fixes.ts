/* Per-table autofix (#530), the pure half.
 *
 * The engine's fix records carry the rule they resolve and the line they
 * anchor to — but not a group. The demo does not need the engine to add one:
 * the page emits the file itself, so it knows exactly which lines are whose
 * (groupOfLine), and scoping stays derived rather than duplicated. The demo
 * never repairs more than the engine offers — these functions only FILTER the
 * engine's own list, never invent an edit.
 */

import { groupOfLine, type Delivery } from "./delivery";

/** What this module reads of the engine's fix record — the (rule, line)
 *  identity the wasm `Fix` carries for cross-linking back to findings. */
export type FixLike = {
  readonly rule: string;
  readonly line: number | null;
};

/** The fixes whose anchor line falls inside one group's block. A whole-file
 *  fix (null line) belongs to no table and therefore to no button. */
export function fixesForGroup<F extends FixLike>(
  fixes: readonly F[],
  delivery: Delivery,
  code: string,
): F[] {
  return fixes.filter(
    (f) => f.line !== null && groupOfLine(delivery, f.line) === code,
  );
}

/** A finding is MANUAL when no computed fix shares its (rule, line) identity
 *  — the fixer will not touch it, and the page badges it so the reader knows
 *  which findings are theirs to resolve. Latent asymmetry, accepted: a
 *  whole-file fix (null line) belongs to no button, yet its finding would
 *  read as fixable here — the demo's emitted text never produces one, and if
 *  the engine ever does, the unspendable budget will show up in e2e. */
export function isManual(
  finding: { readonly rule: string; readonly line: number | null },
  fixes: readonly FixLike[],
): boolean {
  return !fixes.some((f) => f.rule === finding.rule && f.line === finding.line);
}
