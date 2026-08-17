import { test, expect, type Page } from "@playwright/test";
import { fileURLToPath } from "node:url";
import { readFileSync } from "node:fs";
import path from "node:path";
import { APP, enterExplore } from "./helpers";

const fixture = (name: string) =>
  path.join(path.dirname(fileURLToPath(import.meta.url)), "fixtures", name);

// Wait until the app has painted. The sample buttons render on the ~30 KB
// tokenizer alone (#353) — the engine may still be arriving — but every caller
// goes on to validate something, which waits for it anyway.
async function ready(page: Page) {
  await page.goto(APP);
  await expect(
    page.getByRole("button", { name: /Clean \(minimal\)/ }),
  ).toBeVisible();
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
    page
      .getByText(/Rule 9/)
      .filter({ visible: true })
      .first(),
  ).toBeVisible();
});

test("an FYI-only file shows an amber informational banner, not red", async ({
  page,
}) => {
  await ready(page);
  // fyi_only.ags = the clean fixture with one extended-ASCII char (é) → a
  // single "FYI (Related to Rule 1)" finding, no errors/warnings.
  await page
    .locator('input[type="file"]')
    .setInputFiles(fixture("fyi_only.ags"));
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
  await page
    .locator('input[type="file"]')
    .setInputFiles(fixture("fixable.ags"));
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
  await page
    .locator('input[type="file"]')
    .setInputFiles(fixture("fixable.ags"));
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

test("Tools → Excel round-trips AGS4 → .xlsx → AGS4, client-side", async ({
  page,
}) => {
  await ready(page);
  await page.locator('input[type="file"]').setInputFiles(fixture("coords.ags"));
  await tab(page, "Tools").click();
  await page.getByRole("button", { name: /^Excel$/ }).click();

  // AGS4 → .xlsx: download the workbook, assert it's a real zip (PK magic) and
  // the success note reports the sheet/row counts.
  const [xlsxDl] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("button", { name: /Download as Excel/ }).click(),
  ]);
  expect(xlsxDl.suggestedFilename()).toMatch(/\.xlsx$/);
  const xlsxBytes = readFileSync(await xlsxDl.path());
  expect(xlsxBytes.subarray(0, 2).toString("latin1")).toBe("PK");
  await expect(page.getByText(/→ \.xlsx/)).toBeVisible();

  // .xlsx → AGS4: feed that workbook back into the import input; selecting a
  // file triggers the conversion + download. Assert the emitted AGS4 carries
  // the LOCA group back (a full client-side round trip).
  const [agsDl] = await Promise.all([
    page.waitForEvent("download"),
    page.locator('input[accept*="xlsx"]').setInputFiles(await xlsxDl.path()),
  ]);
  expect(agsDl.suggestedFilename()).toMatch(/\.ags$/);
  const agsText = readFileSync(await agsDl.path(), "utf8");
  expect(agsText).toContain("GROUP");
  expect(agsText).toContain("LOCA");
});

// The engine runs in TWO workers (#354, ags-wiki/design/dec-engine-tiering.md):
// the always-on one behind Validate/Fix/Export and most of Tools, and a second
// created only when Explore or Tools → Excel is opened — the one that carries
// the much larger tier-2 engine from #355 on.
//
// Nothing about that split shows in the UI, so these two tests are the whole
// guard. A `startTier2Worker()` call that drifted to module scope, or a validate
// re-routed to the second worker, would put every visitor back on the big engine
// with every feature still working and every size gate still green.
const tier2Workers = (page: Page) =>
  page.workers().filter((w) => /tier2\.worker/.test(w.url()));

test("Tools → Excel brings the second engine worker up; validating never does", async ({
  page,
}) => {
  await ready(page);
  // A full validate, end to end, in the always-on worker.
  await page.getByRole("button", { name: /Clean \(minimal\)/ }).click();
  await expect(page.getByText(/Clean — 0 findings/)).toBeVisible();
  expect(tier2Workers(page)).toHaveLength(0);

  // Tools itself doesn't count — Dictionary is served by the same worker.
  await tab(page, "Tools").click();
  await page.getByRole("button", { name: /^Dictionary$/ }).click();
  await expect(page.getByPlaceholder(/Search/).first()).toBeVisible();
  expect(tier2Workers(page)).toHaveLength(0);

  // Opening Excel does, before any conversion is asked for.
  await page.getByRole("button", { name: /^Excel$/ }).click();
  await expect.poll(() => tier2Workers(page).length).toBe(1);
});

test("Explore brings the second engine worker up, and only one of it", async ({
  page,
}) => {
  await ready(page);
  expect(tier2Workers(page)).toHaveLength(0);

  // No file loaded, so the pane renders its "load one first" prompt: it is
  // opening the TAB that creates the worker, not an ingest — and no DuckDB.
  await tab(page, "Explore").click();
  await expect(page.getByText(/Data explorer/)).toBeVisible();
  await expect.poll(() => tier2Workers(page).length).toBe(1);

  // The other consumer reuses it rather than starting a second.
  await tab(page, "Tools").click();
  await page.getByRole("button", { name: /^Excel$/ }).click();
  await expect(page.getByText(/AGS4 → Excel/)).toBeVisible();
  expect(tier2Workers(page)).toHaveLength(1);
});

