/* The demo's one piece of state (#397, #398).
 *
 * Live, no submit: the delivery is a signal, the emitted text is a memo of it,
 * and the report is recomputed whenever the text changes. There is no Check
 * button because there is nothing to check — the file and the findings are
 * simply always current.
 *
 * The engine loads EAGER-IDLE (#531, reversing the touch-gated decision):
 * the fetch starts after first paint, in an idle callback, because the demo
 * is the page's thesis and a reader who scrolls to it should find the
 * findings already there — and because the wasm travels brotli-compressed on
 * the real delivery path, the cost the old gate guarded against was smaller
 * than the blank pane it caused. First render is never blocked: nothing
 * engine-shaped happens before the page has painted. `arm()` stays idempotent
 * and every interactive surface still calls it, so a reader who beats the
 * idle callback to the demo starts the load that instant.
 */

import {
  createMemo,
  createResource,
  createRoot,
  createSignal,
  onCleanup,
  type Accessor,
} from "solid-js";
import {
  SEEDED,
  addRow as addRowTo,
  deleteGroup as deleteGroupFrom,
  deleteRow as deleteRowFrom,
  emit,
  parse,
  restoreGroup as restoreGroupTo,
  setCell as setCellIn,
  type Delivery,
} from "./delivery";
import {
  EMPTY,
  record,
  redo as redoIn,
  undo as undoIn,
  type History,
} from "./history";
import { keyHeadings } from "./schema";
import { fixesForGroup, isManual } from "./fixes";
import {
  applyFixesText,
  computeFixesText,
  engine,
  validateText,
  type Finding,
  type Fix,
  type Report,
} from "./engine";

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
const { report, busy, counted } = createRoot(() => {
  const [res] = createResource(
    () => (armed() ? delivery() : false),
    // One fetch, all three answers: the findings, the fixes, AND the delivery
    // they were computed against. Keying the fetcher on the delivery (not its
    // text) lets the result carry that snapshot, so a fix's line is only ever
    // mapped to a group through the SAME revision the engine saw — mixing a
    // stale fix list with a fresh delivery could count a fix against the
    // wrong table during the revalidation window (#530). The cost is a
    // second engine pass per revalidation — accepted for coherence.
    async (snapshot: Delivery) => {
      const current = emit(snapshot);
      return {
        report: await validateText(current),
        fixes: await computeFixesText(current),
        snapshot,
      };
    },
  );
  // Named accessors rather than the resource itself, so callers read `report()`
  // and `busy()` without knowing this is a resource — and so the resource's
  // `undefined`-before-first-fetch becomes the `null` the UI already handles.
  const report: Accessor<Report | null> = () => res()?.report ?? null;
  const busy: Accessor<boolean> = () => res.loading;
  const counted: Accessor<
    {
      readonly fixes: readonly Fix[];
      readonly snapshot: Delivery;
    } | null
  > = () => res() ?? null;
  return { report, busy, counted };
});

export { armed, busy, delivery, focusLine, picked, report, text };
export { setFocusLine, setPicked };

/** Start the engine once the page has painted and the thread is idle (#531).
 *  Called from the page root; any interaction that calls arm() first wins. */
export function armWhenIdle(): void {
  if (armed()) return;
  // Feature-detected: older Safari lacks requestIdleCallback, and a missing
  // idle hook must degrade to "shortly", not "never".
  if ("requestIdleCallback" in window) {
    requestIdleCallback(
      () => {
        arm();
      },
      { timeout: 2000 },
    );
  } else {
    setTimeout(() => {
      arm();
    }, 200);
  }
}

/** Load the engine. Idempotent; the first caller pays and the rest await.
 *
 * Arming flips the resource's source from `false` to the current text, which
 * is what starts the first validation. Since #531 the usual first caller is
 * the idle callback above; an interactive surface that beats it there wins,
 * which is why every one of them still calls this. */
export function arm(): void {
  if (armed()) return;
  setArmed(true);
  void engine();
}

/* The undo stack (#525). Not a signal: nothing renders undo state — the only
   readers are the keyboard shortcuts — so a plain module variable avoids
   subscribing anything to a value that changes on every edit. */
let history: History<Delivery> = EMPTY;

/** Every model mutation funnels through here so undo covers all of them —
 *  cell edits, row adds and deletes, the engine's fixes, the demo reset, and
 *  whatever joins them later. Identity short-circuit: a no-op mutation (an
 *  unknown group, an out-of-range row) must not burn an undo step. */
