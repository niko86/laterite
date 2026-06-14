import { test, expect, type Page } from "@playwright/test";
import { fileURLToPath } from "node:url";
import { readFileSync } from "node:fs";
import path from "node:path";
import { enterExplore } from "./helpers";

const APP = "/laterite/";
const fixture = (name: string) =>
  path.join(path.dirname(fileURLToPath(import.meta.url)), "fixtures", name);

// Wait until the wasm validator is live (the sample buttons only render once
// the worker reports ready — see App's wasmReady gate).
async function ready(page: Page) {
  await page.goto(APP);
  await expect(page.getByRole("button", { name: /Clean \(minimal\)/ })).toBeVisible();
}

const tab = (page: Page, name: string) =>
  page.getByRole("tab", { name: new RegExp(`^${name}$`) });

test("app loads with all five tabs", async ({ page }) => {
  await ready(page);
  for (const t of ["Validate", "Fix", "Explore", "Tools", "Export"]) {
    await expect(tab(page, t)).toBeVisible();
  }
});

test("clean sample validates with no findings", async ({ page }) => {
  await ready(page);
  await page.getByRole("button", { name: /Clean \(minimal\)/ }).click();
  await expect(page.getByText(/Clean — 0 findings/)).toBeVisible();
});

test("unknown-heading sample surfaces a Rule 9 finding", async ({ page }) => {
  await ready(page);
  await page.getByRole("button", { name: /Rule 9.*unknown heading/ }).click();
  // The findings UI (banner + filter chip + group header) names Rule 9. Scope
  // to a VISIBLE match: the sample-loader button is also "Rule 9 …" but it
  // collapses once a file is loaded, so .first() alone would hit the hidden one.
  await expect(page.getByText("✗").first()).toBeVisible();
  await expect(
    page.getByText(/Rule 9/).filter({ visible: true }).first(),
  ).toBeVisible();
});

test("an FYI-only file shows an amber informational banner, not red", async ({
  page,
}) => {
  await ready(page);
  // fyi_only.ags = the clean fixture with one extended-ASCII char (é) → a
  // single "FYI (Related to Rule 1)" finding, no errors/warnings.
  await page.locator('input[type="file"]').setInputFiles(fixture("fyi_only.ags"));
  await expect(page.getByText(/informational \(FYI\) finding/)).toBeVisible();
  // The red error banner (✗) must NOT appear for an FYI-only file.
  await expect(page.getByText("✗")).toHaveCount(0);
});

test("a file with one error among FYI findings stays red (not amber)", async ({
  page,
}) => {
  await ready(page);
  // mixed_error_fyi.ags = fyi_only.ags + an unknown PROJ_ZZZZ heading (Rule 9
  // error). The é still yields a Rule 1 FYI — so it's a MIX, which must render
  // red, never the FYI-only amber banner (reportIsOnlyFyi boundary).
  await page
    .locator('input[type="file"]')
    .setInputFiles(fixture("mixed_error_fyi.ags"));
  await expect(page.getByText("✗").first()).toBeVisible();
  await expect(page.getByText(/informational \(FYI\) finding/)).toHaveCount(0);
  // The red banner breaks findings down by severity — the é is a Rule 1 FYI,
  // so the breakdown names it separately rather than lumping one red total.
  await expect(page.getByText(/\d+ informational/)).toBeVisible();
});

test("a fixable file offers a safe fix and applying it clears the safe set", async ({
  page,
}) => {
  await ready(page);
  await page.locator('input[type="file"]').setInputFiles(fixture("fixable.ags"));
  await tab(page, "Fix").click();

  // The 1-dp value "123.4" under LOCA_NATE's 2DP type is a safe Rule 8
  // reformat (→ "123.40"). (The fixture is already CRLF, so Rule 2a doesn't
  // fire — Rule 8 is the safe fix this asserts.)
  const fixAll = page.getByRole("button", { name: /Fix all safe \(\d+\)/ });
  await expect(fixAll).toBeVisible();
  await expect(fixAll).not.toHaveText(/\(0\)/);

  // The persistent download must be present BEFORE applying (regression: it
  // used to live inside FixesPanel and vanish exactly when the file went clean).
  const download = page.getByRole("button", { name: /Download \.ags/ });
  await expect(download).toBeVisible();

  await fixAll.click();
  // After applying, no safe fixes remain …
  await expect(
    page.getByRole("button", { name: /Fix all safe \(0\)/ }),
  ).toBeVisible();
  // … but the download stays — that's exactly when you'd want to save it.
  await expect(download).toBeVisible();
});