test("Tools → Transport round-trips a file through .zst.age, client-side", async ({
  page,
}) => {
  // The passphrase KDF is scrypt log_N 18 — deliberately expensive, and slow on
  // a memory-starved CI runner (256 MiB buffer). Give it plenty of room.
  test.setTimeout(180_000);
  await ready(page);
  await page.locator('input[type="file"]').setInputFiles(fixture("coords.ags"));
  const original = readFileSync(fixture("coords.ags"));
  await tab(page, "Tools").click();
  await page.getByRole("button", { name: /^Transport$/ }).click();

  // Encrypt the loaded file with a passphrase → a real age file (magic bytes).
  await page
    .getByPlaceholder("Passphrase")
    .first()
    .fill("correct horse battery");
  const [ageDl] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("button", { name: /Encrypt & download/ }).click(),
  ]);
  expect(ageDl.suggestedFilename()).toMatch(/\.zst\.age$/);
  const lockedPath = await ageDl.path();
  expect(readFileSync(lockedPath).subarray(0, 18).toString("latin1")).toBe(
    "age-encryption.org",
  );

  // Decrypt it back and assert byte-for-byte recovery of the original.
  await page.locator('input[accept*="age"]').setInputFiles(lockedPath);
  await page
    .getByPlaceholder("Passphrase")
    .nth(1)
    .fill("correct horse battery");
  const [outDl] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("button", { name: /Decrypt & download/ }).click(),
  ]);
  expect(readFileSync(await outDl.path()).equals(original)).toBe(true);
});

test("Tools → Anonymiser pseudonymises IDs (cross-refs intact) + hashes PROJ_ID", async ({
  page,
}) => {
  await ready(page);
  // strata.ags carries LOCA_ID across LOCA/GEOL/SAMP/TREG — a real cross-ref set.
  await page.locator('input[type="file"]').setInputFiles(fixture("strata.ags"));
  await tab(page, "Tools").click();
  await page.getByRole("button", { name: /^Anonymiser$/ }).click();

  // Default preset "All identifying" → download the redacted file.
  const [dl] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("button", { name: /Download redacted/ }).click(),
  ]);
  expect(dl.suggestedFilename()).toMatch(/\.anon\.ags$/);
  const out = readFileSync(await dl.path(), "utf8");

  // The real LOCA_ID "BH01" is gone EVERYWHERE — replaced by one stable
  // pseudonym (ID000N) shared across every group it keyed, so cross-references
  // survive. The token appears more than once (multiple group sections).
  expect(out).not.toContain('"BH01"');
  const m = out.match(/"ID\d{4}"/);
  expect(m).not.toBeNull();
  expect(out.split(m![0]).length - 1).toBeGreaterThan(1);

  // PROJ_ID "P1" → the file's full 64-hex content hash (Phase 2, #581: the web
  // now drives the shared Rust engine, which uses the full-width hash — a KEY
  // field, so collision-safe — not the old 16-hex TS truncation).
  expect(out).not.toMatch(/"DATA","P1"/);
  expect(out).toMatch(/"[0-9a-f]{64}"/);
});

test("an empty pane's 'Go to Validate' button jumps to the Validate tab", async ({
  page,
}) => {
  await ready(page);
  // No file loaded → Explore shows its empty state, now with an actionable
  // button (was a dead end: text saying 'load a file in Validate' + no way to).
  await tab(page, "Explore").click();
  await page
    .getByRole("button", { name: /Go to Validate to load a file/ })
    .click();
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
  await page
    .getByRole("button", { name: /Load map \(OpenStreetMap\)/ })
    .click();
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
  // The injected manifest link, start_url and scope must all sit under the
  // deploy base — under a non-root base a bare href 404s, and a start_url that
  // doesn't match scope makes the app uninstallable. At base "/" the prefix
  // half of that is trivially true; the fetch below is what still has teeth.
  await expect(page.locator('link[rel="manifest"]')).toHaveAttribute(
    "href",
    `${APP}manifest.webmanifest`,
  );
  // The manifest parses and declares the installability essentials (a 512px
  // icon, standalone display, base-scoped start_url/scope).
  const manifest = await page.evaluate(async () => {
    const href = document
      .querySelector('link[rel="manifest"]')
      ?.getAttribute("href");
    return href ? await (await fetch(href)).json() : null;
  });
  expect(manifest?.start_url).toBe(APP);
  expect(manifest?.scope).toBe(APP);
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
        // The tiny tokenizer wasm (#533) is boot-critical: first render is
        // gated on it, so a precache drop breaks the app offline entirely.
        paths.some((p) => /\/ags4_tokenizer_bg-.*\.wasm$/.test(p)) &&
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
    // precached index.html), every JS/CSS chunk, and the tier-1 engine wasm —
    // all from the precache.
    await page.reload();
    await expect(
      page.getByRole("button", { name: /Clean \(minimal\)/ }),
    ).toBeVisible();

    // And the engine genuinely runs offline: the precached tier-1 wasm boots in
    // its worker and validates a precached sample with no network.
    await page.getByRole("button", { name: /Clean \(minimal\)/ }).click();
    await expect(page.getByText(/Clean — 0 findings/)).toBeVisible();

    // Tools works offline too, which is what tier 1 bought and what the
    // "Validate, Fix, Export & Tools now work offline" notice now claims (#355).
    // The dictionary is served by the same worker and the same precached
    // artifact, so this is the claim itself rather than a proxy for it.
    await tab(page, "Tools").click();
    await page.getByRole("button", { name: /^Dictionary$/ }).click();
    await expect(page.getByPlaceholder(/Search/).first()).toBeVisible();
  } finally {
    // Restore so the shared browser/context can't leak offline into reuse.
    await page.context().setOffline(false);
  }
});

