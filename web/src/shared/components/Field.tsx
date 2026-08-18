import { Show, type Component, type JSX } from "solid-js";

// Label-above-control wrapper.
//
// The fixed-height control box is the whole point and the easiest part to drop:
// an input, a select and a checkbox have three different intrinsic heights, so
// in a row of Fields they land on three different baselines. Centring each one
// in a --control-h box lines them up, and putting the hint OUTSIDE that box
// means a Field with a hint doesn't push its own control out of step with the
// Fields beside it.

export const Field: Component<{
  label: JSX.Element;
  /** The quiet line under the control — units, format, a constraint. */
  hint?: JSX.Element;
  class?: string;
  children: JSX.Element;
}> = (props) => (
  <label
    class={`flex flex-col gap-1 text-micro text-fg-muted min-w-0 ${props.class ?? ""}`}
  >
    {props.label}
    <span class="grid content-center min-h-[--control-h] min-w-0">
      {props.children}
    </span>
    <Show when={props.hint}>
      <span class="text-fg-dim">{props.hint}</span>
    </Show>
  </label>
);
