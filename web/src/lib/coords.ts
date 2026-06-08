// Coordinate transforms for the LOCA grid → WGS84 tool, factored out of the
// component so the projection maths + GeoJSON emit are unit-testable (the
// OSTN15 path is checked against Ordnance Survey's official test vectors).
//
// Two accuracy tiers for British National Grid (EPSG:27700):
//   • Helmert 7-parameter (the +towgs84 below) — instant, no download, ~5 m.
//   • OSTN15 NTv2 grid shift — official OS, sub-metre, but needs the ~14.5 MB
//     `.gsb` grid lazy-loaded and registered with proj4 (+nadgrids). This is
//     what matters when the lat/lon is *exported for use* (GeoJSON consumed in
//     a GIS), not just eyeballed — a 5 m error becomes baked-in data there.
// The Irish Grid stays Helmert (no NTv2 grid bundled for it).
//
// proj4 is NOT imported here (it's multi-100 kB and must stay lazy / out of
// the entry chunk): the helpers take a proj4 instance the caller dynamically
// imports. coords.test.ts imports proj4 directly under Node.

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type Proj4Like = any;

export const WGS84 = "EPSG:4326";

/** The grid asset lazy-fetched from the app's own origin (privacy: never a
 *  third-party host). Committed under public/grids/ — provenance, SHA-256 and
 *  licence in public/grids/README.md. */
export const GRID_FILE = "OSTN15_NTv2_OSGBtoETRS.gsb";
/** proj4 nadgrid registration key, referenced by the +nadgrids def below. */
export const GRID_KEY = "OSTN15";

// BSD licence condition: the OS copyright notice must travel with the software
// that incorporates the transformation. Shown in-tool whenever OSTN15 is
// active, and embedded in the GeoJSON metadata.
export const OS_ATTRIBUTION =
  "Contains OS data © Crown copyright and database rights, Ordnance Survey Limited 2016. " +
  "OSTN15 transformation licensed under the OSI BSD Licence.";

export type SystemId = "osgb" | "irish";

export interface CrsDef {
  epsg: string;
  label: string;
  /** Helmert (towgs84) variant — always available, no grid needed. */
  helmert: string;
  /** OSTN15 (+nadgrids) variant — only when the grid is registered. */
  nadgrids?: string;
  /** Does this system support the sub-metre OSTN15 grid? (GB only.) */
  precise: boolean;
}

export const CRS: Record<SystemId, CrsDef> = {
  osgb: {
    epsg: "EPSG:27700",
    label: "British National Grid (OSGB36)",
    helmert:
      "+proj=tmerc +lat_0=49 +lon_0=-2 +k=0.9996012717 +x_0=400000 +y_0=-100000 +ellps=airy +towgs84=446.448,-125.157,542.06,0.15,0.247,0.842,-20.489 +units=m +no_defs",
    nadgrids:
      "+proj=tmerc +lat_0=49 +lon_0=-2 +k=0.9996012717 +x_0=400000 +y_0=-100000 +ellps=airy +nadgrids=" +
      GRID_KEY +
      " +units=m +no_defs",
    precise: true,
  },
  irish: {
    epsg: "EPSG:29903",
    label: "Irish Grid (TM65)",
    helmert:
      "+proj=tmerc +lat_0=53.5 +lon_0=-8 +k=1.000035 +x_0=200000 +y_0=250000 +ellps=mod_airy +towgs84=482.5,-130.6,564.6,-1.042,-0.214,-0.631,8.15 +units=m +no_defs",
    precise: false,
  },
};

/** Human label for the transform actually used — surfaced in the UI and the
 *  GeoJSON metadata so a consumer always knows the accuracy of the data. */
export function transformLabel(system: SystemId, precise: boolean): string {
  return precise && CRS[system].precise
    ? "OSTN15 NTv2 (official OS, sub-metre)"
    : "Helmert 7-parameter (~5 m)";
}

/** Register the OSTN15 grid (ArrayBuffer of the .gsb) with proj4. Idempotent
 *  per proj4 instance — re-registering just overwrites. */
export function registerOstn15(proj4: Proj4Like, grid: ArrayBuffer): void {
  proj4.nadgrid(GRID_KEY, grid);
}

/** Install the CRS definitions. When `precise`, OSGB uses its +nadgrids
 *  variant (requires {@link registerOstn15} to have run first). */
export function applyDefs(proj4: Proj4Like, precise: boolean): void {
  const osgb = CRS.osgb;
  proj4.defs(osgb.epsg, precise && osgb.nadgrids ? osgb.nadgrids : osgb.helmert);
  proj4.defs(CRS.irish.epsg, CRS.irish.helmert);
}

/** Project one grid coordinate to WGS84 [lon, lat], or null if non-finite.
 *  Assumes {@link applyDefs} (and, for precise OSGB, {@link registerOstn15})
 *  has run on this proj4 instance. */
export function project(
  proj4: Proj4Like,
  system: SystemId,
  e: number,
  n: number,
): [number, number] | null {
  if (!isFinite(e) || !isFinite(n)) return null;
  const [lon, lat] = proj4(CRS[system].epsg, WGS84, [e, n]);
  if (!isFinite(lon) || !isFinite(lat)) return null;
  return [lon, lat];
}

export interface ConvertedPoint {
  id: string;
  e: string;
  n: string;
  lat: number | null;
  lon: number | null;
}

/** ~1 mm at GB latitudes; keeps OSTN15's sub-metre precision without emitting
 *  spurious digits. */
const round8 = (x: number) => Math.round(x * 1e8) / 1e8;

/**
 * Build an RFC 7946 FeatureCollection from converted points. GeoJSON is always
 * WGS84 and coordinates are [longitude, latitude] (x, y) — the classic gotcha.
 * A top-level `metadata` foreign member (RFC 7946 §6.1 permits these) records
 * the source, the transform used (so the consumer knows the accuracy), and the
 * OS attribution when OSTN15 was applied. Points that failed to convert are
 * dropped (a Feature must have a geometry).
 */
export function toGeoJson(
  points: ConvertedPoint[],
  opts: { system: SystemId; precise: boolean },
): string {
  const usedOstn15 = opts.precise && CRS[opts.system].precise;
  const fc = {
    type: "FeatureCollection",
    metadata: {
      source: "AGS4 LOCA (LOCA_NATE / LOCA_NATN)",
      sourceGrid: `${CRS[opts.system].label} [${CRS[opts.system].epsg}]`,
      transform: transformLabel(opts.system, opts.precise),
      crs: "WGS84 [EPSG:4326]",
      ...(usedOstn15 ? { attribution: OS_ATTRIBUTION } : {}),
    },
    features: points
      .filter((p) => p.lon != null && p.lat != null)
      .map((p) => ({
        type: "Feature",
        geometry: {
          type: "Point",
          coordinates: [round8(p.lon!), round8(p.lat!)],
        },
        properties: {
          LOCA_ID: p.id,
          easting: p.e,
          northing: p.n,
        },
      })),
  };
  return JSON.stringify(fc, null, 2);
}
