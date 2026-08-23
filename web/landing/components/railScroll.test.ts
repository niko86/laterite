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
  depthAt,
  depthLabel,
  railY,
  scrollProgress,
} from "./railScroll";
import { SECTIONS } from "../sections";

describe("scrollProgress", () => {
  it("is 0 at the top and 1 at the bottom", () => {
    expect(scrollProgress(0, 800, 4000)).toBe(0);
    expect(scrollProgress(3200, 800, 4000)).toBe(1);
  });

  it("is linear in between", () => {
    expect(scrollProgress(1600, 800, 4000)).toBeCloseTo(0.5);
  });

  it("clamps rather than overshooting on elastic scroll", () => {
    // iOS rubber-banding reports a negative scrollY and one past the end.
    expect(scrollProgress(-120, 800, 4000)).toBe(0);
    expect(scrollProgress(9999, 800, 4000)).toBe(1);
  });

  it("answers 0 for a page shorter than the viewport", () => {
    // The division by zero. NaN here would paint the veil over the whole rail.
    expect(scrollProgress(0, 800, 800)).toBe(0);
    expect(scrollProgress(0, 800, 400)).toBe(0);
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
    expect(depthLabel(depthAt(scrollProgress(0, 800, 4000), total))).toBe(
      "0.00",
    );
    expect(depthLabel(depthAt(scrollProgress(3200, 800, 4000), total))).toBe(
      "25.00",
    );
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
