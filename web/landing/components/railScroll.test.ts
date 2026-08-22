/* The rail's arithmetic (#399). jsdom reports every scroll dimension as 0, so
 * a component test here would assert nothing; these are the numbers instead.
 */

import { describe, expect, it } from "vitest";
import {
  RAIL_INSET_PCT,
  bandBounds,
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
  it("keeps the pill on screen at both extremes", () => {
    // The failure this prevents: a pill clipped in half against the top edge.
    expect(railY(0)).toBe(RAIL_INSET_PCT);
    expect(railY(1)).toBe(100 - RAIL_INSET_PCT);
  });

  it("still travels monotonically between them", () => {
    expect(railY(0.5)).toBeCloseTo(50);
    expect(railY(0.25)).toBeLessThan(railY(0.75));
  });

  it("does not clamp the DEPTH to keep the pill visible", () => {
    // Decoupled on purpose — a rail that shortened the hole to fit its own
    // pill would lie about how deep the reader is.
    expect(depthLabel(depthAt(1, 25))).toBe("25.00");
    expect(railY(1)).toBeLessThan(100);
  });

  it("positions a tick and the pill identically at the same depth (#524)", () => {
    // The defect this pins, stated CROSS-function because that is where it
    // lived: the component places section ticks at band TOPS (bandBounds) and
    // the pill at railY(progress), and pre-#524 those were two different runs
    // — bandBounds on a plain 0-100%, the pill inset — so the pill's number
    // never matched the label beside it. Against that arithmetic this fails
    // on every band but the first; two identical calls to one function would
    // prove nothing.
    const n = SECTIONS.length;
    for (let i = 0; i < n; i++) {
      const tickY = bandBounds(i, n).top;
      const pillYAtTickDepth = railY(i / n);
      expect(tickY).toBeCloseTo(pillYAtTickDepth);
    }
    // And the terminal tick: the pill's journey ends where the last band does.
    const last = bandBounds(n - 1, n);
    expect(railY(1)).toBeCloseTo(last.top + last.height);
  });
});

describe("bandBounds", () => {
  it("tiles contiguously, with no gap and no overlap", () => {
    const bands = SECTIONS.map((_, i) => bandBounds(i, SECTIONS.length));
    for (let i = 1; i < bands.length; i++) {
      const prev = bands[i - 1];
      const here = bands[i];
      if (!prev || !here) throw new Error("bandBounds returned a hole");
      expect(here.top).toBeCloseTo(prev.top + prev.height);
    }
  });

  it("gives every section an equal band, whatever its pixel height", () => {
    const heights = new Set(
      SECTIONS.map((_, i) => bandBounds(i, SECTIONS.length).height),
    );
    expect(heights.size).toBe(1);
  });
});

describe("the rail and the page agree on the sequence", () => {
  it("has one band per section", () => {
    // #399 specifies seven bands for seven sections. If a section is added and
    // the ramp is not extended, this is where it surfaces.
    expect(SECTIONS).toHaveLength(7);
  });
});