// The whole four-tier design (#338) reduces to one fact about the install: the
// precache carries tier 1 and NOT tier 2. Everything else about the split is
// size-based and has headroom for the full engine — the raw ceiling, the gzip
// ceiling, `maximumFileSizeToCacheInBytes` — so this negative assertion is the
// only guard here that would actually fail.
//
// The trap it guards is not hypothetical: both engines come out of one crate and
// wasm-pack names them both `ags4_wasm_bg.wasm` by default. Same stem, two
// hashes, and `assets/ags4_wasm_bg-*.wasm` matches BOTH — the install carries the
// full engine again, every feature still works, and every number still passes.
//
// Falsified before being trusted, and the run taught the ordering: widening that
// glob to match both engines is caught FIRST by the 3 MiB
// `maximumFileSizeToCacheInBytes` (a build warning, not a failure — tier 2 is
// 5.31 MB), and only once the cap is also lifted does tier 2 reach the precache.
// At that point this test fails here, naming the leaked artifact, and it is the
// only check in the repo that does.
test("PWA: the precache carries the tier-1 engine and not tier 2", async ({
  page,
}) => {
  await ready(page);
  await waitForServiceWorker(page);

  // Scoped to the PRECACHE specifically — identified as the cache holding
  // index.html — not to "any cache". Tier 2 is *expected* in a runtime cache
  // once something fetches it (a later test asserts exactly that), and since
  // #356 the idle warm puts it there on any capable device without a tab being
  // opened at all; a whole-storage check would fail for the one behaviour the
  // design wants.
  const wasm = await page.evaluate(async () => {
    for (const key of await caches.keys()) {
      const reqs = await (await caches.open(key)).keys();
      const names = reqs.map((r) => new URL(r.url).pathname.split("/").pop());
      if (names.some((n) => n === "index.html"))
        return names.filter((n) => n?.endsWith(".wasm"));
    }
    return null;
  });

  expect(wasm, "no precache cache holding index.html").not.toBeNull();
  // Tier 1 (the engine minus arrow + excel) and tier 0 (the tokenizer first
  // render waits on) — both precached, both required offline.
  expect(wasm?.filter((n) => /^ags4_wasm_bg-.*\.wasm$/.test(n ?? ""))).toEqual([
    expect.stringMatching(/^ags4_wasm_bg-/),
  ]);
  expect(
    wasm?.filter((n) => /^ags4_tokenizer_bg-.*\.wasm$/.test(n ?? "")),
  ).toEqual([expect.stringMatching(/^ags4_tokenizer_bg-/)]);
  // Tier 2 — the assertion the tiering rests on.
  expect(wasm?.filter((n) => /^ags4_wasm_full_bg-/.test(n ?? ""))).toEqual([]);
});

/** How many tier-2 wasm entries the CacheFirst bucket holds. Shared by the
 *  test that fills it on a real Excel open and the one that fills it on idle
 *  — the same question asked of the same cache, so the same reader. */
const tier2Cached = (page: Page) =>
  page.evaluate(async () => {
    const names = await caches.keys();
    if (!names.includes("ags-engine-tier2")) return 0;
    const keys = await (await caches.open("ags-engine-tier2")).keys();
    return keys.filter((k) => /ags4_wasm_full_bg-.*\.wasm$/.test(k.url)).length;
  });

test("PWA: the tier-2 engine lands in its own runtime cache on first Excel use", async ({
  page,
}) => {
  // The positive half of the same rule, and the half that fails SILENTLY: if the
  // tier-2 response ever stopped being cacheable under `statuses: [200]`, nothing
  // would error — the entry would simply never be written and every Explore or
  // Excel open would re-download 5.2 MB, reported nowhere. Same shape as the
  // DuckDB test below, for the same reason, on our own artifact this time.
  await ready(page);
  // Control BEFORE the fetch: a runtime rule only sees what the worker
  // intercepts, and a cold first visit can open Excel while the SW is still
  // installing — the cache would then be empty for a reason unrelated to the
  // rule under test.
  await waitForServiceWorker(page);
  await tab(page, "Tools").click();
  await page.getByRole("button", { name: /^Excel$/ }).click();

  await expect
    .poll(() => tier2Cached(page), {
      timeout: 30_000,
      message:
        "the tier-2 wasm never reached the ags-engine-tier2 runtime cache — " +
        "CacheFirst is refetching the full engine on every open",
    })
    .toBeGreaterThan(0);
});

test("PWA: the DuckDB engine actually lands in its runtime cache", async ({
  page,
}) => {
  // #339 tightened both CacheFirst rules from `statuses: [0, 200]` to `[200]`,
  // so a refused cross-origin fetch can no longer be cached as an opaque
  // response and then served — or rather, thrown at — forever. The unit guard
  // (src/lib/sw-cache-policy.test.ts) asserts that policy over the rule set.
  //
  // This asserts the half a config test CANNOT see, and it is the half that
  // fails SILENTLY: if a response ever stopped being cacheable under the
  // tightened rule, nothing would error — the entry would simply never be
  // written and CacheFirst would re-download ~36 MB on every single page load,
  // reported nowhere. Only a real browser against a real service worker can
  // tell "cached" from "silently refetched every time".
  //
  // Rides an Explore load the suite already pays for; the engine is same-origin
  // out of dist/ here, because VITE_DUCKDB_CDN is a deploy-only setting.
  await ready(page);
  // Wait for control BEFORE the engine is fetched. A runtime-caching rule only
  // sees fetches the worker intercepts, and on a cold first visit the SW may
  // still be installing when Explore fires — in which case DuckDB loads
  // straight off the network and the cache stays empty for a reason that has
  // nothing to do with the rule under test.
  await waitForServiceWorker(page);
  await page.getByRole("button", { name: /Rule 9.*unknown heading/ }).click();
  await enterExplore(page);

  await expect
    .poll(
      () =>
        page.evaluate(async () => {
          const names = await caches.keys();
          if (!names.includes("ags-duckdb-wasm")) return 0;
          const keys = await (await caches.open("ags-duckdb-wasm")).keys();
          return keys.filter((k) => /duckdb-(eh|mvp)-.*\.wasm$/.test(k.url))
            .length;
        }),
      {
        timeout: 30_000,
        message:
          "the DuckDB wasm never reached the ags-duckdb-wasm runtime cache — " +
          "CacheFirst is refetching it on every load",
      },
    )
    .toBeGreaterThan(0);
});

