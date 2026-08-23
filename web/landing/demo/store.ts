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
  batch,
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
  reparseGuarded,
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
  abbreviationCells,
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

/* The pane's raw draft (#635). While the pane editor holds the delivery,
   the engine must judge the DRAFT'S OWN BYTES: the tolerant parse launders
   garbage into an empty delivery, and validating THAT emitted an
   earned-looking all-clear — #638's lie through another door. Non-null only
   between pane commits; any other writer (a structured commit, an undo
   step, the editor closing) makes the emitted text the truth again. */
const [paneRaw, setPaneRaw] = createSignal<string | null>(null);

/** One key, two sites: commit() skips clearing paneRaw for exactly the
 *  commits that are about to re-stamp it. A string in only one place would
 *  fail silently as an extra engine pass per commit, not a visible break. */
const PANE_EDIT = "pane-edit";

/** Whether the pane's draft currently outranks the emitted text — the
 *  component uses this to tell its own commit echoing back from an outside
 *  writer reclaiming the store. */
export const paneDraftActive = (): boolean => paneRaw() !== null;

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
    () => (armed() ? { snapshot: delivery(), raw: paneRaw() } : false),
    // One fetch, all three answers: the findings, the fixes, AND the delivery
    // they were computed against. Keying the fetcher on the delivery (not its
    // text) lets the result carry that snapshot, so a fix's line is only ever
    // mapped to a group through the SAME revision the engine saw — mixing a
    // stale fix list with a fresh delivery could count a fix against the
    // wrong table during the revalidation window (#530). The cost is a
    // second engine pass per revalidation — accepted for coherence.
    async (src: { snapshot: Delivery; raw: string | null }) => {
      // The raw draft outranks the emitted text while it stands (#635):
      // the reader is looking at THOSE bytes, so the verdict must be about
      // them — including the engine's refusal when they are not AGS4.
      const current = src.raw ?? emit(src.snapshot);
      return {
        report: await validateText(current),
        fixes: await computeFixesText(current),
        snapshot: src.snapshot,
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
/** The exported pick setter is a RUN BOUNDARY (#550): coalescing survives
 *  only an unbroken stay in one card, so opening, moving, or closing the
 *  editor ends any open run — a reopened cell records fresh rather than
 *  folding into the old run's base. Value-style only; no caller uses the
 *  updater form. (The store's own internal setPicked calls sit inside commit
 *  flows whose unkeyed commits already break the run.) */
function pickCell(p: { group: string; row: number; col: number } | null): void {
  history = { ...history, key: null };
  setPicked(p);
}

export { setFocusLine, pickCell as setPicked };

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
function commit(
  next: (d: Delivery) => Delivery,
  coalesce: string | null = null,
): boolean {
  const current = delivery();
  const changed = next(current);
  if (changed === current) return false;
  history = record(history, current, 100, coalesce);
  // One batch, deliberately: effects watching text() must see the pane
  // draft's fate and the new delivery TOGETHER — written separately,
  // Solid runs them between the writes, and the pane's follow-the-writer
  // effect read a stale still-standing draft and skipped (#635).
  batch(() => {
    setDelivery(changed);
    dropStalePick(changed);
    // A structured writer reclaims the truth from the pane's draft — the
    // pane commit re-stamps its own raw right after this.
    if (coalesce !== PANE_EDIT) setPaneRaw(null);
  });
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
  opts?: { coalesce?: boolean },
): void {
  arm();
  /* The carousel commits every keystroke (live revalidation), so it asks for
   * coalescing: one cell's consecutive commits share a key and undo as one
   * step (#550). The fine-pointer table already commits once per cell edit
   * and stays unkeyed — an unkeyed commit also BREAKS any open run, as does
   * every other mutation below. */
  const key = opts?.coalesce ? `cell:${group}:${row}:${col}` : null;
  commit((d) => setCellIn(d, group, row, col, value), key);
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
  // Batched for the same reason as commit(): the pane's follow-the-writer
  // effect must see the draft released and the stepped delivery at once.
  batch(() => {
    setDelivery(() => stepped.present);
    dropStalePick(stepped.present);
    setPaneRaw(null);
  });
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

/** Whole-delivery replacement from the pane's draft (#635). The tables get
 *  the tolerant parse's view; the ENGINE gets the raw bytes (paneRaw
 *  above), so garbage is refused rather than laundered. Never throws —
 *  parse is tolerant by design — and consecutive pane commits coalesce
 *  into one undo step, so a typing session unwinds in one Cmd+Z. Stale
 *  cell picks are commit's own concern (dropStalePick). */
export function replaceFromText(raw: string): void {
  const next = parse(raw);
  // Only a successful parse feeds the structured store (the recorded
  // mechanism): a draft the parser finds no groups in — the same
  // emptiness the engine refuses as "no GROUP rows" — leaves the tables
  // on the last good delivery, while the verdict surfaces carry the
  // refusal. Emptying every table under a refused banner would punish
  // the reader twice for one typo.
  // Batched with the raw stamp: written separately, the follow-the-writer
  // effect runs between them, sees a not-yet-stamped draft, and re-seeds
  // the textarea mid-typing wherever emit(parse(draft)) differs from the
  // draft's own bytes.
  batch(() => {
    if (next.length > 0) commit(() => next, PANE_EDIT);
    setPaneRaw(raw);
  });
}

/** The pane editor closing (#635): the emitted text is the truth again,
 *  and the verdict re-keys to it. */
export function releasePaneDraft(): void {
  setPaneRaw(null);
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
 * revision.
 *
 * "refused" is the third outcome (#582), distinct from both a count and a
 * zero: parse is narrow and silently drops what it does not recognise —
 * the mechanism that hid #574 — so a reparse of the engine's fixed text
 * that LOSES data is never committed. The guard itself (reparseGuarded,
 * with losesData's directional rule) lives in the pure model where it is
 * tested; this function only routes its answer — and the routing has no
 * test at any altitude, stated per #582's brief rather than papered over:
 * the unit lane is plain node and cannot import this Solid store, and no
 * e2e can reach a refusal since #574 closed the one route that made parse
 * drop a line. On refusal the delivery is
 * left exactly as it was and no undo step is recorded, because commit is
 * simply not called. The refusal deliberately does not enter the findings
 * list — held by shape, not by assertion: the outcome is a return value
 * the type system keeps out of Finding, and the report resource is keyed
 * on the delivery, which a refusal never moves. The scoreboard tallies
 * the engine's report and the UI never decides how bad — the caller
 * renders the refusal as a note beside the button that was clicked. */
export async function applyGroupFixes(
  group: string,
): Promise<number | "refused"> {
  arm();
  const base = delivery();
  const current = emit(base);
  const all = await computeFixesText(current);
  const mine = fixesForGroup(all, base, group);
  if (!mine.length) return 0;
  const fixed = await applyFixesText(current, mine);
  const next = reparseGuarded(base, fixed);
  if (next === "refused") return "refused";
  commit(() => next);
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
 *  break the moment a row is added above. Since #590 this also resolves Rule
 *  16's prose-addressed findings against the CURRENT delivery, so every cell
 *  carrying the undefined abbreviation is marked — all of them or none, never
 *  an arbitrary first. The resolve rescans the group's rows per call, and the
 *  table calls this per cell: fine at the demo's table sizes, and the
 *  per-cell memos in GroupTable are what keep it off the keystroke path. */
export function findingsForCell(
  group: string,
  rowIndex: number,
  heading: string,
): Finding[] {
  const findings = report()?.findings ?? [];
  const direct = findings.filter(
    (f) =>
      f.group === group &&
      f.heading === heading &&
      f.dataRow !== null &&
      f.dataRow - 1 === rowIndex,
  );
  const data = delivery().find((g) => g.code === group);
  if (!data) return direct;
  const abbr = findings.filter(
    (f) =>
      f.group === group &&
      abbreviationCells(f, data.headings, data.rows).some(
        (c) => c.row === rowIndex && data.headings[c.col] === heading,
      ),
  );
  return [...direct, ...abbr];
}

/** Findings that condemn a whole ROW — heading-less but row-pinned, the
 *  shape Rule 10c's orphan arrives in (#590). Rendered as the row variant of
 *  the severity grammar, distinct from a cell verdict: the engine is saying
 *  "this row has no parent", not "this value is wrong". */
export function findingsForRow(group: string, rowIndex: number): Finding[] {
  return (report()?.findings ?? []).filter(
    (f) =>
      f.group === group &&
      f.heading === null &&
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
 * Since #590 the same findings ALSO map to the cells carrying the value
 * (findingsForCell resolves them against the delivery — all carrying cells,
 * which answers the old caveat that pinning one row would lie). This broader
 * name-match remains for the field card being EDITED: a reader typing into a
 * clean SAMP_TYPE cell still needs to see what the column's rule says, and
 * the carousel dedupes the overlap at its call site. */
export function groupFindingsNaming(group: string, heading: string): Finding[] {
  return findingsForGroup(group).filter((f) => f.desc.includes(heading));
}

export type { Finding, Report };