function commit(next: (d: Delivery) => Delivery): boolean {
  const current = delivery();
  const changed = next(current);
  if (changed === current) return false;
  history = record(history, current);
  setDelivery(changed);
  dropStalePick(changed);
  return true;
}

/** A pick can outlive its row — a row delete, an undo of an add. Close rather
 *  than clamp: the row under the pick is GONE, and silently re-aiming the
 *  editor at a neighbouring row would edit data the reader never chose. */
function dropStalePick(d: Delivery): void {
  const p = picked();
  if (!p) return;
  const rows = d.find((g) => g.code === p.group)?.rows.length ?? 0;
  if (p.row >= rows) setPicked(null);
}

export function setCell(
  group: string,
  row: number,
  col: number,
  value: string,
): void {
  arm();
  commit((d) => setCellIn(d, group, row, col, value));
}

export function addRow(group: string, parent: string | null): void {
  arm();
  commit((d) => addRowTo(d, group, parent, keyHeadings(group)));
}

export function deleteGroup(group: string): void {
  arm();
  commit((d) => deleteGroupFrom(d, group));
  /* No pick handling of its own: dropStalePick reads an absent group as zero
     rows, so a pick inside the deleted group already closes in commit(). */
}

export function restoreGroup(group: string): void {
  arm();
  commit((d) => restoreGroupTo(d, group));
}

export function deleteRow(group: string, row: number): void {
  arm();
  const p = picked();
  const changed = commit((d) => deleteRowFrom(d, group, row));
  /* A pick names a POSITION. Deleting at or above it makes that position mean
     a different row's data, and dropStalePick only catches the out-of-range
     case — so close it here. Below the deletion the rows above are untouched
     and the pick still means what the reader chose. */
  if (changed && p && p.group === group && p.row >= row) setPicked(null);
}

function step(walk: typeof undoIn): void {
  const stepped = walk(history, delivery());
  if (!stepped) return;
  history = stepped.history;
  setDelivery(() => stepped.present);
  dropStalePick(stepped.present);
}

export function undo(): void {
  step(undoIn);
}

export function redo(): void {
  step(redoIn);
}

/** Ctrl/Cmd+Z and +Shift+Z, bound at the window so both editors are covered
 *  (#525). An open text input keeps its NATIVE undo — the model shortcut
 *  would yank the delivery out from under a half-typed value. Call from a
 *  component's setup; the unbind registers on that owner's onCleanup. */
export function bindUndoShortcuts(): void {
  const target = window;
  const onKey = (e: KeyboardEvent) => {
    if (!(e.metaKey || e.ctrlKey) || e.key.toLowerCase() !== "z") return;
    const el = e.target;
    if (el instanceof HTMLElement && el.closest("input, textarea")) return;
    e.preventDefault();
    if (e.shiftKey) redo();
    else undo();
  };
  target.addEventListener("keydown", onKey);
  onCleanup(() => {
    target.removeEventListener("keydown", onKey);
  });
}

export function reset(): void {
  commit(() => SEEDED);
}

/** Apply one group's share of the engine's OWN fixes (#530).
 *
 * Deliberately not a bespoke landing-page repair: hand-rolling more than the
 * shipped fixer does would make the page repair more than `lat fix`, and a
 * reader who tried the same file on their own machine would get a different
 * answer from the demo that sold them the tool. The engine is the authority,
 * exactly as it is for severity — this function only FILTERS its list.
 *
 * Recomputed fresh against the current text rather than read from the
 * resource, so a click can never apply a list computed for an older
 * revision. */
export async function applyGroupFixes(group: string): Promise<number> {
  arm();
  const current = text();
  const all = await computeFixesText(current);
  const mine = fixesForGroup(all, delivery(), group);
  if (!mine.length) return 0;
  const fixed = await applyFixesText(current, mine);
  commit(() => parse(fixed));
  return mine.length;
}

/** One table's fix budget (#530): how many of the engine's fixes anchor in
 *  its block — or null before the first count exists, because "zero" and
 *  "not yet counted" are different claims (the scoreboard's own rule). Lines
 *  are mapped through the snapshot the fixes were computed against, never a
 *  fresher delivery. */
export function groupFixCount(group: string): number | null {
  const c = counted();
  if (!c) return null;
  return fixesForGroup(c.fixes, c.snapshot, group).length;
}

/** True when the fixer will not touch this finding — the "manual" badge on
 *  strips and tooltips (#530). */
export function isManualFinding(finding: Finding): boolean {
  return isManual(finding, counted()?.fixes ?? []);
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
