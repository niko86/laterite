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
 * The band appears in exactly two places for a group: its chip and its table
 * cap. It was three until #590 retired the band-coloured KEY region — on the
 * red-brown ramp it read as a verdict anyway, the exact confusion this rule
 * exists to prevent — so the region now wears the theme's structural stone
 * tint (landing.css). Severity is carried by the error tint and the corner
 * flag (#616), which are the same on every group.
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
import { Button, Icon, Popover } from "@shared/components";
import type { DemoGroup, DemoHeading } from "./schema";
import { singleLine, type Group } from "./delivery";
import { FindingCallout } from "./FindingCallout";
import { coarsePointer } from "./pointer";
import {
  severityCellTint,
  severityRowEdge,
  severityRowTint,
  worstSeverity,
} from "./severity";
import { findingsForCell, findingsForRow, isManualFinding } from "./store";

/** The in-place value input (#525), its own component for two reasons: mount
 *  focus/select read PROPS rather than signals inside a lifecycle callback,
 *  and commit-vs-cancel settles locally — whichever key closes it marks it
 *  done, so the blur a close causes cannot commit a second time. Uncontrolled
 *  on purpose: the DOM holds the draft and the model hears about it at
 *  COMMIT — a per-keystroke setCell would revalidate half-typed values the
 *  reader never asserted. (The carousel commits live by design: its card IS
 *  the editor.) */
/** The status half of a header's tooltip (#616): the marks are decorative,
 *  so this is where the grammar is spelled out. KEY and REQUIRED are
 *  independent rules — 10a is identity, 10b is non-empty — which is why the
 *  words name them separately rather than ranking one under the other. */
function statusWords(heading: DemoHeading): string {
  if (heading.key && heading.required)
    return " · KEY and REQUIRED: part of the row's identity, never empty";
  if (heading.key) return " · KEY: part of the row's identity";
  if (heading.required) return " · REQUIRED: must not be empty";
  return "";
}

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
  /** #530: how many of the engine's fixes anchor in this group — or null
   *  before the engine's first count, which renders no button at all: "Fix 0"
   *  before anything was counted would be a false zero (#531's scoreboard
   *  rule). Count and action both come from the store's scoping of the
   *  engine's own list — this component only renders them. */
  fixCount: number | null;
  /** #582: the last fix click was refused because reparsing the engine's
   *  fixed text would have LOST data. Rendered as a note beside the button —
   *  never as a finding, because the scoreboard tallies the engine's report
   *  and this verdict is the page's own. */
  fixRefused: boolean;
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
  let tableEl: HTMLTableElement | undefined;
  const fine = () => !coarsePointer();

  /* Freeze the geometry for the life of an edit session (#593): the auto
     layout re-solves on every keystroke into the editor — thirteen
     characters into SAMP_TYPE moved all six columns. Measured BEFORE the
     editor mounts, because a type-to-replace open swaps a long value for a
     one-character input, which is itself enough to move the columns; then
     fixed layout holds that solution until the editor closes, when auto
     layout resumes so a committed value still earns its column the usual
     way. */
  const freezeColumns = () => {
    const el = tableEl;
    if (!el || el.style.tableLayout === "fixed") return;
    for (const th of el.querySelectorAll("th")) {
      th.style.width = `${th.getBoundingClientRect().width}px`;
    }
    el.style.tableLayout = "fixed";
  };
  const unfreezeColumns = () => {
    const el = tableEl;
    if (!el) return;
    for (const th of el.querySelectorAll("th")) th.style.width = "";
    el.style.tableLayout = "";
  };
  const openEditor = (cell: {
    row: number;
    col: number;
    seed: string | null;
  }) => {
    freezeColumns();
    setEditing(cell);
  };
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
    unfreezeColumns();
    props.onCommit(open.row, open.col, value);
    if (thenMove) move(0, thenMove);
    else focusCell(open.row, open.col);
  };

  const cancelEdit = () => {
    const open = editing();
    setEditing(null);
    unfreezeColumns();
    if (open) focusCell(open.row, open.col);
  };

  /* One handler for the spreadsheet keys. Bubbling from the open input never
     reaches the branches below (`editing()` guards them), and modifier chords
     other than copy/paste fall through so the page-level undo hears them.

     THE CLIPBOARD CONTRACT (#551), stated as observed rather than designed:
     on a SELECTED cell, Ctrl/Cmd+C writes the cell's raw value (never the
     status glyph) via the async clipboard API, and Ctrl/Cmd+V commits the
     clipboard string verbatim through the store funnel — one commit, so undo
     unwinds a paste in one step exactly like a typed edit. An OPEN editor is
     deliberately not ours: the INPUT early-return above hands the browser its
     native input clipboard, and the carousel's cards are the same story.
     "Verbatim" stops at the line terminators (#574): the handler is the one
     entry path a browser does not sanitize for us, so it applies the rule a
     single-line input applies to a paste — each break becomes a space — via
     `singleLine`. `setCell` enforces the same invariant on the model, which
     is what makes it true rather than merely usual; the call here is where a
     reader SEES it happen. */
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
        /* #580: the clipboard resolves LATER, and between the keypress and
           the read the reader can delete the row, undo, apply fixes or
           reset — after which these indices name data they never chose. The
           recorded decision is dropStalePick's: ABANDON, never re-aim. The
           value is the target's identity as far as one exists (rows carry no
           ids): a pick still on the same position over the same value is the
           same target; anything else is a moved one and the paste dies
           silently. A swapped-in row with a byte-identical cell value is the
           one residual this cannot see. */
        const before = cellValue(at.row, at.col);
        void navigator.clipboard
          .readText()
          // eslint-disable-next-line solid/reactivity -- event-handler continuation: reading the pick and cell AT RESOLVE TIME is the #580 fix, not a missed tracking scope
          .then((value) => {
            const now = props.picked;
            if (!now || now.row !== at.row || now.col !== at.col) return;
            if (cellValue(at.row, at.col) !== before) return;
            commitTo(at.row, at.col, singleLine(value));
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
      openEditor({ row: at.row, col: at.col, seed: null });
    } else if (e.key.length === 1 && !e.altKey) {
      e.preventDefault();
      openEditor({ row: at.row, col: at.col, seed: e.key });
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
      style={{ "--band": `var(${props.band})` }}
    >
      {/* The cap: a single solid band, not the masthead's four-band gradient —
          a gradient here would read as four groups rather than one. */}
      <div aria-hidden="true" class="h-[3px] w-full bg-(--band)" />

      {/* The table's own header (#530): the fix budget lives ON the table it
          repairs, disabled at zero rather than hidden so "nothing fixable
          here" stays a visible fact, not an absence — but only once a count
          EXISTS. Before the engine's first pass the bar stays empty rather
          than stating a zero nobody computed. */}
      <Show when={props.fixCount !== null}>
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
        {/* The refusal note (#582): the neutral tier, no rule and no engine
            severity — this is the page refusing its own commit, not the
            engine finding anything. It sits under the button it answers. */}
        <Show when={props.fixRefused}>
          <div class="border-b border-laterite-200 px-2 py-1.5">
            <FindingCallout severity="note">
              These repairs came back missing part of the table, so nothing was
              changed. The file is exactly as you left it.
            </FindingCallout>
          </div>
        </Show>
      </Show>

      <div class="overflow-x-auto overscroll-x-contain">
        <table ref={tableEl} class="w-full border-collapse text-left">
          <caption class="sr-only">
            {props.schema.code} · {props.schema.description}
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
                      {/* The status marks (#616): a key glyph for KEY, the
                          glyph boxed for KEY+REQUIRED, the form-convention
                          `*` for REQUIRED alone, nothing for OTHER. The two
                          axes are independent rules — KEY is 10a (identity),
                          REQUIRED is 10b (non-empty) — and both occur
                          separately on this page's own headings. Decorative:
                          the words live in the header's title attribute. */}
                      <Show when={heading.key}>
                        {/* The system's key, not a drawn one (icons.ts's
                            rule), at the sheet's ~10px in the band ink. */}
                        <span
                          aria-hidden="true"
                          class="self-center text-(--band)"
                          classList={{
                            "rounded-xs border border-current p-px":
                              heading.required,
                          }}
                        >
                          <Icon name="key-round" size={10} class="block" />
                        </span>
                      </Show>
                      <Show when={!heading.key && heading.required}>
                        <span aria-hidden="true" class="text-(--band)">
                          *
                        </span>
                      </Show>
                      <span
                        title={`${heading.description} (${heading.type})${statusWords(heading)}`}
                      >
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
              {(row, rowIndex) => {
                /* The row's own verdict (#590), worst-of like the cell's. */
                const rowWorst = createMemo(() =>
                  worstSeverity(findingsForRow(props.schema.code, rowIndex())),
                );
                const rowFailing = () => rowWorst() !== null;
                return (
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
                              // Verdicts outrank the region colour, and rank
                              // among themselves: a failing CELL's tint beats
                              // the row wash, which beats the key tint (#590).
                              "bg-(--key-tint)":
                                heading.key && !failing() && !rowFailing(),
                              [severityRowTint(rowWorst() ?? "error")]:
                                rowFailing() && !failing(),
                              [severityRowEdge(rowWorst() ?? "error")]:
                                rowFailing() && col() === 0,
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
                               pinned cell — the row wash included, which is
                               why the backing also stands down for it. */
                              "bg-surface dark:bg-surface-raised":
                                col() === 0 && !failing() && !rowFailing(),
                            }}
                          >
                            <Show
                              when={isEditing()}
                              fallback={
                                /* The finding text lives ON the cell (#591):
                                   hover, select and keyboard focus each
                                   summon the callouts the strip used to
                                   carry — cell findings when the cell has
                                   its own, else the row's, so the orphan
                                   pops from any of its cells. Content, not
                                   the popover, carries the severity tint. */
                                <Popover
                                  class="w-full"
                                  content={
                                    failing() || rowFailing() ? (
                                      <For
                                        each={
                                          failing()
                                            ? cellFindings()
                                            : findingsForRow(
                                                props.schema.code,
                                                rowIndex(),
                                              )
                                        }
                                      >
                                        {(f) => (
                                          <FindingCallout
                                            severity={f.severity}
                                            rule={f.rule}
                                            manual={isManualFinding(f)}
                                          >
                                            {f.desc}
                                          </FindingCallout>
                                        )}
                                      </For>
                                    ) : undefined
                                  }
                                >
                                  <button
                                    type="button"
                                    data-cell={`${rowIndex()}-${col()}`}
                                    onClick={() => {
                                      // Fine pointer, spreadsheet feel: the
                                      // first click selects, the second opens
                                      // in place. Coarse pointers pick — the
                                      // carousel edits.
                                      if (fine() && isPicked())
                                        openEditor({
                                          row: rowIndex(),
                                          col: col(),
                                          seed: null,
                                        });
                                      else props.onPick(rowIndex(), col());
                                    }}
                                    aria-label={`Edit ${heading.name} on row ${rowIndex() + 1} of ${props.schema.code}`}
                                    class="relative w-full rounded-xs px-1 text-left font-mono transition-colors hover:bg-accent-quiet focus-visible:outline-hidden focus-visible:[box-shadow:var(--focus-ring)]"
                                    classList={{
                                      "bg-accent-quiet": isPicked(),
                                    }}
                                  >
                                    <Show
                                      when={row[col()]}
                                      fallback={
                                        <span class="text-fg-dim">–</span>
                                      }
                                    >
                                      {row[col()]}
                                    </Show>
                                    <Show when={failing()}>
                                      {/* The corner flag (#616): the
                                        spreadsheet convention for an
                                        annotated cell, replacing the inline
                                        ✗ that read as part of the value
                                        ("11.8 ✗", worse at 390px). It
                                        borrows the cell's severity ink via
                                        currentColor; the popover above
                                        carries what the engine said. */}
                                      <span
                                        aria-hidden="true"
                                        class="absolute top-0 right-0 border-t-[7px] border-l-[7px] border-t-current border-l-transparent"
                                      />
                                    </Show>
                                  </button>
                                </Popover>
                              }
                            >
                              <CellEditor
                                label={`${heading.name} on row ${rowIndex() + 1} of ${props.schema.code}`}
                                initial={
                                  editing()?.seed ??
                                  cellValue(rowIndex(), col())
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
                          // Belt over the blur's braces: the input's blur has
                          // already committed and unfrozen by the time a click
                          // lands in every engine we ship to, but that is
                          // event ORDERING, not a contract — and an editor
                          // outliving its row would leave the table frozen at
                          // stale widths for good (#593).
                          if (editing()) cancelEdit();
                          props.onDeleteRow(rowIndex());
                        }}
                        aria-label={`Delete row ${rowIndex() + 1} of ${props.schema.code}`}
                        class="rounded-xs px-1 text-caption text-fg-faint transition-colors hover:bg-err-quiet hover:text-err focus-visible:outline-hidden focus-visible:[box-shadow:var(--focus-ring)]"
                      >
                        ✕
                      </button>
                    </td>
                  </tr>
                );
              }}
            </For>
          </tbody>
        </table>
      </div>
    </div>
  );
};
