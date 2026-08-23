/* The one-card presentation (#592, generalised by #595) — what a stack of
 * full-width cards becomes when the viewport cannot afford it. One card
 * shows; swipe, the paging buttons, or RowCarousel's Alt+Arrow idiom move
 * through the rest, wrapping in both directions. Two chromes, each an issue's
 * recorded choice: the findings surfaces read a "2 / 4" counter (#592), the
 * install deck reads position dots (#595).
 *
 * EVERY card stays in the DOM, parked with `hidden`, and only the current one
 * shows. That is a testing contract as much as a rendering one: the e2e
 * suite's absence assertions ("Rule 14 is gone") count `li`s, and a carousel
 * that mounted only the current card would let a card hide from them on an
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

import { Index, Show, createSignal, type JSX } from "solid-js";
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
  const go = (delta: number) => {
    setDir(delta > 0 ? 1 : -1);
    setRaw(stepIndex(at(), delta, count()));
  };
  const jumpTo = (i: number) => {
    if (i === at()) return;
    setDir(i > at() ? 1 : -1);
    setRaw(clampIndex(i, count()));
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
        class={`list-none p-0 ${props.class ?? "mt-3"}`}
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
        {/* Index (#534): re-renders mint fresh items, and only a genuine page
            turn should re-fire the entrance. The slide-and-fade rides
            motion-safe, so reduced motion gets the swap with no transition
            at all. */}
        <Index each={props.items}>
          {(item, i) => (
            <li
              hidden={i !== at()}
              class="motion-safe:transition-[opacity,translate] motion-safe:duration-(--dur-fast) motion-safe:starting:opacity-0"
              classList={{
                "motion-safe:starting:translate-x-4": dir() === 1,
                "motion-safe:starting:-translate-x-4": dir() === -1,
              }}
            >
              {props.card(item)}
            </li>
          )}
        </Index>
      </ul>
      <Show when={count() > 1}>
        <div class="mt-2 flex items-center gap-2" onKeyDown={onKeys}>
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
