// The policy half of #339: no CacheFirst rule may accept an opaque response.
// Asserted over EVERY runtime rule rather than the two literals that were
// wrong, because the next such rule gets written by copying one of these and a
// test naming `ags-duckdb-wasm` would not look at it. Why it matters is on the
// rules themselves in vite.config.ts, next to the value being constrained.
//
// The behavioural half — that a healthy response still reaches the cache — is
// in e2e/app.spec.ts, because no config assertion can see it and it is the
// direction that fails silently.
import { describe, expect, it } from "vitest";

import { RUNTIME_CACHING } from "../../vite.config";

// Workbox's `cacheableResponse.statuses` defaults to undefined = "cache
// whatever came back", so a rule that omits the option entirely is the same bug
// spelled differently — an opaque response would still be written. Both shapes
// have to fail, which is why this reads the option rather than the array alone.
const opaqueAccepted = (rule: (typeof RUNTIME_CACHING)[number]) => {
  const statuses = rule.options?.cacheableResponse?.statuses;
  return statuses === undefined || statuses.includes(0);
};

describe("service worker runtime caching", () => {
  it("has rules to check", () => {
    // Vacuity guard, and only that: an emptied array would pass the filter
    // below with nothing to report. A rename or a move fails at typecheck
    // instead, so this is not guarding against those.
    expect(RUNTIME_CACHING.length).toBeGreaterThan(0);
  });

  it("never caches an opaque response under CacheFirst", () => {
    const offenders = RUNTIME_CACHING.filter(
      (rule) => rule.handler === "CacheFirst" && opaqueAccepted(rule),
    ).map((rule) => rule.options?.cacheName ?? "(unnamed)");

    expect(offenders).toEqual([]);
  });
});
