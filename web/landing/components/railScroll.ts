/* The rail's arithmetic (#399), extracted from the component.
 *
 * The probe's mapping and depth are the only parts of the borehole rail that
 * can be WRONG rather than merely ugly, and neither is testable through a
 * component in the node lane — jsdom reports every scroll dimension as 0. So
 * they live here as pure functions of numbers, and the component does nothing
 * but read the viewport and pass them on.
 *
 * The wink in `depthLabel` is that 2DP is a real AGS TYPE, and the total is the
 * `LOCA_FDEP` the seeded delivery gives BH01 — read from the fixture, not
 * written here, so changing the seeded depth moves the rail with it.
 */

/** The probe's fraction of the hole, keyed to the DATUM line (#615).
 *
 * The retired mapping rode the document's scroll fraction while the ticks
 * rode section-height shares — two domains that agreed only at the surface
 * and the floor, so a jump could land the pill 1.37 m past its own tick.
 * This one reads whichever section sits under the datum (the masthead's
 * measured bottom plus the datum-gap token, `datumOffset` in viewport px):
 * each section's LANDING scroll position — where a jump puts its top on the
 * datum — becomes a knot at that section's tick fraction, and the probe
 * interpolates between knots. A landing therefore reads its tick exactly,
 * by construction rather than by coincidence.
 *
 * Two recorded, accepted trades from the #615 trial: the probe changes speed
 * where sections change height, and the run from the last landable knot to
 * `maxScroll` lerps to 1 — the stretched tail that lets the floor read the
 * full depth at page bottom while the deepest landing line sits well above
 * it, compressing the final section into the tail.
 *
 * Degenerates stay honest: an unscrollable page answers 0, unmeasured
 * sections (tops all zero, before the ResizeObserver's first delivery)
 * collapse every knot and leave the plain document fraction, and elastic
 * overscroll clamps rather than painting the veil past the rail. */
export function datumFraction(
  scrollY: number,
  datumOffset: number,
  tops: readonly number[],
  heights: readonly number[],
  maxScroll: number,
): number {
  if (maxScroll <= 0) return 0;
  const fractions = bandFractions(heights);
  /* Knots must strictly increase in BOTH coordinates or the lerp divides by
     zero: a landing clamped into its predecessor (the first section starts
     above the datum; unmeasured tops collapse to 0) folds into the knot
     already there, and a landing at or past maxScroll is unreachable — the
     tail from the last landable knot covers that scroll instead. */
  const knots: [number, number][] = [[0, 0]];
  let lastScroll = 0;
  let lastFraction = 0;
  for (let i = 0; i < tops.length; i++) {
    const landing = (tops[i] ?? 0) - datumOffset;
    /* startFrom, not a parallel accumulation: the knot's fraction and the
       tick's (bandStartFraction) must come from the one walker, or the
       exactness this mapping exists for is two computations agreeing by
       luck. */
    const start = startFrom(fractions, i);
    if (landing > lastScroll && landing < maxScroll && start > lastFraction) {
      knots.push([landing, start]);
      lastScroll = landing;
      lastFraction = start;
    }
  }
  knots.push([maxScroll, 1]);
  const y = Math.min(maxScroll, Math.max(0, scrollY));
  /* The landing snap: browsers land an anchor jump on a whole pixel while
     the measured geometry is fractional, and one pixel of scroll is enough
     to flip the depth's second decimal. Within a pixel of a landing line
     the probe cannot resolve the difference, so it reads the tick exactly —
     which is the landing's DEFINITION, not a fudge. Monotonicity survives:
     the unsnapped fraction a pixel out is within a pixel's slope of the
     knot's. */
  for (const [s, f] of knots) {
    if (Math.abs(y - s) <= 1) return f;
  }
  let prevScroll = 0;
  let prevFraction = 0;
  for (const [s, f] of knots) {
    if (y <= s) {
      if (s === prevScroll) return f;
      return (
        prevFraction +
        ((y - prevScroll) / (s - prevScroll)) * (f - prevFraction)
      );
    }
    prevScroll = s;
    prevFraction = f;
  }
  return 1;
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
