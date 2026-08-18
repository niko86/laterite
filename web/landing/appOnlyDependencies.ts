// The apex's dependency firewall — the half of #394's "separate builds" that is
// a check rather than an arrangement.
//
// Two builds share one dependency set (web/package.json), so nothing about the
// arrangement stops `import { AsyncDuckDB } from "@duckdb/duckdb-wasm"` from
// reaching a shared component and riding into the landing bundle. #334 names
// that failure by name: the landing page quietly becoming a worse copy of the
// app. Review is the wrong instrument for it — a heavy dependency arrives as
// one plausible import in a diff about something else.
//
// So the rule is inverted from the obvious one. Not "these packages are
// banned" — a denylist has to be extended every time the app grows, by the same
// person who just added the thing it should catch. Instead: **everything the
// app declares is forbidden here unless the landing page is explicitly allowed
// it**. Adding DuckDB, Arrow, ECharts or Leaflet to the app therefore guards
// the apex against it on the same commit, with nobody remembering to.
//
// Widening `SHARED_PACKAGES` is the deliberate act, and the only one.

/** A forbidden package, and the first module that pulled it into the graph. */
export interface ForbiddenModule {
  readonly pkg: string;
  readonly moduleId: string;
}

/**
 * What the landing bundle is allowed to carry out of the shared dependency set.
 *
 * `solid-js` is here because the shared primitives (#406) are Solid components
 * and a button must exist once, not twice. The three font families are the
 * self-hosted pairing (#394) — they are the payload, not a leak. `vite` and the
 * two Tailwind packages appear in the module graph as build machinery (the
 * modulepreload polyfill, the `@import "tailwindcss"` entry), never as page
 * weight.
 */
export const SHARED_PACKAGES: readonly string[] = [
  "solid-js",
  "@fontsource-variable/figtree",
  "@fontsource-variable/public-sans",
  "@fontsource/ibm-plex-mono",
  "tailwindcss",
  "@tailwindcss/vite",
  "vite",
];

/**
 * Every package the web project declares that the landing page may NOT carry.
 *
 * Transitive dependencies are deliberately absent: a package the app never
 * declares is either something an allowed package needs (solid-js's `seroval`)
 * or something that arrived under a forbidden parent, which already fails.
 */
export function appOnlyPackages(
  declared: Readonly<Record<string, string>>,
  shared: readonly string[] = SHARED_PACKAGES,
): string[] {
  const allowed = new Set(shared);
  return Object.keys(declared)
    .filter((pkg) => !allowed.has(pkg))
    .sort();
}

/**
 * The forbidden packages present in a module graph, one entry per package.
 *
 * Matching is on the `node_modules/<pkg>/` path segment rather than the bare
 * name, so `leaflet-draw` does not read as `leaflet` and a hoisted-vs-nested
 * install answers the same either way. Ids that are not paths — Vite's virtual
 * modules, which carry a leading NUL — cannot contain the marker and fall out
 * on their own.
 */
export function findForbiddenModules(
  moduleIds: Iterable<string>,
  forbidden: readonly string[],
): ForbiddenModule[] {
  const found = new Map<string, string>();
  for (const rawId of moduleIds) {
    const id = rawId.replaceAll("\\", "/");
    for (const pkg of forbidden) {
      if (found.has(pkg)) continue;
      const at = id.indexOf(`node_modules/${pkg}/`);
      // Position 0 (a repo-relative id) or preceded by a separator — anything
      // else is a package whose name merely ends in the one we're looking for.
      if (at === 0 || (at > 0 && id[at - 1] === "/")) found.set(pkg, rawId);
    }
  }
  return [...found]
    .map(([pkg, moduleId]) => ({ pkg, moduleId }))
    .sort((a, b) => a.pkg.localeCompare(b.pkg));
}
