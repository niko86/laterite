/* The rail's arithmetic (#399; remapped by #585). jsdom reports every scroll
 * dimension as 0, so a component test here would assert nothing; these are
 * the numbers instead.
 */

import { describe, expect, it } from "vitest";
import {
  RAIL_INSET_PCT,
  bandBounds,
  bandFractions,
  bandStartFraction,
  datumFraction,
  depthAt,
  depthLabel,
  railY,
} from "./railScroll";
import { SECTIONS } from "../sections";

/* A page shaped like the real one: seven contiguous sections stacked below a
 * masthead, a datum one gap-step under the masthead's bottom, and a viewport
 * short enough that the last section's landing line is reachable. Fixture
 * numbers, not measurements — the mapping's PROPERTIES are what these pin. */
const TOPS = [53, 472, 782, 1129, 1514, 1900, 3271];
const HEIGHTS = [419, 310, 347, 385, 385, 1371, 707];
const DATUM = 65;
const MAX = 3238;

/* Fixture reads are in range by construction; the helper answers the
   compiler's noUncheckedIndexedAccess without an assertion. */
const at = (xs: readonly number[], i: number): number => xs[i] ?? 0;

describe("datumFraction — the probe keyed to the datum line (#615)", () => {
  it("is exact at every section's landing line", () => {
    // The whole point of the remap: a jump puts a section top ON the datum,
    // and the probe there must read the section's own tick fraction — the
    // retired document-fraction mapping overshot it (the incident is
    // recorded on datumFraction itself).
    for (let i = 0; i < TOPS.length; i++) {
      const landing = Math.max(0, at(TOPS, i) - DATUM);
      expect(datumFraction(landing, DATUM, TOPS, HEIGHTS, MAX)).toBeCloseTo(
        bandStartFraction(i, HEIGHTS),
        10,
      );
    }
  });

  it("reads 0 at the top of the page", () => {
    // The first section starts ABOVE the datum (the masthead covers it), so
    // its landing clamps to scroll 0 — where the fraction is 0 exactly, not
    // the sliver of band the datum has already descended past.
    expect(datumFraction(0, DATUM, TOPS, HEIGHTS, MAX)).toBe(0);
  });

  it("reaches 1 at max scroll — the stretched tail", () => {
    // The deepest landing line sits well above the page bottom; from there
    // the fraction lerps to 1 so the floor reads the full depth.
    expect(datumFraction(MAX, DATUM, TOPS, HEIGHTS, MAX)).toBe(1);
    const lastLanding = at(TOPS, TOPS.length - 1) - DATUM;
    const midTail = (lastLanding + MAX) / 2;
    const f = datumFraction(midTail, DATUM, TOPS, HEIGHTS, MAX);
    expect(f).toBeGreaterThan(bandStartFraction(TOPS.length - 1, HEIGHTS));
    expect(f).toBeLessThan(1);
  });

  it("reads the tick exactly from a whole pixel away — the landing snap", () => {
    // Browsers land an anchor jump on a whole pixel; the measured geometry
    // is fractional. A pixel of scroll can flip the depth's second decimal,
    // so within one the probe reads the tick — the e2e's EXACT text match
    // rides on this.
    const landing = at(TOPS, 3) - DATUM;
    for (const off of [-1, 1]) {
      expect(datumFraction(landing + off, DATUM, TOPS, HEIGHTS, MAX)).toBe(
        bandStartFraction(3, HEIGHTS),
      );
    }
    // And only from a pixel: two out is a real position, not a landing.
    expect(
      datumFraction(landing + 2, DATUM, TOPS, HEIGHTS, MAX),
    ).toBeGreaterThan(bandStartFraction(3, HEIGHTS));
  });

  it("is monotonic through every section and the tail", () => {
    let prev = -1;
    for (let y = 0; y <= MAX; y += 7) {
      const f = datumFraction(y, DATUM, TOPS, HEIGHTS, MAX);
      expect(f).toBeGreaterThanOrEqual(prev);
      prev = f;
    }
  });

  it("clamps rather than overshooting on elastic scroll", () => {
    // iOS rubber-banding reports a negative scrollY and one past the end.
    expect(datumFraction(-120, DATUM, TOPS, HEIGHTS, MAX)).toBe(0);
    expect(datumFraction(MAX + 999, DATUM, TOPS, HEIGHTS, MAX)).toBe(1);
  });

  it("answers 0 for a page shorter than the viewport", () => {
    // No scrollable height is a division by zero, and the honest answer is
    // the surface — not NaN painting the veil over the whole rail.
    expect(datumFraction(0, DATUM, TOPS, HEIGHTS, 0)).toBe(0);
    expect(datumFraction(0, DATUM, TOPS, HEIGHTS, -400)).toBe(0);
  });

  it("degrades to the plain document fraction before first measure", () => {
    // Unmeasured tops all clamp into the origin knot, leaving a single
    // linear run — the first paint is the retired mapping, not NaN.
    const zeros = SECTIONS.map(() => 0);
    expect(datumFraction(1619, DATUM, zeros, zeros, MAX)).toBeCloseTo(
      1619 / MAX,
    );
  });

  it("drops an unreachable landing into the tail", () => {
    // A landing at or past max scroll can never sit on the datum; its knot
    // would put two fractions at one scroll position. The tail from the
    // last landable knot absorbs it, still monotonic, still 1 at the floor.
    const shortMax = 3100; // below the last section's landing
    const atLast = datumFraction(
      at(TOPS, 5) - DATUM,
      DATUM,
      TOPS,
      HEIGHTS,
      shortMax,
    );
    expect(atLast).toBeCloseTo(bandStartFraction(5, HEIGHTS), 10);
    expect(datumFraction(shortMax, DATUM, TOPS, HEIGHTS, shortMax)).toBe(1);
    let prev = -1;
    for (let y = 0; y <= shortMax; y += 7) {
      const f = datumFraction(y, DATUM, TOPS, HEIGHTS, shortMax);
      expect(f).toBeGreaterThanOrEqual(prev);
      prev = f;
    }
  });

  it("keeps a landing on its tick when a section below the datum grows", () => {
    // Findings arrive and the file section doubles: everything above it
    // lands unmoved, and the probe still reads each tick exactly.
    const grown = [...HEIGHTS.slice(0, 5), at(HEIGHTS, 5) * 2, at(HEIGHTS, 6)];
    const grownTops = [at(TOPS, 0)];
    for (let i = 1; i < grown.length; i++) {
      grownTops.push(at(grownTops, i - 1) + at(grown, i - 1));
    }
    const grownMax = MAX + at(HEIGHTS, 5);
    for (let i = 0; i < grownTops.length; i++) {
      const landing = Math.max(0, at(grownTops, i) - DATUM);
      expect(
        datumFraction(landing, DATUM, grownTops, grown, grownMax),
      ).toBeCloseTo(bandStartFraction(i, grown), 10);
    }
  });
});

