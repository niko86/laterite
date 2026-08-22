/* The undo stack (#525): a bounded snapshot history over an immutable value.
 *
 * The delivery is ONE immutable signal, which is what makes undo this small —
 * a snapshot is a reference, not a copy, so the stack costs pointers. Pure
 * data over arrays with the present value held OUTSIDE (in the store's
 * signal): `record` is called with the state a mutation is about to replace,
 * and undo/redo hand back the state to show plus the stack that remembers the
 * road not taken.
 */

export type History<T> = {
  /** States that have been replaced, oldest first. */
  readonly past: readonly T[];
  /** States undone away from, nearest first. */
  readonly future: readonly T[];
  /** The open coalescing run, if any — see `record` (#550). */
  readonly key: string | null;
};

export const EMPTY: History<never> = { past: [], future: [], key: null };

/** A mutation happened: remember what it replaced. One timeline — an edit
 *  after an undo abandons the redo branch, the way every editor does it.
 *
 *  A `key` names a coalescing RUN (#550): consecutive records sharing one
 *  key keep only the run's base snapshot, so a word typed into a carousel
 *  card is one undo step, not one per keystroke. Any differently-keyed or
 *  unkeyed record breaks the run, and undo/redo end it below — an edit after
 *  an undo must record fresh, or the abandoned branch's base would swallow
 *  it. */
export function record<T>(
  h: History<T>,
  replaced: T,
  cap = 100,
  key: string | null = null,
): History<T> {
  // Mid-run the cap needs no re-check: `past` is reused unchanged, so the
  // bound set when the run's base was pushed is preserved, never re-verified.
  if (key !== null && key === h.key) return { past: h.past, future: [], key };
  return { past: [...h.past, replaced].slice(-cap), future: [], key };
}

export function undo<T>(
  h: History<T>,
  present: T,
): { history: History<T>; present: T } | null {
  const target = h.past.at(-1);
  if (target === undefined) return null;
  return {
    history: {
      past: h.past.slice(0, -1),
      future: [present, ...h.future],
      key: null,
    },
    present: target,
  };
}

export function redo<T>(
  h: History<T>,
  present: T,
): { history: History<T>; present: T } | null {
  const [target, ...rest] = h.future;
  if (target === undefined) return null;
  return {
    history: { past: [...h.past, present], future: rest, key: null },
    present: target,
  };
}
