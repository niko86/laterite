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
 * review reversed it, and #585 is the record.
 *
 * One OUTPUT run, one INPUT line (#615): pill, ticks, veil and probe all
 * position through railY, which is what keeps a tick's label equal to the
 * pill's reading at the same height (the #524 invariant) — and the probe's
 * fraction is keyed to the DATUM line, the section under it (railScroll's
 * datumFraction), not the document's scroll fraction. #585 accepted the
 * drift between those two domains as ornament; the owner's pass-2 review
 * reversed that — a jump landed the pill visibly past its own tick — and
 * #615 is the record. The datum itself is ONE token, `--datum-offset` (the
 * masthead's measured bottom, written to `--surface-y` by the observer
 * below, plus the `--datum-gap` spacing step): the scale wrapper's top, the
 * sections' scroll-margins and the mapping all consume it, so where a jump
 * lands, where 0.00 m sits and what the probe reads cannot drift apart.
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
  datumFraction,
  depthAt,
  depthLabel,
  railY,
} from "./railScroll";

export const Rail: Component = () => {
  const [progress, setProgress] = createSignal(0);
  /* One entry per section, all zero until the observer's first delivery —
     which bandFractions answers with equal shares, not NaN. */
  const [heights, setHeights] = createSignal<number[]>(SECTIONS.map(() => 0));
  const total = seededFinalDepth();

  onMount(() => {
    const sections = SECTIONS.map((s) => document.getElementById(s.id));
    const header = document.querySelector("header");
    /* The mapping's inputs that are not signals: nothing in the JSX reads
       them, so they stay plain values the frame handler closes over. */
    let tops = SECTIONS.map(() => 0);
    let datumOffset = 0;
    let queued = false;
    const read = () => {
      queued = false;
      setProgress(
        datumFraction(
          window.scrollY,
          datumOffset,
          tops,
          heights(),
          document.documentElement.scrollHeight - window.innerHeight,
        ),
      );
    };
    const onScroll = () => {
      if (queued) return;
      queued = true;
      requestAnimationFrame(read);
    };

    /* The sections and the masthead re-measure themselves: findings arrive,
       tables grow rows, the viewport turns — each delivery re-weighs the
       bands through the same pure mapping the unit lane pins, and re-reads
       the probe, because a section growing ABOVE the datum moves the
       fraction without any scroll happening.

       The masthead's height lands in `--surface-y` on the root, and the
       datum the mapping keys to is read BACK from a section's resolved
       scroll-margin — the same `--datum-offset` the jumps and the scale
       wrapper consume — so the mapping cannot disagree with where a jump
       actually lands (#615; the retired static `scroll-mt-16` disagreed
       with the masthead this observer measures, which was exactly the
       drift — landing.css carries the incident). */
    const measure = () => {
      setHeights(sections.map((el) => el?.offsetHeight ?? 0));
      if (header) {
        document.documentElement.style.setProperty(
          "--surface-y",
          `${header.getBoundingClientRect().height}px`,
        );
      }
      tops = sections.map((el) =>
        el ? el.getBoundingClientRect().top + window.scrollY : 0,
      );
      const first = sections.find((el) => el);
      if (first) {
        datumOffset = parseFloat(getComputedStyle(first).scrollMarginTop) || 0;
      }
      onScroll();
    };
    const observer = new ResizeObserver(measure);
    for (const el of sections) if (el) observer.observe(el);
    if (header) observer.observe(header);
    measure();
    window.addEventListener("scroll", onScroll, { passive: true });
    window.addEventListener("resize", onScroll, { passive: true });

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
      {/* Anchored to the datum line — the masthead's measured bottom plus
          the datum-gap step (#615, restoring D2-01's air under the gradient
          bar) — so railY(0) lands 0.00 m exactly where a jump lands a
          section top. The token is the anchor: `--surface-y` is written by
          the observer above, and the same `--datum-offset` calc drives the
          sections' scroll-margins. Off the bottom so the terminal tick is
          not flush to the edge. OFFSETS, not padding: every child is
          absolutely positioned, and an absolute child anchors to its
          containing block's padding box (#524). */}
      <div class="absolute inset-x-0 top-(--datum-offset) bottom-6">
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
