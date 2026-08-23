import { Show, type Component, type JSX } from "solid-js";
import { createTooltipMachine } from "./tooltipMachine";

// The icon-control tooltip: a maroon pill after a uniform delay. Native `title`
// delays are browser-controlled and feel inconsistent between them, which is
// the whole reason this exists — long-form field help stays on `title`, where
// the inconsistency does not matter.
//
// Where the pieces live: tooltipDelay.ts owns what should happen and after
// how long; tooltipMachine.ts owns the signal and the timer (shared with
// Popover, #591); this owns the pill.

export const Tooltip: Component<{
  /** The label. Nothing renders when absent, so a conditional tip is safe. */
  tip?: string;
  placement?: "top" | "bottom";
  class?: string;
  children: JSX.Element;
}> = (props) => {
  const { shown, trigger } = createTooltipMachine();

  return (
    <span
      class={`relative inline-flex ${props.class ?? ""}`}
      onMouseEnter={() => {
        trigger("pointer-enter");
      }}
      onMouseLeave={() => {
        trigger("pointer-leave");
      }}
      onFocusIn={() => {
        trigger("focus");
      }}
      onFocusOut={() => {
        trigger("blur");
      }}
    >
      {props.children}
      <Show when={shown() && props.tip}>
        <span
          role="tooltip"
          class={[
            "absolute left-1/2 -translate-x-1/2 z-(--z-tooltip) w-max max-w-[22rem]",
            "bg-(--laterite-900) text-fg-on-cta border border-white/[0.18]",
            "text-caption leading-normal text-left rounded-sm px-[0.5rem] py-[0.22rem]",
            "shadow-(--shadow-tooltip)",
            // Never intercepts the pointer — a tooltip that can be hovered can
            // trap the pointer that summoned it.
            "pointer-events-none",
            (props.placement ?? "top") === "bottom"
              ? "top-[calc(100%+7px)]"
              : "bottom-[calc(100%+7px)]",
          ].join(" ")}
        >
          {props.tip}
        </span>
      </Show>
    </span>
  );
};
