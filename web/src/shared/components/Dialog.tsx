import {
  createEffect,
  onCleanup,
  Show,
  type Component,
  type JSX,
} from "solid-js";
import { Button } from "./Button";
import { FOCUSABLE_SELECTOR, isDismissKey, nextFocusIndex } from "./focusTrap";

// Modal dialog: maroon-tinted scrim (never neutral black, never blurred), 8px
// panel, --shadow-dialog, header row with a ghost ✕.
//
// The system's reference is a presentational shell — it draws the scrim and the
// panel and stops there. A modal that cannot be closed with Escape, lets Tab
// walk out into the page behind it, and never moves focus into itself is not
// modal for anyone not using a mouse, so the keyboard contract is implemented
// here. The arithmetic behind it is in focusTrap.ts, where it is tested.
//
// `fixed`, not the reference's `absolute`: a dialog positioned against whatever
// ancestor happens to be relative will sit inside a scrolled pane instead of
// over the page.

export const Dialog: Component<{
  open?: boolean;
  title: string;
  hint?: JSX.Element;
  /** Panel width; the system's two steps are --dialog-w and --dialog-w-wide. */
  width?: string;
  onClose?: () => void;
  footer?: JSX.Element;
  children?: JSX.Element;
}> = (props) => {
  let panel!: HTMLDivElement;

  // `panel` is assigned by the ref during render, before any effect or key
  // handler here can run.
  const focusable = (): HTMLElement[] => [
    ...panel.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR),
  ];

  const onKeyDown = (e: KeyboardEvent) => {
    if (isDismissKey(e.key, e.altKey || e.ctrlKey || e.metaKey || e.shiftKey)) {
      e.preventDefault();
      props.onClose?.();
      return;
    }
    if (e.key !== "Tab") return;
    const items = focusable();
    const to = nextFocusIndex(
      items.length,
      items.indexOf(document.activeElement as HTMLElement),
      e.shiftKey,
    );
    if (to < 0) return;
    // Always preventDefault once we are trapping, or the browser advances too
    // and focus lands two controls on.
    e.preventDefault();
    items[to]?.focus();
  };

  createEffect(() => {
    if (!props.open) return;
    // Where focus came from, so it can go back — otherwise closing a dialog
    // drops a keyboard user at the top of the document.
    const returnTo = document.activeElement as HTMLElement | null;
    focusable()[0]?.focus();
    document.addEventListener("keydown", onKeyDown);
    onCleanup(() => {
      document.removeEventListener("keydown", onKeyDown);
      returnTo?.focus();
    });
  });

  return (
    <Show when={props.open ?? true}>
      <div
        class="fixed inset-0 z-[--z-dialog] flex items-start justify-center p-8 bg-[--scrim]"
        // Dismissing on a scrim click, but only when the press STARTED there —
        // a drag that begins inside the panel and releases on the scrim (a text
        // selection overshooting) is not a dismissal.
        onMouseDown={(e) => {
          if (e.target === e.currentTarget) props.onClose?.();
        }}
      >
        <div
          ref={panel}
          role="dialog"
          aria-modal="true"
          aria-label={props.title}
          class="bg-surface border border-line rounded-xl shadow-[--shadow-dialog] pt-4 px-5 pb-5"
          style={{ width: `min(${props.width ?? "var(--dialog-w)"}, 100%)` }}
        >
          <div class="flex justify-between items-center gap-[0.8rem] mb-[0.4rem]">
            <strong class="text-title font-semibold text-fg">
              {props.title}
            </strong>
            <Button
              variant="ghost"
              aria-label="Close"
              onClick={() => props.onClose?.()}
            >
              ✕
            </Button>
          </div>
          <Show when={props.hint}>
            <p class="text-caption text-fg-muted leading-normal mt-0 mb-[0.6rem]">
              {props.hint}
            </p>
          </Show>
          <div class="text-body text-fg">{props.children}</div>
          <Show when={props.footer}>
            <div class="flex justify-end gap-[0.4rem] mt-4">{props.footer}</div>
          </Show>
        </div>
      </div>
    </Show>
  );
};
