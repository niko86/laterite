// The service worker's runtime caching policy — which heavy assets are cached
// on first fetch, under what cache names, accepting which statuses. Consumed by
// `src/sw.ts` (which turns each rule into a coalesced CacheFirst route) and
// asserted over by `sw-cache-policy.test.ts`, so the policy stays reachable
// from a unit test — the rule that made a server fault permanent per-device
// (#339) was the kind only a test over the whole array catches. The rules
// lived inline in vite.config.ts while the SW was generated (`generateSW`);
// #366 moved SW authorship into `src/sw.ts`, and the policy came here so the
// test could keep reading it without importing a service-worker global scope.

/** One CacheFirst rule, shaped like the `generateSW` config it replaced so the
 *  policy test keeps both historical failure modes representable: a rule that
 *  omits `cacheableResponse` entirely means "cache whatever came back" —
 *  opaque failures included — which is the same bug as listing status 0. */
export interface RuntimeCacheRule {
  urlPattern: (context: { url: URL }) => boolean;
  handler: "CacheFirst";
  options: {
    cacheName: string;
    cacheableResponse?: { statuses?: number[] };
    expiration: {
      maxEntries: number;
      maxAgeSeconds: number;
      purgeOnQuotaError: boolean;
    };
  };
}

export const RUNTIME_CACHING: RuntimeCacheRule[] = [
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
    // The TIER-2 engine wasm (#355) — the full build, several MB, fetched the
    // first time Explore or Tools → Excel is opened and never on a visit that
    // opens neither. Fingerprinted + immutable, so CacheFirst is safe and the
    // second visit to either tab compiles from cache, offline included.
    urlPattern: ({ url }) =>
      /\/ags4_wasm_full_bg-[^/]*\.wasm$/.test(url.pathname),
    handler: "CacheFirst",
    options: {
      cacheName: "ags-engine-tier2",
      // 200 only — the DuckDB rule above says why (#339). Same-origin here, so
      // an opaque response is not even reachable; the `0` that broke DuckDB was
      // copied from a rule that could not use it either, and copying a default
      // rather than deciding it is exactly how that comes back.
      cacheableResponse: { statuses: [200] },
      expiration: {
        // Current build + one stale generation as an update-window fallback.
        maxEntries: 2,
        maxAgeSeconds: 60 * 60 * 24 * 60,
        purgeOnQuotaError: true,
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
