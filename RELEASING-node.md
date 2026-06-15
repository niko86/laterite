# Releasing `laterite` (npm)

The Node package (`rust-packages/laterite-node`) ships as **`laterite`** on npm,
with three per-platform native packages (`@laterite/native-darwin-arm64`,
`@laterite/native-linux-x64-gnu`, `@laterite/native-win32-x64-msvc`) pulled in as
`optionalDependencies`. `release.yml`'s `build-node` + `npm-publish` jobs do the
work on a final `v*` tag — but only after the **one-time owner setup** below.

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
2. **Create a classic _Automation_ token** (npm → *Access Tokens* → *Generate New
   Token* → **Classic Token** → **Automation**). Add it as the **repo secret
   `NPM_TOKEN`** (repo Settings → Secrets and variables → Actions).
   - **Use Automation, not a granular/publish token, if your account has 2FA.**
     Automation tokens are the documented exception that **bypass 2FA** in CI;
     granular and classic-publish tokens respect the account's
     "two-factor for writes" setting and the publish fails with **`EOTP` — "this
     operation requires a one-time password"** (learned the hard way on the
     0.1.0 release). Account-wide scope is fine; delete it once OIDC is wired.
3. **Create the `npm` GitHub environment** (repo Settings → Environments → *New
   environment* `npm`) with a **deployment branch/tag policy** → *Selected* →
   add a tag rule **`node-v*`**. This is the environment-level half of the publish
   gate (mirrors the `pypi` env).

### After the first release — switch to OIDC (optional, recommended)

Once the four packages exist on npm:

1. For **each** of `laterite`, `@laterite/native-darwin-arm64`,
   `@laterite/native-linux-x64-gnu`, `@laterite/native-win32-x64-msvc`: package
   Settings → *Trusted Publisher* → GitHub Actions, with Repository = the repo
   hosting `release.yml`, Workflow = `release.yml`, Environment = `npm`.
2. Next release uses OIDC with no workflow change (npm prefers the trusted
   publisher when `id-token` is present). You can then **delete the `NPM_TOKEN`
   secret**.

## Cutting a release

The npm package releases on its **own tag namespace, `node-v*`** — fully
independent of the Python `v*` tags (a `node-v*` tag never builds wheels; a `v*`
tag never publishes to npm). So the Node version is decoupled from the Python
packages' versions; it starts at `0.1.0`.

The version lives in `rust-packages/laterite-node/package.json`, and the tag must
match it (`node-v<version>`) — `npm-publish`'s tag-check asserts it. `napi
prepublish` syncs the three platform packages to the same version automatically.

> **Public repo:** releases run from the public mirror (`niko86/laterite`), which
> is also where npm + the `npm` environment are configured. Cut the `node-v*` tag
> there (after the public-tree sync), not on the private dev repo.

1. Bump `rust-packages/laterite-node/package.json` `"version"` (e.g. 0.1.0 →
   0.1.1), commit, and let the public-tree sync carry it to `niko86/laterite`.
2. On **`niko86/laterite`** → **Releases → Draft a new release**:
   - **Choose a tag** → type **`node-v0.1.1`** → *Create new tag on publish*.
   - **Target**: the default branch (the synced commit).
   - **Publish release.**

The workflow accepts both the web-UI `release` event and a `git push` of the tag
(if you have a checkout): `git tag -m "…" node-v0.1.1 && git push origin
node-v0.1.1`. Either way, the `node-v0.1.1` tag triggers `release.yml`:

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
and `publishConfig.access=public` (scoped-private `E402`). Next: wire OIDC
trusted publishers and delete `NPM_TOKEN`.

> **After any publish — regenerate the lockfile.** The `@laterite/native-*`
> optional deps are pinned to the package version. While a version is unpublished
> the lockfile records them as *unresolved*; once published, `npm ci` resolves
> them against the registry and the empty lockfile entry no longer matches →
> **`EUSAGE` "lock file ... does not satisfy"**. So after a release: `cd
> rust-packages/laterite-node && npm install`, then commit the updated
> `package-lock.json`. (Don't use `npm ci --omit=optional` to dodge it — that
> also drops rollup's own platform binary and breaks the `tsup` build.)
