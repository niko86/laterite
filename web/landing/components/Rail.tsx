/* The borehole rail (#399) — scrolling the page as descending a borehole.
 *
 * A fixed left gutter carrying the strata strip, a veil that uncovers as the
 * reader descends, the steel probe, the depth pill and the depth scale. It is
 * the page's signature and the reason the section rhythm exists.
 *
 * ## Bands weigh what they mark (#585)
 *
 * Each band's height is its section's MEASURED share of the page — a
 * ResizeObserver per section feeds the pure mapping (railScroll.ts) — so
 * depth maps onto scroll distance and a long section gets the fat stratum it
 * costs to scroll. #524 recorded equal bands as a design choice; the owner's
 * review reversed it, and #585 is the record. The datum moved with it: the
 * scale wrapper anchors to the measured bottom of the masthead — the page's
 * gradient "surface" — so 0.00 m sits level with the bar instead of at an
 * arbitrary inset.
 *
 * One OUTPUT run, two inputs, honestly: pill, ticks, veil and probe all
 * position through railY, which is what keeps a tick's label equal to the
 * pill's reading at the same height (the #524 invariant). The probe's input
 * is the document's scroll fraction while band tops ride section-height
 * shares — two domains that agree exactly at the surface and the floor and
 * drift a little between (the masthead, the footer and the viewport
 * subtraction all live in one and not the other). Mapping scroll into
 * section-space would trade that small drift for a probe that jumps at
 * section boundaries; the drift is the better ornament.
 *
 * ## Mostly decorative, on purpose
 *
 * The strip, veil, probe, pill and depth numbers are `aria-hidden`: every
 * NAME on the rail is already a heading in the document, and the depth is an
 * ornament. The section LABELS are the #585 exception — real links that
 * scroll to their sections, so the rail is a working elevator panel for
 * pointer and keyboard alike. They are the only things on the rail in the
 * accessibility tree, and the only things that take pointer events.
 *
 * ## Motion
 *
 * A passive scroll listener and nothing else. The rail has no self-running
 * animation, so a reduced-motion preference needs no special case — it
 * responds only to a scroll the reader is already performing, and the label
 * links ride the document's own scroll-behavior rule (#589): smooth when
 * motion is welcome, instant when it is not. Reads are batched into an
 * animation frame so a fast scroll cannot queue a layout read per event.
 *
 * ## Narrow widths
 *
 * Below the collapse breakpoint it becomes an 8px strip at the page edge with
 * the probe and the veil and no depth scale — inheriting the weighted mapping
 * and nothing else (#585's one out-of-scope line). At 8px the bands read as
 * stripes rather than strata and the reveal is easy to miss — that trade is
 * acknowledged rather than solved.
 */

import {
  createMemo,
  createSignal,
  For,
  onCleanup,
  onMount,
  type Component,
} from "solid-js";
import { SECTIONS, bandVar } from "../sections";
import { seededFinalDepth } from "../demo/delivery";
import {
  bandBounds,
  depthAt,
  depthLabel,
  railY,
  scrollProgress,
} from "./railScroll";

