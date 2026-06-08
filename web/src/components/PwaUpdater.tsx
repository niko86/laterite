import { createSignal, Show, type Component } from "solid-js";
import { useRegisterSW } from "virtual:pwa-register/solid";

// Registers the service worker and owns its two bits of UI:
//   • "Ready to work offline" — a one-shot confirmation the first time the
//     app-shell precache lands (so the user knows offline now works).
//   • "New version available — Reload" — shown when an updated SW is waiting.
//     We use registerType:'prompt' (see vite.config.ts), so taking the update
//     is the user's call (no yanking the page out from under a mid-validate /
//     mid-query session). On click, `applyUpdate` tells the waiting worker to
//     skipWaiting and OWNS the reload itself (reload on controllerchange, plus
//     a timed fallback) — leaning on the plugin's implicit reload alone could
//     silently no-op, so the button visibly always reloads.
//
// Because this is a long-lived single-page tab (users rarely navigate), the
// browser would otherwise only check for a new SW on a manual reload — so a
// fresh deploy could go unnoticed for a whole session. onRegisteredSW adds a
// modest poll (hourly + on tab refocus) that re-fetches ONLY the tiny sw.js
// (never the precache or the globIgnored 92 MB assets), so the update toast
// actually appears.
//
// Toast only — no layout impact, dismissible. A bottom-right card on desktop;
// on a phone it spans the bottom (inset-x-4) clear of the home-indicator
// safe-area so it doesn't crowd into the right edge of content. If the SW
// can't register (unsupported browser, file:// , etc.) nothing renders.

// Re-check for a new service worker hourly. update() only re-fetches sw.js.
const UPDATE_POLL_MS = 60 * 60 * 1000;
export const PwaUpdater: Component = () => {
  const [offlineDismissed, setOfflineDismissed] = createSignal(false);

  const {
    needRefresh: [needRefresh, setNeedRefresh],
    offlineReady: [offlineReady, setOfflineReady],
    updateServiceWorker,
  } = useRegisterSW({
    onRegisteredSW(_swUrl, r) {
      if (!r) return;
      // Poll for a new SW so a long single-page session still surfaces the
      // update toast. Guard on onLine so we don't spam failed fetches offline.
      const poll = () => {
        if (navigator.onLine) void r.update();
      };
      setInterval(poll, UPDATE_POLL_MS);
      // Also check the moment the tab regains focus (a likely point for a
      // deploy to have happened while it sat in the background).
      document.addEventListener("visibilitychange", () => {
        if (document.visibilityState === "visible") poll();
      });
    },
    onRegisterError(err) {
      // Non-fatal: the app works without the SW, just without offline.
      console.warn("[pwa] service worker registration failed", err);
    },
  });

  const dismissOffline = () => {
    setOfflineReady(false);
    setOfflineDismissed(true);
  };
  const close = () => setNeedRefresh(false);

  // Take the update. The button MUST visibly do something — earlier it called
  // updateServiceWorker(true) and leaned on the plugin's own controllerchange
  // reload, which could silently no-op (no waiting worker yet, or the
  // controllerchange never arriving). So we own the reload: tell the waiting
  // worker to skipWaiting (reloadPage:false — we don't want the plugin to also
  // reload), reload the moment it takes control, and hard-fall-back after a
  // short grace so the click can never appear to do nothing.
  const applyUpdate = () => {
    let done = false;
    const reload = () => {
      if (done) return;
      done = true;
      window.location.reload();
    };
    navigator.serviceWorker?.addEventListener("controllerchange", reload, {
      once: true,
    });
    void updateServiceWorker(false);
    setTimeout(reload, 3000);
  };

  return (
    <Show when={needRefresh() || (offlineReady() && !offlineDismissed())}>
      <div
        role="status"
        aria-live="polite"
        class="fixed inset-x-4 bottom-4 z-50 mb-[env(safe-area-inset-bottom)] rounded-lg border border-line-strong bg-surface-raised p-3 text-sm text-fg shadow-lg sm:inset-x-auto sm:right-4 sm:max-w-xs"
      >
        <Show
          when={needRefresh()}
          fallback={
            <div class="flex items-center gap-3">
              {/* Scoped deliberately: the precache covers Validate/Fix/the
                  dictionary; the heavy DuckDB engine (Explore) + OSTN15 grid
                  (Coordinates) aren't precached, so don't over-promise them. */}
              <span>Validate &amp; Fix now work offline.</span>
              <button
                type="button"
                class="rounded border border-line-strong px-2 py-0.5 text-xs text-fg-soft hover:bg-chip"
                onClick={dismissOffline}
              >
                Dismiss
              </button>
            </div>
          }
        >
          <p class="mb-2 font-medium">A new version is available.</p>
          <div class="flex items-center gap-2">
            <button
              type="button"
              class="rounded bg-accent px-3 py-1 text-xs font-medium text-canvas hover:opacity-90"
              onClick={applyUpdate}
            >
              Reload
            </button>
            <button
              type="button"
              class="rounded border border-line-strong px-2 py-1 text-xs text-fg-soft hover:bg-chip"
              onClick={close}
            >
              Later
            </button>
          </div>
        </Show>
      </div>
    </Show>
  );
};
