import {
  createMemo,
  createResource,
  createSignal,
  For,
  Show,
  type Component,
} from "solid-js";
import { splitAgsFields } from "../../lib/agsline";
import { fileStore } from "../../lib/fileStore";
import { downloadBlob, baseName } from "../../lib/download";
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
  type SystemId,
} from "../../lib/coords";
import { CoordinateMap } from "./CoordinateMap";
import { controlClass } from "../../lib/controls";

// Coordinate tool: convert a file's LOCA national-grid eastings / northings
// (LOCA_NATE / LOCA_NATN) to WGS84 latitude / longitude, entirely client-side
// via proj4. Export to CSV or GeoJSON. An OpenStreetMap basemap is available
// but OPT-IN behind explicit consent — map tiles reveal the site location to a
// third-party server, so it's off by default (see CoordinateMap). proj4 +
// Leaflet are dynamically imported so they stay out of the entry chunk.
//
// Two accuracy tiers (British National Grid only): the default Helmert
// transform (~5 m, instant) and an opt-in OSTN15 mode (sub-metre, official OS)
// that lazy-fetches the ~14.5 MB NTv2 grid. OSTN15 is what you want when the
// lat/lon is *exported for use* (GeoJSON in a GIS), not just eyeballed. The
// projection maths + GeoJSON emit live in ../../lib/coords (unit-tested against
// OS's official vectors).

// Lazy, cached fetch of the bundled grid from the app's own origin (never a
// third-party host — privacy). Registered with proj4 exactly once.
let gridPromise: Promise<ArrayBuffer> | null = null;
let gridRegistered = false;
function loadGrid(): Promise<ArrayBuffer> {
  if (!gridPromise) {
    const url = `${import.meta.env.BASE_URL}grids/${GRID_FILE}`;
    gridPromise = fetch(url)
      .then((r) => {
        if (!r.ok) throw new Error(`HTTP ${r.status}`);
        return r.arrayBuffer();
      })
      .catch((e: unknown) => {
        gridPromise = null; // allow a later retry
        throw e;
      });
  }
  return gridPromise;
}

interface Loca {
  id: string;
  e: string;
  n: string;
}

const decode = (b: Uint8Array) =>
  new TextDecoder("utf-8", { fatal: false }).decode(b);

const val = (
  cps: string[],
  f: { valueStart: number; valueEnd: number } | undefined,
) => (f ? cps.slice(f.valueStart, f.valueEnd).join("") : "");

/** Pull (LOCA_ID, LOCA_NATE, LOCA_NATN) from the LOCA group's DATA rows. */
function parseLoca(text: string): Loca[] {
  let inLoca = false;
  let hi = { id: -1, e: -1, n: -1 };
  const out: Loca[] = [];
  for (const line of text.split(/\r?\n/)) {
    if (line.trim() === "") continue;
    const fields = splitAgsFields(line);
    const cps = Array.from(line);
    const tag = val(cps, fields[0]);
    if (tag === "GROUP") {
      inLoca = val(cps, fields[1] ?? fields[0]) === "LOCA";
      hi = { id: -1, e: -1, n: -1 };
    } else if (tag === "HEADING" && inLoca) {
      const names = fields.slice(1).map((f) => val(cps, f));
      hi = {
        id: names.indexOf("LOCA_ID"),
        e: names.indexOf("LOCA_NATE"),
        n: names.indexOf("LOCA_NATN"),
      };
    } else if (tag === "DATA" && inLoca && hi.e >= 0 && hi.n >= 0) {
      const cells = fields.slice(1).map((f) => val(cps, f));
      out.push({
        id: hi.id >= 0 ? (cells[hi.id] ?? "") : "",
        e: cells[hi.e] ?? "",
        n: cells[hi.n] ?? "",
      });
    }
  }
  return out;
}

interface ConvertResult {
  points: ConvertedPoint[];
  /** The grid these points were converted FROM. Carried on the result (not
   *  read live) so an export taken mid-refresh can't pair stale points with a
   *  newly-changed source-grid selection — which would mislabel the GeoJSON
   *  metadata and, worse, drop the OS attribution on OSTN15 coordinates. */
  system: SystemId;
  /** Whether OSTN15 was *actually* applied (false if not requested, not
   *  supported for this grid, or the grid failed to load). */
  precise: boolean;
  /** Set when precise was requested but the grid couldn't be loaded. */
  gridError: string | null;
}

