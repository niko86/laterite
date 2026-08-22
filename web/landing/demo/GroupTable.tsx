/* One group's table (#396, editable in #398).
 *
 * THE REPETITION IS THE RELATIONSHIP. AGS4 has no ids and no joins: a child
 * group restates its parent's KEY headings verbatim, which is why LLPL is nine
 * columns wide and why SAMP_ID appears in three of the four descent tables.
 * The design lets that land rather than hiding it — it is the most surprising
 * thing about the format to a newcomer, and the page's best four seconds.
 *
 * ## The table stays a table
 *
 * It scrolls sideways with the first KEY column pinned, and type is NEVER
 * shrunk. A KEY chain at 12px is unreadable, and shrinking it would defeat the
 * whole point of showing it. The scroller lives on the table's own wrapper, so
 * no ancestor ever scrolls horizontally — nine columns on a 390px phone must
 * move the table, not the page.
 *
 * ## Band colour means group identity, never severity
 *
 * The band appears in exactly three places for a group: its chip, its table cap,
 * and its KEY-column region. Nowhere else on the page uses band colour, so it
 * can never be read as "how bad is this". Severity is carried by the error tint
 * and the ✗ marker, which are the same on every group.
 *
 * The seven KEY columns of LLPL are tinted as ONE continuous region rather than
 * seven striped columns — same tint, no per-column edges, and a single solid
 * band rule at the boundary where the key ends and the data begins.
 *
 * ## Two editors, split by MODALITY (#525)
 *
 * On a fine pointer this table edits like a spreadsheet: a click selects, a
 * second click or Enter opens the value in place, arrows move the selection,
 * typing replaces. On a coarse pointer the cell buttons only PICK, and the
 * row carousel below the table does the editing — nine columns at 390px need
 * paged field cards, not a caret in a 40px cell. The split reads
 * `pointer: coarse` (see pointer.ts), never the viewport width.
 */

import {
  For,
  Show,
  createMemo,
  createSignal,
  onMount,
  type Component,
} from "solid-js";
import { Button, Tooltip } from "@shared/components";
import type { DemoGroup } from "./schema";
import type { Group } from "./delivery";
import { coarsePointer } from "./pointer";
import { severityCellTint, worstSeverity } from "./severity";
import { findingsForCell, isManualFinding } from "./store";

/** The in-place value input (#525), its own component for two reasons: mount
 *  focus/select read PROPS rather than signals inside a lifecycle callback,
 *  and commit-vs-cancel settles locally — whichever key closes it marks it
 *  done, so the blur a close causes cannot commit a second time. Uncontrolled
 *  on purpose: the DOM holds the draft and the model hears about it at
 *  COMMIT — a per-keystroke setCell would revalidate half-typed values the
 *  reader never asserted. (The carousel commits live by design: its card IS
 *  the editor.) */
const CellEditor: Component<{
  label: string;
  initial: string;
  /** Enter / second click select the value to replace; a seeded keystroke
   *  has already replaced it and types on. */
  selectAll: boolean;
  onCommit: (value: string, thenMove?: number) => void;
  onCancel: () => void;
}> = (props) => {
  let el!: HTMLInputElement;
  let done = false;
  onMount(() => {
    el.focus();
    if (props.selectAll) el.select();
  });
  return (
    <input
      ref={el}
      aria-label={props.label}
      class="w-full rounded-xs bg-surface px-1 font-mono text-caption text-fg outline-hidden [box-shadow:var(--focus-ring)]"
      value={props.initial}
      onKeyDown={(e) => {
        if (e.key === "Enter") {
          e.preventDefault();
          done = true;
          props.onCommit(e.currentTarget.value);
        } else if (e.key === "Escape") {
          e.preventDefault();
          done = true;
          props.onCancel();
        } else if (e.key === "Tab") {
          e.preventDefault();
          done = true;
          props.onCommit(e.currentTarget.value, e.shiftKey ? -1 : 1);
        }
      }}
      onBlur={(e) => {
        // A click elsewhere commits, like a spreadsheet.
        if (!done) props.onCommit(e.currentTarget.value);
      }}
    />
  );
};

