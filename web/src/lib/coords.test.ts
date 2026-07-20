import { describe, expect, it } from "vitest";
import { existsSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";
import proj4 from "proj4";
import {
  CRS,
  GRID_FILE,
  OS_ATTRIBUTION,
  applyDefs,
  project,
  registerOstn15,
  toGeoJson,
  transformLabel,
  type ConvertedPoint,
} from "./coords";

const pt = (
  id: string,
  e: string,
  n: string,
  lon: number | null,
  lat: number | null,
): ConvertedPoint => ({ id, e, n, lon, lat });

describe("transformLabel", () => {
  it("names OSTN15 only when precise AND supported", () => {
    expect(transformLabel("osgb", true)).toMatch(/OSTN15/);
    expect(transformLabel("osgb", false)).toMatch(/Helmert/);
    // Irish grid has no OSTN15 — precise=true must still report Helmert.
    expect(transformLabel("irish", true)).toMatch(/Helmert/);
  });
});

describe("CRS defs", () => {
  it("OSGB carries both a Helmert and an OSTN15 nadgrids variant", () => {
    expect(CRS.osgb.helmert).toContain("+towgs84=");
    expect(CRS.osgb.nadgrids).toContain("+nadgrids=OSTN15");
    expect(CRS.osgb.precise).toBe(true);
  });
  it("Irish grid is Helmert-only", () => {
    expect(CRS.irish.nadgrids).toBeUndefined();
    expect(CRS.irish.precise).toBe(false);
  });
});

describe("toGeoJson", () => {
  const rows = [
    pt("BH01", "400000", "300000", -1.2345678912, 52.123456789),
    pt("BH02", "401000", "301000", -1.22, 52.13),
    pt("BAD", "x", "y", null, null), // failed conversion → dropped
  ];

  it("emits an RFC 7946 FeatureCollection with [lon, lat] order", () => {
    const fc = JSON.parse(toGeoJson(rows, { system: "osgb", precise: true }));
    expect(fc.type).toBe("FeatureCollection");
    // The null-conversion row is dropped (a Feature needs a geometry).
    expect(fc.features).toHaveLength(2);
    const f = fc.features[0];
    expect(f.type).toBe("Feature");
    expect(f.geometry.type).toBe("Point");
    // GeoJSON is x,y → [longitude, latitude].
    expect(f.geometry.coordinates[0]).toBeCloseTo(-1.23456789, 6);
    expect(f.geometry.coordinates[1]).toBeCloseTo(52.12345679, 6);
    expect(f.properties).toMatchObject({
      LOCA_ID: "BH01",
      easting: "400000",
      northing: "300000",
    });
  });

  it("rounds coordinates to 8 dp (drops spurious precision)", () => {
    const fc = JSON.parse(toGeoJson(rows, { system: "osgb", precise: true }));
    // -1.2345678912 → -1.23456789
    expect(fc.features[0].geometry.coordinates[0]).toBe(-1.23456789);
  });

  it("includes the OS attribution only when OSTN15 was actually used", () => {
    const precise = JSON.parse(
      toGeoJson(rows, { system: "osgb", precise: true }),
    );
    expect(precise.metadata.attribution).toBe(OS_ATTRIBUTION);
    expect(precise.metadata.transform).toMatch(/OSTN15/);

    const helmert = JSON.parse(
      toGeoJson(rows, { system: "osgb", precise: false }),
    );
    expect(helmert.metadata.attribution).toBeUndefined();
    expect(helmert.metadata.transform).toMatch(/Helmert/);

    // Irish + precise requested still degrades to Helmert (no grid) → no attr.
    const irish = JSON.parse(
      toGeoJson(rows, { system: "irish", precise: true }),
    );
    expect(irish.metadata.attribution).toBeUndefined();
  });
});

// Engine-consistency guard: prove the proj4 +nadgrids path reproduces Ordnance
// Survey's *own* published OSTN15 results to sub-metre. Skips when the ~14.5 MB
// grid binary isn't checked out (a lean CI clone) — same pattern as the Python
// suite skipping the absent large.ags fixture.
//
// The reference points below are a spread of OS's 40 official test vectors
// (OSTN15_TestInput/Output_OSGBtoETRS), reproduced under the OSI BSD Licence —
// Contains OS data © Crown copyright and database rights, Ordnance Survey 2016.
const OS_VECTORS: { e: number; n: number; lat: number; lon: number }[] = [
  { e: 91492.146, n: 11318.804, lat: 49.9222639373, lon: -6.29977752014 }, // TP01
  { e: 241124.584, n: 220332.641, lat: 51.858908964, lon: -4.3085247696 }, // TP10
  { e: 422242.186, n: 433818.701, lat: 53.8002151963, lon: -1.66379168242 }, // TP20
  { e: 267056.768, n: 846176.972, lat: 57.4862500072, lon: -4.21926398555 }, // TP30
  { e: 395999.668, n: 1138728.951, lat: 60.1330809166, lon: -2.07382822798 }, // TP40
  { e: 639821.835, n: 169565.858, lat: 51.3744702555, lon: 1.44454730409 }, // TP07 (east)
];

const here = path.dirname(fileURLToPath(import.meta.url));
const gridPath = path.join(here, "..", "..", "public", "grids", GRID_FILE);
const hasGrid = existsSync(gridPath);

describe.skipIf(!hasGrid)("OSTN15 transform vs OS official vectors", () => {
  it("reproduces all OS test points to sub-metre", () => {
    const buf = readFileSync(gridPath);
    const ab = buf.buffer.slice(
      buf.byteOffset,
      buf.byteOffset + buf.byteLength,
    );
    registerOstn15(proj4, ab);
    applyDefs(proj4, true);

    let maxErr = 0;
    for (const v of OS_VECTORS) {
      const ll = project(proj4, "osgb", v.e, v.n);
      expect(ll).not.toBeNull();
      const [lon, lat] = ll!;
      const dlat = (lat - v.lat) * 111_320;
      const dlon = (lon - v.lon) * 111_320 * Math.cos((v.lat * Math.PI) / 180);
      maxErr = Math.max(maxErr, Math.hypot(dlat, dlon));
    }
    // OS publishes to 11 dp; our crude deg→m conversion adds a little noise.
    // Anything under 10 cm proves the rigorous grid is wired correctly (the
    // ~5 m Helmert fallback would blow this assertion apart).
    expect(maxErr).toBeLessThan(0.1);
  });
});
