import {
  createEffect,
  createSignal,
  onCleanup,
  Show,
  type Component,
  type JSX,
} from "solid-js";
import { createTooltipMachine } from "./tooltipMachine";

// The cell popover (#591): Tooltip's grammar — the same delay policy through
// the same machine — for content richer than a one-line label. Its deltas
// are each a requirement of surfacing finding text at a table cell:
//
// - JSX content instead of a string, so the severity-tinted callout stack
//   rides inside and the popover itself decides nothing about how bad.
// - FIXED positioning, measured from the anchor at open. The cells live in
//   an overflow-x-auto scroller, and specifying overflow-x computes
//   overflow-y to auto — an absolutely-positioned panel above the first row
//   lost its top edge to the scroller's clip box. Fixed escapes the clip,
//   and any scroll while shown re-measures the anchor, so the panel tracks
//   the cell instead of floating detached — dismissal stays the pointer's
//   and the keyboard's, where the reader can see it happen.
// - Escape dismisses even when the anchor is only HOVERED — a document
//   listener alive exactly while shown — but never on the editor's own
//   Escape: that keystroke targets an INPUT and already means "cancel the
//   edit", and closing an unrelated cell's popover with it reads as a
//   glitch.
// - Coarse pointers are ignored outright: a tap is a PICK on these anchors
//   (#525's modality split), and this surface is the strip's replacement
//   for fine pointers only.

export const Popover: Component<{
  /** The floating content. Nothing renders — and no trigger opens — when absent. */
  content?: JSX.Element;
  placement?: "top" | "bottom";
  class?: string;
  children: JSX.Element;
}> = (props) => {
  const machine = createTooltipMachine({
    // Read per TRIGGER, so it cannot go stale the way the mount-time
    // matchMedia reads src/lib/media.ts exists to retire do — an event-time
    // read is fresh by construction, and a reactive signal would buy
    // nothing over it here.
    ignore: () => window.matchMedia("(pointer: coarse)").matches,
  });
  let anchor: HTMLSpanElement | undefined;
  const [at, setAt] = createSignal<{
    x: number;
    top: number;
    bottom: number;
  } | null>(null);

  createEffect(() => {
    if (!machine.shown()) return;
    if (anchor) {
      const r = anchor.getBoundingClientRect();
      setAt({ x: r.left + r.width / 2, top: r.top, bottom: r.bottom });
    }
    const onKey = (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      if (e.target instanceof HTMLElement && e.target.tagName === "INPUT")
        return;
      machine.hide();
    };
    const onScroll = () => {
      if (!anchor) return;
      const r = anchor.getBoundingClientRect();
      setAt({ x: r.left + r.width / 2, top: r.top, bottom: r.bottom });
    };
    document.addEventListener("keydown", onKey);
    document.addEventListener("scroll", onScroll, {
      capture: true,
      passive: true,
    });
    onCleanup(() => {
      document.removeEventListener("keydown", onKey);
      document.removeEventListener("scroll", onScroll, { capture: true });
    });
  });

  return (
    <span
      ref={anchor}
      class={`inline-flex ${props.class ?? ""}`}
      onMouseEnter={() => {
        machine.trigger("pointer-enter");
      }}
      onMouseLeave={() => {
        machine.trigger("pointer-leave");
      }}
      onFocusIn={() => {
        machine.trigger("focus");
      }}
      onFocusOut={() => {
        machine.trigger("blur");
      }}
    >
      {props.children}
      <Show when={machine.shown() && props.content ? at() : null}>
        {(pos) => (
          <span
            role="tooltip"
            class={[
              "fixed -translate-x-1/2 z-(--z-tooltip) w-max max-w-[24rem]",
              "block rounded-md border border-line bg-surface p-1 text-left",
              "shadow-(--shadow-tooltip) space-y-1 dark:bg-surface-raised",
              "pointer-events-none",
            ].join(" ")}
            style={
              (props.placement ?? "top") === "bottom"
                ? { left: `${pos().x}px`, top: `${pos().bottom + 7}px` }
                : {
                    left: `${pos().x}px`,
                    bottom: `calc(100vh - ${pos().top - 7}px)`,
                  }
            }
          >
            {props.content}
          </span>
        )}
      </Show>
    </span>
  );
};
