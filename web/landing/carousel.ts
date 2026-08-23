/* The finding carousel's paging arithmetic (#592), pure per
 * dec-web-test-altitude: the wrap and the swipe threshold are the decisions,
 * lifted out of the component so the unit lane can hold them.
 */

/** One page forward or back, wrapping in both directions — the carousel is a
 *  loop, not a walled track. */
export function stepIndex(
  current: number,
  delta: number,
  length: number,
): number {
  if (length <= 0) return 0;
  return (((current + delta) % length) + length) % length;
}

/** A live position against a list that revalidation can shrink under it: a
 *  stranded index lands on the last real card, never off the end. */
export function clampIndex(current: number, length: number): number {
  if (length <= 0) return 0;
  return Math.max(0, Math.min(current, length - 1));
}

/* Under this, a drag is a tap that wandered: paging on it would make every
 * imprecise tap on the card a navigation. */
const SWIPE_MIN_PX = 40;

/** The paging verb a horizontal drag means: leftward reads as forward (the
 *  next card slides in from the right), rightward as back, and anything
 *  shorter than a real swipe as nothing. */
export function swipeStep(dx: number): -1 | 0 | 1 {
  if (dx <= -SWIPE_MIN_PX) return 1;
  if (dx >= SWIPE_MIN_PX) return -1;
  return 0;
}
