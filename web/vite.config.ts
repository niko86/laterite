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

// The runtime caching rules live in `src/lib/swPolicy.ts` now: the service
// worker is hand-written (`src/sw.ts`, see below) and consumes them there, and
// the policy test (`src/lib/sw-cache-policy.test.ts`) asserts over them there.

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
  resolve: {
    alias: {
      // The shared surface layer (#394): tokens now, primitives next (#406).
      // landing/vite.config.ts resolves this to the same directory, which is
      // what makes "a button exists once" structural rather than a convention.
      "@shared": resolve(import.meta.dirname, "src/shared"),
    },
  },
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
    // files, and the **tier-1** engine wasm — held to gzip and raw ceilings by
    // `tools/release/check-wasm-tier1.mjs`, which is where those numbers live. So
    // Validate, Fix, Export and ALL of Tools work fully offline after one visit,
    // and the Explore/Charts/Coordinates UIs render offline too; only their heavy
    // *engines* are deferred.
    //
    // The install's total weight is deliberately NOT stated here (#345):
    // vite-plugin-pwa prints it as `precache N entries (… KiB)` on every build, so
    // read it from a build. Three hand-written copies of that reading rotted, this
    // one included. See CLAUDE.md, *Conventions* — measured values go in gates,
    // not comments.
    //
    // What the tiering did to that install is the part worth keeping, and it is
    // history rather than a reading — the precached engine went 3.3 MB (AGS4
    // producer) → 4.8 MB (content-addressed keychain, #303) → 6.6 MB (Excel), and
    // then #355 swapped it for the engine minus `arrow` and `excel`, cutting the
    // install by about 4.5 MiB in a day.
    //
    // NEVER precached: the DuckDB engine wasm (36 MB EH + 41 MB MVP), the 15 MB
    // OSTN15 grid, and now the **tier-2** engine — the full build, which
    // only Explore and Tools → Excel need. All three are `globIgnore`d here and
    // instead runtime-cached CacheFirst the FIRST time they're actually fetched
    // (the idle-warm in lib/prefetch.ts only fetches DuckDB on a fast,
    // non-metered link; otherwise it waits for a real Explore/Coordinates click)
    // — so they cost nothing until used, then work offline thereafter.
    VitePWA({
      // `injectManifest`, not `generateSW`, since #366: the coalescing the
      // warm-vs-worker race needs is a custom handler around CacheFirst, and
      // `generateSW`'s `runtimeCaching` rules cannot carry one. The worker
      // itself is `src/sw.ts`; this plugin still computes and injects the
      // precache manifest from the globs below, so the #355 locks keep living
      // here, beside each other.
      strategies: "injectManifest",
      srcDir: "src",
      filename: "sw.ts",
      // 'prompt', not 'autoUpdate': a user mid-analysis (file loaded, query
      // running) shouldn't have the page reloaded out from under them. The
      // PwaUpdater toast lets them choose when to take the new version.
      // (src/sw.ts carries the SKIP_WAITING listener this mode needs.)
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
        // The pairing's dark canvas (= --canvas in dark, shared colors.css)
        // for a polished splash + task-switcher; the running app's status bar
        // is themed dynamically by the media-queried <meta name="theme-color">
        // in index.html.
        theme_color: "#14100f",
        background_color: "#14100f",
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
      injectManifest: {
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
        //
        // `ags4_wasm_full_bg-*.wasm` is tier 2 (#355), and it is the fragile one.
        // The two engine builds come out of the same crate, and with the same
        // `--out-name` they would fingerprint to two hashes with the same STEM —
        // at which case the tier-1 glob above matches BOTH, the install carries
        // the full engine again, and nothing errors. The distinct `--out-name` in
        // `web/package.json` is what makes them separable; this is the second
        // lock, and the e2e assertion that tier 2 is ABSENT from the precache is
        // the third — the only one that actually fails, since every size ceiling
        // here has room for it.
        globIgnores: [
          "assets/duckdb-*.wasm",
          "assets/ags4_wasm_full_bg-*.wasm",
          "grids/**",
          "**/*.map",
          // Fontsource's @font-face blocks are unicode-range split per script
          // (#403): a latin reader fetches the latin files and nothing else.
          // Precaching the other scripts would be the ONE way those files ever
          // get requested — the install would download what no page view does.
          "assets/*-latin-ext-*.woff2",
          "assets/*-vietnamese-*.woff2",
        ],
        // 3 MiB, down from 8 (#355) — and the drop is what makes this a guard
        // again. The precache carries TIER 1 now, not the full engine, so this
        // cap can sit in the GAP between the two builds: above the raw ceiling
        // `tools/release/check-wasm-tier1.mjs` holds tier 1 to, and below what
        // tier 2 weighs. At 8 MiB it could not catch a leaked tier 2 at all — the
        // full engine fits under it, which is exactly why the globs above needed
        // a third lock rather than a bigger number. Both bounds live where they
        // are enforced; restating either here is how the pair silently uncouples.
        //
        // The two ceilings move together: raise the tier-1 gate past this and the
        // engine stops being precached, which costs offline validate. That is
        // not silent — vite-plugin-pwa prints "<file> is N MB, and won't be
        // precached" — but a warning in a build log is not a failing check, so
        // the tier-1 gate above is what actually holds the pair apart.
        //
        // Measured, not assumed: widening the glob above to match both engines
        // makes this cap fire on tier 2 with exactly that warning, and the app
        // still builds. Size history: 4 MiB/~3.3 MB (AGS4 producer) →
        // 6 MiB/~4.8 MB (content-addressed keychain, #303) → 8 MiB/~6.6 MB
        // (Excel) → 3 MiB/2.1 MB (the tier split, which took Excel and Arrow
        // back out).
        maximumFileSizeToCacheInBytes: 3 * 1024 * 1024,
        // Everything `generateSW` used to be CONFIGURED to emit beyond the
        // manifest — clientsClaim, cleanupOutdatedCaches, the navigation
        // fallback and its denylist, the runtime CacheFirst routes — is
        // authored in src/sw.ts now, each piece marked with the option it
        // replaces. (One entry that used to sit in the denylist is gone for
        // good: `/docs/` guarded the MkDocs site back when it published into
        // this app's own output — one origin, inside this worker's scope. The
        // docs answer on their own host now, so the guard could never fire.)
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
