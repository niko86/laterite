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

/** The inset the whole rail runs inside, in percentage points at each end, so
 *  the depth pill is never clipped against the top or bottom edge. */
export const RAIL_INSET_PCT = 4;

/** The rail's ONE vertical mapping: a 0–1 fraction of the hole to a percentage
 *  of the strip. Bands, ticks, veil, probe and pill all position through it —
 *  #524 was two mappings (ticks on a plain 0–100 run, the pill on the inset
 *  run), so the pill's number never matched the label beside it.
 *
 * The inset applies to POSITION only while depth still runs a true 0–total.
 * The two are deliberately decoupled: clamping the DEPTH to keep the pill on
 * screen would make the rail lie about how deep the reader is. */
export function railY(fraction: number, inset = RAIL_INSET_PCT): number {
  return inset + fraction * (100 - inset * 2);
}

/** The band a section occupies, top and height, as percentages of the strip —
 *  on the shared run, so a band top IS its tick's position. Equal bands: the
 *  rail marks the sequence of sections, not their pixel heights, so a long
 *  section does not get a fatter stratum. */
export function bandBounds(
  index: number,
  count: number,
): { top: number; height: number } {
  const height = (100 - RAIL_INSET_PCT * 2) / count;
  return { top: railY(index / count), height };
}
