import { configDefaults, defineConfig } from "vitest/config";

// Unit tests cover the PURE logic — severity classification, CSV/analytics
// helpers, and the rest of the wasm-free modules. Component rendering (Solid +
// DOM) is exercised end-to-end by the Playwright suite (web/e2e), so these run
// in a plain node env with no plugins — fast, no JSDOM, no Rust/wasm toolchain.
// That sentence WAS the whole decision, recorded where only someone editing
// this file would meet it (#431). It now has a page —
// ags-wiki/design/dec-web-test-altitude.md — which adds the half it was
// missing: when a guard is worth pinning below e2e, EXTRACT the decision
// into a pure function and test it here. A test that wants a DOM is a
// signal the logic wants lifting out of the component, not that this lane
// wants widening.
//
// The tokenizer-dependent tests (agsline display helpers + fix-preview
// geometry) now call the wasm-backed tokenizer (#533), so they run in the
// separate wasm lane (vitest.wasm.config.ts, in the e2e job) and are excluded
// here to keep this lane pure + fast. The tokenizer's own invariants are pinned
// in Rust (laterite-ags4-parse's `display_spans.rs` proptest).
export default defineConfig({
  test: {
    environment: "node",
    // `landing/**` is the apex's own build (#394): its dependency firewall is
    // pure logic that must be tested, and it lives beside the config it guards
    // rather than in the app's source tree.
    // `scripts/**` joins them for the same reason (#401): the docs' copy of the
    // shared token layer is generated, and the rules deciding what that copy
    // says are pure functions that must not be tested by eyeballing the output.
    include: [
      "src/**/*.test.ts",
      "landing/**/*.test.ts",
      "scripts/**/*.test.mjs",
    ],
    exclude: [
      ...configDefaults.exclude,
      "src/lib/agsline.test.ts",
      "src/lib/fixpreview.test.ts",
    ],
    coverage: {
      // v8 provider — no Istanbul instrumentation step, and matches the
      // engine the bundle ships on. Scope is the files the unit suite
      // actually imports (the pure-logic modules); components + the worker
      // are covered by the Playwright e2e suite, not measured here.
      provider: "v8",
      // text → CI log; lcov → web/coverage/lcov.info for the Codecov upload
      // (web flag). Only uploaded from the public repo (see e2e.yml guard).
      reporter: ["text", "lcov"],
      // The wasm-pack output is GENERATED glue, not code this repo writes or
      // tests. It is gitignored and absent when CI's `unit` job runs (that job
      // builds no wasm), so leaving it un-excluded made the number depend on
      // whether the developer happened to have built wasm: with it present the
      // run reports ~47% and FAILS the 95 floor, for no reason anyone changed.
      // Excluded so a local run and the CI run measure the same denominator.
      exclude: ["src/wasm/**", "src/wasm-full/**", "src/wasm-tokenizer/**"],
      // Floor gate, not a target. Introduced at 65 (68.76% baseline); ratcheted to
      // 95, and now to 99 after duckTypes / loadSensitive / the relationship
      // walkers got suites. A new untested pure module — or deleting a tested one
      // — drops this and fails the `unit` job, which is the point.
      //
      // BRANCHES had no floor at all until now, and that is why the Codecov badge
      // read ~85% while this gate sat green at 95+: a line whose branch is only
      // half-taken counts as HIT by lcov and as a PARTIAL by Codecov. Lines alone
      // cannot see that, so branch coverage drifted to 81% unnoticed. Codecov's is
      // the stricter measure and this is the floor that tracks it.
      //
      // Branches ratcheted 89 → 95 once the fetch doors, the join-mode chart
      // refs and the template's search loop got suites (actual 96.34, lines 100,
      // Codecov-strict 97.5%). The ~10 branches still short are `?.`/`??` guards
      // TypeScript's narrowing needs but no input can reach — chasing those would
      // move the number without testing anything, so the floor keeps a margin
      // rather than pinning the current value.
      thresholds: { lines: 99, branches: 95 },
    },
  },
});
