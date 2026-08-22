/* The row carousel (#398) — the COARSE pointer's editor.
 *
 * Tapping any cell opens THAT WHOLE ROW as a paged set of field cards, landing
 * on the field that was tapped. This is the pattern chosen to carry LLPL's nine
 * columns on a 390px phone: a plain grid cannot, and shrinking type to make one
 * fit destroys the KEY chain the page exists to show.
 *
 * It used to be the editing pattern AT EVERY WIDTH; #525 split the editors by
 * modality instead. A fine pointer edits in the table itself — selection,
 * arrows, type-to-replace — because on a desktop this tray is an indirection:
 * click a cell, look away from it, edit somewhere else. The split reads
 * `pointer: coarse` (pointer.ts), not the viewport width — a phone in
 * landscape is still a touch device.
 *
 * It is a PANEL IN THE PAGE FLOW, between the table and what follows it. It
 * never floats over the content and never covers the findings the reader is
 * working from — which is also why it is not the shared Dialog: a dialog traps
 * focus and dims the page, and both are wrong for something you edit while
 * reading the findings it changes.
 *
 * ## The 820px policy does not apply here
 *
 * The design bundle contradicts itself — the foundations artboard hides
 * interactive demos below 820px, and the options artboard exists precisely to
 * find an editing pattern that works at 390px. The editing pattern wins. A
 * landing page whose one interactive proof is switched off for mobile visitors
 * is a landing page that does not land.
 */

import { For, Show, createMemo, type Component } from "solid-js";
import { Button } from "@shared/components";
import type { DemoGroup } from "./schema";
import type { Group } from "./delivery";
import { findingsForCell, groupFindingsNaming, setCell } from "./store";

/** Rule 8's message names the TYPE but not what it means. A newcomer editing
 *  `11.8` needs "two decimal places", not "does not match its declared TYPE
 *  2DP" — so the plain-words half is written here, keyed by the AGS TYPE the
 *  dictionary gives. Unknown types fall through to the engine's own wording,
 *  which is always present above it. */
const TYPE_IN_WORDS: Record<string, string> = {
  ID: "a unique identifier — free text, but it has to be unique in its group",
  X: "free text",
  PA: "one of the abbreviations the ABBR group defines for this heading",
  XN: "text or a number",
  "0DP": "a whole number, no decimal point",
  "1DP": "a number written to one decimal place",
  "2DP": "a number written to exactly two decimal places — 11.80, not 11.8",
  "3DP": "a number written to three decimal places",
  DT: "a date or timestamp in the format the UNIT row declares",
};