export const CoordinateTool: Component = () => {
  const [crs, setCrs] = createSignal<SystemId>("osgb");
  const [precise, setPrecise] = createSignal(false);
  // Reactive mirror of the module-level gridRegistered, so the "Loading the
  // grid (~14.5 MB)…" message shows only on the genuine first fetch — not on
  // later re-conversions (file change, toggle) that reuse the cached grid.
  const [gridReady, setGridReady] = createSignal(gridRegistered);

  // OpenStreetMap basemap — opt-in. Consent (acknowledging that tiles load from
  // a third-party server) is remembered in localStorage; whether the map is
  // shown is a per-session toggle. Nothing loads until the user explicitly asks.
  const CONSENT_KEY = "ags-coords-osm-consent";
  const readConsent = () => {
    try {
      return localStorage.getItem(CONSENT_KEY) === "1";
    } catch {
      return false;
    }
  };
  const [consented, setConsented] = createSignal(readConsent());
  const [showMap, setShowMap] = createSignal(false);
  const [askConsent, setAskConsent] = createSignal(false);
  const persistConsent = (v: boolean) => {
    setConsented(v);
    try {
      if (v) localStorage.setItem(CONSENT_KEY, "1");
      else localStorage.removeItem(CONSENT_KEY);
    } catch {
      /* private mode — keep the in-memory consent, skip persistence */
    }
  };
  const requestMap = () =>
    consented() ? setShowMap(true) : setAskConsent(true);
  const confirmConsent = () => {
    persistConsent(true);
    setAskConsent(false);
    setShowMap(true);
  };
  const forgetConsent = () => {
    persistConsent(false);
    setShowMap(false);
  };

  const text = createMemo(() => {
    const b = fileStore.bytes();
    return b ? decode(b) : "";
  });
  const loca = createMemo(() => (text() ? parseLoca(text()) : []));
  const preciseSupported = () => CRS[crs()].precise;

  const [converted] = createResource(
    () => ({ rows: loca(), crs: crs(), precise: precise() }),
    async ({ rows, crs, precise }): Promise<ConvertResult> => {
      if (rows.length === 0)
        return { points: [], system: crs, precise: false, gridError: null };
      const proj4 = (await import("proj4")).default;
      let eff = precise && CRS[crs].precise;
      let gridError: string | null = null;
      if (eff) {
        try {
          const buf = await loadGrid();
          if (!gridRegistered) {
            registerOstn15(proj4, buf);
            gridRegistered = true;
          }
          setGridReady(true);
        } catch (e) {
          eff = false; // fall back to Helmert, surface why
          gridError = e instanceof Error ? e.message : String(e);
        }
      }
      applyDefs(proj4, eff);
      const points: ConvertedPoint[] = rows.map((r) => {
        const ll = project(proj4, crs, parseFloat(r.e), parseFloat(r.n));
        return { ...r, lon: ll ? ll[0] : null, lat: ll ? ll[1] : null };
      });
      return { points, system: crs, precise: eff, gridError };
    },
  );

  const points = () => converted()?.points ?? [];
  const ok = () => points().filter((r) => r.lat != null);
  const effPrecise = () => converted()?.precise ?? false;
  // The first grid download is in flight (precise requested, grid not yet
  // cached). Re-conversions after the grid is ready show "Converting…" instead.
  const loadingGrid = () =>
    converted.loading && precise() && preciseSupported() && !gridReady();

  // RFC 4180 field quoting — a LOCA_ID (or raw easting/northing) can legally
  // carry a comma, quote, or newline, which would otherwise shift columns.
  const csv = (v: string) =>
    /[",\r\n]/.test(v) ? `"${v.replace(/"/g, '""')}"` : v;

  const exportCsv = () => {
    const head = "LOCA_ID,easting,northing,latitude,longitude";
    const body = points()
      .map(
        (r) =>
          `${csv(r.id)},${csv(r.e)},${csv(r.n)},${r.lat?.toFixed(8) ?? ""},${r.lon?.toFixed(8) ?? ""}`,
      )
      .join("\r\n");
    downloadBlob(
      `${head}\r\n${body}\r\n`,
      `${baseName(fileStore.name())}.latlon.csv`,
      "text/csv",
    );
  };

  const exportGeoJson = () => {
    const c = converted();
    if (!c) return;
    downloadBlob(
      toGeoJson(c.points, { system: c.system, precise: c.precise }),
      `${baseName(fileStore.name())}.latlon.geojson`,
      "application/geo+json",
    );
  };

  return (
    <Show
      when={fileStore.bytes()}
      fallback={
        <div class="rounded-lg border border-dashed border-line-strong bg-surface p-10 text-center">
          <p class="text-lg font-medium text-fg-soft">Coordinate converter</p>
          <p class="mx-auto mt-2 max-w-prose text-sm text-fg-faint">
            Load an AGS4 file in the Validate tab to convert its LOCA grid
            coordinates to latitude / longitude. No map, no tiles — nothing
            leaves your browser.
          </p>
        </div>
      }
    >
      <div class="flex min-w-0 flex-col gap-3">
        <p class="text-sm text-fg-soft">
          Convert <span class="mono">LOCA_NATE</span> /{" "}
          <span class="mono">LOCA_NATN</span> to WGS84 lat/lon, then export CSV
          or GeoJSON. An OpenStreetMap basemap is available on request — it's
          off by default because map tiles reveal the site location to a
          third-party server.
        </p>

        <Show
          when={loca().length > 0}
          fallback={
            <p class="text-sm text-fg-muted">
              No LOCA group with <span class="mono">LOCA_NATE</span> /{" "}
              <span class="mono">LOCA_NATN</span> columns in this file.
            </p>
          }
        >
          <div class="flex flex-wrap items-center gap-3 text-sm">
            <label class="flex items-center gap-1.5 text-fg-muted">
              Source grid
              <select
                class={controlClass}
                value={crs()}
                onChange={(e) => setCrs(e.currentTarget.value as SystemId)}
              >
                <For each={Object.entries(CRS)}>
                  {([k, d]) => <option value={k}>{d.label}</option>}
                </For>
              </select>
            </label>

            <label
              class="flex items-center gap-1.5"
              classList={{
                "text-fg-muted": preciseSupported(),
                "text-fg-dim": !preciseSupported(),
              }}
              title={
                preciseSupported()
                  ? "Use the official OSTN15 grid (sub-metre). Downloads ~14.5 MB once."
                  : "OSTN15 covers Great Britain only"
              }
            >
              <input
                type="checkbox"
                checked={precise() && preciseSupported()}
                disabled={!preciseSupported()}
                onChange={(e) => setPrecise(e.currentTarget.checked)}
              />
              Precise (OSTN15, sub-metre)
              <Show when={!preciseSupported()}>
                <span class="text-xs">(GB only)</span>
              </Show>
            </label>

            <button
              type="button"
              class="rounded-md bg-cta px-3 py-1.5 font-medium text-fg-on-cta hover:bg-cta-hover disabled:opacity-45"
              disabled={ok().length === 0 || converted.loading}
              onClick={exportCsv}
            >
              Download CSV ({ok().length})
            </button>
            <button
              type="button"
              class="rounded-md bg-cta px-3 py-1.5 font-medium text-fg-on-cta hover:bg-cta-hover disabled:opacity-45"
              disabled={ok().length === 0 || converted.loading}
              onClick={exportGeoJson}
            >
              Download GeoJSON ({ok().length})
            </button>
            <Show when={!showMap()}>
              <button
                type="button"
                class="rounded-md border border-line-strong px-3 py-1.5 text-fg-soft hover:bg-chip disabled:opacity-45"
                disabled={ok().length === 0}
                onClick={requestMap}
              >
                Show on map
              </button>
            </Show>
          </div>

          {/* Accuracy + provenance line — always tells the consumer what they
              get, and carries the OS attribution when OSTN15 is active. */}
          <p class="text-xs text-fg-muted">
            Accuracy:{" "}
            <span class="text-fg-soft">
              {transformLabel(crs(), effPrecise())}
            </span>
            <Show when={!effPrecise() && preciseSupported()}>
              {" "}
              — tick <span class="text-fg-soft">Precise</span> for survey-grade
              output (recommended when the GeoJSON will be used as data).
            </Show>
            <Show when={effPrecise()}>
              {" · "}
              <span class="text-fg-dim">{OS_ATTRIBUTION}</span>
            </Show>
          </p>

          <Show when={converted()?.gridError}>
            {(err) => (
              <p class="rounded-sm border border-warn/45 bg-warn-quiet px-3 py-2 text-xs text-warn">
                Couldn't load the OSTN15 grid ({err()}) — showing ~5 m Helmert
                results instead. The grid ships at{" "}
                <span class="mono">grids/{GRID_FILE}</span>.
              </p>
            )}
          </Show>

          {/* Opt-in OpenStreetMap basemap — consent-gated because tiles reveal
              the site location to a third-party server. Nothing loads until the
              user confirms; the CoordinateMap (Leaflet) only mounts when shown.
              Rendered ABOVE the results table so clicking "Show on map" gives
              immediate visible feedback instead of appending off-screen below it. */}
          <Show when={askConsent()}>
            <div class="rounded-lg border border-warn/45 bg-warn-quiet p-3">
              <p class="text-sm font-medium text-warn">
                Show these points on a map?
              </p>
              <p class="mt-1 max-w-prose text-xs text-fg-muted">
                Plotting loads map tiles from{" "}
                <span class="font-medium">OpenStreetMap</span>, a third-party
                server. That request reveals the site's approximate location
                (and your IP) to OpenStreetMap. Nothing else in this app leaves
                your browser. Only continue if that's acceptable for this data.
              </p>
              <div class="mt-2 flex gap-2 text-sm">
                <button
                  type="button"
                  class="rounded-md bg-cta px-3 py-1.5 font-medium text-fg-on-cta hover:bg-cta-hover"
                  onClick={confirmConsent}
                >
                  Load map (OpenStreetMap)
                </button>
                <button
                  type="button"
                  class="rounded-md border border-line-strong px-3 py-1.5 text-fg-soft hover:bg-chip"
                  onClick={() => setAskConsent(false)}
                >
                  Cancel
                </button>
              </div>
            </div>
          </Show>

          <Show when={showMap()}>
            <div class="flex flex-col gap-2">
              <div class="flex flex-wrap items-center justify-between gap-2 text-xs text-fg-muted">
                <span>
                  Basemap © OpenStreetMap — only tile requests leave your
                  browser.
                </span>
                <span class="flex gap-3">
                  <button
                    type="button"
                    class="hover:text-fg"
                    onClick={() => setShowMap(false)}
                  >
                    Hide map
                  </button>
                  <button
                    type="button"
                    class="hover:text-err"
                    title="Stop loading tiles and forget the saved consent"
                    onClick={forgetConsent}
                  >
                    Forget consent
                  </button>
                </span>
              </div>
              <CoordinateMap points={ok} />
            </div>
          </Show>

          <Show
            when={!converted.loading}
            fallback={
              <p class="text-sm text-fg-muted">
                {loadingGrid()
                  ? "Loading the OSTN15 grid (~14.5 MB, first use only)…"
                  : "Converting…"}
              </p>
            }
          >
            <div class="scroll-region rounded-lg border border-line">
              <table class="min-w-full text-xs">
                <thead class="sticky top-0 z-10 bg-surface-raised text-fg-soft [&_th]:border-b [&_th]:border-line">
                  <tr>
                    <th class="px-3 py-1.5 text-left font-medium">LOCA_ID</th>
                    <th class="px-3 py-1.5 text-right font-medium">Easting</th>
                    <th class="px-3 py-1.5 text-right font-medium">Northing</th>
                    <th class="px-3 py-1.5 text-right font-medium">Latitude</th>
                    <th class="px-3 py-1.5 text-right font-medium">
                      Longitude
                    </th>
                  </tr>
                </thead>
                <tbody class="mono">
                  <For each={points()}>
                    {(r) => (
                      <tr class="border-t border-line-subtle hover:bg-surface-raised">
                        <td class="px-3 py-1 text-accent">{r.id || "—"}</td>
                        <td class="px-3 py-1 text-right text-fg-soft">
                          {r.e || "—"}
                        </td>
                        <td class="px-3 py-1 text-right text-fg-soft">
                          {r.n || "—"}
                        </td>
                        <td class="px-3 py-1 text-right text-fg-soft">
                          {r.lat?.toFixed(6) ?? "—"}
                        </td>
                        <td class="px-3 py-1 text-right text-fg-soft">
                          {r.lon?.toFixed(6) ?? "—"}
                        </td>
                      </tr>
                    )}
                  </For>
                </tbody>
              </table>
            </div>
          </Show>
        </Show>
      </div>
    </Show>
  );
};
