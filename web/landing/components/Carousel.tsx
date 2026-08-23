/* The one-card presentation (#592, generalised by #595) — what a stack of
 * full-width cards becomes when the viewport cannot afford it. One card
 * shows; swipe, the paging buttons, or RowCarousel's Alt+Arrow idiom move
 * through the rest, wrapping in both directions. Two chromes, each an issue's
 * recorded choice: the findings surfaces read a "2 / 4" counter (#592), the
 * install deck reads position dots (#595).
 *
 * EVERY card stays in the DOM, parked invisible in a one-cell grid stack
 * (#622), and only the current one shows. The stack is the height story:
 * all cards occupy the same cell, so the ul stands as tall as its tallest
 * card and paging never reflows the page below it — display parking sized
 * the deck to whichever card was showing. Visibility parking keeps parked
 * cards out of the accessibility tree and the tab order exactly as display
 * parking did, and the testing contract survives too: the e2e suite's
 * absence assertions ("Rule 14 is gone") count `li`s, and a carousel that
 * mounted only the current card would let a card hide from them on an
 * unshown page. Presence stays countable; visibility is the presentation.
 *
 * The ul keeps a stack's list role and takes the caller's aria-label, so an
 * accessible name like "LLPL findings" survives the change of dress — and the
 * root stays a fragment so the ul's parent is still the caller's column,
 * which the strip-shares-a-column e2e contract reads.
 *
 * One card renders bare: no counter, no dots, no buttons, and no swipe
 * surface either — the pointer handlers disarm and `touch-pan-y` stands
 * down, so a lone card constrains no gesture it cannot answer.
 */

import { Index, Show, createSignal, untrack, type JSX } from "solid-js";
import { Button } from "@shared/components";
import { clampIndex, stepIndex, swipeStep } from "../carousel";

export function Carousel<T>(props: {
  label: string;
  items: readonly T[];
  /** The card body — the caller's own flavour (a finding callout, an install
   *  card), never decided here. */
  card: (item: () => T) => JSX.Element;
  /** Which position chrome the caller's issue recorded. */
  chrome: "counter" | "dots";
  /** What one card IS, for the controls' accessible names — "finding" on the
   *  findings surfaces, "card" on the install deck. */
  noun: string;
  /** Spacing utilities for the ul — the stack being replaced owns the rhythm. */
  class?: string;
}): JSX.Element {
  const [raw, setRaw] = createSignal(0);
  /* Which side the incoming card enters from — forward slides in from the
     right, back from the left. Motion only; reduced motion never reads it. */
  const [dir, setDir] = createSignal<1 | -1>(1);
  const count = () => props.items.length;
  /* Clamped on read, not by an effect: the list can shrink under a live
     position any time, and a derived position is never stale. */
  const at = () => clampIndex(raw(), count());
  /* A reversal is TWO renders, not one. The parked nudge is rendered
     state, and a CSS transition starts from the last COMPUTED style — so
     flipping direction and position in the same recalc would slide the
     entering card in from the PREVIOUS turn's side. Frame one renders the
     new pose, frame two turns (two rAFs: the first runs before the recalc
     it needs to get past). The target is computed when it fires, so every
     call still lands exactly one turn and rapid input settles where a
     synchronous turn would have. */
  const turn = (d: 1 | -1, target: () => number) => {
    const flipped = d !== dir();
    setDir(d);
    if (!flipped) {
      setRaw(target());
      return;
    }
    requestAnimationFrame(() => {
      requestAnimationFrame(() => setRaw(target()));
    });
  };
  const go = (delta: number) => {
    /* untrack: the target reads position state at FIRE time by design —
       a deferred turn must step from wherever the deck stands when it
       lands, never subscribe to it. */
    turn(delta > 0 ? 1 : -1, () =>
      untrack(() => stepIndex(at(), delta, count())),
    );
  };
  const jumpTo = (i: number) => {
    if (i === at()) return;
    turn(i > at() ? 1 : -1, () => untrack(() => clampIndex(i, count())));
  };

  /* The swipe is a straight read of two pointer events — down records where
     the finger landed, up measures how far it travelled. `touch-pan-y` leaves
     vertical scrolling with the browser, so only a deliberately horizontal
     drag ever reaches the threshold. */
  let startX: number | null = null;
  /* A mouse fires `click` after ANY down/up pair on a common ancestor — a
     drag has no browser-side threshold — so a swipe that starts and ends on
     a control inside the card (a copy button, a link that focuses a file
     line) would also activate it. A page turn must not drag that along. */
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
        class={`grid list-none p-0 ${props.class ?? "mt-3"}`}
        classList={{ "touch-pan-y": count() > 1 }}
        onKeyDown={onKeys}
        /* Capture phase, so the swallow runs BEFORE the card's own click
           handlers — a bubble listener would arrive after they had already
           fired. Attached by ref: capture has no home in the delegated JSX
           handlers. */
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
        {/* Index (#534): re-renders mint fresh items, so only a genuine
            page turn re-fires the entrance. The entrance is the parked
            pose released: a parked card waits transparent, nudged toward
            the side it would enter from (`turn` re-renders the nudge
            before a reversal, so the side is this turn's, not the last's),
            and unparking transitions it into place — while visibility,
            deliberately NOT in the transition list, flips at once,
            vanishing the outgoing card the way display parking did. Rides
            motion-safe; reduced motion gets the swap with no transition
            at all. */}
        <Index each={props.items}>
          {(item, i) => (
            <li
              class="col-start-1 row-start-1"
              classList={{
                /* The transition rides the ACTIVE card only: a parked
                   card's pose must SNAP — on a reversal the nudge flips
                   sides, and a parked card that eased between them would
                   still be mid-flip when the turn released it. */
                "motion-safe:transition-[opacity,translate] motion-safe:duration-(--dur-fast)":
                  i === at(),
                "invisible opacity-0": i !== at(),
                "translate-x-4": i !== at() && dir() === 1,
                "-translate-x-4": i !== at() && dir() === -1,
              }}
            >
              {props.card(item)}
            </li>
          )}
        </Index>
      </ul>
      <Show when={count() > 1}>
        {/* Centred under the card (#622): the row spans the column, so
            without it the chrome hugged the left edge under centred cards. */}
        <div
          class="mt-2 flex items-center justify-center gap-2"
          onKeyDown={onKeys}
        >
          <Button
            variant="default"
            size="sm"
            aria-label={`Previous ${props.noun}`}
            onClick={() => {
              go(-1);
            }}
          >
            ‹
          </Button>
          <Show
            when={props.chrome === "dots"}
            fallback={
              <span class="font-mono text-micro text-fg-muted tabular-nums">
                {at() + 1} / {count()}
              </span>
            }
          >
            <span class="flex items-center gap-1.5">
              <Index each={props.items}>
                {(_, i) => (
                  <button
                    type="button"
                    aria-label={`Go to ${props.noun} ${i + 1}`}
                    aria-current={i === at() || undefined}
                    class="size-2.5 rounded-full border border-line"
                    classList={{
                      "bg-accent": i === at(),
                      "bg-transparent": i !== at(),
                    }}
                    onClick={() => {
                      jumpTo(i);
                    }}
                  />
                )}
              </Index>
            </span>
          </Show>
          <Button
            variant="default"
            size="sm"
            aria-label={`Next ${props.noun}`}
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
}
