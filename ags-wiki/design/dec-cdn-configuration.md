---
type: decision
title: "CDN configuration is recorded and asserted, never applied from CI"
status: accepted
tags: [design, decision, deploy, cloudflare]
decided: "2026-08-21"
supersedes: []
from_gap: []
related: [validator-site, docs-site]
sources: []
owns: [cdn-configuration]
---

# CDN configuration is recorded and asserted, never applied from CI

## Context

Two settings on `cdn.laterite.dev` sit between a visitor and the Explore tab, and
both live in the Cloudflare account rather than in this repository:

- the **CORS configuration** on the `laterite-cdn` R2 bucket, without which the
  browser refuses the engine fetch;
- a **Cache Rule** on the `laterite.dev` zone, without which every engine request
  goes to the R2 origin.

Neither has an in-tree trace. A rebuilt bucket or a reverted rule breaks the app
with nothing in CI to notice, and the CORS failure surfaces as a **hang, not an
error** — which is what made 2026-08-16 expensive (see [[validator-site]] for the
incident, and the service-worker half that made it survive a reload).

Three properties were being collapsed into one phrase, "it lives only in the
Cloudflare account", and separating them is what let the two settings land
differently:

| | meaning |
|---|---|
| **recorded** | the value is written down here, reviewable, diffable |
| **applied** | something pushes it to Cloudflare |
| **asserted** | something proves the live state matches |

## Options considered

1. **Record only** — commit both, plus a runbook. Nothing checks them.
2. **Record + assert** — CI proves the live state and fails when it is wrong.
3. **Record + apply + assert** — CI is the source of truth and pushes on deploy.

## Decision

**Option 2, at a rung chosen per setting.** Nothing is applied from CI.

| | bucket CORS | zone Cache Rule |
|---|---|---|
| recorded | `repo:web/cloudflare/r2-cors.json`, in the shape `wrangler --file` consumes | this page — see *The Cache Rule, as applied* below |
| applied | by hand, from that file | by hand, in the dashboard |
| asserted | the deploy, **fatally**, before the Workers ship | the nightly, off the deploy path |

## Why

**Applying from CI would cost two broad, long-lived credentials.** Editing bucket
configuration needs an R2 token with **Admin Read & Write** — which also grants
create and delete on buckets — and the Cache Rule needs zone Rulesets edit, which
can rewrite caching for every host on `laterite.dev`. The deploy's existing token
only needs `Object Read & Write`. Widening it, or adding a second, would put more
standing authority in GitHub for a setting that changes perhaps twice a year, and
this repo has been moving the other way (the engine now publishes with no stored
credential at all).

**Asserting catches strictly more than applying.** Apply-on-deploy fixes the live
state and therefore cannot report that someone changed it in the dashboard; the
assertion catches both that and a fresh bucket. The thing apply buys is that a
rebuilt bucket self-heals — traded here for a red deploy that names the fix.

**The two settings are not one thing, and one rung for both would be wrong.**
They differ in scope (bucket vs zone), in credential, in whether Wrangler can
manage them at all (it cannot manage Cache Rules), and — decisively — in failure
signature. Missing CORS means Explore **never starts**. A reverted Cache Rule
means Explore still works and a first-time visitor far from the bucket waits
longer for a 7.6 MB transfer, which the originating issue argued down itself. A
fatal deploy gate on a latency regression is how a red run learns to be ignored.

**Where each assertion runs follows from that.** The CORS check sits between the
R2 upload and the Worker deploys, so a bucket without CORS stops the release
instead of publishing an app whose engine cannot start — the same ordering
rationale already written into `repo:.github/workflows/deploy-validator.yml` for
uploading before deploying. The accepted cost: a CORS regression blocks a
docs-only deploy. The Cache Rule check runs in
`repo:.github/workflows/nightly.yml`, where a red night is already a tracked
signal and no deploy is held up.

**A stand-in object, because nothing else has a stable URL.** Every engine bundle
is fingerprinted, so a probe URL naming one rots on the next engine build — and a
rotted URL returns 404, which reads exactly like a broken CDN rather than a broken
probe. The alternative, parsing the live app for the URL it was built with, was
rejected after checking: the reference is not in the entry chunk but in a lazily
loaded one, so finding it means walking the module graph, and a probe that cannot
find a URL looks identical to one that passed. So the deploy uploads
`repo:web/cloudflare/canary.txt` at a fixed key and both checks point there. This
is sound rather than convenient: **neither setting is scoped to an object** — CORS
belongs to the whole bucket, and the Cache Rule matches on hostname with no path
predicate — so what the canary establishes holds for the bundles.

**Localhost origins came out of the allowlist.** They let a dev build fetch the
real CDN, but the dev path does not need them: `VITE_DUCKDB_CDN` is opt-in, and
unset the engine stays in the app's own assets (`repo:web/vite.config.ts`). While
they were listed, any page on those ports — including an unrelated project's dev
server — could read from the bucket. Reproducing a CDN-specific bug locally is
rare enough to be a temporary dashboard change.

## The Cache Rule, as applied

Recorded here rather than as a committed file, because Wrangler cannot manage
Cache Rules — dashboard, API or Terraform only — so a JSON in the tree would be a
description nothing reads and nothing diffs. On the `laterite.dev` zone:

- **Match:** `http.host eq "cdn.laterite.dev"`
- **Cache eligibility:** eligible for cache
- **Edge TTL:** ignore the origin's cache-control header and use this TTL — 1 year
- **Status Code TTL:** status `>= 400` → **No store**

The last entry is not decoration. Cache eligibility applies regardless of status,
so without it a 404 served in the window between a Worker deploy and its objects
landing sticks at the edge for every visitor to that datacentre until it expires.
That hazard was introduced by the rule itself and closed in the same rule, and the
nightly asserts both halves for exactly that reason.

Safe at a 1-year TTL because the objects are fingerprinted: a bundle's content
never changes under its name, and a new build produces a new name.

## Consequences

- A rebuilt bucket or a reverted rule now **fails loudly** instead of shipping a
  hang; the deploy's error names the committed file and the exact command.
- A rebuilt bucket does **not** self-heal. Someone must reapply, and the red
  deploy is what tells them to.
- The Cache Rule's record is prose, so it can drift from the dashboard without
  anything diffing it. The nightly asserting its *effect* is the mitigation, and
  it is a weaker guarantee than the CORS file gets — knowingly, because the
  alternative is a file that no tool consumes.
- If applying from CI is ever wanted, this decision is what to revisit, and the
  credential scope is the thing that was traded.

## Related

[[validator-site]] · [[docs-site]] · `repo:web/cloudflare/README.md`
