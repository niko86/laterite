# Releasing `laterite` (npm) — platform setup + publish gotchas

**Companion to [`RELEASING.md`](RELEASING.md).** The npm `laterite` package
(`rust-packages/laterite-node`) ships with three per-platform native packages
(`@laterite/native-darwin-arm64`, `@laterite/native-linux-x64-gnu`,
`@laterite/native-win32-x64-msvc`) as `optionalDependencies`. As of #372 it
**shares the single project version** with the wheel + CLI:
`tools/release/bump-version.sh` stamps `package.json` and regenerates
`package-lock.json`, so the version bump is covered in RELEASING.md —
including how to cut the `node-v*` tag on the mirror. **This doc keeps only the
npm-specific one-time owner setup and the publish-path gotchas** (they bit us on
the 0.1.0 shakedown). `release.yml`'s `build-node` + `npm-publish` jobs do the
work on a final `node-v*` tag — but only after the **one-time owner setup** below.

## One-time npm setup (owner)

Until this is done, `npm-publish` will fail (or the tag won't reach it). It harms
nothing before — the tag gate keeps the job dormant on PRs/branches.

**Why a token for the first release (not pure OIDC):** npm — unlike PyPI — has no
"pending" trusted publishers. You can only configure a trusted publisher on a
package that **already exists**, so OIDC can't bootstrap the four brand-new
packages. The first release authenticates with a scoped token (creating all
four); after that you add the trusted publishers and npm switches to OIDC
**automatically** (the workflow keeps `id-token: write`), and the token can be
deleted. `--provenance` works under both.

1. **Create the `@laterite` org** on npm (free, for public packages). Done — it
   owns the three scoped native packages.
2. ~~**Create a classic _Automation_ token**~~ — **NO LONGER POSSIBLE (checked
   2026-07-29).** npm has removed classic tokens from the UI: only *granular*
   access tokens can be created, and npm actively advises against the
   bypass-2FA option they replaced. It also prints, on any publish using a
   2FA-bypassing credential, that such tokens *"are being restricted for account
   changes and direct publishing"*
   ([deprecation notice](https://gh.io/npm-gat-bypass2fa-deprecation)).

   The historical reasoning, kept because it explains the shape of what follows:
   Automation tokens were the documented exception that bypassed 2FA in CI, while
   granular and classic-publish tokens respect the account's "two-factor for
   writes" setting and fail with **`EOTP` — "this operation requires a one-time
   password"** (learned on the 0.1.0 release). With Automation tokens gone, **there
   is no CI-usable credential that dodges 2FA** — which is why the bootstrap route
   below changed.

   **What to do instead:** don't create a token at all. Bootstrap a new package
   with ONE authenticated publish from a workstation (`npm login`, then
   `npm publish <dir> --access public --otp=<code>`), then wire its trusted
   publisher so every later release comes from CI via OIDC with provenance and no
   credential in the loop. See RELEASING-wasm.md, which walks the full sequence
   including the three separate auth failures it surfaces.

   > **If a `NPM_TOKEN` secret still exists, treat it as a liability, not a
   > fallback.** This repo's expired on **2026-06-22** and nobody noticed for five
   > weeks — `node-v0.7.0` (2026-07-08) and `node-v0.8.0` (2026-07-28) both
   > published fine, because OIDC covered the four existing packages and the token
   > was never consulted. It only surfaced when a package that *cannot* use OIDC —
   > a brand-new one — fell back to it and got `E404`. Deleting the token once OIDC
   > is live (step 2 of the next section) would have made that loud immediately.
3. **Create the `npm` GitHub environment** (repo Settings → Environments → *New
   environment* `npm`) with a **deployment branch/tag policy** → *Selected* →
   add a tag rule **`node-v*`**. This is the environment-level half of the publish
   gate (mirrors the `pypi` env).

### After the first release — switch to OIDC — **DONE**

Once the four packages exist on npm:

1. For **each** of `laterite`, `@laterite/native-darwin-arm64`,
   `@laterite/native-linux-x64-gnu`, `@laterite/native-win32-x64-msvc`: package
   Settings → *Trusted Publisher* → GitHub Actions, with Repository = the repo
   hosting `release.yml`, Workflow = `release.yml`, Environment = `npm`.
   **Done** — proven by `node-v0.7.0` and `node-v0.8.0` publishing cleanly on an
   already-expired token, which only OIDC can explain.
2. Next release uses OIDC with no workflow change (npm prefers the trusted
   publisher when `id-token` is present).
3. **Delete the token from the job.** Done for `npm-publish` — it passes no
   `NODE_AUTH_TOKEN` at all. This is not tidying: a token behind a working
   trusted publisher is never read, so its only reachable effect is to turn "the
   publisher is misconfigured" into a misleading `E404`. Removing it changes the
   failure path, not the success path. See the job comment and RELEASING-wasm.md.
4. **Delete the `NPM_TOKEN` repository secret.** Nothing in `release.yml`
   references it any more — owner action, not something CI can do.

## What the `node-v*` tag triggers

The npm package publishes on its own tag namespace, **`node-v*`** (a `node-v*`
tag never builds wheels; a `v*` tag never publishes to npm) — but the version
number is now unified, so you cut `node-v0.6.0` alongside `v0.6.0` from the same
release (RELEASING.md step 3). The tag must match `package.json` `"version"` —
`npm-publish`'s tag-check asserts it; `napi prepublish` syncs the three platform
packages to that version automatically.

The `node-v*` tag triggers `release.yml`:

- **`build-node`** — builds `laterite-node.<triple>.node` on each of the three
  platform runners (linux/macOS/windows), uploads them as `node-<target>`
  artifacts.
- **`npm-publish`** (final `node-v*` tags only, no `rc`/`dev`) — downloads the
  artifacts, places them into the `@laterite/native-*` dirs (`napi
  create-npm-dir` + `napi artifacts`), builds the dual ESM/CJS dist (`tsup`),
  publishes the platform packages (`napi prepublish`), then `npm publish
  --provenance` the main `laterite` package.
  - **The scoped `@laterite/native-*` packages must publish _public_.** Scoped
    packages default to **private** (a paid feature) → `npm publish` fails with
    **`E402` — "you must sign up for private packages"**. `napi prepublish` has
    no `--access` flag, so the root `package.json` carries
    `"publishConfig": { "access": "public" }`, which `napi create-npm-dir`
    **propagates** into every generated platform `package.json`. Don't remove it.

### Pre-releases

A tag containing `rc` or `dev` (e.g. `node-v0.2.0rc1`) builds the artifacts but
**skips `npm-publish`** — same as the PyPI side.

## First-release validation

The publish path can't be fully exercised without a real tag + the npm setup
above, so treat the **first** `npm-publish` run as a shakedown: watch that `napi
artifacts` places the three `.node`s correctly and that each `npm publish`
authenticates. If OIDC + napi misbehave, use the token fallback. Subsequent
releases are routine.

**Done — `0.1.0` shipped 2026-06-15** (`laterite` + the three `@laterite/native-*`,
provenance-signed; a real `npm install laterite` reads + validates against the
published binary). The shakedown surfaced three gotchas, all now guarded against
above: the committed napi loader (else the publish-only `build:ts` breaks — see
the packaging smoke in the `node` CI job), the **Automation** token (2FA/`EOTP`),
and `publishConfig.access=public` (scoped-private `E402`). The trusted publishers
are wired and the job no longer passes a token; only deleting the now-unreferenced
`NPM_TOKEN` secret remains.

## Why the native pins are NOT in the committed `package.json` (the `EUSAGE` cycle)

This bit twice, so the fix is structural rather than procedural.

The `@laterite/native-*` optional deps used to be pinned in the committed
manifest at the package version. While that version is unpublished npm cannot
resolve them, so `package-lock.json` records `{"optional": true}` placeholders;
once the version publishes, `npm ci` validates the lock against the registry, the
placeholders no longer satisfy the manifest, and every install fails with
**`EUSAGE` "lock file ... does not satisfy"**.

The first fix was procedural — have `bump-version.sh` regenerate the lock during
the bump. That cannot work, because the bump happens *before* the publish: the
regenerated lock just gets fresh placeholders. `npm ci` duly broke on `main`
again immediately after 0.8.0 published (#155).

So the pins are gone from the committed manifest entirely. `napi pre-publish`
rewrites the whole `optionalDependencies` block at publish time anyway, deriving
each pin from `napi.targets` and the package version — a committed copy could
only ever be redundant, and could never make the published pins more correct than
the tool that writes them. With nothing pinned, there is nothing unresolvable to
lock.

`test_version_faithful.py::test_node_package_and_native_deps_match` asserts the
pins stay absent, and that `napi.targets` matches `release.yml`'s `build-node`
matrix — the input napi actually derives them from.

If you ever hit `EUSAGE` out-of-band anyway: `cd rust-packages/laterite-node &&
npm install --package-lock-only` and commit the lock. **Don't reach for
`npm ci --omit=optional`** — it does not dodge the error (the manifest/lock sync
check runs before install strategy, verified), and it would drop rollup's own
platform binary and break the `tsup` build.