test("explore ingests a file into DuckDB-wasm", async ({ page }) => {
  await ready(page);
  await page.getByRole("button", { name: /Rule 9.*unknown heading/ }).click();
  // DuckDB-wasm loads (multi-MB) then the dashboard reports the parsed groups.
  // enterExplore dismisses the cold-engine gate if a low-end CI fingerprint
  // tripped it (otherwise "data rows" would never appear).
  await enterExplore(page);
});

test("Explore chart builder renders a chart, and the SQL builder composes a query", async ({
  page,
}) => {
  await ready(page);
  // fixable.ags has a LOCA group with a numeric LOCA_NATE column to plot.
  await page.locator('input[type="file"]').setInputFiles(fixture("fixable.ags"));
  await enterExplore(page);

  // Chart builder: pick LOCA, and an ECharts canvas renders.
  await page.getByRole("button", { name: "Charts" }).click();
  await page.getByLabel("Table").selectOption("LOCA");
  await expect(page.locator("canvas").first()).toBeVisible({ timeout: 90_000 });

  // SQL builder: the controls panel + console both appear, and "Use this SQL"
  // populates the editor.
  await page.getByRole("button", { name: "SQL" }).click();
  await expect(page.getByText(/Build a query with controls/)).toBeVisible();
  await page.getByText(/Build a query with controls/).click();
  await page.getByRole("button", { name: /Use this SQL/ }).click();
  await expect(page.locator("textarea")).toHaveValue(/SELECT/);
});

test("Tools → Dictionary loads the per-edition standard dict (searchable + selectable)", async ({
  page,
}) => {
  await ready(page);
  await tab(page, "Tools").click();
  // The engine's per-edition standard dictionary loaded (auto → 4.1.1) — not
  // the old static scaffolded JSON whose descriptions were mostly empty.
  await expect(page.getByText(/AGS 4\.1\.1 standard dictionary/)).toBeVisible();
  // Description search now works: "depth" matches a heading DESCRIPTION (the
  // bug was ~91% empty descriptions), surfacing SAMP (SAMP_TOP = depth to top).
  await page.getByPlaceholder(/Search groups/).fill("depth");
  await expect(page.getByText("SAMP", { exact: true }).first()).toBeVisible();
  // Edition is selectable → the dictionary reloads for 4.2.
  await page.getByLabel("AGS edition").selectOption("4.2");
  await expect(page.getByText(/AGS 4\.2 standard dictionary/)).toBeVisible();
});

