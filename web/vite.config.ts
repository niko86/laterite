import { defineConfig, type Plugin } from "vite";
import solid from "vite-plugin-solid";
import tailwindcss from "@tailwindcss/vite";
import { VitePWA } from "vite-plugin-pwa";
import {
  copyFileSync,
  existsSync,
  globSync,
  mkdirSync,
  renameSync,
} from "node:fs";
import { dirname, resolve } from "node:path";

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

// A cold visit (no service worker yet) to an unknown in-scope path must still
// boot the app rather than hard-404. On Cloudflare that is `not_found_handling:
// "single-page-application"` in wrangler.jsonc, which needs no file — this 404
// copy is the equivalent trick for a host that has no such setting and serves
// 404.html instead, which is what still fronts the legacy niko86.github.io URLs.
// Routing is hash-only (every shared link is <base>#…), so it only ever matters
// for stray paths; it costs one file, so it stays until that host does not.
// Runs in closeBundle, after index.html is finalised (manifest link injected)
// and after the SW precache manifest is computed, so 404.html is intentionally
// NOT itself precached (it's only for the pre-SW cold case).
function spa404Fallback(): Plugin {
  let outDir = "dist";
  return {
    name: "spa-404-fallback",
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

// The DuckDB engine wasm cannot live with the rest of the app.
//
// Cloudflare caps a SINGLE static asset at 25 MiB — on Workers and on Pages
// alike, so this is not a choice between the two hosts. The two DuckDB bundles
// are 34 MiB (EH) and 39 MiB (MVP), and `wrangler deploy` refuses the whole
// upload over either. Cloudflare's own answer for anything larger is R2.
//
// So when VITE_DUCKDB_CDN is set, those two files are served from there and
// MOVED OUT of the build output. Both halves are required: rewriting the URL
// without moving the file still ships 73 MiB the deploy rejects, and moving it
// without rewriting gives a 404 on first Explore click.
//
// Unset (dev, and any build that isn't deploying to Cloudflare) changes
// nothing — the files stay put and load from the same origin, exactly as
// before. The DuckDB wasm is already `globIgnore`d from the PWA precache, so
// removing it cannot disturb the service worker manifest.
const DUCKDB_CDN = process.env.VITE_DUCKDB_CDN?.replace(/\/*$/, "/");
const DUCKDB_WASM = /(^|\/)duckdb-(eh|mvp)-[\w-]+\.wasm$/;

function offloadDuckdbWasm(): Plugin {
  let outDir = "dist";
  return {
    name: "offload-duckdb-wasm-to-r2",
    apply: "build",
    configResolved(cfg) {
      outDir = cfg.build.outDir;
    },
    // closeBundle, so it runs after the PWA plugin has computed its precache
    // manifest — the files are excluded from it either way, but moving them
    // earlier would make that dependency load-bearing and invisible.
    closeBundle() {
      if (!DUCKDB_CDN) return;
      const staging = resolve(outDir, "..", "r2-upload");
      for (const rel of globSync("assets/*.wasm", { cwd: outDir })) {
        if (!DUCKDB_WASM.test(rel)) continue;
        const to = resolve(staging, rel);
        mkdirSync(dirname(to), { recursive: true });
        renameSync(resolve(outDir, rel), to);
        console.log(`[duckdb-r2] ${rel} -> r2-upload/${rel}`);
      }
    },
  };
}

// Exported so `src/lib/sw-cache-policy.test.ts` can assert the caching policy
// over every rule. Inline in the VitePWA options it was unreachable from a
// test, and the rule that made a server fault permanent per-device (#339) was
// the kind only a test over the whole array catches.
// Derived from VitePWA's own options rather than imported from `workbox-build`.
// That package supplies the type, but it reaches us only as a transitive dep of
// vite-plugin-pwa — importing it directly would be a phantom dependency that
// `tsc` resolves today purely because npm happens to hoist it.
type RuntimeCachingRule = NonNullable<
  NonNullable<
    NonNullable<Parameters<typeof VitePWA>[0]>["workbox"]
  >["runtimeCaching"]
>[number];

export const RUNTIME_CACHING: RuntimeCachingRule[] = [
  {
    // DuckDB engine wasm — 36 MB (EH) + 41 MB (MVP). Fingerprinted +
    // immutable, so CacheFirst is safe and avoids any revalidation.
    urlPattern: ({ url }) =>
      /\/duckdb-(eh|mvp)-[^/]*\.wasm$/.test(url.pathname),
    handler: "CacheFirst",
    options: {
      cacheName: "ags-duckdb-wasm",
      // 200 only. Status 0 is an OPAQUE response — what a cross-origin fetch
      // degrades to when it is refused — and CacheFirst never revalidates, so
      // accepting it writes a failure that is then served until expiry. That is
      // not hypothetical: on 2026-08-16 this bucket had no CORS configuration,
      // the fetch was blocked, and the failure was cached; the server fix was
      // minutes, but each affected device needed `caches.delete()` in a console
      // to recover. Nothing here is opaque — the CDN answers with
      // `access-control-allow-origin` and no `no-cors` fetch exists in the app —
      // so 0 kept nothing alive except the bug.
      cacheableResponse: { statuses: [200] },
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
      // 200 only — see the DuckDB rule above. This one is same-origin out of
      // the app's own dist/, so it could never have been opaque in the first
      // place; the 0 was copied, not reasoned about.
      cacheableResponse: { statuses: [200] },
      expiration: {
        maxEntries: 2,
        maxAgeSeconds: 60 * 60 * 24 * 180,
        purgeOnQuotaError: true,
      },
    },
  },
];

// `base` is the single deploy-location knob, and it lives here and nowhere
// else because a wrong base 404s every asset on the site.
//
// It is `/` because the site now answers on its own domain (laterite.dev),
// where GitHub serves the repo at the root. It was `/laterite/` for the
// project-Pages path, and `deploy-validator.yml` still exposes `base` as a
// workflow_dispatch input — which is what makes the cutover a single build
// rather than a broken window: the domain and this default cannot both change
// atomically, so dispatch with the other value to bridge.
export default defineConfig({
  base: process.env.VITE_BASE ?? "/",
  experimental: {
    // Rewrite ONLY the two oversized DuckDB bundles to the R2 origin. Every
    // other asset keeps the default base-relative URL; returning undefined
    // is how this hook says "leave it alone".
    renderBuiltUrl(filename) {
      if (DUCKDB_CDN && DUCKDB_WASM.test(filename))
        return DUCKDB_CDN + filename;
      return undefined;
    },
  },
  plugins: [
    solid(),
    tailwindcss(),
    stripDuckdbWorkerSourcemaps(),
    offloadDuckdbWasm(),
    // PWA: installable + offline. The caching split is the whole point here.
    // PRECACHE (downloaded at install, then served offline) = the full app
    // shell: EVERY JS/CSS chunk — including the Explore/Charts/Coordinates UI
    // and the DuckDB *worker* glue — plus the reference JSONs, the sample
    // files, and the ~6.9 MiB *validator* wasm. So Validate/Fix/the dictionary
    // work fully offline after one visit, and the Explore/Charts/Coordinates
    // UIs render offline too; only their heavy *engines* are deferred.
    //
    // That install costs **~11.9 MiB across 45 entries** — the figure vite-plugin-pwa
    // prints as `precache N entries (… KiB)` on every build, which is where to
    // re-read it rather than trusting this comment. It said ~5.85 MiB until
    // 2026-08-16 and had been wrong by more than 2x since the Excel converter
    // landed: the validator wasm alone grew 3.3 MB -> 4.8 MB -> 6.6 MB across
    // three features (see maximumFileSizeToCacheInBytes below, which tracks the
    // same growth and WAS kept current, because a stale number there fails the
    // build and a stale number here fails nobody).
    //
    // NEVER precached: the DuckDB engine wasm (36 MB EH + 41 MB MVP)
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
        // to Vite's `base`, so they track the deploy knob rather than pinning
        // a second copy of it here.
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
        // "load sample"); the two validator wasms are named in explicitly (the
        // `.wasm` extension is deliberately NOT in the glob above so the heavy
        // DuckDB engine can't slip in). The tiny tokenizer wasm (#533) is
        // boot-critical — the app gates first render on it — so it MUST be
        // precached or the app never renders offline.
        globPatterns: [
          "**/*.{js,css,html,ico,svg,json,webmanifest,ags,txt,png,woff,woff2}",
          "assets/ags4_wasm_bg-*.wasm",
          "assets/ags4_tokenizer_bg-*.wasm",
        ],
        // Belt-and-braces with the runtimeCaching rules below: keep the heavy
        // assets out of the install precache no matter how the globs evolve.
        globIgnores: ["assets/duckdb-*.wasm", "grids/**", "**/*.map"],
        // 8 MiB clears the ~6.6 MB validator wasm (the Excel converter — calamine
        // + rust_xlsxwriter for xlsx_to_ags4/ags4_to_xlsx — pushed it past the old
        // 6 MiB cap, which silently failed the whole SW build) but still sits far
        // below the 36 MB DuckDB wasm, so even if a glob slipped, the engine can't
        // precache. Size history: 4 MiB/~3.3 MB (AGS4 producer) → 6 MiB/~4.8 MB
        // (content-addressed keychain, #303) → 8 MiB/~6.6 MB (Excel).
        maximumFileSizeToCacheInBytes: 8 * 1024 * 1024,
        cleanupOutdatedCaches: true,
        // First-install SW controls the page immediately, so an offline reload
        // right after the first visit is already served from cache.
        clientsClaim: true,
        // SPA offline reload → serve the app shell. The plugin base-prefixes
        // this under Vite's `base`. Only fires for navigation requests (Workbox
        // guards on request.mode === 'navigate'), so asset/JSON fetches are
        // untouched.
        navigateFallback: "index.html",
        // Never answer a top-level navigation with the app shell when a real
        // file should serve itself: a final path segment with a dot-extension,
        // which is how the runtime-cached .wasm/.gsb reach their own handlers.
        //
        // A second entry for `/docs/` used to sit here because MkDocs published
        // into the app's own output, one origin, where this worker's scope
        // covered it. The docs now answer on their own host, outside the scope
        // — so the guard cannot fire, and keeping it would only make a future
        // reader work out that it can't.
        navigateFallbackDenylist: [/\/[^/?]+\.[^/?]+$/],
        runtimeCaching: RUNTIME_CACHING,
      },
    }),
    // After VitePWA, so it copies the FINAL index.html (manifest link injected).
    spa404Fallback(),
  ],
  build: {
    // The wasm + (Phase 2) DuckDB/ECharts chunks are large; raise the
    // warn limit so the build log isn't noisy about expected sizes.
    chunkSizeWarningLimit: 4096,
  },
});
