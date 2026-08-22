/* The restore stub (#529): what a deleted group leaves in the table's place.
 *
 * The absence is the lesson — the delivery just lost a block and the engine
 * now says so — but the way back has to be discoverable exactly where the
 * table was, or the only recovery is the global reset. Restore returns the
 * SEED's rows, not the reader's edits: carrying edits through a delete/restore
 * cycle would need shadow state that can rot, so the stub says which contract
 * it offers. Undo is the other verb — it walks the timeline and does bring
 * the edits back — which is why the copy names both.
 */

import type { Component } from "solid-js";
import { Button } from "@shared/components";

export const GroupStub: Component<{
  code: string;
  band: string;
  onRestore: () => void;
}> = (props) => (
  <div
    class="rounded-lg border border-dashed border-line bg-surface p-4 dark:bg-surface-raised"
    style={{ "border-left": `3px solid var(${props.band})` }}
  >
    <p class="font-mono text-caption font-semibold text-fg">
      {props.code} deleted
    </p>
    <p class="mt-1 max-w-[52ch] text-caption text-fg-soft">
      The delivery just lost its {props.code} block — if a rule needs it, the
      findings now say so. Restore brings back the seeded rows, not your edits;
      undo walks the timeline and does.
    </p>
    <div class="mt-3">
      <Button
        variant="outline"
        size="sm"
        aria-label={`Restore ${props.code}`}
        onClick={() => {
          props.onRestore();
        }}
      >
        Restore {props.code}
      </Button>
    </div>
  </div>
);