describe("depthLabel", () => {
  it("always writes two decimal places — 2DP is a real AGS TYPE", () => {
    expect(depthLabel(0)).toBe("0.00");
    expect(depthLabel(25)).toBe("25.00");
    expect(depthLabel(12.5)).toBe("12.50");
  });

  it("reads 0.00 at the top and the seeded total at the bottom", () => {
    const total = 25;
    expect(
      depthLabel(depthAt(datumFraction(0, DATUM, TOPS, HEIGHTS, MAX), total)),
    ).toBe("0.00");
    expect(
      depthLabel(depthAt(datumFraction(MAX, DATUM, TOPS, HEIGHTS, MAX), total)),
    ).toBe("25.00");
  });
});

describe("railY — the one vertical mapping", () => {
  it("puts the datum at the top edge and keeps the floor off the bottom", () => {
    // #585's ruling, superseding #524's symmetric inset: fraction 0 IS the
    // surface — the wrapper anchors to the masthead bar, so an inset above
    // the datum would move 0.00 m off the very line that defines it. The
    // BOTTOM inset survives for the terminal pill's sake.
    expect(railY(0)).toBe(0);
    expect(railY(1)).toBe(100 - RAIL_INSET_PCT);
  });

  it("still travels monotonically between them", () => {
    expect(railY(0.25)).toBeLessThan(railY(0.75));
  });

  it("does not clamp the DEPTH to keep the pill visible", () => {
    // Decoupled on purpose — a rail that shortened the hole to fit its own
    // pill would lie about how deep the reader is.
    expect(depthLabel(depthAt(1, 25))).toBe("25.00");
    expect(railY(1)).toBeLessThan(100);
  });
});

