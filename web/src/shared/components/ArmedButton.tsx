import {
  createEffect,
  createSignal,
  Show,
  type Component,
  type JSX,
} from "solid-js";
import { Button, type ButtonSize, type ButtonVariant } from "./Button";
import {
  onArmedTrigger,
  type ArmedEvent,
  type ArmedState,
} from "./armedConfirm";

// A destructive action's button (#408). Several tools replace or discard the
// user's loaded file; this is the restraint contract for all of them — arm on
// the first press (danger repaint, the label becomes the question), act on the
// second. No dialog: the button itself asks.
//
// When an exit disarms and why: armedConfirm.ts. This owns the DOM events and
// the label swap; that owns what they mean.

export const ArmedButton: Component<{
  /** The question the second press answers — "Discard all fixes?". */
  confirm: string;
  onConfirm: () => void;
  /** Arm only while true (default). False = nothing to lose yet, so a plain
   *  one-press button. Named for the condition, not the state — `armed`
   *  would collide with what `state()` already calls itself. */
  armWhen?: boolean;
  variant?: ButtonVariant;
  size?: ButtonSize;
  disabled?: boolean;
  class?: string;
  title?: string;
  children: JSX.Element;
}> = (props) => {
  const [state, setState] = createSignal<ArmedState>("idle");

  const trigger = (event: ArmedEvent) => {
    const action = onArmedTrigger(state(), event);
    setState(action.state);
    if (action.fire) props.onConfirm();
  };

  // The guard flipping mid-question (busy, selection emptied, the thing to
  // lose going away) stands it down — a control that re-enables must ask
  // again, never fire from stale intent.
  createEffect(() => {
    if (props.disabled || !(props.armWhen ?? true)) trigger("disable");
  });

  return (
    <Button
      variant={props.variant}
      size={props.size}
      disabled={props.disabled}
      title={props.title}
      class={props.class}
      tone={state() === "armed" ? "danger" : undefined}
      onClick={() => {
        if (props.armWhen ?? true) trigger("press");
        else props.onConfirm();
      }}
      onMouseLeave={() => {
        trigger("pointer-leave");
      }}
      onBlur={() => {
        trigger("blur");
      }}
      onKeyDown={(e) => {
        if (e.key === "Escape") trigger("escape");
      }}
    >
      <Show when={state() === "armed"} fallback={props.children}>
        {props.confirm}
      </Show>
    </Button>
  );
};
