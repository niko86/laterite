/* One group's section (#396, #398): the chip, the prose, the table, the editor.
 *
 * The prose beside each table is what makes the chain teachable — a reader who
 * cannot see that SAMP hangs off LOCA hangs off PROJ gets nothing from breaking
 * a row later. The parent named in each line comes from the generated schema's
 * `parent` field, so a dictionary edition that re-parented a group would change
 * this sentence without a code edit.
 *
 * What the section OWNS is the pairing: the prose column, the alternation, and
 * which affordances a descent group offers. The table column is the shared
 * harness (#549), which the TRAN cover sheet renders too.
 */

import { Show, createMemo, type Component } from "solid-js";
import { EditableGroup } from "./EditableGroup";
import { DEMO_GROUPS } from "./schema";

/** What each group is FOR, in one sentence a geotechnical newcomer can hold.
 *  Editorial rather than derived: the dictionary's own descriptions ("Location
 *  Details") name the group without explaining why it exists. */
const BLURB: Record<string, string> = {
  PROJ: "Every delivery starts here. One row, naming the job; every other group in the file hangs off it.",
  LOCA: "The holes. One row per borehole or trial pit, with its ground level and how deep it went.",
  SAMP: "What came out of the hole. Note that LOCA_ID reappears; that repetition IS the link back to the borehole.",
  LLPL: "Atterberg limits, one row per specimen. Nine columns, seven of them KEY: five restating SAMP's key, two naming the specimen. That is what a join looks like in a format with no joins.",
};

export const GroupSection: Component<{
  code: "PROJ" | "LOCA" | "SAMP" | "LLPL";
  band: string;
  tableFirst: boolean;
}> = (props) => {
  /* The section narrows on the SCHEMA alone. A schema without matching data
     means the reader deleted the group, and the harness answers that with the
     restore stub; a missing schema is the other story — a generated-file /
     fixture mismatch the Python gate fails on — and there is nothing to say
     about a group the dictionary does not describe. */
  const schema = createMemo(() =>
    DEMO_GROUPS.find((g) => g.code === props.code),
  );

  return (
    <Show when={schema()}>
      {(sch) => (
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
              <Show when={sch().parent}>
                {(parent) => (
                  <span class="font-normal text-fg-muted">
                    child of {parent()}
                  </span>
                )}
              </Show>
            </p>

            <h2 class="mt-3 font-display text-h2 font-extrabold tracking-(--track-tight) text-accent">
              {sch().description}
            </h2>
            <p class="mt-2 text-fg-soft">{BLURB[props.code]}</p>
          </div>

          <div
            class="min-w-0"
            classList={{ "min-[64rem]:order-1": props.tableFirst }}
          >
            {/* Both affordances, unlike the cover sheet: a descent group takes
                as many rows as the reader wants, and this is where the page
                first says how to edit one. */}
            <EditableGroup
              code={props.code}
              band={props.band}
              canAddRow
              showEditHint
            />
          </div>
        </div>
      )}
    </Show>
  );
};
