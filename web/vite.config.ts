import { defineConfig, type Plugin } from "vite";
import solid from "vite-plugin-solid";
import tailwindcss from "@tailwindcss/vite";
import { VitePWA } from "vite-plugin-pwa";
import { copyFileSync, existsSync } from "node:fs";
import { resolve } from "node:path";

// The DuckDB-wasm worker bundles carry a `//# sourceMappingURL=…worker.js.map`
// comment, but we copy the worker via `?url` (fingerprinted) and never emit
// the .map — so the browser resolves the comment to a non-existent
// `assets/duckdb-browser-*.worker.js.map` and logs a harmless 404 (noticeable
// in mobile / Safari consoles). Strip the comment from the emitted worker
// assets; a third-party prebuilt worker's source map isn't useful to us.
function stripDuckdbWorkerSourcemaps(): Plugin {
  return {
    name: "strip-duckdb-worker-sourcemaps",
    apply: "build",
    generateBundle(_options, bundle) {
      for (const [file, item] of Object.entries(bundle)) {
        if (item.type !== "asset") continue;
        if (!/duckdb-browser-.*\.worker.*\.js$/.test(file)) continue;
        const src =
          typeof item.source === "string"
            ? item.source
            : new TextDecoder().decode(item.source);
        item.source = src.replace(/\/\/[#@]\s*sourceMappingURL=\S+\s*$/m, "");
      }
    },
  };
}

// GitHub Pages serves 404.html for any path it doesn't recognise. Copy the
// built index.html to 404.html so a COLD visit (no service worker yet) to a
// mistyped or unknown in-scope URL still boots the app instead of a hard 404.
// Routing is hash-only (every shared link is /laterite/#…), so this only
// matters for stray paths — but it's the conventional Pages SPA hardening and
// costs one file. Runs in closeBundle, after index.html is finalised (manifest
// link injected) and after the SW precache manifest is computed, so 404.html
// is intentionally NOT itself precached (it's only for the pre-SW cold case).
function githubPagesSpaFallback(): Plugin {
  let outDir = "dist";
  return {
    name: "gh-pages-404-fallback",
    apply: "build",
    configResolved(cfg) {
      outDir = cfg.build.outDir;
    },
    closeBundle() {
      const idx = resolve(outDir, "index.html");
      if (existsSync(idx)) copyFileSync(idx, resolve(outDir, "404.html"));
    },
  };
}

// `base` is the single deploy-location knob. Private test repo now
// (GitHub Pages serves it at /laterite/); the future public home
// niko86/laterite is a one-line flip via VITE_BASE=/laterite/. A wrong
// base 404s every asset, so it lives here and nowhere else.
export default defineConfig({
  base: process.env.VITE_BASE ?? "/laterite/",
  plugins: [
    solid(),
    tailwindcss(),
    stripDuckdbWorkerSourcemaps(),
    // PWA: installable + offline. The caching split is the whole point here.
    // PRECACHE (downloaded at install, ~5.85 MiB, then served offline) = the
    // full app shell: EVERY JS/CSS chunk — including the Explore/Charts/
    // Coordinates UI and the DuckDB *worker* glue — plus the reference JSONs,
    // the sample files, and the 2.2 MB *validator* wasm. So Validate/Fix/the
    // dictionary work fully offline after one visit, and the Explore/Charts/
    // Coordinates UIs render offline too; only their heavy *engines* are
    // deferred. NEVER precached: the DuckDB engine wasm (36 MB EH + 41 MB MVP)
    // and the 15 MB OSTN15 grid — 92 MB we refuse to pull on every install.
    // Those are `globIgnore`d here and instead runtime-cached CacheFirst the
    // FIRST time they're actually fetched (the idle-warm in lib/prefetch.ts
    // only fetches DuckDB on a fast, non-metered link; otherwise it waits for a
    // real Explore/Coordinates click) — so they cost nothing until used, then
    // work offline thereafter.
    VitePWA({
      // 'prompt', not 'autoUpdate': a user mid-analysis (file loaded, query
      // running) shouldn't have the page reloaded out from under them. The
      // PwaUpdater toast lets them choose when to take the new version.
      registerType: "prompt",
      // PwaUpdater.tsx registers via `virtual:pwa-register/solid` so it can
      // own the update/offline-ready UI — don't also auto-inject a registrar.
      injectRegister: false,
      // (No `includeAssets`: apple-touch-icon is referenced in index.html so
      // the plugin auto-includes it, and every icons/*.png is already matched
      // by the `png` globPattern below — listing it again only duplicated the
      // precache entry.)
      manifest: {
        name: "AGS4 Validator + Data Explorer",
        short_name: "AGS4 Validator",
        description:
          "Validate, fix and explore AGS4 geotechnical transfer files entirely in your browser — nothing is uploaded.",
        // Dark Primer canvas (= app's --canvas in dark) for a polished splash
        // + task-switcher; the running app's status bar is themed dynamically
        // by the media-queried <meta name="theme-color"> in index.html.
        theme_color: "#0d1117",
        background_color: "#0d1117",
        display: "standalone",
        orientation: "any",
        categories: ["productivity", "utilities"],
        // scope + start_url intentionally omitted → the plugin defaults them
        // to Vite's `base` (/laterite/), so they track the deploy knob.
        icons: [
          { src: "icons/icon-128.png", sizes: "128x128", type: "image/png" },
          { src: "icons/icon-256.png", sizes: "256x256", type: "image/png" },
          // No `maskable` variant: the brand mark carries a wordmark in its
          // lower fifth that Android's adaptive-icon safe-zone would crop —
          // declaring only "any" letterboxes it cleanly instead.
          { src: "icons/icon-512.png", sizes: "512x512", type: "image/png" },
        ],
      },
      workbox: {
        // App-shell precache. `ags` pulls in the tiny sample files (offline
        // "load sample"); the validator wasm is named in explicitly.
        globPatterns: [
          "**/*.{js,css,html,ico,svg,json,webmanifest,ags,txt,png,woff,woff2}",
          "assets/ags4_wasm_bg-*.wasm",
        ],
        // Belt-and-braces with the runtimeCaching rules below: keep the heavy
        // assets out of the install precache no matter how the globs evolve.
        globIgnores: ["assets/duckdb-*.wasm", "grids/**", "**/*.map"],
        // 6 MiB clears the ~4.8 MB validator wasm (the content-addressed keychain
        // — #303 Phase 5 — added ~0.8 MB: laterite-ags4-core's dictionary registry;
        // gzip is ~1.15 MB) but still sits far below the 36 MB DuckDB wasm, so even
        // if a glob slipped, the engine can't precache. Was 4 MiB / ~3.3 MB when
        // the AGS4 producer (to_ags4 + to_ags4_ipc) was the last size bump.
        maximumFileSizeToCacheInBytes: 6 * 1024 * 1024,
        cleanupOutdatedCaches: true,
        // First-install SW controls the page immediately, so an offline reload
        // right after the first visit is already served from cache.
        clientsClaim: true,
        // SPA offline reload → serve the app shell. The plugin base-prefixes
        // this to /laterite/index.html. Only fires for navigation requests
        // (Workbox guards on request.mode === 'navigate'), so asset/JSON
        // fetches are untouched.
        navigateFallback: "index.html",
        // Never answer a top-level navigation with the app shell when it should
        // resolve elsewhere: (1) file-like URLs (a final path segment with a
        // dot-extension) — let real assets / the runtime-cached .wasm/.gsb serve
        // themselves; (2) `/docs/` — the MkDocs site (published alongside the app
        // at /laterite/docs/) is its own static site, so the app's service worker
        // must not intercept a navigation into it.
        navigateFallbackDenylist: [/\/[^/?]+\.[^/?]+$/, /\/docs\//],
        runtimeCaching: [
          {
            // DuckDB engine wasm — 36 MB (EH) + 41 MB (MVP). Fingerprinted +
            // immutable, so CacheFirst is safe and avoids any revalidation.
            urlPattern: ({ url }) =>
              /\/duckdb-(eh|mvp)-[^/]*\.wasm$/.test(url.pathname),
            handler: "CacheFirst",
            options: {
              cacheName: "ags-duckdb-wasm",
              cacheableResponse: { statuses: [0, 200] },
              expiration: {
                // selectBundle() picks ONE variant per browser (EH or MVP), so
                // a device caches one ~38 MB wasm per build. Cap at 2 ⇒ at most
                // the current build + one stale generation as an update-window
                // fallback, never an unbounded pile of old fingerprinted wasm.
                maxEntries: 2,
                maxAgeSeconds: 60 * 60 * 24 * 60,
                purgeOnQuotaError: true, // evict under storage pressure, don't error
              },
            },
          },
          {
            // OSTN15 NTv2 grid — ~15 MB, fetched only when "Precise (OSTN15)"
            // coordinates are ticked. Immutable.
            urlPattern: ({ url }) => /\/grids\/.*\.gsb$/.test(url.pathname),
            handler: "CacheFirst",
            options: {
              cacheName: "ags-ostn15-grid",
              cacheableResponse: { statuses: [0, 200] },
              expiration: {
                maxEntries: 2,
                maxAgeSeconds: 60 * 60 * 24 * 180,
                purgeOnQuotaError: true,
              },
            },
          },
        ],
      },
    }),
    // After VitePWA, so it copies the FINAL index.html (manifest link injected).
    githubPagesSpaFallback(),
  ],
  build: {
    // The wasm + (Phase 2) DuckDB/ECharts chunks are large; raise the
    // warn limit so the build log isn't noisy about expected sizes.
    chunkSizeWarningLimit: 4096,
  },
});
