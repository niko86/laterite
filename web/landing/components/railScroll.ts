/* The rail's arithmetic (#399), extracted from the component.
 *
 * Scroll progress and depth are the only parts of the borehole rail that can be
 * WRONG rather than merely ugly, and neither is testable through a component in
 * the node lane — jsdom reports every scroll dimension as 0. So they live here
 * as pure functions of numbers, and the component does nothing but read the
 * viewport and pass them on.
 *
 * The wink in `depthLabel` is that 2DP is a real AGS TYPE, and the total is the
 * `LOCA_FDEP` the seeded delivery gives BH01 — read from the fixture, not
 * written here, so changing the seeded depth moves the rail with it.
 */

/** Scroll position over scrollable height, clamped to 0–1.
 *
 * A page shorter than its viewport has no scrollable height; that is a division
 * by zero, and the honest answer is 0 (the probe sits at the surface) rather
 * than NaN painting the veil over the whole rail. */
export function scrollProgress(
  scrollY: number,
  viewportHeight: number,
  documentHeight: number,
): number {
  const scrollable = documentHeight - viewportHeight;
  if (scrollable <= 0) return 0;
  return Math.min(1, Math.max(0, scrollY / scrollable));
}

/** Depth in metres at `progress` down a hole of `total` metres. */
export function depthAt(progress: number, total: number): number {
  return progress * total;
}

/** Two decimal places, always — `0.00` at the top and the seeded total at the
 *  bottom. Trailing zeros are the point; `0` would be a different AGS TYPE. */
export function depthLabel(depth: number): string {
  return depth.toFixed(2);
}

/** The inset at the BOTTOM of the run, in percentage points, so the terminal
 *  pill is never clipped against the viewport floor. The top inset retired
 *  with #585: the datum anchors to the masthead bar now, and an inset above
 *  it would move 0.00 m off the very line that defines it. */
export const RAIL_INSET_PCT = 4;

/** The rail's ONE vertical mapping: a 0–1 fraction of the hole to a percentage
 *  of the strip. Bands, ticks, veil, probe and pill all position through it —
 *  #524 was two mappings (ticks on a plain 0–100 run, the pill on the inset
 *  run), so the pill's number never matched the label beside it.
 *
 * The inset applies to POSITION only while depth still runs a true 0–total.
 * The two are deliberately decoupled: clamping the DEPTH to keep the pill on
 * screen would make the rail lie about how deep the reader is. */
export function railY(fraction: number): number {
  return fraction * (100 - RAIL_INSET_PCT);
}

/** Each section's share of the hole, from its MEASURED height (#585,
 *  superseding #524's recorded equal-bands choice): depth now maps onto
 *  scroll distance, so a long section gets the fat stratum it costs to
 *  scroll. Zero-total heights — the first paint, before the ResizeObserver
 *  delivers — fall back to equal shares rather than NaN. */
export function bandFractions(heights: readonly number[]): number[] {
  const total = heights.reduce((a, b) => a + b, 0);
  if (total <= 0) return heights.map(() => 1 / Math.max(1, heights.length));
  return heights.map((h) => h / total);
}

/* The one walker over the shares, so bandStartFraction and bandBounds
   cannot disagree about where a band starts. */
function startFrom(fractions: readonly number[], index: number): number {
  let before = 0;
  for (let i = 0; i < index; i++) before += fractions[i] ?? 0;
  return before;
}

/** Where section `index`'s band starts, as a 0–1 fraction of the hole — the
 *  fraction its depth tick labels, so tick depth and pill depth read off the
 *  same run. */
export function bandStartFraction(
  index: number,
  heights: readonly number[],
): number {
  return startFrom(bandFractions(heights), index);
}

/** The band a section occupies — its start fraction, and top and height as
 *  percentages of the strip — on the shared run, so a band top IS its tick's
 *  position, weighted by the heights the sections really rendered (#585).
 *  `start` rides along so a caller labelling the tick's depth does not walk
 *  the shares a second time. */
export function bandBounds(
  index: number,
  heights: readonly number[],
): { start: number; top: number; height: number } {
  const fractions = bandFractions(heights);
  const start = startFrom(fractions, index);
  return {
    start,
    top: railY(start),
    height: (fractions[index] ?? 0) * (100 - RAIL_INSET_PCT),
  };
}
