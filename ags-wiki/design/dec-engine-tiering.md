---
type: decision
title: "Four engine tiers in the browser: render on 30 KB, validate on 839 KiB, defer the rest (#338)"
status: accepted
tags: [design, decision, wasm, web, performance]
decided: "2026-08-16"
supersedes: []
from_gap: []
related: [validator-site, tech-stack-wasm, laterite-ags4-wasm, playwright-e2e, dec-laterite-ags4-types-leaf, dec-ags4-censor-leaf]
sources: []
---

# Four engine tiers in the browser

## Context

Before this, the browser app precached **one** engine artifact — the whole thing,
6.7 MB raw — and gated its **first paint** on that artifact instantiating.
`repo:web/src/App.tsx` waited on `Promise.all([validatorReady(),
tokenizerReady()])`, and `repo:web/src/lib/validator.worker.ts` instantiates
eagerly at module scope, so nothing rendered until the full engine had
downloaded and compiled. Someone who opened the app, read the page and left had
paid for the Excel converter, the Arrow stack, the certificate machinery, diff,
merge and censor.

#330 made that artifact feature-gated, which made tiering possible. This page is
what the tiering should be.

Two facts found while designing it contradicted the ticket that proposed it, and
both changed the answer:

- **`certify` is on the Validate tab**, not Tools
  (`repo:web/src/components/validate/DownloadCertificate.tsx`). The ticket's
  table put it in tier 2, which would have moved a primary-tab button behind a
  1 MB engine swap.
- **`read` is Explore-only** (`repo:web/src/components/explore/ExplorePane.tsx`
  is its sole caller), though the ticket listed it in tier 1 beside validate and
  fix.

## Options considered

**Where the tier boundary sits.** Measured per-feature, `wasm-opt`'d, gzip -9:

| build | gzip | raw |
|---|---:|---:|
| npm slim (`--no-default-features`) | 757.3 KiB | 1868.9 KiB |
| **slim + `certify` + `diff` + `merge` + `censor`** | **839.2 KiB** | **2093.9 KiB** |
| full (`default`) | 1771.1 KiB | 5189.8 KiB |

1. **Tier 1 = the npm slim artifact.** One build serves npm and the app.
2. **Tier 1 = everything except `arrow` and `excel`.** ← chosen
3. **Tier 1 = slim + `arrow`.** Explore's ingest precached; only Excel defers.

The four "cheap" features cost **+82 KiB gzipped between them**; `arrow` and
`excel` account for **932 KiB** of the 1,014 KiB gap. Option 1 pays a full engine
swap to save 82 KiB and breaks a Validate-tab button doing it. Option 3 lands
tier 1 at ~1.17 MB and gives most of the saving back.

## Decision

Four tiers.

| tier | what | size | when |
|---|---|---:|---|
| **0** | `laterite-ags4-tokenizer-wasm` | ~30 KB | gates first render |
| **1** | engine − `arrow` − `excel` | 839 KiB gz / 2094 KiB raw | precached; ready by file-load |
| **2** | full engine | 1771 KiB gz / 5190 KiB raw | warm-fetched on idle, compiled on intent |
| **3** | DuckDB | 36 MB | unchanged — first Explore click |

**First render gates on tier 0 alone.** The engine comes out of `App.tsx`'s
`Promise.all`. Tier 1 loads during the window where a human is choosing a file —
seconds, not milliseconds — which is the whole point: the engine's real deadline
is *when a file is loaded*, never *when the page paints*.

**Tier 1 is precached** (`globPatterns` in `repo:web/vite.config.ts`) and serves
Validate, Fix, Export **and all of Tools**. All of that now works offline.

**Tier 2 is warm-*fetched*, never speculatively compiled**, behind the same
`saveData` + `isLowEndDevice()` gate DuckDB uses in
`repo:web/src/lib/prefetch.ts` (the predicate itself is
`repo:web/src/lib/device.ts`). It is `globIgnore`d and served by its own
`CacheFirst` runtime rule with `statuses: [200]` (see #339 — status 0 is never
cacheable here again). It compiles on intent, in a **second worker created
lazily** on first Explore or Excel open. That worker owns `ParsedDataset`, whose
only consumer is Explore.

**Sequenced, not parallel.** Tier 2's warm starts only once tier 1 is up, which
is what `repo:web/src/lib/prefetch.ts` already does — its `warmLazyAssets()`
fires from a `createEffect` gated on engine readiness.