test("Tools → Coordinates converts LOCA grid and exports GeoJSON (Helmert + OSTN15)", async ({
  page,
}) => {
  await ready(page);
  await page.locator('input[type="file"]').setInputFiles(fixture("coords.ags"));
  await tab(page, "Tools").click();
  await page.getByRole("button", { name: /^Coordinates$/ }).click();

  // Default Helmert conversion renders a lat/lon table. BH02 is OS test point
  // TP20 (lat 53.8002…) — assert the converted latitude shows.
  await expect(page.getByText("BH02").first()).toBeVisible();
  await expect(page.getByText(/53\.80/).first()).toBeVisible();

  // Export GeoJSON in the default (Helmert) mode — no grid download needed.
  const [dlH] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("button", { name: /Download GeoJSON/ }).click(),
  ]);
  expect(dlH.suggestedFilename()).toMatch(/\.latlon\.geojson$/);
  const helmert = JSON.parse(readFileSync(await dlH.path(), "utf8"));
  expect(helmert.type).toBe("FeatureCollection");
  expect(helmert.features).toHaveLength(3);
  // GeoJSON is [lon, lat] (x, y).
  expect(helmert.features[0].geometry.coordinates).toHaveLength(2);
  expect(helmert.metadata.transform).toMatch(/Helmert/);
  expect(helmert.metadata.attribution).toBeUndefined();

  // Opt into OSTN15 — the ~14.5 MB grid lazy-loads from the app's own origin,
  // then the accuracy line flips to the rigorous OS transform.
  await page.getByRole("checkbox", { name: /Precise/ }).check();
  await expect(page.getByText(/OSTN15 NTv2/)).toBeVisible({ timeout: 90_000 });

  // The precise export carries OSTN15 + the OS attribution (BSD requirement).
  const [dlP] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("button", { name: /Download GeoJSON/ }).click(),
  ]);
  const precise = JSON.parse(readFileSync(await dlP.path(), "utf8"));
  expect(precise.metadata.transform).toMatch(/OSTN15/);
  expect(precise.metadata.attribution).toMatch(/Ordnance Survey/);
});

test("an empty pane's 'Go to Validate' button jumps to the Validate tab", async ({
  page,
}) => {
  await ready(page);
  // No file loaded → Explore shows its empty state, now with an actionable
  // button (was a dead end: text saying 'load a file in Validate' + no way to).
  await tab(page, "Explore").click();
  await page.getByRole("button", { name: /Go to Validate to load a file/ }).click();
  await expect(tab(page, "Validate")).toHaveAttribute("aria-selected", "true");
});

test("a shared link restores the tab AND the sub-view (#tab=tools&tool=coords)", async ({
  page,
}) => {
  // Sub-view state is persisted + shareable (lib/settings), so a link lands on
  // the exact tool the sender saw — not just the tab, then the default tool.
  await page.goto(APP + "#tab=tools&tool=coords");
  await expect(tab(page, "Tools")).toHaveAttribute("aria-selected", "true");
  await expect(page.getByText(/Coordinate converter/)).toBeVisible();
});

test("Tools → Coordinates: the OpenStreetMap basemap is consent-gated", async ({
  page,
}) => {
  // Never actually hit OSM tile servers from a test — abort tile requests. The
  // Leaflet map still initialises (container + attribution + SVG markers); only
  // the tile imagery is missing. This also proves no tile loads before consent.
  await page.route(/tile\.openstreetmap\.org/, (r) => r.abort());

  await ready(page);
  await page.locator('input[type="file"]').setInputFiles(fixture("coords.ags"));
  await tab(page, "Tools").click();
  await page.getByRole("button", { name: /^Coordinates$/ }).click();

  // No map until the user asks.
  await expect(page.locator(".leaflet-container")).toHaveCount(0);

  // "Show on map" → the consent gate appears, still NO map mounted.
  await page.getByRole("button", { name: /Show on map/ }).click();
  await expect(page.getByText(/Show these points on a map\?/)).toBeVisible();
  await expect(page.locator(".leaflet-container")).toHaveCount(0);

  // Consent → Leaflet mounts (container + OSM attribution control render).
  await page.getByRole("button", { name: /Load map \(OpenStreetMap\)/ }).click();
  await expect(page.locator(".leaflet-container")).toBeVisible();
  await expect(page.locator(".leaflet-control-attribution")).toContainText(
    /OpenStreetMap/,
  );

  // Hide map → the container is gone again.
  await page.getByRole("button", { name: /^Hide map$/ }).click();
  await expect(page.locator(".leaflet-container")).toHaveCount(0);
});

// --- PWA / offline -------------------------------------------------------

