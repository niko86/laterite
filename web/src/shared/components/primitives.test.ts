import { describe, expect, it, vi } from "vitest";
import { FOCUSABLE_SELECTOR, isDismissKey, nextFocusIndex } from "./focusTrap";
import {
  dismiss,
  enqueue,
  MAX_QUEUED,
  visible,
  type ToastItem,
} from "./toastQueue";
import { onTooltipTrigger, TOOLTIP_DELAY_MS } from "./tooltipDelay";

// The three behaviours #406 calls out by name. Everything else about these
// primitives is a class string, which a test would only restate.

describe("focus trap", () => {
  it("walks forward and wraps at the end", () => {
    expect(nextFocusIndex(3, 0, false)).toBe(1);
    expect(nextFocusIndex(3, 1, false)).toBe(2);
    expect(nextFocusIndex(3, 2, false)).toBe(0);
  });

  it("walks backward and wraps at the start", () => {
    expect(nextFocusIndex(3, 2, true)).toBe(1);
    expect(nextFocusIndex(3, 1, true)).toBe(0);
    expect(nextFocusIndex(3, 0, true)).toBe(2);
  });

  // Arming the trap, and recovering after a click on the scrim: focus is
  // nowhere in the dialog, and Tab has to pull it back in rather than let it
  // continue through the page behind.
  it("pulls focus in from outside, at the end the key implies", () => {
    expect(nextFocusIndex(3, -1, false)).toBe(0);
    expect(nextFocusIndex(3, -1, true)).toBe(2);
  });

  it("treats an out-of-range index as outside", () => {
    expect(nextFocusIndex(3, 9, false)).toBe(0);
    expect(nextFocusIndex(3, 9, true)).toBe(2);
  });

  it("leaves focus alone when there is nothing focusable", () => {
    expect(nextFocusIndex(0, -1, false)).toBe(-1);
    expect(nextFocusIndex(0, 0, true)).toBe(-1);
  });

  it("wraps a single element onto itself rather than escaping", () => {
    expect(nextFocusIndex(1, 0, false)).toBe(0);
    expect(nextFocusIndex(1, 0, true)).toBe(0);
  });

  it("dismisses on a bare Escape only", () => {
    expect(isDismissKey("Escape", false)).toBe(true);
    expect(isDismissKey("Escape", true)).toBe(false);
    expect(isDismissKey("Enter", false)).toBe(false);
    expect(isDismissKey("Esc", false)).toBe(false);
  });

  it("excludes disabled controls and tabindex -1 from the trap", () => {
    expect(FOCUSABLE_SELECTOR).toContain("button:not([disabled])");
    expect(FOCUSABLE_SELECTOR).toContain('[tabindex]:not([tabindex="-1"])');
  });
});

describe("toast queue", () => {
  const t = (id: number): ToastItem => ({ id, message: `m${id}` });

  it("shows the head, and nothing when empty", () => {
    expect(visible([])).toBeUndefined();
    expect(visible([t(1), t(2)])?.id).toBe(1);
  });

  it("queues in arrival order — one at a time, not a pile", () => {
    const q = enqueue(enqueue([], t(1)), t(2));
    expect(q.map((x) => x.id)).toEqual([1, 2]);
    expect(visible(q)?.id).toBe(1);
  });

  // The eviction rule: when a loop fires toasts faster than they expire, the
  // one being read stays, and the newest outcome stays. The stale middle goes.
  it("evicts the oldest WAITING toast when full, never the visible one", () => {
    let q: ToastItem[] = [];
    for (const id of [1, 2, 3, 4]) q = enqueue(q, t(id));
    expect(q).toHaveLength(MAX_QUEUED);
    expect(q.map((x) => x.id)).toEqual([1, 3, 4]);
    expect(visible(q)?.id).toBe(1);
  });

  it("keeps evicting from the middle as more arrive", () => {
    let q: ToastItem[] = [];
    for (const id of [1, 2, 3, 4, 5, 6]) q = enqueue(q, t(id));
    expect(q.map((x) => x.id)).toEqual([1, 5, 6]);
  });

  it("dismisses by id, so a click racing the timer cannot pop the wrong one", () => {
    const q = [t(1), t(2), t(3)];
    expect(dismiss(q, 2).map((x) => x.id)).toEqual([1, 3]);
    // The visible one going reveals the next.
    expect(visible(dismiss(q, 1))?.id).toBe(2);
  });

  it("ignores a dismissal for a toast already gone", () => {
    expect(dismiss([t(1)], 99).map((x) => x.id)).toEqual([1]);
  });

  it("carries the undo affordance only when the action is reversible", () => {
    const undo = vi.fn();
    const q = enqueue([], { id: 1, message: "fixed 40 cells", onUndo: undo });
    visible(q)?.onUndo?.();
    expect(undo).toHaveBeenCalledOnce();
    expect(visible(enqueue([], t(2)))?.onUndo).toBeUndefined();
  });
});

describe("tooltip delay", () => {
  it("delays a pointer, because a pointer crosses controls it did not mean to", () => {
    expect(onTooltipTrigger("pointer-enter")).toEqual({
      kind: "delay",
      ms: TOOLTIP_DELAY_MS,
    });
    expect(TOOLTIP_DELAY_MS).toBe(300);
  });

  // Deliberately not delayed — see tooltipDelay.ts.
  it("shows immediately on keyboard focus", () => {
    expect(onTooltipTrigger("focus")).toEqual({ kind: "show" });
  });

  it("hides on both exits, which is also what cancels a pending delay", () => {
    expect(onTooltipTrigger("pointer-leave")).toEqual({ kind: "hide" });
    expect(onTooltipTrigger("blur")).toEqual({ kind: "hide" });
  });
});
