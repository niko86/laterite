// `project()`'s two non-finite guards.
//
// A borehole's easting/northing comes from a text cell, so a blank, a dash or a
// typo reaches here as `NaN`. Both guards return `null` so the caller can drop
// the point — and both matter for different reasons:
//
//   * the INPUT guard stops a NaN entering proj4 at all. Without it proj4 is
//     handed NaN and hands something back, and a map marker appears at a
//     position no one can explain;
//   * the OUTPUT guard catches a projection that ran but produced a non-finite
//     result — which is what a coordinate far outside the CRS's valid domain
//     does. A borehole at the wrong end of the country plots silently; one that
//     projects to Infinity must not plot at all.
import proj4 from "proj4";
import { describe, expect, it } from "vitest";

import { applyDefs, project } from "./coords";

// The Helmert definitions are enough for these cases; the precise OSGB variant
// needs the OSTN15 grid file, which is a fetch this unit test has no business
// making.
applyDefs(proj4, false);

describe("project", () => {
  // Official OS test points (the same TP vectors coords.test.ts checks the
  // rigorous OSTN15 grid against). Here the Helmert definitions are in force, so
  // the tolerance is the ~5 m that approximation is documented to cost — enough
  // to prove the projection is wired up without re-testing proj4's arithmetic.
  const OS_POINTS = [
    { e: 91492.146, n: 11318.804, lat: 49.9222639373, lon: -6.29977752014 },
    { e: 422242.186, n: 433818.701, lat: 53.8002151963, lon: -1.66379168242 },
    { e: 395999.668, n: 1138728.951, lat: 60.1330809166, lon: -2.07382822798 },
  ];

  it("projects the official OS test points to within the Helmert tolerance", () => {
    // The control. Without a case that succeeds, "returns null" proves nothing.
    for (const p of OS_POINTS) {
      const out = project(proj4, "osgb", p.e, p.n);
      expect(out).not.toBeNull();
      const [lon, lat] = out!;
      const metres = Math.hypot(
        (lat - p.lat) * 111_320,
        (lon - p.lon) * 111_320 * Math.cos((p.lat * Math.PI) / 180),
      );
      expect(metres).toBeLessThan(10);
    }
  });

  it("projects an Irish grid reference into Ireland", () => {
    const out = project(proj4, "irish", 315904, 234671);
    expect(out).not.toBeNull();
    const [lon, lat] = out!;
    expect(lat).toBeGreaterThan(51);
    expect(lat).toBeLessThan(56);
    expect(lon).toBeLessThan(-5);
    expect(lon).toBeGreaterThan(-11);
  });

  it("refuses a non-finite easting or northing before projecting", () => {
    // `Number("")` and `Number("—")` are both NaN, and both are ordinary things
    // to find in a LOCA_NATE cell.
    expect(project(proj4, "osgb", Number.NaN, 623009)).toBeNull();
    expect(project(proj4, "osgb", 429157, Number.NaN)).toBeNull();
    expect(project(proj4, "osgb", Number.NaN, Number.NaN)).toBeNull();
  });

  it("refuses an infinite input", () => {
    expect(project(proj4, "osgb", Number.POSITIVE_INFINITY, 623009)).toBeNull();
    expect(project(proj4, "osgb", 429157, Number.NEGATIVE_INFINITY)).toBeNull();
  });

  it("refuses a projection whose OUTPUT is not finite", () => {
    // The second guard, and the one the INPUT guard cannot stand in for: 1e300
    // is a perfectly finite number, so it walks straight past the first check.
    // The transform then runs and hands back something unusable, and returning
    // that would put a marker at an undefined position rather than dropping the
    // point.
    expect(Number.isFinite(1e300)).toBe(true);
    expect(project(proj4, "osgb", 1e300, 1e300)).toBeNull();
    expect(project(proj4, "osgb", 1e12, -1e12)).toBeNull();
  });

  it("never returns a tuple containing NaN or Infinity", () => {
    // The property every caller depends on: if `project` returns a pair, both
    // numbers are plottable. Asserted across the plausible and the hostile in
    // one pass, so a new failure mode shows up as a changed shape rather than a
    // silently skipped case.
    const inputs: [number, number][] = [
      [429157, 623009], // in Great Britain
      [0, 0], // the grid origin — off the coast, but real
      [-1e6, 1e6], // far outside the grid, still projects
      [1e12, -1e12], // beyond the CRS's domain entirely
    ];
    const out = inputs.map(([e, n]) => project(proj4, "osgb", e, n));
    expect(
      out.map((r) => r === null || (isFinite(r[0]) && isFinite(r[1]))),
    ).toEqual([true, true, true, true]);
    // Exactly one of them was undroppable-but-unplottable, which is what makes
    // the assertion above more than vacuously true.
    expect(out.filter((r) => r === null)).toHaveLength(1);
  });
});
