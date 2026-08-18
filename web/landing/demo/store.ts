/* The demo's one piece of state (#397, #398).
 *
 * Live, no submit: the delivery is a signal, the emitted text is a memo of it,
 * and the report is recomputed whenever the text changes. There is no Check
 * button because there is nothing to check — the file and the findings are
 * simply always current.
 *
 * The engine is ARMED rather than loaded: nothing is fetched until the reader
 * first touches the demo, so a visitor who came for an install command never
 * pays for it. `arm()` is idempotent and safe to call from every interactive
 * surface, which is why each of them calls it rather than one of them owning it.
 */

import {
  createMemo,
  createResource,
  createRoot,
  createSignal,
  type Accessor,
} from "solid-js";
import {
  SEEDED,
  addRow as addRowTo,
  emit,
  parse,
  setCell as setCellIn,
  type Delivery,
} from "./delivery";
import { keyHeadings } from "./schema";
import { engine, validateText, type Finding, type Report } from "./engine";

const [delivery, setDelivery] = createSignal<Delivery>(SEEDED);
const [armed, setArmed] = createSignal(false);
const [focusLine, setFocusLine] = createSignal<number | null>(null);

/** The cell the row carousel is open on, or null when it is closed. */
const [picked, setPicked] = createSignal<{
  group: string;
  row: number;
  col: number;
} | null>(null);

/** The AGS4 text the engine sees — and, byte for byte, what the output pane
 *  renders. One source, so a finding's line number always points at the line
 *  the reader is looking at. */
const text = createMemo(() => emit(delivery()));

/* The validation pass, as a resource rather than a hand-rolled async effect.
 *
 * The first version of this was `createEffect(on([armed, text], async …))` with
 * a monotonic counter to drop stale results. It looked right and it was subtly
 * broken: the report SIGNAL held the correct new value while every subscriber
 * stayed one update behind, so editing 11.8 to 11.80 cleared Rule 8 from the
 * engine's output and left the page still showing it — the demo contradicting
 * the engine it exists to demonstrate, which is the single worst thing this
 * page can do.
 *
 * `createResource` is the primitive for exactly this shape. It owns the async
 * lifecycle, discards superseded fetches itself, and exposes `.loading` — so
 * there is no counter to get wrong. The source returns `false` until armed,
 * which is how a resource is told not to fetch yet.
 *
 * `createRoot` because this graph lives at module scope: without an owner Solid
 * warns that the computations can never be disposed, and it is right — this
 * store IS the page's lifetime, so the root is stated rather than left implicit.
 */
const { report, busy } = createRoot(() => {
  const [res] = createResource(
    () => (armed() ? text() : false),
    (current: string) => validateText(current),
  );
  // Named accessors rather than the resource itself, so callers read `report()`
  // and `busy()` without knowing this is a resource — and so the resource's
  // `undefined`-before-first-fetch becomes the `null` the UI already handles.
  const report: Accessor<Report | null> = () => res() ?? null;
  const busy: Accessor<boolean> = () => res.loading;
  return { report, busy };
});

export { armed, busy, delivery, focusLine, picked, report, text };
export { setFocusLine, setPicked };

/** Load the engine. Idempotent; the first caller pays and the rest await.
 *
 * Arming flips the resource's source from `false` to the current text, which is
 * what starts the first validation — the reader's first interaction shows them
 * findings rather than an empty panel. */
export function arm(): void {
  if (armed()) return;
  setArmed(true);
  void engine();
}

export function setCell(
  group: string,
  row: number,
  col: number,
  value: string,
): void {
  arm();
  setDelivery((d) => setCellIn(d, group, row, col, value));
}

export function addRow(group: string, parent: string | null): void {
  arm();
  setDelivery((d) => addRowTo(d, group, parent, keyHeadings(group)));
}

export function reset(): void {
  setDelivery(SEEDED);
}

/** Run the engine's OWN fixer over the current delivery.
 *
 * Deliberately not a bespoke landing-page repair. #398 describes Fix as
 * correcting the decimal places, the abbreviation case and the missing TRAN
 * group; the shipped fixer mechanically repairs only the first, and reports the
 * rest as findings a human has to decide about. Hand-rolling the other two here
 * would make the page repair more than `lat fix` does, so a reader who tried the
 * same file on their own machine would get a different answer from the demo that
 * sold them the tool. The engine is the authority, exactly as it is for severity.
 *
 * What survives is the argument, intact and now true of the real tool: a
 * validator tells you what is wrong, and a human decides what is right.
 */
export async function applyEngineFixes(): Promise<number> {
  arm();
  const m = await engine();
  const bytes = new TextEncoder().encode(text());
  const fixes = m.compute_fixes(bytes);
  if (!fixes.length) return 0;
  const fixed = m.apply_fixes(bytes, null, fixes);
  setDelivery(parse(new TextDecoder().decode(fixed)));
  return fixes.length;
}

/** The findings that land on one cell — used to mark it in the table and to
 *  write the failing rule under its field card. Matched on (group, data row,
 *  heading), which is the identity the engine reports; a line-number match would
 *  break the moment a row is added above. */
export function findingsForCell(
  group: string,
  rowIndex: number,
  heading: string,
): Finding[] {
  return (report()?.findings ?? []).filter(
    (f) =>
      f.group === group &&
      f.heading === heading &&
      f.dataRow !== null &&
      f.dataRow - 1 === rowIndex,
  );
}

/** Findings that name a group but no particular cell — Rule 16's abbreviation
 *  findings, and Rule 10c's orphan. Shown against the table rather than lost. */
export function findingsForGroup(group: string): Finding[] {
  return (report()?.findings ?? []).filter(
    (f) => f.group === group && f.heading === null,
  );
}

/** Group-level findings whose text names this heading.
 *
 * Rule 16 reports "Abbreviation \"b\" under SAMP_TYPE is not defined" against the
 * GROUP, with no heading and no data row — correctly, because it is a statement
 * about the group's use of that abbreviation and not about one row. But #398
 * asks the field card to write the failing rule under the field being edited,
 * and a reader typing in SAMP_TYPE needs to see it.
 *
 * So the carousel shows these ALONGSIDE the cell findings rather than as them.
 * The distinction is kept: the cell is not marked failing (no ✗, no red value),
 * because attaching a group finding to row 1 would be a lie in a file where rows
 * 1 and 3 both carried the bad value. */
export function groupFindingsNaming(group: string, heading: string): Finding[] {
  return findingsForGroup(group).filter((f) => f.desc.includes(heading));
}

export type { Finding, Report };
