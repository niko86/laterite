/* The one-card findings presentation (#592) — what a stack of callouts
 * becomes below the page's layout breakpoint, where four full-width cards
 * cost most of a phone screen. One card shows; swipe, the paging buttons,
 * RowCarousel's Alt+Arrow idiom or the counter's arithmetic (carousel.ts)
 * move through the rest, wrapping in both directions.
 *
 * EVERY card stays in the DOM, parked with `hidden`, and only the current one
 * shows. That is a testing contract as much as a rendering one: the e2e
 * suite's absence assertions ("Rule 14 is gone") count `li`s, and a carousel
 * that mounted only the current card would let a finding hide from them on an
 * unshown page. Presence stays countable; visibility is the presentation.
 *
 * The ul keeps the stack's list role and aria-label, so the strip's
 * accessible name ("LLPL findings") survives the change of dress — and the
 * root stays a fragment so the ul's parent is still the caller's column,
 * which the strip-shares-a-column e2e contract reads.
 *
 * One finding renders as a bare card: no counter, no buttons, and no swipe
 * surface either — the pointer handlers disarm and `touch-pan-y` stands
 * down, so a lone card constrains no gesture it cannot answer.
 */

import { Index, Show, createSignal, type Component, type JSX } from "solid-js";
import { Button } from "@shared/components";
import { clampIndex, stepIndex, swipeStep } from "./carousel";
import type { Finding } from "./engine";

export const FindingsCarousel: Component<{
  label: string;
  findings: readonly Finding[];
  /** The card body — the caller's callout flavour (the strip's manual badge,
   *  the panel's group chip and click-to-focus), never decided here. */
  card: (f: () => Finding) => JSX.Element;
}> = (props) => {
  const [raw, setRaw] = createSignal(0);
  /* Which side the incoming card enters from — forward slides in from the
     right, back from the left. Motion only; reduced motion never reads it. */
  const [dir, setDir] = createSignal<1 | -1>(1);
  const count = () => props.findings.length;
  /* Clamped on read, not by an effect: revalidation can shrink the list any
     time, and a derived position is never stale. */
  const at = () => clampIndex(raw(), count());
  const go = (delta: number) => {
    setDir(delta > 0 ? 1 : -1);
    setRaw(stepIndex(at(), delta, count()));
  };

  /* The swipe is a straight read of two pointer events — down records where
     the finger landed, up measures how far it travelled. `touch-pan-y` leaves
     vertical scrolling with the browser, so only a deliberately horizontal
     drag ever reaches the threshold. */
  let startX: number | null = null;
  /* A mouse fires `click` after ANY down/up pair on a common ancestor — a
     drag has no browser-side threshold — so a swipe that starts and ends on
     the card's own button would also click it. The panel's cards focus a
     file line on click; a page turn must not drag that along. */
  let swallowClick = false;

  /* RowCarousel's keyboard idiom, carried over: plain arrows stay with
     whatever is focused, Alt+Arrow pages. On the ul AND the controls row,
     so it answers from a focused card or a focused button alike. */
  const onKeys = (e: KeyboardEvent) => {
    if (count() < 2 || !e.altKey) return;
    if (e.key === "ArrowLeft") go(-1);
    if (e.key === "ArrowRight") go(1);
  };

  return (
    <Show when={count()}>
      <ul
        aria-label={props.label}
        class="mt-3 list-none p-0"
        classList={{ "touch-pan-y": count() > 1 }}
        onKeyDown={onKeys}
        /* Capture phase, so the swallow runs BEFORE the card button's own
           click handler — a bubble listener would arrive after the line had
           already been focused. Attached by ref: capture has no home in the
           delegated JSX handlers. */
        ref={(el) => {
          el.addEventListener(
            "click",
            (e) => {
              if (!swallowClick) return;
              swallowClick = false;
              e.preventDefault();
              e.stopPropagation();
            },
            true,
          );
        }}
        onPointerDown={(e) => {
          swallowClick = false;
          if (count() > 1) startX = e.clientX;
        }}
        onPointerCancel={() => {
          startX = null;
        }}
        onPointerUp={(e) => {
          if (startX === null) return;
          const delta = swipeStep(e.clientX - startX);
          startX = null;
          if (delta !== 0) {
            swallowClick = true;
            go(delta);
          }
        }}
      >
        {/* Index for the panel's reason (#534): revalidation mints fresh
            finding objects, and only a genuine page turn should re-fire the
            entrance. The slide-and-fade rides motion-safe, so reduced motion
            gets the swap with no transition at all. */}
        <Index each={props.findings}>
          {(f, i) => (
            <li
              hidden={i !== at()}
              class="motion-safe:transition-[opacity,translate] motion-safe:duration-(--dur-fast) motion-safe:starting:opacity-0"
              classList={{
                "motion-safe:starting:translate-x-4": dir() === 1,
                "motion-safe:starting:-translate-x-4": dir() === -1,
              }}
            >
              {props.card(f)}
            </li>
          )}
        </Index>
      </ul>
      <Show when={count() > 1}>
        <div class="mt-2 flex items-center gap-2" onKeyDown={onKeys}>
          <Button
            variant="default"
            size="sm"
            aria-label="Previous finding"
            onClick={() => {
              go(-1);
            }}
          >
            ‹
          </Button>
          <span class="font-mono text-micro text-fg-muted tabular-nums">
            {at() + 1} / {count()}
          </span>
          <Button
            variant="default"
            size="sm"
            aria-label="Next finding"
            onClick={() => {
              go(1);
            }}
          >
            ›
          </Button>
        </div>
      </Show>
    </Show>
  );
};