// The tier-2 idle warm (#356). Everything about it is invisible in the UI — a
// warm that never fires, fires twice, or quietly compiles 5.2 MB of wasm all
// look identical on screen, and all three are regressions these tests exist to
// catch.

/** Present capable HARDWARE and the given connection to the page, before a line
 *  of app script runs. The hardware half is not decoration: CI runners routinely
 *  fingerprint low-end (≤ 2 cores, < 4 GB) — `helpers.ts` already works around
 *  the same thing for the Explore engine gate — and the warm skips those BY
 *  DESIGN, so without it a warm test on a runner asserts the opposite of what it
 *  means to and passes for the wrong reason. The connection is what each test
 *  actually varies. */
async function poseAsDevice(page: Page, connection: NetworkInformation) {
  // `configurable: true` inside a try, matching the two low-end poses this file
  // and perf.spec.ts already use: a browser that ships one of these as an own
  // non-configurable property would otherwise throw here and take the page with
  // it, and a pose is never worth failing a test it isn't about.
  await page.addInitScript((conn) => {
    try {
      const fixed = (k: string, v: unknown) =>
        Object.defineProperty(navigator, k, {
          configurable: true,
          get: () => v,
        });
      fixed("hardwareConcurrency", 8);
      fixed("deviceMemory", 8);
      fixed("connection", conn);
    } catch {
      /* leave the real values in place */
    }
  }, connection);
}

interface NetworkInformation {
  saveData: boolean;
  effectiveType: string;
}

/** Every request for URLs matching `pattern`, split by who issued it. A request
 *  the SERVICE WORKER made is a real network download; one it did not is the
 *  page or a worker asking, which CacheFirst may well answer from cache. Only
 *  the first kind costs a user megabytes, so it is counted apart. */
function watchFetches(page: Page, pattern: RegExp) {
  const network: string[] = [];
  const all: string[] = [];
  page.context().on("request", (r) => {
    if (!pattern.test(r.url())) return;
    all.push(r.url());
    if (r.serviceWorker()) network.push(r.url());
  });
  return { network, all };
}

const TIER2_WASM = /ags4_wasm_full_bg-[^/]*\.wasm$/;
const DUCKDB_WASM = /duckdb-(eh|mvp)-[^/]*\.wasm$/;

const watchTier2Fetches = (page: Page) => watchFetches(page, TIER2_WASM);

/** A visit with the service worker already in control, which is what the warm
 *  needs to be observable: on a cold FIRST visit the SW is still installing when
 *  the idle tick fires, so the fetch goes straight to the network and the
 *  runtime rule never sees it. Every repeat visitor is in the controlled state. */
async function controlledVisit(page: Page) {
  await ready(page);
  await waitForServiceWorker(page);
  await ready(page);
  await waitForServiceWorker(page);
}

test("the warm waits for tier 1 — the two engines are never in flight together", async ({
  page,
}) => {
  test.setTimeout(180_000);
  // The criterion the design argues hardest for ("Why sequenced": tier 1 is on
  // the critical path, tier 2 is speculative, and fetching them together lets
  // the speculative one steal bandwidth from the needed one — landing exactly on
  // the sample-file path, where a user can go from cold paint to needing the
  // engine in milliseconds).
  //
  // It holds because App.tsx fires `warmLazyAssets()` from an effect gated on
  // engine readiness. Nothing else observed that, so moving the call out of the
  // gate — the one edit that breaks this — left every other test green.
  //
  // A COLD first visit, deliberately: no service worker in control yet, so both
  // engines come off the network where their order is visible. The link is
  // throttled because localhost finishes 2.1 MB before an overlap could be seen
  // at all, which would make this pass on any build.
  const cdp = await page.context().newCDPSession(page);
  await cdp.send("Network.emulateNetworkConditions", {
    offline: false,
    latency: 20,
    downloadThroughput: (4 * 1024 * 1024) / 8, // ~4 Mbps
    uploadThroughput: (1 * 1024 * 1024) / 8,
  });

  const order: string[] = [];
  page.context().on("requestfinished", (r) => {
    if (/ags4_wasm_bg-[^/]*\.wasm$/.test(r.url())) order.push("tier1-done");
  });
  page.context().on("request", (r) => {
    if (/ags4_wasm_full_bg-[^/]*\.wasm$/.test(r.url()))
      order.push("tier2-start");
  });

  await poseAsDevice(page, { saveData: false, effectiveType: "4g" });
  await ready(page);
  await expect
    .poll(() => order.includes("tier2-start"), {
      timeout: 120_000,
      message: "the tier-2 warm never fired, so there is no ordering to judge",
    })
    .toBe(true);

  // Everything that happened before the warm's FIRST request has to include tier
  // 1 finishing. Stated as a slice rather than two index comparisons so the one
  // failure that matters — the warm starting first, which leaves the slice empty
  // — reports itself as that, and not as "tier 1 was never fetched". (Only the
  // first of each is in play: the service worker precaches tier 1 too, and those
  // later duplicates say nothing about when the app's own engine became ready.)
  expect(
    order.slice(0, order.indexOf("tier2-start")),
    "the tier-2 warm started before tier 1 had finished downloading — the " +
      "speculative fetch is competing with the one on the critical path",
  ).toContain("tier1-done");
});

