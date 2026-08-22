/* One group's section (#396, #398): the chip, the prose, the table, the editor.
 *
 * The prose beside each table is what makes the chain teachable — a reader who
 * cannot see that SAMP hangs off LOCA hangs off PROJ gets nothing from breaking
 * a row later. The parent named in each line comes from the generated schema's
 * `parent` field, so a dictionary edition that re-parented a group would change
 * this sentence without a code edit.
 */

import { For, Show, createMemo, type Component } from "solid-js";
import { Button } from "@shared/components";
import { FindingCallout } from "./FindingCallout";
import { DEMO_GROUPS } from "./schema";
import { RowCarousel } from "./RowCarousel";
import { GroupTable } from "./GroupTable";
import { coarsePointer } from "./pointer";
import {
  addRow,
  arm,
  deleteRow,
  delivery,
  findingsForGroup,
  picked,
  setCell,
  setPicked,
} from "./store";

/** What each group is FOR, in one sentence a geotechnical newcomer can hold.
 *  Editorial rather than derived: the dictionary's own descriptions ("Location
 *  Details") name the group without explaining why it exists. */
const BLURB: Record<string, string> = {
  PROJ: "Every delivery starts here. One row, naming the job — and every other group in the file hangs off it.",
  LOCA: "The holes. One row per borehole or trial pit, with its ground level and how deep it went.",
  SAMP: "What came out of the hole. Note that LOCA_ID reappears — that repetition IS the link back to the borehole.",
  LLPL: "Atterberg limits, one row per specimen. Nine columns, seven of them KEY: five restating SAMP's key, two naming the specimen. That is what a join looks like in a format with no joins.",
};

export const GroupSection: Component<{
  code: "PROJ" | "LOCA" | "SAMP" | "LLPL";
  band: string;
  tableFirst: boolean;
}> = (props) => {
  /* One memo rather than two accessors, so a single `Show` narrows both. The
     lint rule forbids the non-null assertions this would otherwise take, and
     the pair is genuinely all-or-nothing: a schema without matching data is a
     generated-file/fixture mismatch, which the Python gate already fails on. */
  const bits = createMemo(() => {
    const schema = DEMO_GROUPS.find((g) => g.code === props.code);
    const data = delivery().find((g) => g.code === props.code);
    return schema && data ? { schema, data } : undefined;
  });

  const open = createMemo(() => {
    const p = picked();
    return p && p.group === props.code ? { row: p.row, col: p.col } : null;
  });

  const groupFindings = createMemo(() => findingsForGroup(props.code));

  return (
    <Show when={bits()}>
      {(b) => (
        <div
          class="grid gap-8 min-[64rem]:items-start"
          classList={{
            /* The table column is the wider of the two, on BOTH sides of the
               alternation. Swapping only the ORDER leaves the table in the
               22rem prose column every other section, which clipped SAMP_ID
               off SAMP and three columns off LLPL. */
            "min-[64rem]:grid-cols-[minmax(0,1fr)_minmax(0,22rem)]":
              props.tableFirst,
            "min-[64rem]:grid-cols-[minmax(0,22rem)_minmax(0,1fr)]":
              !props.tableFirst,
          }}
        >
          {/* `min-w-0` on BOTH children: below the breakpoint the grid has no
              explicit template, so the items keep `min-width: auto` and the
              nowrap tables' min-content sizes the PAGE (#523) — a 390px phone
              got a 783px layout viewport. Above it, the `minmax(0,1fr)`
              columns already hold the floor. */}
          <div
            class="min-w-0"
            classList={{ "min-[64rem]:order-2": props.tableFirst }}
          >
            {/* The group chip: band tint, a solid band rule inset on the left,
                and MAROON text. Never white or black on a band fill — the
                mid-ramp bands fail contrast in both directions. */}
            <p
              class="inline-flex items-center gap-2 rounded-sm border-l-[3px] py-1 pr-3 pl-2 font-mono text-micro font-semibold tracking-(--track-micro) text-accent"
              style={{
                "border-left-color": `var(${props.band})`,
                background: `color-mix(in srgb, var(${props.band}) var(--chip-tint-pct), transparent)`,
              }}
            >
              {props.code}
              <Show when={b().schema.parent}>
                {(parent) => (
                  <span class="font-normal text-fg-muted">
                    child of {parent()}
                  </span>
                )}
              </Show>
            </p>

            <h2 class="mt-3 font-display text-h2 font-extrabold tracking-(--track-tight) text-accent">
              {b().schema.description}
            </h2>
            <p class="mt-2 text-fg-soft">{BLURB[props.code]}</p>
          </div>

          <div
            class="min-w-0"
            classList={{ "min-[64rem]:order-1": props.tableFirst }}
          >
            <GroupTable
              schema={b().schema}
              data={b().data}
              band={props.band}
              picked={open()}
              onPick={(row, col) => {
                arm();
                setPicked({ group: props.code, row, col });
              }}
              onCommit={(row, col, value) => {
                setCell(props.code, row, col, value);
              }}
              onDeleteRow={(row) => {
                deleteRow(props.code, row);
              }}
            />

            {/* Group-level findings strip — attached to the TABLE it judges,
                not the prose column beside it (#526): a finding rendered in
                the essay reads as commentary; on the table it reads as a
                verdict. Same callout as the carousel and the panel. */}
            <Show when={groupFindings().length}>
              <ul
                aria-label={`${props.code} findings`}
                class="mt-3 list-none space-y-2 p-0"
              >
                <For each={groupFindings()}>
                  {(f) => (
                    <li>
                      <FindingCallout severity={f.severity} rule={f.rule}>
                        {f.desc}
                      </FindingCallout>
                    </li>
                  )}
                </For>
              </ul>
            </Show>

            <div class="mt-3 flex flex-wrap items-center gap-3">
              <Button
                variant="add"
                onClick={() => {
                  addRow(props.code, b().schema.parent);
                }}
              >
                + row
                <Show when={b().schema.parent}>
                  {(parent) => (
                    <span class="text-fg-faint">
                      {" "}
                      (inherits {parent()}'s key)
                    </span>
                  )}
                </Show>
              </Button>
              <span class="text-caption text-fg-faint">
                {/* The hint follows the editor the reader actually has (#525). */}
                {coarsePointer()
                  ? "Tap any cell to edit the row."
                  : "Click a cell, then type — Enter commits, Esc cancels."}
              </span>
            </div>

            {/* The carousel is the COARSE pointer's editor (#525); on a fine
                pointer the pick is a spreadsheet selection and opening a tray
                under the table would double the editing surface. */}
            <Show when={coarsePointer() ? open() : null}>
              {(cell) => (
                <RowCarousel
                  schema={b().schema}
                  data={b().data}
                  band={props.band}
                  row={cell().row}
                  col={cell().col}
                  onMove={(col) =>
                    setPicked({ group: props.code, row: cell().row, col })
                  }
                  onClose={() => setPicked(null)}
                  onDelete={() => {
                    deleteRow(props.code, cell().row);
                  }}
                />
              )}
            </Show>
          </div>
        </div>
      )}
    </Show>
  );
};