describe("bandFractions — measured heights become shares of the hole", () => {
  it("is each section's share of the summed heights", () => {
    expect(bandFractions([100, 200, 100])).toEqual([0.25, 0.5, 0.25]);
  });

  it("doubling one section's height doubles its band and shifts what follows", () => {
    // The #585 criterion verbatim — "double one section's height, its band
    // doubles, ticks move" — pinned as the RATIO between bands: a share of
    // a grown total cannot literally double, but the doubled section's band
    // now stands twice as tall as an unchanged peer's, and every later tick
    // sits lower.
    const before = [100, 100, 100, 100];
    const after = [100, 200, 100, 100];
    expect(bandBounds(1, after).height).toBeCloseTo(
      2 * bandBounds(0, after).height,
    );
    expect(bandStartFraction(2, after)).toBeGreaterThan(
      bandStartFraction(2, before),
    );
  });

  it("falls back to equal shares when nothing has measured yet", () => {
    // First paint runs before the ResizeObserver's first delivery; equal
    // bands are the honest placeholder, not NaN.
    expect(bandFractions([0, 0, 0])).toEqual([1 / 3, 1 / 3, 1 / 3]);
  });
});

describe("bandBounds — weighted, contiguous, on the one run", () => {
  const HEIGHTS = [900, 1400, 1100, 1300, 1700, 2600, 800];

  it("tiles contiguously, with no gap and no overlap", () => {
    for (let i = 1; i < HEIGHTS.length; i++) {
      const prev = bandBounds(i - 1, HEIGHTS);
      const here = bandBounds(i, HEIGHTS);
      expect(here.top).toBeCloseTo(prev.top + prev.height);
    }
  });

  it("gives a taller section a taller band, in proportion", () => {
    const short = bandBounds(6, HEIGHTS); // 800
    const tall = bandBounds(5, HEIGHTS); // 2600
    expect(tall.height / short.height).toBeCloseTo(2600 / 800);
  });

  it("reproduces equal bands for equal heights", () => {
    const equal = SECTIONS.map(() => 1000);
    const heights = new Set(
      SECTIONS.map((_, i) => bandBounds(i, equal).height.toFixed(6)),
    );
    expect(heights.size).toBe(1);
  });

  it("positions a tick and the pill identically at the same depth (#524)", () => {
    // The invariant #585's remap must not break: the component places
    // section ticks at band TOPS and the pill at railY(progress) — one run,
    // so a tick's label IS what the pill reads when centred on it.
    for (let i = 0; i < HEIGHTS.length; i++) {
      expect(bandBounds(i, HEIGHTS).top).toBeCloseTo(
        railY(bandStartFraction(i, HEIGHTS)),
      );
    }
    // And the terminal tick: the pill's journey ends where the last band does.
    const last = bandBounds(HEIGHTS.length - 1, HEIGHTS);
    expect(railY(1)).toBeCloseTo(last.top + last.height);
  });
});

describe("the rail and the page agree on the sequence", () => {
  it("has one band per section", () => {
    // #399 specifies seven bands for seven sections. If a section is added and
    // the ramp is not extended, this is where it surfaces.
    expect(SECTIONS).toHaveLength(7);
  });
});