export const GroupTable: Component<{
  schema: DemoGroup;
  data: Group;
  /** The CSS variable naming this group's band, e.g. `--laterite-500`. */
  band: string;
  onPick: (row: number, col: number) => void;
  picked: { row: number; col: number } | null;
  onCommit: (row: number, col: number, value: string) => void;
  onDeleteRow: (row: number) => void;
  /** #530: how many of the engine's fixes anchor in this group, and what
   *  clicking the header button applies. Count and action both come from the
   *  store's scoping of the engine's own list — this component only renders
   *  them. */
  fixCount: number;
  onFix: () => void;
}> = (props) => {
  const lastKey = () =>
    props.schema.headings.reduce((at, h, i) => (h.key ? i : at), -1);

  /* The in-place editor's cell, fine pointers only. Component-local where the
     pick is store-global: which cell is SELECTED matters to the carousel and
     the findings jump, but a half-open input is this table's own business.
     `seed` carries the keystroke that opened it — type-to-replace — or null
     for Enter/second-click, which edits the current value selected. */
  const [editing, setEditing] = createSignal<{
    row: number;
    col: number;
    seed: string | null;
  } | null>(null);

  let root: HTMLDivElement | undefined;
  const fine = () => !coarsePointer();
  const cellValue = (row: number, col: number) =>
    props.data.rows[row]?.[col] ?? "";

  /* Keyboard selection must MOVE focus, not just the highlight: focus is what
     scrolls the cell into the scroller's view and what keyboard events hang
     off. Microtask, so a cell created this frame is queryable. */
  const focusCell = (row: number, col: number) => {
    queueMicrotask(() => {
      root
        ?.querySelector<HTMLButtonElement>(`[data-cell="${row}-${col}"]`)
        ?.focus();
    });
  };

  const move = (dRow: number, dCol: number) => {
    const at = props.picked;
    if (!at) return;
    const row = Math.min(
      Math.max(at.row + dRow, 0),
      props.data.rows.length - 1,
    );
    const col = Math.min(
      Math.max(at.col + dCol, 0),
      props.schema.headings.length - 1,
    );
    props.onPick(row, col);
    focusCell(row, col);
  };

  const commitEdit = (value: string, thenMove = 0) => {
    const open = editing();
    if (!open) return;
    setEditing(null);
    props.onCommit(open.row, open.col, value);
    if (thenMove) move(0, thenMove);
    else focusCell(open.row, open.col);
  };

  const cancelEdit = () => {
    const open = editing();
    setEditing(null);
    if (open) focusCell(open.row, open.col);
  };

  /* One handler for the spreadsheet keys. Bubbling from the open input never
     reaches the branches below (`editing()` guards them), and modifier chords
     other than copy/paste fall through so the page-level undo hears them. */
  const onKeys = (e: KeyboardEvent) => {
    const at = props.picked;
    // The target check, not just `editing()`: the editor's own Enter has
    // ALREADY closed it by the time the event bubbles here, and acting on it
    // would reopen the editor the reader just committed.
    if (e.target instanceof HTMLElement && e.target.tagName === "INPUT") return;
    if (!fine() || !at || editing()) return;
    if (e.metaKey || e.ctrlKey) {
      const key = e.key.toLowerCase();
      if (key === "c") {
        void navigator.clipboard
          .writeText(cellValue(at.row, at.col))
          .catch(() => undefined);
      } else if (key === "v") {
        const commitTo = props.onCommit;
        void navigator.clipboard
          .readText()
          .then((value) => {
            commitTo(at.row, at.col, value);
          })
          .catch(() => undefined);
      }
      return;
    }
    const arrows: Record<string, readonly [number, number]> = {
      ArrowUp: [-1, 0],
      ArrowDown: [1, 0],
      ArrowLeft: [0, -1],
      ArrowRight: [0, 1],
    };
    const step = arrows[e.key];
    if (step) {
      // preventDefault: an arrow moves the SELECTION, never the scroller.
      e.preventDefault();
      move(step[0], step[1]);
    } else if (e.key === "Enter") {
      e.preventDefault();
      setEditing({ row: at.row, col: at.col, seed: null });
    } else if (e.key.length === 1 && !e.altKey) {
      e.preventDefault();
      setEditing({ row: at.row, col: at.col, seed: e.key });
    }
  };

  return (
    <div
      ref={root}
      onKeyDown={onKeys}
      /* No shadow class. #396 asks for a warm shadow, but the shared elevation
         layer resolves `--shadow-card` to `none` on the argument that a card
         lifts by border and surface step — and #400 requires exactly that in
         dark ("no card depends on a shadow to read as raised"). Following the
         token keeps one answer instead of two. */
      class="overflow-hidden rounded-lg border border-laterite-200 bg-surface dark:bg-surface-raised"
      style={{
        "--band": `var(${props.band})`,
        // 18% in light. #400 dials every tint down to 14% on the dark canvas,
        // where a saturated wash glows instead of tinting.
        "--key-tint": `color-mix(in srgb, var(${props.band}) var(--key-tint-pct), transparent)`,
      }}
    >
      {/* The cap: a single solid band, not the masthead's four-band gradient —
          a gradient here would read as four groups rather than one. */}
      <div aria-hidden="true" class="h-[3px] w-full bg-(--band)" />

      {/* The table's own header (#530): the fix budget lives ON the table it
          repairs, disabled at zero rather than hidden so "nothing fixable
          here" stays a visible fact, not an absence. */}
      <div class="flex items-center justify-end border-b border-laterite-200 px-2 py-1">
        <Button
          variant="action"
          size="sm"
          disabled={props.fixCount === 0}
          aria-label={`Fix ${props.fixCount} auto-fixable in ${props.schema.code}`}
          onClick={() => {
            props.onFix();
          }}
        >
          Fix {props.fixCount} auto-fixable
        </Button>
      </div>

      <div class="overflow-x-auto overscroll-x-contain">
        <table class="w-full border-collapse text-left">
          <caption class="sr-only">
            {props.schema.code} — {props.schema.description}
          </caption>
          <thead>
            <tr class="border-b border-line">
              <For each={props.schema.headings}>
                {(heading, col) => (
                  <th
                    scope="col"
                    class="px-3 py-2 font-mono text-micro font-semibold whitespace-nowrap"
                    classList={{
                      "bg-(--key-tint) text-accent": heading.key,
                      "text-fg-muted": !heading.key,
                      "border-r-[3px] border-r-(--band)": col() === lastKey(),
                      "sticky left-0 z-10 bg-surface dark:bg-surface-raised":
                        col() === 0,
                    }}
                  >
                    <span class="flex items-baseline gap-1">
                      <Show when={heading.key}>
                        {/* The KEY marker. Decorative — the column's role is
                            already in the header's title attribute. */}
                        <span
                          aria-hidden="true"
                          class="text-[0.6em] text-(--band)"
                        >
                          ◆
                        </span>
                      </Show>
                      <span title={`${heading.description} (${heading.type})`}>
                        {heading.name}
                      </span>
                    </span>
                    <span class="mt-0.5 block font-normal text-fg-faint">
                      {heading.type}
                      {heading.unit ? ` · ${heading.unit}` : ""}
                    </span>
                  </th>
                )}
              </For>
              {/* The delete column's header: a name for screen readers, no
                  visible label — the ✕ on each row is the affordance.
                  `relative` is load-bearing: `sr-only` positions absolutely,
                  and without a positioned ancestor its containing block skips
                  past the scroller to the PAGE, parking an invisible 1px box
                  at this cell's static position — out past the table's own
                  width — silently reintroducing the #523 document overflow. */}
              <th scope="col" class="relative px-1 py-2">
                <span class="sr-only">delete row</span>
              </th>
            </tr>
          </thead>
          <tbody>
            <For each={props.data.rows}>
              {(row, rowIndex) => (
                <tr class="border-b border-line-subtle last:border-b-0">
                  <For each={props.schema.headings}>
                    {(heading, col) => {
                      /* `createMemo` + `classList`, not a joined class string.
                         A `class={[...].join(" ")}` here computed ONCE and never
                         re-ran when the report arrived, so a failing cell stayed
                         unmarked while the findings panel beside it was correct
                         — the page contradicting itself in the one place a
                         reader is looking. classList takes accessors and is
                         unambiguously reactive. */
                      const cellFindings = createMemo(() =>
                        findingsForCell(
                          props.schema.code,
                          rowIndex(),
                          heading.name,
                        ),
                      );
                      const failing = () => cellFindings().length > 0;
                      /* The engine's worst tier on this cell drives tint,
                         text and marker alike — one severity grammar across
                         every table on the page (#526), never decided
                         here. */
                      const worst = createMemo(() =>
                        worstSeverity(cellFindings()),
                      );
                      const isPicked = createMemo(() => {
                        const p = props.picked;
                        return p?.row === rowIndex() && p.col === col();
                      });
                      const isEditing = createMemo(() => {
                        const open = editing();
                        return open?.row === rowIndex() && open.col === col();
                      });
                      return (
                        <td
                          class="px-3 py-1.5 text-caption whitespace-nowrap"
                          classList={{
                            // One region, one tint — no per-column striping.
                            // A failing cell's severity tint REPLACES the key
                            // tint: the verdict outranks the region colour.
                            "bg-(--key-tint)": heading.key && !failing(),
                            "border-r-[3px] border-r-(--band)":
                              col() === lastKey(),
                            "font-semibold": failing(),
                            [severityCellTint(worst() ?? "error")]: failing(),
                            "text-fg": !failing(),
                            "sticky left-0 z-10": col() === 0,
                            /* The sticky column's opaque backing yields to a
                               verdict: both are backgrounds, and leaving both
                               classes on lets STYLESHEET ORDER pick — column
                               0 untinted while columns 1+ tint. The quiet
                               tokens are opaque, so nothing ghosts under the
                               pinned cell. */
                            "bg-surface dark:bg-surface-raised":
                              col() === 0 && !failing(),
                          }}
                        >
                          <Show
                            when={isEditing()}
                            fallback={
                              <button
                                type="button"
                                data-cell={`${rowIndex()}-${col()}`}
                                onClick={() => {
                                  // Fine pointer, spreadsheet feel: the first
                                  // click selects, the second opens in place.
                                  // Coarse pointers pick — the carousel edits.
                                  if (fine() && isPicked())
                                    setEditing({
                                      row: rowIndex(),
                                      col: col(),
                                      seed: null,
                                    });
                                  else props.onPick(rowIndex(), col());
                                }}
                                aria-label={`Edit ${heading.name} on row ${rowIndex() + 1} of ${props.schema.code}`}
                                class="w-full rounded-xs px-1 text-left font-mono hover:bg-accent-quiet focus-visible:outline-hidden focus-visible:[box-shadow:var(--focus-ring)]"
                                classList={{ "bg-accent-quiet": isPicked() }}
                              >
                                <Show
                                  when={row[col()]}
                                  fallback={<span class="text-fg-dim">—</span>}
                                >
                                  {row[col()]}
                                </Show>
                                <Show when={failing()}>
                                  {/* The marker inherits the cell's severity
                                      colour; the tooltip carries what the
                                      engine actually said (#526). */}
                                  <Tooltip
                                    tip={cellFindings()
                                      .map(
                                        (f) =>
                                          `${f.rule} — ${f.desc}${isManualFinding(f) ? " — manual" : ""}`,
                                      )
                                      .join("  ·  ")}
                                  >
                                    <span aria-hidden="true" class="ml-1">
                                      ✗
                                    </span>
                                  </Tooltip>
                                </Show>
                              </button>
                            }
                          >
                            <CellEditor
                              label={`${heading.name} on row ${rowIndex() + 1} of ${props.schema.code}`}
                              initial={
                                editing()?.seed ?? cellValue(rowIndex(), col())
                              }
                              selectAll={editing()?.seed == null}
                              onCommit={commitEdit}
                              onCancel={cancelEdit}
                            />
                          </Show>
                        </td>
                      );
                    }}
                  </For>
                  {/* Row delete (#525): without it the Rule 13 teach-loop —
                      add a second PROJ row, watch it flagged — could only be
                      unwound by resetting the whole demo. */}
                  <td class="px-1 py-1.5 whitespace-nowrap">
                    <button
                      type="button"
                      onClick={() => {
                        props.onDeleteRow(rowIndex());
                      }}
                      aria-label={`Delete row ${rowIndex() + 1} of ${props.schema.code}`}
                      class="rounded-xs px-1 text-caption text-fg-faint hover:bg-err-quiet hover:text-err focus-visible:outline-hidden focus-visible:[box-shadow:var(--focus-ring)]"
                    >
                      ✕
                    </button>
                  </td>
                </tr>
              )}
            </For>
          </tbody>
        </table>
      </div>
    </div>
  );
};
