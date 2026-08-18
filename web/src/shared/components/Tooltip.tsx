import {
  createSignal,
  onCleanup,
  Show,
  type Component,
  type JSX,
} from "solid-js";
import { onTooltipTrigger } from "./tooltipDelay";

// The icon-control tooltip: a maroon pill after a uniform delay. Native `title`
// delays are browser-controlled and feel inconsistent between them, which is
// the whole reason this exists — long-form field help stays on `title`, where
// the inconsistency does not matter.
//
// When the timing decisions live: tooltipDelay.ts. This owns the timer and the
// DOM; that owns what should happen and after how long.

export const Tooltip: Component<{
  /** The label. Nothing renders when absent, so a conditional tip is safe. */
  tip?: string;
  placement?: "top" | "bottom";
  class?: string;
  children: JSX.Element;
}> = (props) => {
  const [shown, setShown] = createSignal(false);
  let timer: ReturnType<typeof setTimeout> | undefined;

  const clear = () => {
    if (timer !== undefined) {
      clearTimeout(timer);
      timer = undefined;
    }
  };
  // A trigger fired, then the control unmounted — without this the timer still
  // fires and sets a signal on a disposed component.
  onCleanup(clear);

  const trigger = (kind: Parameters<typeof onTooltipTrigger>[0]) => {
    const action = onTooltipTrigger(kind);
    clear();
    if (action.kind === "delay") {
      timer = setTimeout(() => {
        setShown(true);
      }, action.ms);
    } else {
      setShown(action.kind === "show");
    }
  };

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
            "absolute left-1/2 -translate-x-1/2 z-[--z-tooltip] w-max max-w-[22rem]",
            "bg-[--laterite-900] text-fg-on-cta border border-white/[0.18]",
            "text-caption leading-normal text-left rounded-sm px-[0.5rem] py-[0.22rem]",
            "shadow-[--shadow-tooltip]",
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