export const Rail: Component = () => {
  const [progress, setProgress] = createSignal(0);
  /* One entry per section, all zero until the observer's first delivery —
     which bandFractions answers with equal shares, not NaN. */
  const [heights, setHeights] = createSignal<number[]>(SECTIONS.map(() => 0));
  /* Where the page's surface sits: the measured bottom of the sticky
     masthead, whose gradient bar is the 0.00 m line (#585's datum ruling).
     The initial 56 is only the first paint's stand-in — the retired `top-14`
     inset it replaces, held one frame until the observer's first delivery
     measures the real chrome. */
  const [surfaceY, setSurfaceY] = createSignal(56);
  const total = seededFinalDepth();

  onMount(() => {
    let queued = false;
    const read = () => {
      queued = false;
      setProgress(
        scrollProgress(
          window.scrollY,
          window.innerHeight,
          document.documentElement.scrollHeight,
        ),
      );
    };
    const onScroll = () => {
      if (queued) return;
      queued = true;
      requestAnimationFrame(read);
    };
    read();
    window.addEventListener("scroll", onScroll, { passive: true });
    window.addEventListener("resize", onScroll, { passive: true });

    /* The sections and the masthead re-measure themselves: findings arrive,
       tables grow rows, the viewport turns — each delivery re-weighs the
       bands through the same pure mapping the unit lane pins. */
    const sections = SECTIONS.map((s) => document.getElementById(s.id));
    const header = document.querySelector("header");
    const measure = () => {
      setHeights(sections.map((el) => el?.offsetHeight ?? 0));
      if (header) setSurfaceY(header.getBoundingClientRect().height);
    };
    const observer = new ResizeObserver(measure);
    for (const el of sections) if (el) observer.observe(el);
    if (header) observer.observe(header);
    measure();

    onCleanup(() => {
      window.removeEventListener("scroll", onScroll);
      window.removeEventListener("resize", onScroll);
      observer.disconnect();
    });
  });

  const probe = () => railY(progress());
  const bounds = (i: number) => bandBounds(i, heights());

  return (
    <div class="pointer-events-none fixed top-0 bottom-0 left-0 z-20 w-2 min-[68rem]:w-24 min-[68rem]:border-r min-[68rem]:border-line min-[68rem]:bg-surface">
      {/* Anchored to the masthead's measured bottom — the gradient bar IS the
          surface, so railY(0) lands 0.00 m exactly on it — and off the bottom
          so the terminal tick is not flush to the edge. OFFSETS, not padding:
          every child is absolutely positioned, and an absolute child anchors
          to its containing block's padding box (#524). */}
      <div
        class="absolute inset-x-0 bottom-6"
        style={{ top: `${surfaceY()}px` }}
      >
        {/* The strip: seven bands, top to bottom, weighted by their sections. */}
        <div
          aria-hidden="true"
          class="absolute inset-y-0 left-0 w-2 overflow-hidden min-[68rem]:left-6 min-[68rem]:w-[26px] min-[68rem]:rounded-sm"
        >
          <For each={SECTIONS}>
            {(_section, i) => {
              const b = createMemo(() => bounds(i()));
              return (
                <div
                  class="absolute inset-x-0"
                  style={{
                    top: `${b().top}%`,
                    height: `${b().height}%`,
                    background: `var(${bandVar(i())})`,
                  }}
                />
              );
            }}
          </For>

          {/* The veil: everything BELOW the probe is still covered, so the
              strata uncover as the reader descends. */}
          <div
            class="absolute inset-x-0 bottom-0 border-t-2 border-t-steel-500 bg-canvas/85"
            style={{ top: `${probe()}%` }}
          />
        </div>

        {/* The probe — a steel line spanning the rail, above the fill. */}
        <div
          aria-hidden="true"
          class="absolute inset-x-0 h-[2px] bg-steel-700"
          style={{ top: `${probe()}%` }}
        />

        {/* The depth pill rides the strip. Depth and nothing else: no unit and
            no group name, because the scale beside it carries both.

            One `text-surface` covers both themes and replaces the light/dark
            foreground pair this used to carry: the fill inverts across themes
            (maroon `accent` in light, sand `laterite-300` in dark) and so does
            `--surface`, so the on-fill text lands light-on-dark and then
            dark-on-light without a `dark:` variant. The #404 mechanism the
            shared Chip records. */}
        <div
          aria-hidden="true"
          class="absolute hidden -translate-y-1/2 rounded-pill bg-accent px-2 py-0.5 font-mono text-[0.62rem] text-surface min-[68rem]:block dark:bg-laterite-300"
          style={{ top: `${probe()}%`, left: "6px" }}
        >
          {depthLabel(depthAt(progress(), total))}
        </div>

        {/* The depth scale, clear of the pill's widest state. */}
        <div class="absolute inset-y-0 left-[56px] hidden w-[44px] min-[68rem]:block">
          <For each={SECTIONS}>
            {(section, i) => {
              const b = createMemo(() => bounds(i()));
              return (
                <div class="absolute inset-x-0" style={{ top: `${b().top}%` }}>
                  <div aria-hidden="true" class="h-px w-3 bg-line-strong" />
                  <p
                    aria-hidden="true"
                    class="mt-0.5 font-mono text-[0.58rem] leading-tight text-fg-faint"
                  >
                    {depthLabel(depthAt(b().start, total))} m
                  </p>
                  {/* The one interactive thing on the rail (#585): the label
                      is a door to its section, keyboard and pointer alike.
                      The aria-label says what the door DOES — a bare "PROJ"
                      names a place, not an action. */}
                  <a
                    href={`#${section.id}`}
                    aria-label={`Jump to ${section.label}`}
                    class="pointer-events-auto block font-mono text-[0.58rem] leading-tight text-fg-muted no-underline transition-colors hover:text-accent focus-visible:outline-hidden focus-visible:[box-shadow:var(--focus-ring)]"
                  >
                    {section.label}
                  </a>
                </div>
              );
            }}
          </For>

          {/* The terminal tick: the hole has a floor, and the scale says so.
              Section ticks mark section TOPS, so without this the deepest
              label was the last section's top and the bottom of the hole went
              unlabelled (#524). Depth only — there is no section down here. */}
          <div
            aria-hidden="true"
            class="absolute inset-x-0"
            style={{ top: `${railY(1)}%` }}
          >
            <div class="h-px w-3 bg-line-strong" />
            <p class="mt-0.5 font-mono text-[0.58rem] leading-tight text-fg-faint">
              {depthLabel(total)} m
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};
