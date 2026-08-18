import { defineConfig, type Plugin } from "vite";
import solid from "vite-plugin-solid";
import tailwindcss from "@tailwindcss/vite";
import { resolve } from "node:path";
// Both written the long way — the import attribute and the file extension —
// because Vite's native config loader (the coming default) rejects the short
// forms with a warning on every build.
import pkg from "../package.json" with { type: "json" };
import {
  appOnlyPackages,
  findForbiddenModules,
} from "./appOnlyDependencies.ts";

// laterite.dev — the apex, built here rather than in the app's config (#394).
//
// ONE dependency set, TWO builds. The app's config is a single-entry SPA build
// carrying app-only machinery: a PWA registration layer, an SPA 404 fallback,
// and a step that relocates the oversized DuckDB wasm out to R2. Spanning those
// across a two-entry build would apply every one of them to a page that wants
// none, and shared-chunk splitting would make it easy to drag the app's heavy
// dependencies into the apex bundle without anyone noticing.
//
// So the separation is the enforcement mechanism, not a preference. #334 names
// the failure mode: the landing page quietly becoming a worse copy of the app.
// Sharing a toolchain is not sharing a bundle.
//
// Solid and Tailwind are here despite this page currently rendering neither a
// component nor a reactive value: the shared primitives (#406) are Solid
// components classed with the shared tokens, and a build that cannot compile
// one is not the gate this ticket is supposed to be.

// The other half of the separation — see appOnlyDependencies.ts for why this is an
// allowlist inverted rather than a list of banned packages.
//
// Scope: the JavaScript module graph. A dependency reaching the page through
// the CSS pipeline instead (a `@import "some-package/x.css"`) may be resolved
// internally by Vite or Tailwind and never registered as a module, so it can
// pass — which is fine for what this guards, since the weight #334 is worried
// about is DuckDB, Arrow, ECharts and Leaflet, and every one of those arrives
// as JavaScript.
function noAppOnlyDependencies(): Plugin {
  const forbidden = appOnlyPackages({
    ...pkg.dependencies,
    ...pkg.devDependencies,
  });
  return {
    name: "no-app-only-dependencies",
    apply: "build",
    buildEnd() {
      const hits = findForbiddenModules(this.getModuleIds(), forbidden);
      if (hits.length === 0) return;
      const lines = hits.map((h) => `  ${h.pkg}  (via ${h.moduleId})`);
      this.error(
        `the landing bundle picked up ${hits.length} app-only ` +
          `${hits.length === 1 ? "dependency" : "dependencies"}:\n` +
          lines.join("\n") +
          `\n\nlaterite.dev is a small page that loads no engine. If one of ` +
          `these genuinely belongs on both surfaces, add it to SHARED_PACKAGES ` +
          `in web/landing/appOnlyDependencies.ts — deliberately, and with the size in ` +
          `mind.`,
      );
    },
  };
}

export default defineConfig({
  root: import.meta.dirname,
  // No VITE_BASE knob, unlike the app: the apex has only ever answered at the
  // root of its own domain, and never went to the project-Pages path the app's
  // knob exists to bridge.
  base: "/",
  resolve: {
    alias: {
      // Both builds resolve this to the same directory, which is what lets a
      // button exist once (#394). The app's config carries the twin entry.
      // landing.css imports the token layer THROUGH it rather than by relative
      // path, so the alias is exercised by every build rather than merely
      // declared until the shared primitives (#406) arrive.
      "@shared": resolve(import.meta.dirname, "../src/shared"),
    },
  },
  plugins: [solid(), tailwindcss(), noAppOnlyDependencies()],
});
