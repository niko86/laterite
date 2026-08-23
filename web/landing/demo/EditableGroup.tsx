/* The editable group harness (#549): one group's table and every control that
 * edits it.
 *
 * Extracted because the four descent sections and the TRAN cover sheet were
 * maintaining the same wiring twice — the same store calls, the same two
 * narrowings, the same coarse-pointer guard — and the copy grew by a shape
 * every ticket that touched it (#527 seeded it, #529 added the restore stub
 * and the delete button, #530 the per-table fix budget).
 *
 * TRAN was a strict SUBSET of a descent group, not a parallel implementation:
 * everything it rendered, GroupSection rendered too, with the same props in
 * the same order. So the two affordances it lacks are named boolean props
 * rather than a children slot — there are exactly two, both fixed per caller,
 * and "+ row" on a one-row transmission header is meaningless rather than
 * merely unwanted. A slot would buy flexibility nothing is asking for.
 *
 * LAYOUT STAYS WITH THE CALLERS. This emits the table, the strip, the actions
 * row and the carousel; the grid column GroupSection places them in and the
 * `mt-3` block the cover sheet places them in belong to those callers, and
 * normalising the difference would be a visual change the extraction has no
 * business making.
 *
 * Sharing the harness did NOT make TRAN an eighth descent section (#527). It
 * keeps its own prose and its own place beside the file it covers, and it adds
 * nothing to the strata ramp — whose length is `web/landing/sections.test.ts`'s
 * to hold, not this comment's to restate.
 */

import { Show, createMemo, type Component } from "solid-js";
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
  /** The "+ row" affordance. Off for the cover sheet: TRAN is the delivery's
   *  one transmission header, so a second row is not a thing a reader can
   *  mean. */
  canAddRow?: boolean;
  /** The line that names the editor the reader actually has (#525). Off for
   *  the cover sheet, which sits below four tables that have already said it. */
  showEditHint?: boolean;
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
          <FindingsStrip code={props.code} findings={groupFindings()} />
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
            onFix={() => {
              void applyGroupFixes(props.code);
            }}
          />

          <FindingsStrip code={props.code} findings={groupFindings()} />

          {/* A ROW only when there is a row's worth of controls. With one
              control the box stays block, which is what the cover sheet had
              before this extraction — and the two are not the same height: an
              inline-flex Button in a block box is sized by the line-box strut,
              which at this button's size is the taller of the two. Flexing it
              would have lifted everything below the cover sheet's table by a
              couple of pixels, and #549's contract is that neither caller's
              spacing moves. */}
          <div
            class="mt-3"
            classList={{
              "flex flex-wrap items-center gap-3":
                props.canAddRow || props.showEditHint,
            }}
          >
            <Show when={props.canAddRow}>
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
            </Show>
            <Button
              variant="ghost"
              size="sm"
              tone="danger"
              aria-label={`Delete the ${props.code} group`}
              onClick={() => {
                deleteGroup(props.code);
              }}
            >
              delete group
            </Button>
            <Show when={props.showEditHint}>
              <span class="text-caption text-fg-faint">
                {coarsePointer()
                  ? "Tap any cell to edit the row."
                  : "Click a cell, then type. Enter commits, Esc cancels."}
              </span>
            </Show>
          </div>

          {/* The carousel is the COARSE pointer's editor (#525); on a fine
              pointer the pick is a spreadsheet selection and opening a tray
              under the table would double the editing surface. */}
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
        </>
      )}
    </Show>
  );
};
