/**
 * When a tooltip should appear, and after how long.
 *
 * The system specifies a uniform 300ms delay, in place of the native `title`
 * delay which each browser picks for itself. The delay exists so a pointer
 * crossing a toolbar does not strobe every control it passes over — which means
 * the cancel path matters as much as the show path, and that is the half a
 * render test would not reach.
 *
 * KEYBOARD FOCUS IS NOT DELAYED, and that is a deliberate departure from
 * reading the rule literally. The delay's whole job is to suppress tooltips the
 * reader did not ask for; a pointer sweeps across controls incidentally, but
 * focus lands on exactly one control because someone tabbed to it. Making a
 * keyboard user wait 300ms for the label of the control they deliberately
 * selected adds nothing and costs them the only label they have.
 */

/** The delay's single definition; the CSS token `--tooltip-delay` mirrors it. */
export const TOOLTIP_DELAY_MS = 300;

export type TooltipTrigger =
  "pointer-enter" | "pointer-leave" | "focus" | "blur";

export type TooltipAction =
  /** Start (or restart) the delay, then show. */
  | { kind: "delay"; ms: number }
  /** Show now — no waiting. */
  | { kind: "show" }
  /** Hide, and cancel any delay still pending. */
  | { kind: "hide" };

export function onTooltipTrigger(trigger: TooltipTrigger): TooltipAction {
  switch (trigger) {
    case "pointer-enter":
      return { kind: "delay", ms: TOOLTIP_DELAY_MS };
    case "focus":
      return { kind: "show" };
    // Both exits cancel a pending delay as well as hiding, which is what stops
    // a tooltip appearing 300ms after the pointer has already moved on.
    case "pointer-leave":
    case "blur":
      return { kind: "hide" };
  }
}
