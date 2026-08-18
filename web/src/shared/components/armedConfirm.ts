// The armed-confirm decisions (#408), kept out of the component the way
// tooltipDelay.ts keeps the tooltip's: this owns what a press or an exit
// MEANS, the component owns the DOM it happens to.
//
// The contract is two presses, not a dialog: the first press repaints the
// control to the error colour and swaps its label for the question it is
// asking; the second press acts. Every other event — the pointer leaving,
// focus leaving, Escape, the control going disabled — stands the question
// down without acting, so stale intent can never fire.

export type ArmedState = "idle" | "armed";

export type ArmedEvent =
  "press" | "pointer-leave" | "blur" | "escape" | "disable";

export interface ArmedAction {
  state: ArmedState;
  fire: boolean;
}

export function onArmedTrigger(
  state: ArmedState,
  event: ArmedEvent,
): ArmedAction {
  if (event === "press") {
    return state === "idle"
      ? { state: "armed", fire: false }
      : { state: "idle", fire: true };
  }
  return { state: "idle", fire: false };
}