export const RowCarousel: Component<{
  schema: DemoGroup;
  data: Group;
  band: string;
  row: number;
  col: number;
  onMove: (col: number) => void;
  onClose: () => void;
  onDelete: () => void;
}> = (props) => {
  /* `Show`'s callback form is what narrows this to a definite heading, rather
     than a non-null assertion the lint rule forbids — and it is honest: a col
     out of range means the caller and the schema disagree, and rendering
     nothing is the right answer to that. */
  const heading = createMemo(() => props.schema.headings[props.col]);
  const value = () => props.data.rows[props.row]?.[props.col] ?? "";
  const failing = createMemo(() => {
    const h = heading();
    return h ? findingsForCell(props.schema.code, props.row, h.name) : [];
  });

  /** What the card WRITES OUT: this cell's findings plus the group-level ones
   *  that name this heading. `failing()` alone drives the red border, so a
   *  group finding explains without accusing the row. */
  const explains = createMemo(() => {
    const h = heading();
    if (!h) return [];
    return [...failing(), ...groupFindingsNaming(props.schema.code, h.name)];
  });

  const step = (by: number) => {
    const next = props.col + by;
    if (next >= 0 && next < props.schema.headings.length) props.onMove(next);
  };

  return (
    <div
      /* The raised step was the landing's canvas EXACTLY — this page retunes
         `--canvas` onto `--surface-raised`'s own value — so the tray had no
         fill of its own and read as bare page inside a border (#452).
         It lifts, and the field card below drops a step to make room, which is
         the order dark has always read in: lifted tray, recessed card. Doing it
         the other way round — recessing the tray onto `--chip` — costs the nav
         buttons their hover, since the default Button hovers to that same fill
         and would dissolve into the tray it sits on. Naming a utility rather
         than a token here would re-emit it into this bundle (#437). */
      class="mt-4 rounded-lg border border-line bg-surface p-4 dark:bg-surface-raised"
      style={{ "--band": `var(${props.band})` }}
      role="group"
      aria-label={`Editing row ${props.row + 1} of ${props.schema.code}`}
      onKeyDown={(e) => {
        // Reachable and dismissable without a pointer.
        if (e.key === "Escape") props.onClose();
        if (e.key === "ArrowLeft" && e.altKey) step(-1);
        if (e.key === "ArrowRight" && e.altKey) step(1);
      }}
    >
      <div class="flex items-center justify-between gap-3">
        <p class="font-mono text-micro uppercase tracking-(--track-micro) text-fg-muted">
          {props.schema.code} · row {props.row + 1} · field {props.col + 1} of{" "}
          {props.schema.headings.length}
        </p>
        <span class="flex items-center gap-1">
          {/* Deleting the open row closes the tray — the store drops a pick
              whose row is gone (#525), so no half-open editor on a ghost row. */}
          <Button
            variant="ghost"
            size="sm"
            tone="danger"
            onClick={props.onDelete}
            aria-label="Delete this row"
          >
            Delete row
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={props.onClose}
            aria-label="Close the row editor"
          >
            ✕
          </Button>
        </span>
      </div>

      <div class="mt-3 flex items-stretch gap-2">
        <Button
          variant="default"
          size="sm"
          onClick={() => {
            step(-1);
          }}
          disabled={props.col === 0}
          aria-label="Previous field in this row"
        >
          ‹
        </Button>

        <Show when={heading()}>
          {(h) => (
            <div
              /* A step BELOW the tray, not level with it (#452): light takes
                 the canvas, dark keeps the surface it always had. This is also
                 what gives the value input its own step in light, where the
                 card and the input were one fill told apart by a border. */
              class="min-w-0 flex-1 rounded-md border border-line bg-canvas p-3 dark:bg-surface"
            >
              <p class="flex flex-wrap items-baseline gap-2">
                <Show when={h().key}>
                  <span aria-hidden="true" class="text-(--band)">
                    ◆
                  </span>
                </Show>
                <span class="font-mono text-body font-semibold text-accent">
                  {h().name}
                </span>
                <span class="font-mono text-micro text-fg-faint">
                  {h().type}
                  {h().unit ? ` · ${h().unit}` : ""}
                </span>
              </p>

              <p class="mt-1 text-caption text-fg-muted">{h().description}</p>

              <input
                class="mt-3 w-full rounded-md border border-line-strong bg-surface px-3 py-2 font-mono text-body text-fg focus-visible:outline-hidden focus-visible:[box-shadow:var(--focus-ring)]"
                classList={{ "border-err": failing().length > 0 }}
                value={value()}
                aria-label={`${h().name} value`}
                aria-invalid={failing().length > 0}
                /* Live, no submit — every keystroke re-emits and revalidates. */
                onInput={(e) => {
                  setCell(
                    props.schema.code,
                    props.row,
                    props.col,
                    e.currentTarget.value,
                  );
                }}
              />

              <p class="mt-2 text-caption text-fg-faint">
                {TYPE_IN_WORDS[h().type] ?? `AGS TYPE ${h().type}`}.
                <Show when={h().key}>
                  {" "}
                  It is a KEY field, so it also has to match the parent row this
                  one hangs off.
                </Show>
              </p>

              <For each={explains()}>
                {(finding) => (
                  <p class="mt-2 rounded-md border border-err/40 bg-err-quiet px-3 py-2 text-caption text-err">
                    <span class="font-semibold">{finding.rule}</span> —{" "}
                    {finding.desc}
                  </p>
                )}
              </For>
            </div>
          )}
        </Show>

        <Button
          variant="default"
          size="sm"
          onClick={() => {
            step(1);
          }}
          disabled={props.col === props.schema.headings.length - 1}
          aria-label="Next field in this row"
        >
          ›
        </Button>
      </div>
    </div>
  );
};
