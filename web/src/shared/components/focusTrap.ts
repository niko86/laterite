/**
 * The keyboard contract of a modal, as arithmetic.
 *
 * A focus trap is mostly DOM plumbing — query the focusable children, listen
 * for Tab, call `.focus()`. The part that is actually easy to get wrong is the
 * index maths at the two ends of the list, and that part needs no DOM at all.
 * Keeping it here means the wrapping, the shift-Tab direction and the
 * focus-escaped-the-dialog case are pinned by tests rather than by opening a
 * dialog and pressing Tab a lot.
 *
 * The selector lives here too, so "what counts as focusable" has one answer.
 */

/** Focusable, in DOM order, excluding anything explicitly removed from the tab order. */
export const FOCUSABLE_SELECTOR = [
  "a[href]",
  "button:not([disabled])",
  "input:not([disabled])",
  "select:not([disabled])",
  "textarea:not([disabled])",
  '[tabindex]:not([tabindex="-1"])',
].join(",");

/**
 * Where Tab should land next.
 *
 * `from` is the index of the currently focused element, or -1 when focus is
 * somewhere outside the dialog entirely — which happens when the trap is first
 * armed, and after a click on the scrim. Both ends wrap, because a modal that
 * lets Tab walk out into the page behind it is not modal.
 *
 * Returns -1 for an empty dialog: there is nothing to focus, and the caller
 * should leave focus where it is rather than blur to `document.body`.
 */
export function nextFocusIndex(
  count: number,
  from: number,
  shift: boolean,
): number {
  if (count <= 0) return -1;
  // Focus outside the trap: Tab enters at the top, shift-Tab at the bottom.
  if (from < 0 || from >= count) return shift ? count - 1 : 0;
  return shift ? (from - 1 + count) % count : (from + 1) % count;
}

/**
 * Whether a keydown should dismiss the dialog.
 *
 * Escape only, and only when it is not modified — Escape is also how a reader
 * cancels an IME composition and how some browsers exit full-screen, and a
 * modified Escape is not a dismissal anyone intended.
 */
export function isDismissKey(key: string, modified: boolean): boolean {
  return key === "Escape" && !modified;
}
