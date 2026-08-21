# Cloudflare configuration for `cdn.laterite.dev`

Two settings behind the Explore tab live in the Cloudflare account, not in code.
Neither is applied from CI. What is here is the **record**; what proves the live
state matches is an **assertion** — see `ags-wiki/design/dec-cdn-configuration.md`
for why those are separate, and why nothing here is pushed automatically.

## `r2-cors.json` — the `laterite-cdn` bucket's CORS rules

Applied by hand, verbatim:

```bash
npx wrangler r2 bucket cors set laterite-cdn --file web/cloudflare/r2-cors.json --force
```

`cors set` is a full replace, not an append, so re-running it is safe and this
file is the whole configuration rather than a delta. It needs an R2 token with
**Admin Read & Write** — `Object Read & Write`, which the deploy uses, cannot
edit bucket configuration.

Two things in the file that look removable and are not:

- **`allowed.headers` includes `range`.** DuckDB issues ranged reads, and a
  `Range` header of a single byte range is CORS-safelisted — so the common path
  issues no preflight and never consults this list. It is here for the cases that
  are not safelisted (a multi-range request, or a value over 128 characters),
  where a preflight _is_ issued and its absence would block the read. Keep it,
  but do not mistake it for the reason ranged reads work.
- **`exposeHeaders` includes `content-range`.** This is the load-bearing half. A
  cross-origin response's headers are not readable by the client unless exposed,
  so without it the browser gets the bytes and the caller cannot read the range
  metadata off the response.

**Localhost origins are deliberately absent.** They were there so a dev build
could fetch the real CDN, but the dev path does not need them: `VITE_DUCKDB_CDN`
is opt-in, and unset the engine stays in the app's own assets and is served
locally (`web/vite.config.ts`). While they were listed, any page served from
those ports — including an unrelated project's dev server — could read from this
bucket. Reproducing a CDN-specific bug locally is rare enough to be a temporary
dashboard change rather than a standing allowlist entry.

## The zone Cache Rule — recorded, not filed

There is no committed file for it, on purpose: Wrangler cannot manage Cache
Rules (dashboard, API or Terraform only), so a JSON here would be a description
nothing reads and nothing diffs. The rule's fields are written out in the
decision page, and the nightly asserts its **effect** instead, which is the part
that can actually be checked.

## `canary.txt` — what the probes point at

Every engine bundle on the CDN is fingerprinted, so no object has a stable URL to
probe. This one does. Both settings are scoped to the bucket and the host rather
than to any object, so proving them here proves them for the bundles.