test("the idle warm fetches tier 2 without compiling it, and the Excel that follows refetches nothing", async ({
  page,
}) => {
  test.setTimeout(180_000);
  await poseAsDevice(page, { saveData: false, effectiveType: "4g" });
  const seen = watchTier2Fetches(page);
  await controlledVisit(page);

  await expect
    .poll(() => tier2Cached(page), {
      timeout: 90_000,
      message:
        "the idle warm never primed the tier-2 engine — Explore and Tools → " +
        "Excel are back to starting a 5.2 MB download on the click",
    })
    .toBeGreaterThan(0);

  // FETCH, never compile. Instantiating tier 2 means creating the second
  // worker, and warming must not: compiling ~5 MB of wasm for two tabs most
  // visitors never open hands back a good part of what the tiering won.
  expect(
    tier2Workers(page),
    "the warm compiled the tier-2 engine instead of only fetching it",
  ).toHaveLength(0);

  const beforeNetwork = seen.network.length;
  const beforeAll = seen.all.length;
  await tab(page, "Tools").click();
  await page.getByRole("button", { name: /^Excel$/ }).click();
  await expect(page.getByText(/AGS4 → Excel/)).toBeVisible();
  await expect.poll(() => tier2Workers(page).length).toBe(1);
  // Wait for the second worker to actually ASK for its engine before judging
  // whether it went to the network — otherwise "nothing was refetched" is just
  // "nothing has happened yet", and the assertion below passes on any build.
  await expect
    .poll(() => seen.all.length, {
      timeout: 30_000,
      message: "the second worker never requested the tier-2 engine at all",
    })
    .toBeGreaterThan(beforeAll);

  expect(
    seen.network.length,
    "opening Excel after a completed warm went back to the network — the warm " +
      "primed a URL the worker does not load, or CacheFirst is not serving it",
  ).toBe(beforeNetwork);
});

test("the idle warm downloads nothing under Data Saver", async ({ page }) => {
  // Counted on REQUESTS rather than on an empty cache: a 5.2 MB download that
  // simply hadn't finished yet would leave the cache empty too, so an absent
  // entry proves nothing. A request that was never made does.
  //
  // Two guards enforce this and either alone is enough — `warmLazyAssets`'
  // explicit Data Saver bail, and `isLowEndDevice()`, which reads `saveData` as
  // low-end in its own right. So this fails only when the warm is ungated
  // altogether, which is the regression worth catching; removing one guard is
  // caught by the unit suite instead.
  // Capable hardware, metered connection — so the ONLY thing that can stop the
  // warm here is Data Saver.
  await poseAsDevice(page, { saveData: true, effectiveType: "4g" });
  const seen = watchTier2Fetches(page);
  await controlledVisit(page);

  // Well past the 4 s requestIdleCallback deadline the warm is queued behind.
  await page.waitForTimeout(8_000);

  expect(
    seen.all,
    "Data Saver was ignored — 5.2 MB was speculatively downloaded on a " +
      "connection the user has told the browser to spare",
  ).toEqual([]);
  expect(await tier2Cached(page)).toBe(0);
});

/** Hold every network response matching `pattern` open until the returned
 *  release is called. Route-level, because the hold must land where the SERVICE
 *  WORKER's own fetch sees it — page-scoped CDP throttling does not reach it
 *  (#366 measured that directly: the SW's "throttled" request completed in
 *  7 ms), so a timing-based hold would silently test nothing. */
async function holdRequests(page: Page, pattern: RegExp) {
  let release!: () => void;
  const held = new Promise<void>((r) => (release = r));
  await page.context().route(pattern, async (route) => {
    try {
      await held;
      await route.continue();
    } catch {
      /* the page navigated or closed while its request was held — the aborted
         request is not the one under test */
    }
  });
  return release;
}

// The mirror of the race `isTier2Started()` guards (#366): not "the tab opened
// first, then the warm fired" but "the warm fired first and is STILL DOWNLOADING
// when the tab opens". The fix lives in the service worker (in-flight coalescing
// around CacheFirst — src/lib/swCoalesce.ts), which is what covers every path
// that creates the worker at once; these two hold the end-to-end claim, one per
// engine, because the warm exists to make exactly these clicks fast and in this
// window used to make them slower than no warm at all.

test("a tier-2 warm still in flight when Excel opens is ONE download, not two", async ({
  page,
}) => {
  test.setTimeout(180_000);
  await poseAsDevice(page, { saveData: false, effectiveType: "4g" });
  const seen = watchTier2Fetches(page);
  const release = await holdRequests(page, TIER2_WASM);

  await controlledVisit(page);
  // The warm's network download is up — and held at the route, so it is still
  // in flight however fast localhost answers.
  await expect
    .poll(() => seen.network.length, {
      timeout: 90_000,
      message:
        "the warm's own download never started, so there is no in-flight " +
        "race to judge (if the SW's fetch is invisible here, the route hold " +
        "cannot have applied either)",
    })
    .toBeGreaterThan(0);

  await tab(page, "Tools").click();
  await page.getByRole("button", { name: /^Excel$/ }).click();
  // The second worker exists while the warm's download is still held open, so
  // the race is real by construction — the flight cannot complete before
  // release(). Its own engine request cannot be watched directly: a request
  // the SW answers emits no context-level event here, which is precisely what
  // a coalesced request is. What stands in: the worker fetches at module
  // scope, within milliseconds of existing, and the settle below dwarfs that —
  // then the converter rendering proves the joined flight genuinely fed it.
  await expect.poll(() => tier2Workers(page).length).toBe(1);
  await page.waitForTimeout(1_500);
  // The hold is still holding — an empty cache at release time is what proves
  // the click landed mid-flight rather than after a warm that quietly
  // finished (if route interception ever stopped reaching the SW's fetch,
  // this is the assertion that says so instead of a vacuous pass).
  expect(await tier2Cached(page)).toBe(0);
  release();

  // The joined download genuinely served the worker: the converter renders and
  // the bytes reached the runtime cache.
  await expect(page.getByText(/AGS4 → Excel/)).toBeVisible({
    timeout: 90_000,
  });
  await expect
    .poll(() => tier2Cached(page), { timeout: 30_000 })
    .toBeGreaterThan(0);

  expect(
    seen.network,
    "opening Excel while the warm was still in flight started a second full " +
      "download — the service worker is not coalescing concurrent requests " +
      "for the same URL",
  ).toHaveLength(1);
});