**Two artifacts, two gates.** Two `wasm-pack` runs in
`repo:.github/workflows/deploy-validator.yml`; tier 2 takes `--out-name
ags4_wasm_full` and a dynamic import. Tier 1's gate is
`repo:tools/release/check-wasm-tier1.mjs` (#352) — 940 KiB gzip / 2350 KiB raw,
both axes, ~12% headroom, over the same instrument
`repo:tools/release/check-wasm-slim.mjs` drives
(`repo:tools/release/wasm-artifact-gate.mjs`). It landed BEFORE the artifact had
a consumer, so the 839.2 / 2093.9 measurement this design rests on has been held
from the moment there was something to hold.

## Why

**Why the boundary is `arrow` + `excel` and nothing else.** Those two are the
only features that are actually heavy. Putting the other four in tier 1 costs
82 KiB and buys: Tools working offline, Tools needing no engine swap, and the
Validate tab keeping its certificate button. The swap then fires only on two
explicit, already-slow user actions — Explore, which is about to wait on 36 MB
of DuckDB anyway, and Excel conversion.

**Why a second worker rather than replacing the engine in place.** The worker is
already the isolation boundary — and since #351 the ops it serves live in
`repo:web/src/lib/engineDispatch.ts`, parameterised by the engine module, so a
second worker is a second `createEngineDispatch(...)` rather than a second copy
of thirteen ops. The parsed-dataset handle lives in that closure, which is what
gives each worker its own; the two engines serve disjoint tabs; the only
stateful handle is Explore's own. So "both resident" is not duplication being
tolerated — it is the process boundary matching the feature split. Opening
Explore can never disturb a running validate. Replacement was the ticket's
preferred option and would have worked — `repo:web/src/lib/fileStore.ts` keeps
`originalBytes` for the page's lifetime, so a re-feed is a re-parse and never a
re-download — but it buys nothing here and has in-flight ops to drain.

**Why warm-fetch and not warm-compile.** `repo:web/src/lib/duck.ts` draws this
line already: `warmFetch()` pulls bytes into cache and stops, and compilation
waits behind `EngineGate`. Compiling ~5 MB of wasm for a tab most visitors never
open would hand back a good part of what the tiering just won.

**Why sequenced.** Tier 1 is on the critical path and tier 2 is speculative;
fetching them together lets the speculative one steal bandwidth from the needed
one. On a ~10 Mbps link tier 1 alone is ~0.7 s and shares to ~2 s — and that
delay lands exactly on the sample-file path, where a user can go from cold paint
to needing the engine in milliseconds.

## Consequences

**Accepted cost: Tools → Excel on a low-end device.** It is the only tier-2
consumer needing no DuckDB, so a skipped warm is the whole delay rather than a
rounding error on a 36 MB wait. A couple of seconds, on an explicit action,
behind a loading state.

**A not-ready window exists on the primary path**, where a file is loaded before
tier 1 is up. Reachable mainly via the sample-file buttons. It surfaces as
Validate's *existing* loading state — no new UI, because "engine still arriving"
and "validate in progress" are the same thing to a user.

> [!warning] Taking tier 1 out of the paint gate must not take its FAILURE with it
> Building this (#353) the first attempt let a dead engine be reported by
> whichever pane asked for it, on the reasoning that the worker replies with the
> init error to every op. It does — and the pane still cannot show it. A Solid
> resource **throws when read after an error** (`if (err !== undefined && !pr)
> throw err`), so `repo:web/src/components/validate/ValidatePane.tsx`'s own
> `Validator error: …` fallback sits behind a `<Show when={report()}>` that
> throws before reaching it, and with no `ErrorBoundary` in the tree the tab
> stays on its spinner for ever — the exact permanent silent state #339 is
> about. So `repo:web/src/App.tsx` still reports a failed engine at page level
> even though it no longer waits for a live one: a rejection is not a wait.
> Pinned by an e2e that aborts the engine fetch, falsified by deleting the
> branch. ValidatePane's own unreachable fallback is pre-existing and left
> alone here — it needs a seam to force an op failure before a test of it could
> be shown to go red, and its `createEffect` reads the resource too.

**A failed tier-2 fetch is partial, and must read as partial.** The tab that
needed it reports and offers retry; tier 1 is precached and untouched. #339's
lesson was that a failed engine fetch must never become a permanent silent
state.

**The precache separation is the fragile part.** Two `wasm-pack` runs both emit
`ags4_wasm_bg.wasm` by default, so Vite fingerprints them to two hashes with the
**same stem** — and `globPatterns`' `assets/ags4_wasm_bg-*.wasm` matches both.
Tier 2 would be precached, the install would carry the full engine again, and
nothing would error. `maximumFileSizeToCacheInBytes` cannot catch it either:
tier 2 is 5.2 MB raw, under the 8 MiB cap. Hence the distinct `--out-name`, a
`globIgnores` entry, **and** an e2e assertion in `repo:web/e2e/app.spec.ts` that
tier 2 is *absent* from the precache. That negative assertion is the only guard
here that would actually fail — every other one is size-based, and this artifact
fits under all of them.

**The app's engine stops being npm's.** `check-wasm-slim.mjs` guards what npm
ships; it no longer describes what the app precaches. Tier 1 needs its own gate
or nothing watches the artifact this whole design depends on staying small — it
has one, and it ran before the app had switched to the artifact it guards.

**The offline claim gets larger.** `repo:web/src/components/PwaUpdater.tsx` says
"Validate & Fix now work offline", which was honestly scoped when written and is
now understated — Tools works offline too.

## Related

- [[validator-site]] — the precache-vs-runtime split this refines.
- [[tech-stack-wasm]] — the engine wasm and its build.
- [[laterite-ags4-wasm]] — the crate and its six cargo features (#330).
- [[playwright-e2e]] — where the precache assertions live.
- #338 (this design) · #330 (the feature gates that made it possible) ·
  #339 (why `statuses: [200]`) · #342 (why the raw axis is gated too).