// Poll until the service worker controls the page. clientsClaim (vite.config)
// makes the first-install SW claim the open page as soon as it activates —
// which Workbox only does AFTER the app-shell precache install completes — so
// a non-null controller is a reliable "precache ready, offline will work"
// signal to gate on before cutting the network.
async function waitForServiceWorker(page: Page) {
  await expect
    .poll(
      () =>
        page.evaluate(
          () =>
            "serviceWorker" in navigator &&
            navigator.serviceWorker.controller !== null,
        ),
      { timeout: 30_000, message: "service worker never took control" },
    )
    .toBe(true);
}

test("PWA: installable — manifest, icons and theme-color are wired", async ({
  page,
}) => {
  await ready(page);
  // The injected manifest link is base-prefixed so it resolves under the
  // /laterite/ deploy base (a bare href would 404 on Pages).
  await expect(page.locator('link[rel="manifest"]')).toHaveAttribute(
    "href",
    "/laterite/manifest.webmanifest",
  );
  // The manifest parses and declares the installability essentials (a 512px
  // icon, standalone display, base-scoped start_url/scope).
  const manifest = await page.evaluate(async () => {
    const href = document
      .querySelector('link[rel="manifest"]')
      ?.getAttribute("href");
    return href ? await (await fetch(href)).json() : null;
  });
  expect(manifest?.start_url).toBe("/laterite/");
  expect(manifest?.scope).toBe("/laterite/");
  expect(manifest?.display).toBe("standalone");
  expect(
    (manifest?.icons ?? []).some(
      (i: { sizes?: string }) => i.sizes === "512x512",
    ),
  ).toBe(true);
  // Both light + dark theme-color metas present.
  await expect(page.locator('meta[name="theme-color"]')).toHaveCount(2);
});

test("PWA: the app loads and validates fully offline after first visit", async ({
  page,
}) => {
  await ready(page);
  await waitForServiceWorker(page);

  // Tie the offline pass to SW PROVENANCE, not Chromium's HTTP cache.
  // setOffline blocks the network but does NOT bypass the browser's own disk
  // cache — so a reload could be served from the HTTP cache even if the
  // precache were broken. Assert the workbox precache itself holds the
  // validator wasm + index.html, so a regression that dropped them from the
  // precache globs fails here instead of passing green off the HTTP cache.
  const precacheHasShell = await page.evaluate(async () => {
    for (const key of await caches.keys()) {
      const reqs = await (await caches.open(key)).keys();
      const paths = reqs.map((r) => new URL(r.url).pathname);
      if (
        paths.some((p) => /\/ags4_wasm_bg-.*\.wasm$/.test(p)) &&
        paths.some((p) => /\/index\.html$/.test(p))
      )
        return true;
    }
    return false;
  });
  expect(precacheHasShell).toBe(true);

  // Cut the network at the browser level — nothing may reach the server now.
  await page.context().setOffline(true);
  try {
    // Offline reload: the SW must serve the app shell (navigateFallback →
    // precached index.html), every JS/CSS chunk, and the 2.2 MB validator
    // wasm — all from the precache.
    await page.reload();
    await expect(
      page.getByRole("button", { name: /Clean \(minimal\)/ }),
    ).toBeVisible();

    // And the engine genuinely runs offline: the precached validator wasm
    // boots in its worker and validates a precached sample with no network.
    await page.getByRole("button", { name: /Clean \(minimal\)/ }).click();
    await expect(page.getByText(/Clean — 0 findings/)).toBeVisible();
  } finally {
    // Restore so the shared browser/context can't leak offline into reuse.
    await page.context().setOffline(false);
  }
});

// Helper: load coords.ags into Explore and open the SQL view.
async function exploreSql(page: Page) {
  await ready(page);
  await page.locator('input[type="file"]').setInputFiles(fixture("coords.ags"));
  await enterExplore(page);
  await page.getByRole("button", { name: "SQL" }).click();
}

