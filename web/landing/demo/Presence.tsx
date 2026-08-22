/* Enter/exit presence for the row editor (#534).
 *
 * Solid's `Show` unmounts on the same frame its condition falls, so a closing
 * carousel could never fade — this wrapper holds the last value through an
 * opacity exit and unmounts on `transitionend`. That event is exactly what
 * the motion layer promises to keep firing: reduced motion collapses
 * `--dur-slow` to 0.01ms rather than 0 so sequencing like this skips instead
 * of hanging (motion.css).
 *
 * Enter is `@starting-style` (Tailwind's `starting:` variant), not a
 * flip-a-class-next-frame effect: the first cut of this scheduled the flip
 * with requestAnimationFrame and the carousel could sit at opacity 0 while
 * fully interactive — CSS owns insertion transitions natively, so let it.
 * (Toast.tsx still uses the rAF flip on the shared surface; its context is
 * not this ticket's, so it stays as the app's own call.)
 *
 * The stated duration is deliberate, not a restated default: `--dur-slow` is
 * the system's enter/exit tier (#408).
 */

import { Show, createEffect, createSignal, type JSX } from "solid-js";

export function Presence<T>(props: {
  when: T | null | undefined | false;
  children: (item: () => T) => JSX.Element;
}): JSX.Element {
  const [held, setHeld] = createSignal<T | null>(null);
  const closing = () => !props.when;
  let el: HTMLDivElement | undefined;

  createEffect(() => {
    const v = props.when;
    if (v) {
      setHeld(() => v);
    } else if (el && getComputedStyle(el).opacity === "0") {
      // The no-transition escape hatch, and it is load-bearing: a close that
      // lands before the enter fade has lifted off applies `opacity-0` while
      // computed opacity is already 0 — target equals current, so the browser
      // starts NO transition and neither `transitionend` nor
      // `transitioncancel` will ever fire. Waiting for an event here left the
      // editor mounted forever as an interactable ghost (the touch delete
      // test found it: tap Edit, tap Delete inside the same enter window). An
      // invisible editor owes no exit fade — unmount it now. This runs after
      // the classList render effect, so the computed value already reflects
      // the close.
      setHeld(null);
    }
  });

  const settle = (e: TransitionEvent & { currentTarget: HTMLDivElement }) => {
    // Only our own settled fade-out unmounts — a bubbling transition from a
    // child must not tear the editor down, and neither may an enter fade.
    // transitioncancel is deliberately NOT handled: a retarget (close during
    // enter, reopen-then-close) cancels the old transition and runs a new
    // one, so `transitionend` still arrives — a cancel-unmount would cut the
    // very fade this component exists to play. The event-less close (target
    // already at 0) is the escape hatch's job above, not an event's.
    if (e.target === e.currentTarget && closing()) setHeld(null);
  };

  return (
    <Show when={held()}>
      {(item) => (
        <div
          ref={el}
          class="transition-opacity duration-(--dur-slow) starting:opacity-0"
          classList={{ "opacity-0": closing() }}
          onTransitionEnd={settle}
        >
          {props.children(item)}
        </div>
      )}
    </Show>
  );
}
