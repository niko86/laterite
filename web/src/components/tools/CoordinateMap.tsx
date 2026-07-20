import { createEffect, onCleanup, onMount, type Component } from "solid-js";
import type * as Leaflet from "leaflet";
import type { ConvertedPoint } from "../../lib/coords";

// Leaflet + OpenStreetMap basemap host. Two reasons it's isolated here and
// mounted ONLY after explicit consent (the gate lives in CoordinateTool):
//   1. Privacy — OSM tile requests reveal the viewport (≈ the site location)
//      and the user's IP to a third-party tile server. The rest of the app
//      never leaves the browser, so this is opt-in.
//   2. Weight — Leaflet + its CSS are multi-100 kB; dynamically importing them
//      in onMount (never a static import) keeps them out of the entry chunk,
//      and means neither the library nor a single tile loads until this
//      component actually mounts.
// Markers are vector circleMarkers (no PNG icon assets) — so there's no
// Leaflet default-marker-icon bundler workaround to do, and no broken image
// requests. Popups are built as DOM nodes (textContent) so a LOCA_ID can't
// inject HTML.
export const CoordinateMap: Component<{
  points: () => ConvertedPoint[];
}> = (props) => {
  let el!: HTMLDivElement;
  // Leaflet is dynamically imported in onMount (kept out of the entry chunk), so
  // these hold its real types via a type-only import — no runtime leaflet here.
  let map: Leaflet.Map | null = null;
  let layer: Leaflet.LayerGroup | null = null;
  let L: typeof Leaflet | null = null;

  const draw = () => {
    if (!map || !L || !layer) return;
    layer.clearLayers();
    const latlngs: [number, number][] = [];
    for (const p of props.points()) {
      if (p.lat == null || p.lon == null) continue;
      const ll: [number, number] = [p.lat, p.lon];
      latlngs.push(ll);
      const popup = document.createElement("div");
      popup.textContent = `${p.id || "(no id)"} — ${p.lat.toFixed(6)}, ${p.lon.toFixed(6)}`;
      L.circleMarker(ll, {
        radius: 6,
        color: "#0ea5e9",
        weight: 2,
        fillColor: "#0ea5e9",
        fillOpacity: 0.5,
      })
        .bindPopup(popup)
        .addTo(layer);
    }
    if (latlngs.length === 1) {
      const only = latlngs[0];
      if (only) map.setView(only, 15);
    } else if (latlngs.length > 1)
      map.fitBounds(L.latLngBounds(latlngs), { padding: [30, 30] });
  };

  onMount(() => {
    let disposed = false;
    void Promise.all([
      import("leaflet"),
      import("leaflet/dist/leaflet.css"),
      // eslint-disable-next-line solid/reactivity -- one-shot init after the dynamic import; the reactive redraw is the createEffect below, not this .then callback
    ]).then(([leaflet]) => {
      if (disposed) return;
      // eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- leaflet's type says .default is always present, but the ESM/CJS interop shape varies by bundler
      L = leaflet.default ?? leaflet;
      map = L.map(el).setView([54.5, -2.5], 5); // GB-ish until fitBounds
      L.tileLayer("https://tile.openstreetmap.org/{z}/{x}/{y}.png", {
        maxZoom: 19,
        attribution:
          '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors',
      }).addTo(map);
      layer = L.layerGroup().addTo(map);
      // The container is visible at mount (only rendered once consented/shown),
      // but a tick of invalidateSize guards against a late layout settle.
      queueMicrotask(() => map?.invalidateSize());
      draw();
    });
    onCleanup(() => {
      disposed = true;
      map?.remove();
      map = null;
    });
  });

  // Re-plot when the converted points change (new file / precise toggle / grid).
  createEffect(() => {
    props.points();
    draw();
  });

  return (
    <div
      ref={el}
      class="relative z-0 rounded-lg border border-line"
      style={{ width: "100%", height: "420px" }}
    />
  );
};
