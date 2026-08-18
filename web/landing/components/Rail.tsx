/* The borehole rail (#399) — scrolling the page as descending a borehole.
 *
 * A fixed left gutter carrying the strata strip, a veil that uncovers as the
 * reader descends, the steel probe, the depth pill and the depth scale. It is
 * the page's signature and the reason the section rhythm exists.
 *
 * ## Decorative, on purpose
 *
 * `aria-hidden`. Every label on it — the group codes, the section names — is
 * already a heading in the document, and announcing them twice makes the page
 * worse for a screen reader while adding nothing. The depth is an ornament, not
 * information: nothing on the page depends on knowing you are 12.50 m down.
 *
 * ## Motion
 *
 * A passive scroll listener and nothing else. The rail has no self-running
 * animation, so a reduced-motion preference needs no special case — it responds
 * only to a scroll the reader is already performing. Reads are batched into an
 * animation frame so a fast scroll cannot queue a layout read per event.
 *
 * ## Narrow widths
 *
 * Below the collapse breakpoint it becomes an 8px strip at the page edge with
 * the probe and the veil and no depth scale. At 8px the bands read as stripes
 * rather than strata and the reveal is easy to miss — that trade is acknowledged
 * rather than solved. It costs nothing and takes no layout, which is why it
 * survives instead of the rail simply vanishing.
 */

import {
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
  probeOffsetPct,
  scrollProgress,
} from "./railScroll";

export const Rail: Component = () => {
  const [progress, setProgress] = createSignal(0);
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
    onCleanup(() => {
      window.removeEventListener("scroll", onScroll);
      window.removeEventListener("resize", onScroll);
    });
  });

  const probe = () => probeOffsetPct(progress());

  return (
    <div
      aria-hidden="true"
      class="pointer-events-none fixed top-0 bottom-0 left-0 z-20 w-2 min-[68rem]:w-24 min-[68rem]:border-r min-[68rem]:border-line min-[68rem]:bg-surface"
    >
      {/* Inset below the masthead so the first tick is not hidden behind it,
          and off the bottom so the last one is not flush to the edge. */}
      <div class="relative h-full pt-14 pb-6">
        {/* The strip: seven equal bands, top to bottom, one per section. */}
        <div class="absolute inset-y-0 left-0 w-2 overflow-hidden min-[68rem]:left-6 min-[68rem]:w-[26px] min-[68rem]:rounded-sm">
          <For each={SECTIONS}>
            {(_section, i) => (
              <div
                class="absolute inset-x-0"
                style={{
                  top: `${bandBounds(i(), SECTIONS.length).top}%`,
                  height: `${bandBounds(i(), SECTIONS.length).height}%`,
                  background: `var(${bandVar(i())})`,
                }}
              />
            )}
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
          class="absolute inset-x-0 h-[2px] bg-steel-700"
          style={{ top: `${probe()}%` }}
        />

        {/* The depth pill rides the strip. Depth and nothing else: no unit and
            no group name, because the scale beside it carries both. */}
        <div
          class="absolute hidden -translate-y-1/2 rounded-pill bg-accent px-2 py-0.5 font-mono text-[0.62rem] text-stone-50 min-[68rem]:block dark:bg-laterite-300 dark:text-stone-950"
          style={{ top: `${probe()}%`, left: "6px" }}
        >
          {depthLabel(depthAt(progress(), total))}
        </div>

        {/* The depth scale, clear of the pill's widest state. */}
        <div class="absolute inset-y-0 left-[56px] hidden w-[44px] min-[68rem]:block">
          <For each={SECTIONS}>
            {(section, i) => (
              <div
                class="absolute inset-x-0"
                style={{ top: `${bandBounds(i(), SECTIONS.length).top}%` }}
              >
                <div class="h-px w-3 bg-line-strong" />
                <p class="mt-0.5 font-mono text-[0.58rem] leading-tight text-fg-faint">
                  {depthLabel((i() / SECTIONS.length) * total)} m
                </p>
                <p class="font-mono text-[0.58rem] leading-tight text-fg-muted">
                  {section.label}
                </p>
              </div>
            )}
          </For>
        </div>
      </div>
    </div>
  );
};
