/* The page's rhythm and its band keying (#395, #396, #399).
 *
 * `bandVar` is arithmetic over the ramp rather than a lookup table, which is
 * what lets #400's dark ramp shift every band by redefining seven CSS variables
 * and nothing else. That arithmetic is worth pinning: an off-by-one here would
 * silently recolour every group chip, table cap and KEY edge on the page, and
 * nothing else in the repo would notice.
 */

import { describe, expect, it } from "vitest";
import { SECTIONS, bandVar, groupBandVar } from "./sections";

describe("SECTIONS", () => {
  it("is the seven-band sequence the rail descends", () => {
    expect(SECTIONS.map((s) => s.id)).toEqual([
      "top",
      "proj",
      "loca",
      "samp",
      "llpl",
      "file",
      "install",
    ]);
  });

  it("draws the four groups in dictionary chain order", () => {
    expect(SECTIONS.filter((s) => s.group).map((s) => s.group)).toEqual([
      "PROJ",
      "LOCA",
      "SAMP",
      "LLPL",
    ]);
  });
});

describe("bandVar", () => {
  it("walks the ramp one step per section", () => {
    expect(bandVar(0)).toBe("--laterite-300");
    expect(bandVar(6)).toBe("--laterite-900");
  });

  it("never runs off the end of the ramp", () => {
    // The ramp stops at 900 for the rail's seven bands. A section added without
    // extending the ramp would ask for --laterite-1000, which resolves to
    // nothing and paints a transparent band.
    const last = bandVar(SECTIONS.length - 1);
    expect(last).toBe("--laterite-900");
  });
});

describe("groupBandVar", () => {
  it("gives each group the band of its own section", () => {
    // #396 names these by hex: PROJ #db7841, LOCA #ce5640, SAMP #be3b2e,
    // LLPL #9b3932 — which are exactly ramp steps 400/500/600/700. The two sets
    // agreeing is the point: a group's colour IS its depth.
    expect(groupBandVar("PROJ")).toBe("--laterite-400");
    expect(groupBandVar("LOCA")).toBe("--laterite-500");
    expect(groupBandVar("SAMP")).toBe("--laterite-600");
    expect(groupBandVar("LLPL")).toBe("--laterite-700");
  });

  it("returns nothing for a group the page does not draw", () => {
    // The guard that matters: band colour encodes group identity and nothing
    // else, so a caller must not be able to colour an arbitrary group with a
    // band it has no claim to. Falling back to a band would do exactly that.
    expect(groupBandVar("GEOL")).toBe("");
    expect(groupBandVar("")).toBe("");
  });
});
