// The service worker, hand-written. It was `generateSW` output until #366,
// which needed the one thing that mode cannot express: `runtimeCaching` rules
// cannot carry a custom handler, and the fix for a warm still in flight when
// its engine's tab opens — one download, not two — IS a custom handler
// (in-flight coalescing around CacheFirst, `lib/swCoalesce.ts`). Everything
// else here reproduces what `generateSW` emitted for the same options, each
// piece marked with the option it replaces; the precache manifest is still
// computed by vite-plugin-pwa (`injectManifest` in vite.config.ts, where the
// glob-side locks from #355 stay) and injected below.

import { CacheableResponsePlugin } from "workbox-cacheable-response";
import { clientsClaim } from "workbox-core";
import { ExpirationPlugin } from "workbox-expiration";
import {
  cleanupOutdatedCaches,
  createHandlerBoundToURL,
  precacheAndRoute,
} from "workbox-precaching";
import { NavigationRoute, registerRoute } from "workbox-routing";
import { CacheFirst } from "workbox-strategies";

import { coalesce, type CoalescableStrategy } from "./lib/swCoalesce";
import { RUNTIME_CACHING } from "./lib/swPolicy";

// The webworker lib is deliberately not in this app's tsconfig (the graph is
// DOM-typed; worker files narrow `self` by hand — see tier2.worker.ts), so
// `ServiceWorkerGlobalScope` resolves to workbox-precaching's augmentation
// alone: the injected `__WB_MANIFEST`. The intersection adds the only two
// other members this file touches.
declare let self: ServiceWorkerGlobalScope & {
  skipWaiting(): Promise<void>;
  addEventListener(
    type: "message",
    listener: (event: { data: unknown }) => void,
  ): void;
};

// `registerType: "prompt"`'s other half: PwaUpdater's reload button messages
// the WAITING worker (workbox-window's messageSkipWaiting), and a worker
// nobody is listening in never activates — the toast would offer a reload
// that does nothing.
self.addEventListener("message", (event) => {
  const data = event.data as { type?: unknown } | null;
  if (data?.type === "SKIP_WAITING") void self.skipWaiting();
});

// App-shell precache: the manifest vite-plugin-pwa computed from the globs in
// vite.config.ts is injected here at build time.
precacheAndRoute(self.__WB_MANIFEST);
// Was `cleanupOutdatedCaches: true`.
cleanupOutdatedCaches();
// Was `clientsClaim: true`: first-install SW controls the page immediately, so
// an offline reload right after the first visit is already served from cache.
clientsClaim();

// Was `navigateFallback` + `navigateFallbackDenylist`. SPA offline reload →
// serve the app shell; NavigationRoute only matches navigation requests, so
// asset/JSON fetches are untouched. Relative "index.html" resolves against
// this worker's own location, which is how it tracks the deploy base the same
// way the generated worker did. The denylist: never answer a top-level
// navigation with the app shell when a real file should serve itself — a final
// path segment with a dot-extension, which is how the runtime-cached
// .wasm/.gsb reach their own handlers.
registerRoute(
  new NavigationRoute(createHandlerBoundToURL("index.html"), {
    denylist: [/\/[^/?]+\.[^/?]+$/],
  }),
);

// Was `runtimeCaching` (the rules live in lib/swPolicy.ts, where the unit
// policy test can reach them) — with the one behaviour `generateSW` could not
// express wrapped around each strategy: concurrent requests for one URL share
// one network flight, so a warm still downloading when its engine starts is
// joined, not duplicated (#366).
for (const rule of RUNTIME_CACHING) {
  const strategy = new CacheFirst({
    cacheName: rule.options.cacheName,
    plugins: [
      ...(rule.options.cacheableResponse
        ? [new CacheableResponsePlugin(rule.options.cacheableResponse)]
        : []),
      new ExpirationPlugin(rule.options.expiration),
    ],
  });
  // The cast bridges workbox's wider handleAll signature — its union admits a
  // MANUAL invocation whose `request` may be a string — down to the
  // router-invoked shape this route can actually receive, where `request` is
  // always a real Request. The full options object passes through untouched.
  registerRoute(
    rule.urlPattern,
    coalesce(strategy as unknown as CoalescableStrategy),
  );
}
