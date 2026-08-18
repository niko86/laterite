/**
 * The toast host's queue.
 *
 * The system's rule is "one host, one at a time" — toasts do not pile into a
 * corner, they wait their turn. That makes the host a queue rather than a list
 * of things to render, and the interesting behaviour is what happens when they
 * arrive faster than they leave.
 *
 * All pure, so the ordering, the cap and the eviction rule are testable without
 * mounting anything. The host owns the timers; this owns the decisions.
 */

export interface ToastItem {
  readonly id: number;
  readonly message: string;
  /** Present when the action is reversible — renders the Undo affordance. */
  readonly onUndo?: () => void;
}

/**
 * Visible one, plus two waiting.
 *
 * A cap is needed at all because a toast is often fired from a loop — "fixed 40
 * cells" can arrive as 40 toasts — and an uncapped queue turns a 4-second
 * notice into three minutes of them. Three is the point where the reader can
 * still believe they saw everything.
 */
export const MAX_QUEUED = 3;

/** The toast the host should be showing: the head, or nothing. */
export function visible(queue: readonly ToastItem[]): ToastItem | undefined {
  return queue[0];
}

/**
 * Add a toast, evicting the OLDEST WAITING one when full.
 *
 * Never the head: that one is on screen and being read, and swapping it out
 * mid-sentence is worse than dropping a message nobody has seen yet. Dropping
 * from the middle rather than refusing the newcomer keeps the most recent
 * outcome — the one the reader just caused — always reachable.
 */
export function enqueue(
  queue: readonly ToastItem[],
  toast: ToastItem,
): ToastItem[] {
  const next = [...queue, toast];
  if (next.length <= MAX_QUEUED) return next;
  // Keep the head, drop the one behind it.
  return [...next.slice(0, 1), ...next.slice(2)];
}

/**
 * Remove a toast by id, whether it is showing or still waiting.
 *
 * By id and not by position because dismissal races the auto-dismiss timer: a
 * reader clicking ✕ as the timer fires would otherwise pop the toast that just
 * became visible, and the message they never saw is the one that vanishes.
 */
export function dismiss(queue: readonly ToastItem[], id: number): ToastItem[] {
  return queue.filter((t) => t.id !== id);
}
