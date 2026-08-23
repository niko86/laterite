/* The editable group harness (#549): one group's table and every control that
 * edits it.
 *
 * Extracted because the four descent sections and the TRAN cover sheet were
 * maintaining the same wiring twice — the same store calls, the same two
 * narrowings, the same coarse-pointer guard — and the copy grew by a shape
 * every ticket that touched it (#527 seeded it, #529 added the restore stub
 * and the delete button, #530 the per-table fix budget).
 *
 * TRAN used to be a strict SUBSET of a descent group — #527 held "+ row"
 * off a one-row transmission header as meaningless, and two boolean props
 * carved that subset. #593 superseded the ruling deliberately: a second TRAN
 * row is itself a teachable state, because the engine has a verdict about
 * it, so every caller now gets the full toolbar and the props went with the
 * distinction they encoded.
 *
 * LAYOUT STAYS WITH THE CALLERS. This emits the table, the carousel, the
 * strip and the actions row; the grid column GroupSection places them in and the
 * `mt-3` block the cover sheet places them in belong to those callers, and
 * normalising the difference would be a visual change the extraction has no
 * business making.
 *
 * Sharing the harness did NOT make TRAN an eighth descent section (#527). It
 * keeps its own prose and its own place beside the file it covers, and it adds
 * nothing to the strata ramp — whose length is `web/landing/sections.test.ts`'s
 * to hold, not this comment's to restate.
 */

import {
  Show,
  createEffect,
  createMemo,
  createSignal,
  on,
  type Component,
} from "solid-js";
import { Button } from "@shared/components";
import { FindingsStrip } from "./FindingsStrip";
import { GroupStub } from "./GroupStub";
import { GroupTable } from "./GroupTable";
import { Presence } from "./Presence";
import { RowCarousel } from "./RowCarousel";
import { DEMO_GROUPS } from "./schema";
import { coarsePointer } from "./pointer";
import {
  addRow,
  applyGroupFixes,
  arm,
  deleteGroup,
  deleteRow,
  delivery,
  findingsForGroup,
  groupFixCount,
  picked,
  restoreGroup,
  setCell,
  setPicked,
} from "./store";

export const EditableGroup: Component<{
  code: string;
  band: string;
}> = (props) => {
  /* Since #529 the pair is no longer all-or-nothing: a schema without matching
     data now means the reader DELETED the group, and the harness answers with
     the restore stub instead of vanishing. */
  const bits = createMemo(() => {
    const schema = DEMO_GROUPS.find((g) => g.code === props.code);
    const data = delivery().find((g) => g.code === props.code);
    return schema && data ? { schema, data } : undefined;
  });

  const open = createMemo(() => {
    const p = picked();
    return p && p.group === props.code ? { row: p.row, col: p.col } : null;
  });

  /* Memoized rather than called per strip: the cover sheet's copy recomputed
     this once for each branch of its narrowing. */
  const groupFindings = createMemo(() => findingsForGroup(props.code));

  /* The lossy-reparse refusal (#582), held HERE because it is not a finding:
     the scoreboard tallies the engine's report and the UI never decides how
     bad, so the refusal lives beside the button that was clicked and nowhere
     near the findings list. Any change to the delivery clears it — a note
     about a commit that did not happen has nothing to say about one that
     did. This wiring has no test at any altitude, and the honest statement
     is the coverage (#582's brief): a refusal cannot be driven end to end
     since #574, and the unit lane cannot import the store this reads. The
     tested half is the guard itself, in the pure model. */
  const [fixRefused, setFixRefused] = createSignal(false);
  createEffect(
    on(
      delivery,
      () => {
        setFixRefused(false);
      },
      { defer: true },
    ),
  );

  return (
    <Show
      when={bits()}
      fallback={
        <>
          <GroupStub
            code={props.code}
            band={props.band}
            onRestore={() => {
              restoreGroup(props.code);
            }}
          />
          <Show when={coarsePointer()}>
            <FindingsStrip code={props.code} findings={groupFindings()} />
          </Show>
        </>
      }
    >
      {(b) => (
        <>
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
            fixCount={groupFixCount(props.code)}
            fixRefused={fixRefused()}
            onFix={() => {
              setFixRefused(false);
              void applyGroupFixes(props.code).then((outcome) => {
                if (outcome === "refused") setFixRefused(true);
              });
            }}
          />

          {/* The carousel is the COARSE pointer's editor (#525); on a fine
              pointer the pick is a spreadsheet selection and opening a tray
              under the table would double the editing surface. It mounts
              directly under the table, ABOVE the strip (#634): the tray
              appears where the tap happened, and the findings it may
              generate stay below it — last in the column, it opened off
              screen. */}
          <Presence when={coarsePointer() ? open() : null}>
            {(cell) => (
              <RowCarousel
                schema={b().schema}
                data={b().data}
                band={props.band}
                row={cell().row}
                col={cell().col}
                onMove={(col) => {
                  setPicked({ group: props.code, row: cell().row, col });
                }}
                onClose={() => {
                  setPicked(null);
                }}
                onDelete={() => {
                  deleteRow(props.code, cell().row);
                }}
              />
            )}
          </Presence>

          {/* The strip is the COARSE pointer's surface now (#591): fine
              pointers read the same callouts on the failing cell itself,
              and the panel beside the file stays the one complete list.
              Below the layout breakpoint the strip pages as a one-card
              carousel (#592) — a dress the strip picks itself. */}
          <Show when={coarsePointer()}>
            <FindingsStrip code={props.code} findings={groupFindings()} />
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
            {/* The toolbar button, danger-repainted (#593) — the ghost's
                transparent border made this verb read as a caption. */}
            <Button
              variant="default"
              tone="danger"
              aria-label={`Delete the ${props.code} group`}
              onClick={() => {
                deleteGroup(props.code);
              }}
            >
              delete group
            </Button>
            <span class="text-caption text-fg-faint">
              {coarsePointer()
                ? "Tap any cell to edit the row."
                : "Click a cell, then type. Enter commits, Esc cancels."}
            </span>
          </div>
        </>
      )}
    </Show>
  );
};
