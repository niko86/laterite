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

import { batch, createEffect, createMemo, createSignal, on } from "solid-js";
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
const [report, setReport] = createSignal<Report | null>(null);
const [armed, setArmed] = createSignal(false);
const [busy, setBusy] = createSignal(false);
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

export { armed, busy, delivery, focusLine, picked, report, text };
export { setFocusLine, setPicked };

/** Load the engine. Idempotent; the first caller pays and the rest await. */
export function arm(): void {
  if (armed()) return;
  setArmed(true);
  void engine();
}

// Revalidate on every change once armed. `on(..., { defer: false })` so arming
// validates the seeded delivery immediately rather than waiting for an edit —
// the reader's first interaction should show them findings, not an empty panel.
let run = 0;
createEffect(
  on([armed, text], async ([isArmed, current]) => {
    if (!isArmed) return;
    const mine = ++run;
    setBusy(true);
    const next = await validateText(current);
    // A keystroke landed while this was in flight; its result is the current one.
    if (mine !== run) return;
    batch(() => {
      setReport(next);
      setBusy(false);
    });
  }),
);

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

export type { Finding, Report };
