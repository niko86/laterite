import { createSignal, onCleanup } from "solid-js";

// Reactive `matchMedia`: an accessor that tracks a breakpoint and UPDATES when
// it's crossed. Replaces the one-shot `window.matchMedia(q).matches` reads that
// seeded a panel's open-state once at mount and then went stale — e.g. the
// Validate FilterBar and SQL console Examples/Saved disclosures stayed open
// after the window was narrowed (or an iPad rotated across 1024px). Must be
// called in a reactive scope (component body) so onCleanup can detach.
export function createMediaQuery(query: string): () => boolean {
  if (typeof window === "undefined" || !window.matchMedia) return () => false;
  const mql = window.matchMedia(query);
  const [matches, setMatches] = createSignal(mql.matches);
  const on = (e: MediaQueryListEvent) => setMatches(e.matches);
  mql.addEventListener("change", on);
  onCleanup(() => mql.removeEventListener("change", on));
  return matches;
}
