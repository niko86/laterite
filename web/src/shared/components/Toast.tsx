import {
  createEffect,
  createSignal,
  onCleanup,
  Show,
  type Component,
} from "solid-js";
import { Icon } from "./Icon";
import { dismiss, enqueue, visible, type ToastItem } from "./toastQueue";

// Bottom-left toast: deep maroon panel, light text, optional Undo. Flies in
// 12px over --dur-slow. One host, one at a time.
//
// The queueing decisions are in toastQueue.ts — this owns the timers, the
// animation and the hover hold.

/** The panel itself. Exported for the rare caller that owns its own placement. */
export const Toast: Component<{
  message: string;
  onUndo?: () => void;
  onDismiss?: () => void;
}> = (props) => (
  <div
    role="status"
    aria-live="polite"
    class={[
      "inline-flex items-center gap-[0.6rem] max-w-[24rem]",
      "bg-[--laterite-900] text-fg-on-cta border border-white/[0.18]",
      "rounded-md px-[0.75rem] py-[0.5rem] text-control shadow-[--shadow-toast]",
    ].join(" ")}
  >
    <span>{props.message}</span>
    <Show when={props.onUndo}>
      <button
        type="button"
        onClick={() => props.onUndo?.()}
        class="font-semibold text-[--laterite-300] px-[0.3rem] py-[0.15rem] rounded-xs cursor-pointer"
      >
        Undo
      </button>
    </Show>
    <button
      type="button"
      aria-label="Dismiss"
      onClick={() => props.onDismiss?.()}
      class="inline-flex items-center text-fg-on-cta/65 hover:text-fg-on-cta p-[0.2rem] rounded-xs cursor-pointer"
    >
      <Icon name="x" size={13} />
    </button>
  </div>
);

/**
 * The single host. Mount once, near the app root.
 *
 * `--toast-life` is read from CSS rather than restated here so the duration and
 * the motion contract stay in one place; the fallback covers a host mounted
 * before the token layer has parsed.
 */
const TOAST_LIFE_FALLBACK_MS = 4000;

function toastLifeMs(): number {
  if (typeof getComputedStyle !== "function") return TOAST_LIFE_FALLBACK_MS;
  const raw = getComputedStyle(document.documentElement)
    .getPropertyValue("--toast-life")
    .trim();
  const ms = Number.parseFloat(raw);
  return Number.isFinite(ms) && ms > 0 ? ms : TOAST_LIFE_FALLBACK_MS;
}

let nextId = 1;
const [queue, setQueue] = createSignal<ToastItem[]>([]);

/** Raise a toast. Returns its id, so a caller can retract one it raised. */
export function toast(message: string, onUndo?: () => void): number {
  const id = nextId++;
  setQueue((q) => enqueue(q, { id, message, onUndo }));
  return id;
}

export function retractToast(id: number): void {
  setQueue((q) => dismiss(q, id));
}

export const ToastHost: Component = () => {
  const current = () => visible(queue());
  const [held, setHeld] = createSignal(false);
  const [shown, setShown] = createSignal(false);

  createEffect(() => {
    const item = current();
    if (!item) {
      setShown(false);
      return;
    }
    // Re-run when the hold is released, so the remaining life restarts rather
    // than the toast vanishing the instant the pointer leaves.
    const holding = held();
    // A frame's delay so the enter transition has a from-state to animate from.
    const raf = requestAnimationFrame(() => {
      setShown(true);
    });
    if (holding) {
      onCleanup(() => {
        cancelAnimationFrame(raf);
      });
      return;
    }
    const timer = setTimeout(() => {
      retractToast(item.id);
    }, toastLifeMs());
    onCleanup(() => {
      cancelAnimationFrame(raf);
      clearTimeout(timer);
    });
  });

  return (
    <div
      class="fixed bottom-4 left-4 z-[--z-toast]"
      onMouseEnter={() => {
        setHeld(true);
      }}
      onMouseLeave={() => {
        setHeld(false);
      }}
    >
      <Show when={current()}>
        {(item) => (
          <div
            class="transition-[opacity,transform] duration-[--dur-slow] ease-[--ease-out]"
            classList={{
              "opacity-0 translate-y-[12px]": !shown(),
              "opacity-100 translate-y-0": shown(),
            }}
          >
            <Toast
              message={item().message}
              onUndo={item().onUndo}
              onDismiss={() => {
                retractToast(item().id);
              }}
            />
          </div>
        )}
      </Show>
    </div>
  );
};