test("Explore SQL: builder WHERE on another group runs without wedging the engine", async ({
  page,
}) => {
  await exploreSql(page);
  // Build a query from a DIFFERENT group (LOCA) with a WHERE filter left at its
  // EMPTY default. Regression: that used to emit `"LOCA_NATE" = ''`, a DuckDB
  // conversion error that hung the console forever ("Running…", stale results).
  await page.getByText(/Build a query with controls/).click();
  await page.getByLabel("Table").selectOption("LOCA");
  await page.getByRole("button", { name: /\+ add/ }).click();
  await page.getByRole("button", { name: /Use this SQL/ }).click();
  await page.getByRole("button", { name: /^Run/ }).click();
  // It runs and returns LOCA rows — not stuck, no error.
  await expect(page.getByText(/Running…/)).toHaveCount(0, { timeout: 15_000 });
  await expect(page.getByText(/SQL error/)).toHaveCount(0);
  await expect(page.getByText("BH02").first()).toBeVisible();
});

test("Explore SQL: a query error surfaces and the engine recovers (no permanent hang)", async ({
  page,
}) => {
  test.setTimeout(60_000);
  await exploreSql(page);
  const box = page.locator("textarea");
  const run = page.getByRole("button", { name: /^Run/ });

  // A bad query (here a missing table) is a DuckDB error that — before the fix —
  // could leave run() unsettled, wedging the console on "Running…" forever. Now
  // it must surface an error instead (directly, or via the run() timeout).
  await box.fill(`SELECT * FROM "NOPE_NOSUCH_TABLE"`);
  await run.click();
  await expect(page.getByText(/SQL error/)).toBeVisible({ timeout: 20_000 });

  // …and the engine recovers: a subsequent good query works.
  await box.fill(`SELECT * FROM "LOCA"`);
  await run.click();
  await expect(page.getByText(/SQL error/)).toHaveCount(0, { timeout: 20_000 });
  await expect(page.getByText("BH02").first()).toBeVisible();
});

test("Fix tab offers a RISKY datetime canonicalisation for a non-ISO DT cell", async ({
  page,
}) => {
  await ready(page);
  // datetime.ags has TRAN_DATE = "18/08/2020" in a yyyy-mm-dd column — a Rule 8
  // miss the new fixer rewrites to ISO. It's risky (dd/mm is a guess), so it
  // lands in the opt-in section, not fix-all-safe.
  await page.locator('input[type="file"]').setInputFiles(fixture("datetime.ags"));
  await tab(page, "Fix").click();
  await expect(page.getByText(/Canonicalise datetime/)).toBeVisible();
  await expect(page.getByText(/2020-08-18/).first()).toBeVisible();
});

test("Explore on a low-end device asks before downloading the engine, then loads on confirm", async ({
  page,
}) => {
  // Emulate a weak machine so the device-capability gate (T1) fires: clicking
  // Explore must NOT silently kick off the 36 MB download/compile — it asks
  // first (T1b), so a slow machine isn't surprised by a multi-second freeze.
  await page.addInitScript(`try {
    Object.defineProperty(navigator, 'deviceMemory', { configurable: true, get: () => 2 });
    Object.defineProperty(navigator, 'hardwareConcurrency', { configurable: true, get: () => 2 });
  } catch (e) {}`);
  await ready(page);
  await page.locator('input[type="file"]').setInputFiles(fixture("fixable.ags"));

  // The dashboard must NOT appear yet — the cold-engine confirmation gates it.
  await tab(page, "Explore").click();
  await expect(page.getByText(/Open the data explorer\?/)).toBeVisible();
  await expect(page.getByText(/data rows/)).toHaveCount(0);

  // Continue → the engine loads and the dashboard renders.
  await page.getByRole("button", { name: /^Continue$/ }).click();
  await expect(page.getByText(/data rows/)).toBeVisible({ timeout: 90_000 });

  // A return to Explore in the same session must NOT re-ask (engine is ready).
  await tab(page, "Validate").click();
  await tab(page, "Explore").click();
  await expect(page.getByText(/Open the data explorer\?/)).toHaveCount(0);
  await expect(page.getByText(/data rows/)).toBeVisible();
});
