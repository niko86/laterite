/* One group's table (#396, editable in #398).
 *
 * THE REPETITION IS THE RELATIONSHIP. AGS4 has no ids and no joins: a child
 * group restates its parent's KEY headings verbatim, which is why LLPL is nine
 * columns wide and why SAMP_ID appears in three of the four tables. The design
 * lets that land rather than hiding it — it is the most surprising thing about
 * the format to a newcomer, and the page's best four seconds.
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
 */

import { For, Show, createMemo, type Component } from "solid-js";
import type { DemoGroup } from "./schema";
import type { Group } from "./delivery";
import { findingsForCell } from "./store";

export const GroupTable: Component<{
  schema: DemoGroup;
  data: Group;
  /** The CSS variable naming this group's band, e.g. `--laterite-500`. */
  band: string;
  onPick: (row: number, col: number) => void;
  picked: { row: number; col: number } | null;
}> = (props) => {
  const lastKey = () =>
    props.schema.headings.reduce((at, h, i) => (h.key ? i : at), -1);

  return (
    <div
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
      <div aria-hidden="true" class="h-[3px] w-full bg-[--band]" />

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
                      "bg-[--key-tint] text-accent": heading.key,
                      "text-fg-muted": !heading.key,
                      "border-r-[3px] border-r-[--band]": col() === lastKey(),
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
                          class="text-[0.6em] text-[--band]"
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
                      const failing = createMemo(
                        () =>
                          findingsForCell(
                            props.schema.code,
                            rowIndex(),
                            heading.name,
                          ).length > 0,
                      );
                      const isPicked = createMemo(() => {
                        const p = props.picked;
                        return p?.row === rowIndex() && p.col === col();
                      });
                      return (
                        <td
                          class="px-3 py-1.5 text-caption whitespace-nowrap"
                          classList={{
                            // One region, one tint — no per-column striping.
                            "bg-[--key-tint]": heading.key,
                            "border-r-[3px] border-r-[--band]":
                              col() === lastKey(),
                            "font-semibold text-err": failing(),
                            "text-fg": !failing(),
                            "sticky left-0 z-10 bg-surface dark:bg-surface-raised":
                              col() === 0,
                          }}
                        >
                          <button
                            type="button"
                            onClick={() => {
                              props.onPick(rowIndex(), col());
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
                              <span aria-hidden="true" class="ml-1 text-err">
                                ✗
                              </span>
                            </Show>
                          </button>
                        </td>
                      );
                    }}
                  </For>
                </tr>
              )}
            </For>
          </tbody>
        </table>
      </div>
    </div>
  );
};