test("a DuckDB warm still in flight when Explore opens is ONE download, not two", async ({
  page,
}) => {
  test.setTimeout(180_000);
  // Same race, third engine tier, ~36 MB: duck.ts's warmFetch no-ops once the
  // engine is INSTANTIATED, but a warm already downloading when Explore opens
  // holds no handle getDuckDb() could join — only the SW can merge the two.
  await poseAsDevice(page, { saveData: false, effectiveType: "4g" });
  const seen = watchFetches(page, DUCKDB_WASM);
  const release = await holdRequests(page, DUCKDB_WASM);

  await controlledVisit(page);
  await expect
    .poll(() => seen.network.length, {
      timeout: 90_000,
      message:
        "the DuckDB warm's own download never started, so there is no " +
        "in-flight race to judge",
    })
    .toBeGreaterThan(0);

  // Explore needs a file; the capable pose above means no engine gate — the
  // 36 MB fetch starts on the tab click alone.
  await page.getByRole("button", { name: /Rule 9.*unknown heading/ }).click();
  await tab(page, "Explore").click();
  // DuckDB spawns its worker BEFORE fetching its wasm, and the worker script
  // is precached — so a live duckdb worker while the warm's download is still
  // held is "the engine asked mid-flight" made observable. (The request itself
  // is invisible from out here once the SW answers it — see the tier-2 twin.)
  await expect
    .poll(
      () => page.workers().filter((w) => /duckdb-browser/.test(w.url())).length,
      {
        timeout: 60_000,
        message: "the Explore engine never started loading DuckDB",
      },
    )
    .toBeGreaterThan(0);
  await page.waitForTimeout(1_500);
  // As in the tier-2 twin: an empty bucket at release time proves the click
  // landed mid-flight, and catches a route hold that silently stopped
  // applying to the SW's own fetch.
  expect(
    await page.evaluate(async () => {
      const names = await caches.keys();
      if (!names.includes("ags-duckdb-wasm")) return 0;
      const keys = await (await caches.open("ags-duckdb-wasm")).keys();
      return keys.filter((k) => /duckdb-(eh|mvp)-.*\.wasm$/.test(k.url)).length;
    }),
  ).toBe(0);
  release();

  await expect(page.getByText(/data rows/)).toBeVisible({ timeout: 120_000 });

  expect(
    seen.network,
    "opening Explore while the DuckDB warm was still in flight started a " +
      "second ~36 MB download — the service worker is not coalescing " +
      "concurrent requests for the same URL",
  ).toHaveLength(1);
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
  // datetime.ags has TRAN_DATE = "05/06/2020" in a yyyy-mm-dd column — a Rule 8
  // miss the fixer rewrites to ISO. Both components are ≤ 12 and differ, so the
  // dd/mm reading is a genuine guess → RISKY, landing in the opt-in section, not
  // fix-all-safe. (An unambiguous date like 18/08/2020 would be safe-by-default.)
  await page
    .locator('input[type="file"]')
    .setInputFiles(fixture("datetime.ags"));
  await tab(page, "Fix").click();
  await expect(page.getByText(/Canonicalise datetime/)).toBeVisible();
  await expect(page.getByText(/2020-06-05/).first()).toBeVisible();
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
  await page
    .locator('input[type="file"]')
    .setInputFiles(fixture("fixable.ags"));

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

// --- boot: first paint gates on the tokenizer, not the engine ------------

test("first paint doesn't wait on the engine, and a file loaded in that window still validates", async ({
  browser,
}) => {
  // #353 (see ags-wiki/design/dec-engine-tiering.md): the engine's deadline is
  // when a FILE is loaded, not when the page paints. This holds it on the wire
  // and releases it by hand, so nothing here races the runner's speed — a
  // timed delay would only be as true as the machine was fast that day.
  //
  // Service workers are blocked for this context because page.route() cannot
  // see a fetch the SW answers (Playwright's own guidance). With the precache
  // live the engine would arrive from cache and the hold below would be a
  // no-op the test could not tell from a pass.
  const ctx = await browser.newContext({ serviceWorkers: "block" });
  const page = await ctx.newPage();
  let release!: () => void;
  const held = new Promise<void>((r) => (release = r));
  let requested = false;
  await page.route(/ags4_wasm_bg-.*\.wasm$/, async (route) => {
    requested = true;
    await held;
    await route.continue();
  });

  const sample = page.getByRole("button", { name: /Clean \(minimal\)/ });
  try {
    await page.goto(APP);

    // Tier 0 (the ~30 KB tokenizer) is the whole gate: the page paints and the
    // sample buttons are live while the engine is still in flight.
    await expect(sample).toBeVisible();
    // …and the hold is REAL — without this the route could simply never have
    // matched, and every assertion below would pass against a warm engine.
    await expect
      .poll(() => requested, {
        message: "the engine wasm was never requested — the hold isn't real",
      })
      .toBe(true);

    // The sample buttons are the path that takes someone from cold paint to
    // needing the engine in milliseconds. It waits inside Validate's EXISTING
    // loading state — no error, no new UI state.
    await sample.click();
    await expect(page.getByText(/Validating…/)).toBeVisible();
    await expect(page.getByText(/Clean — 0 findings/)).toHaveCount(0);

    // Engine lands → the queued request produces the normal result.
    release();
    await expect(page.getByText(/Clean — 0 findings/)).toBeVisible();
  } finally {
    release();
    await ctx.close();
  }
});

test("an engine that never arrives is reported, not left spinning", async ({
  browser,
}) => {
  // The other half of taking the engine out of the paint gate. Deferring the
  // FAILURE too is what #339 says must never happen, and it is not theoretical:
  // a Solid resource THROWS when read after an error, so ValidatePane's own
  // "Validator error: …" fallback never renders — with the failure left to the
  // panes this exact flow sat on "Validating…" for ever, the only trace an
  // uncaught `TypeError: Failed to fetch`. So App reports a dead engine even
  // though it no longer waits for a live one.
  const ctx = await browser.newContext({ serviceWorkers: "block" });
  const page = await ctx.newPage();
  await page.route(/ags4_wasm_bg-.*\.wasm$/, (r) => r.abort());
  try {
    await page.goto(APP);
    await expect(
      page.getByText(/Failed to load the validator engine/),
    ).toBeVisible();
    await expect(page.getByText(/Validating…/)).toHaveCount(0);
  } finally {
    await ctx.close();
  }
});

// --- a tier-2 engine that won't fetch: partial, and recoverable -----------

// Tier 1 is precached, so a tier-2 fetch failure is genuinely PARTIAL and has
// to read that way (#357, ags-wiki/design/dec-engine-tiering.md): only the tab
// that needed the second engine is out, and only until a retry succeeds.
//
// Both tests below block service workers for the reason the two boot tests
// above do — page.route() cannot see a fetch the SW answers. Here it matters
// twice over: with the CacheFirst rule live, a retry could be served from cache
// and pass without anything having been re-fetched, which is the one thing
// these tests exist to prove.
//
// What makes them falsifiable is the fetch counter. An abort route that never
// matched, or a retry that re-read a settled rejection instead of asking the
// network again, both leave it standing still — so the count is asserted to
// GROW across the retry, not merely to be non-zero.
async function blockTier2(page: Page) {
  const state = { blocked: true, fetches: 0 };
  await page.route(/ags4_wasm_full_bg-[^/]*\.wasm$/, async (route) => {
    state.fetches++;
    if (state.blocked) await route.abort();
    else await route.continue();
  });
  return state;
}

test("Tools → Excel reports an engine it can't fetch, and converts once a retry succeeds", async ({
  browser,
}) => {
  const ctx = await browser.newContext({ serviceWorkers: "block" });
  const page = await ctx.newPage();
  const tier2 = await blockTier2(page);
  try {
    await page.goto(APP);
    await page.getByRole("button", { name: /Clean \(minimal\)/ }).click();
    await expect(page.getByText(/Clean — 0 findings/)).toBeVisible();

    await tab(page, "Tools").click();
    await page.getByRole("button", { name: /^Excel$/ }).click();
    await page.getByRole("button", { name: /Download as Excel/ }).click();

    // Reported on the tab that needed it — not a spinner, and not silence.
    await expect(page.getByText(/engine couldn't be downloaded/)).toBeVisible();
    expect(tier2.fetches).toBeGreaterThan(0); // the block is real
    const beforeRetry = tier2.fetches;

    // …and the failure is scoped to it. The page-level engine banner belongs to
    // tier 1 and must stay silent: a precached engine did not fail.
    await expect(
      page.getByText(/Failed to load the validator engine/),
    ).toHaveCount(0);

    // The cause is fixed. The retry has to reach the network again — which it
    // can only do if the dead worker was dropped rather than kept.
    tier2.blocked = false;
    const [xlsx] = await Promise.all([
      page.waitForEvent("download"),
      page.getByRole("button", { name: /^Try again$/ }).click(),
    ]);
    expect(xlsx.suggestedFilename()).toMatch(/\.xlsx$/);
    await expect(page.getByText(/→ \.xlsx/)).toBeVisible();
    await expect(page.getByText(/engine couldn't be downloaded/)).toHaveCount(
      0,
    );
    expect(tier2.fetches).toBeGreaterThan(beforeRetry);

    // No page reload happened, and tier 1 never stopped working: a fresh
    // validate still runs in the always-on worker. The sample list collapses
    // once a file is loaded, so re-open it before reaching for a sample.
    await tab(page, "Validate").click();
    await page.getByText(/Or try a sample/).click();
    await page.getByRole("button", { name: /Rule 9.*unknown heading/ }).click();
    await expect(page.getByText("✗").first()).toBeVisible();
  } finally {
    await ctx.close();
  }
});

test("Explore reports an engine it can't fetch, and ingests once a retry succeeds", async ({
  browser,
}) => {
  // DuckDB is fetched and compiled before the parse that fails, and with the
  // service worker blocked none of it comes from a cache.
  test.setTimeout(180_000);
  const ctx = await browser.newContext({ serviceWorkers: "block" });
  const page = await ctx.newPage();
  const tier2 = await blockTier2(page);
  try {
    await page.goto(APP);
    await expect(
      page.getByRole("button", { name: /Clean \(minimal\)/ }),
    ).toBeVisible();
    await page
      .locator('input[type="file"]')
      .setInputFiles(fixture("coords.ags"));

    await tab(page, "Explore").click();
    // The low-end gate holds the DuckDB download on a runner fingerprinted
    // low-end (see enterExplore in helpers.ts); race it against the failure so
    // a capable runner pays no extra wait.
    const failure = page.getByText(/engine couldn't be downloaded/);
    const gate = page.getByRole("button", { name: /^Continue$/ });
    await expect(failure.or(gate).first()).toBeVisible({ timeout: 120_000 });
    if (await gate.isVisible().catch(() => false)) await gate.click();

    // The pane REPORTS. It reached this by rendering an errored resource's
    // fallback — the state that used to spin for ever because reading the
    // resource threw before its own error branch could paint.
    await expect(failure).toBeVisible({ timeout: 120_000 });
    expect(tier2.fetches).toBeGreaterThan(0);
    const beforeRetry = tier2.fetches;

    tier2.blocked = false;
    await page.getByRole("button", { name: /^Try again$/ }).click();
    await expect(page.getByText(/data rows/)).toBeVisible({ timeout: 120_000 });
    expect(tier2.fetches).toBeGreaterThan(beforeRetry);
  } finally {
    await ctx.close();
  }
});

// --- a worker that dies is not reused --------------------------------------

// The other way an engine goes missing (#363): not a wasm that won't download,
// but a WORKER that won't run — a script that fails to load, or one that dies.
// Blocking the chunk is what reaches it, and the failure it guards against is
// subtler than "no engine": rejecting the requests in flight was never the whole
// job, because the channel then still pointed at the corpse. Those requests
// reported and every request AFTER them was posted into silence — and a hang
// never rejects, so no error branch was ever reached.
//
// Opening Explore starts the worker before any parse, so the parse that follows
// is exactly one of those later requests. That is why this goes red on the tab
// rather than merely losing a retry.
test("a dead engine worker is not reused: Explore reports, and the always-on worker carries on", async ({
  browser,
}) => {
  const ctx = await browser.newContext({ serviceWorkers: "block" });
  const page = await ctx.newPage();
  let blocked = 0;
  // The tier-2 worker's own chunk, not its wasm — this is the script that never
  // arrives, so the Worker fires `error` and no engine ever gets as far as
  // failing to instantiate.
  await page.route(/tier2\.worker-[^/]*\.js$/, async (route) => {
    blocked++;
    await route.abort();
  });
  try {
    await page.goto(APP);
    await page.getByRole("button", { name: /Clean \(minimal\)/ }).click();
    await expect(page.getByText(/Clean — 0 findings/)).toBeVisible();

    await tab(page, "Explore").click();
    const failure = page.getByText(/engine (stopped|couldn't be downloaded)/);
    const gate = page.getByRole("button", { name: /^Continue$/ });
    await expect(failure.or(gate).first()).toBeVisible({ timeout: 120_000 });
    if (await gate.isVisible().catch(() => false)) await gate.click();
    await expect(failure).toBeVisible({ timeout: 120_000 });
    await expect(
      page.getByRole("button", { name: /^Try again$/ }),
    ).toBeVisible();
    expect(blocked).toBeGreaterThan(0); // the block is real

    // Each attempt spawns a fresh worker, so each attempt REJECTS — the
    // reporting is repeatable rather than a one-off from the batch that
    // happened to be in flight when it died.
    const beforeRetry = blocked;
    await page.getByRole("button", { name: /^Try again$/ }).click();
    await expect(failure).toBeVisible({ timeout: 120_000 });
    expect(blocked).toBeGreaterThan(beforeRetry);

    // One worker dying leaves the other's tabs alone — the process boundary is
    // half the reason the split exists. A fresh validate still runs.
    await tab(page, "Validate").click();
    await page.getByText(/Or try a sample/).click();
    await page.getByRole("button", { name: /Rule 9.*unknown heading/ }).click();
    await expect(page.getByText("✗").first()).toBeVisible();
    await expect(
      page.getByText(/Failed to load the validator engine/),
    ).toHaveCount(0);
  } finally {
    await ctx.close();
  }
});

// #379: the transport worker's death must not wedge the tool. Its old client
// rejected the requests in flight but kept the dead worker, so the NEXT
// encrypt posted into the corpse and its promise never settled — spinner for
// ever, no error, and the pane's own error branch unreachable. Now the channel
// retires a dead worker, so the user's own retry is the recovery: no banner,
// no dedicated button, just the next click working (the asymmetry with
// Explore/Excel is deliberate — see #379).
test("Tools → Transport reports a dead worker, and the plain retry gets a fresh one", async ({
  browser,
}) => {
  test.slow(); // scrypt at the shipped work factor costs real seconds, twice
  const ctx = await browser.newContext({ serviceWorkers: "block" });
  const page = await ctx.newPage();
  let blocked = 0;
  let arm = true;
  // The worker's own chunk — the script never arrives, the Worker fires
  // `error`, and no zstd/age ever gets as far as initialising.
  await page.route(/transport\.worker-[^/]*\.js$/, async (route) => {
    if (arm) {
      blocked++;
      await route.abort();
    } else {
      await route.continue();
    }
  });
  try {
    await page.goto(APP);
    await page.getByRole("button", { name: /Clean \(minimal\)/ }).click();
    await expect(page.getByText(/Clean — 0 findings/)).toBeVisible();

    await tab(page, "Tools").click();
    await page.getByRole("button", { name: /^Transport$/ }).click();
    await page
      .locator('input[type="password"]')
      .first()
      .fill("correct-horse-e2e");
    await page.getByRole("button", { name: /Encrypt & download/ }).click();

    // The failure must REPORT — the hang this replaces showed nothing at all.
    await expect(page.locator("p.text-err")).toBeVisible({ timeout: 120_000 });
    expect(blocked).toBeGreaterThan(0); // the block is real, not a passed test by accident

    // Release the network. The plain retry must spawn a FRESH worker and
    // finish: with the old client this click posted into the dead one and the
    // download below never fired.
    arm = false;
    const [dl] = await Promise.all([
      page.waitForEvent("download", { timeout: 120_000 }),
      page.getByRole("button", { name: /Encrypt & download/ }).click(),
    ]);
    expect(dl.suggestedFilename()).toMatch(/\.zst\.age$/);
    await expect(page.getByText(/Encrypted /)).toBeVisible();
  } finally {
    await ctx.close();
  }
});
