# Releasing `@laterite/ags4-wasm` (npm) — the browser package

**Companion to [`RELEASING.md`](RELEASING.md) and
[`RELEASING-node.md`](RELEASING-node.md).** The browser build ships as
**`@laterite/ags4-wasm`** — the wasm-pack output of
`rust-packages/laterite-ags4-wasm`, published from its own **`wasm-v*`** tag.

## Why its own tag train

`node-v*` drives the Node package, `v*` drives the wheel + CLI, `wasm-v*` drives
the browser package. A browser-only fix can then ship without moving the Node
package — which is exactly the situation that prompted it: 0.8.0 shipped
`synthesise_metadata` on Python and Node, but the wasm fix (#149) landed two
hours after the tag, and nothing about Node had changed.

Under today's **lockstep** versioning all three still carry the same number, so
the split buys nothing yet. It is the shape [#153](https://github.com/niko86/laterite/issues/153)
(per-surface release tagging) needs, built while the release plumbing was open.

## One-time owner setup

**The `wasm-v*` tag rule on the `npm` GitHub environment — DONE 2026-07-29.**
`wasm-publish` is bound to `environment: npm`, whose `deployment_branch_policy`
allowed only `node-v*` refs, so a `wasm-v*` tag would have been rejected before a
step ran. Environments take multiple rules (the `pypi` env already has
`branch: main` *and* `tag: v*`), so this was one added rule, not a second
environment. The `npm` env now carries both:

    tag: node-v*   (id 51972923)
    tag: wasm-v*   (id 55964567)

Recorded for the next person who wonders where the gate lives. To recreate it:

> repo Settings → Environments → `npm` → Deployment branches and tags →
> **Add rule** → `wasm-v*`

Or:

```bash
gh api -X POST repos/niko86/laterite/environments/npm/deployment-branch-policies \
  -f name='wasm-v*' -f type='tag'
```

**The trusted publisher — DONE 2026-07-30.** Package Settings → *Trusted
Publisher* → GitHub Actions, Repository = this repo, Workflow = `release.yml`,
Environment = `npm`. Do this immediately after the bootstrap below; it is what
makes every subsequent release credential-free.

**And then delete the token from the job.** `wasm-publish` no longer passes
`NODE_AUTH_TOKEN` at all (#172). Once a trusted publisher exists, a token is not a
fallback behind it — it is the thing that hides a misconfiguration. It is why the
first `wasm-v0.8.1` attempt was slow to read: OIDC could not engage, npm fell back
to a credential that had expired five weeks earlier, and the answer was `E404`,
which names neither fact. With nothing to fall back to, "the publisher is wrong"
reports itself as an auth error instead of a missing package.

`npm-publish` (the node train) still carries its token. By this same argument it is
also dead weight — `node-v0.7.0` and `node-v0.8.0` both published on that expired
token because OIDC was already covering those four packages — so drop it on the next
`node-v*`, when a release can actually prove it.

## Bootstrapping a new package name (how 0.8.1 actually shipped)

npm cannot pre-configure a trusted publisher for a package that does not exist, so
**OIDC cannot create a name** — and as of 2026-07-29 there is no CI-usable
credential that can either: npm removed classic *Automation* tokens (the documented
2FA-bypass exception) and restricts 2FA-bypassing credentials for direct publishing.

So the name is created by **one authenticated publish from a workstation**, after
which CI takes over. That is what happened for `0.8.1`:

```bash
# `-- --no-default-features` builds the SLIM artifact — since #330 that is what
# @laterite/ags4-wasm is. The cargo flags MUST follow `--`; wasm-pack exits ZERO
# when they land in the wrong place, having produced nothing, so the size check
# below is what tells you the build was real.
wasm-pack build rust-packages/laterite-ags4-wasm --target web --release \
  --out-dir /tmp/wasm-pkg -- --no-default-features
node tools/release/check-wasm-slim.mjs /tmp/wasm-pkg  # slim surface + size ceiling
tools/release/prepare-wasm-package.sh /tmp/wasm-pkg
tools/release/verify-npm-notice.sh /tmp/wasm-pkg     # prove the notice rides
npm login
npm publish /tmp/wasm-pkg --access public --otp=<6-digit>
```

(The `0.8.1` run predates #330 and had no `--no-default-features`; it published
the full engine.)

**Three separate auth failures sit on that path, in this order.** Each reports a
different code and none of them names its real cause:

1. **`E404` on PUT** while the CI job used `NPM_TOKEN`. Not "missing" — *refused*.
   npm will not confirm whether a scoped name exists, so an unusable credential on
   a new scoped package surfaces as 404 rather than 401/402. The token had **expired
   on 2026-06-22** and nobody noticed for five weeks, because OIDC covered the four
   existing packages and never consulted it (`node-v0.7.0` and `node-v0.8.0` both
   published fine after it died). A `NPM_TOKEN` left in place after OIDC is wired is
   a liability: it converts "no credential" into a misleading 404.
2. **`EOTP` — "this operation requires a one-time password."** The account has
   two-factor required for writes, and no CI-usable token now dodges that. Hence
   `--otp=` on the manual publish.
3. **`E403` "You cannot publish over the previously published versions: 0.8.1"**
   *while* `GET` on the package still returned 404. The write side is authoritative
   and the read replica lags — 0.8.1 took **~45 s** to become readable. Do not
   re-attempt on the strength of a 404; check the direct registry endpoint first.

**Consequence for the tag.** `0.8.1` was published by hand, so the `wasm-v0.8.1`
CI run's publish job **stays red permanently** — the version already exists and npm
will not accept it twice. Nothing is broken and the tag still correctly marks the
source commit.

**`0.8.2` closed it out (2026-07-30) — and it worked first try.** A patch release
with no code change at all, cut for two reasons: to give the package the provenance
attestation `0.8.1` structurally could not have, and to prove the tokenless path.
Both landed. The registry now answers `200` on
`/-/npm/v1/attestations/@laterite/ags4-wasm@0.8.2` and `404` on `@0.8.1`, and the
publish log reads *"Signed provenance statement with source and build information
from GitHub Actions"* (Sigstore log index 2285019141). OIDC authenticated with **no
credential in the job**.

One expectation that proved wrong at the time, worth recording: **on that run the
`npm` environment did not pause for approval.** It carried deployment tag policies
and no required reviewer, so the tag push published with no human checkpoint after
the tag.

**That has since been fixed, and the fix is the operative state:** the `npm`
environment now carries `required_reviewers` alongside its `branch_policy`, so a
`wasm-v*` or `node-v*` tag push **waits for an approval** before anything reaches
the registry. Confirm with

```bash
gh api repos/niko86/laterite/environments/npm --jq '.protection_rules[].type'
# branch_policy
# required_reviewers
```

This paragraph is the one claim in this file nothing in the repo can gate — the
environment's protection rules live in GitHub's settings and leave no in-tree
trace. Re-run the command above rather than trusting the sentence.

## What a `wasm-v*` tag triggers

- **`build-wasm`** — `wasm-pack build --target web --release
  -- --no-default-features` (the SLIM artifact, #330), then
  `tools/release/check-wasm-slim.mjs` and `tools/release/prepare-wasm-package.sh`,
  uploaded as the `wasm-pkg` artifact.
  Built **once**; `wasm-verify` and `wasm-publish` both consume those same bytes,
  so what was checked is what ships.
- **`wasm-verify`** — packaging checks with **no environment and no secrets**, so
  they are reachable on any ref via `workflow_dispatch`. Asserts the ©AGS notice
  rides in the tarball, the published name is `@laterite/ags4-wasm`, and
  `publishConfig.access` is `public`.
- **`wasm-publish`** — final `wasm-v*` tags only (no `rc`/`dev`), tag-vs-version
  checked, notice re-verified against the artefact it is about to upload, then
  `npm publish --provenance`.

A `wasm-v*` tag never builds wheels and never cuts a GitHub release — the Python
jobs exclude the prefix explicitly. Keep that exclusion in step with the trigger
list: those guards are written as "everything except the other release trains",
so a new prefix added to `on.push.tags` without a matching exclusion silently
enrols itself in the Python release.

## The two traps this path already accounts for

**Scoped packages default to private, and the manifest alone is not enough.**
`@laterite/ags4-wasm` is scoped, so without `access: public` a publish is refused.
`prepare-wasm-package.sh` sets `publishConfig.access` and `wasm-verify` asserts it
— but **`npm publish` must ALSO pass `--access public`**, and the first
`wasm-v0.8.1` attempt proved it: with the token present, provenance signed, and
`publishConfig.access: "public"` in the manifest, the PUT still failed with

    npm error code E404
    npm error 404 Not Found - PUT https://registry.npmjs.org/@laterite%2fags4-wasm
    npm error 404  ... could not be found or you do not have permission to access it.

**Read that 404 as "refused", not "missing".** npm will not confirm whether a
scoped name exists, so a permission or visibility refusal on a *new* scoped
package surfaces as 404 rather than 402 or 401 — the error names neither cause,
which is what makes it slow to diagnose. The sibling `laterite` publish has always
passed the flag explicitly; the `@laterite/native-*` packages get it from
`napi pre-publish` rather than from a bare `npm publish`, which is why nothing in
the existing setup had exercised this path. (The related `E402 you must sign up for
private packages` is the same root cause wearing its other error code — that is how
the 0.1.0 shakedown failed for the native packages.)

**The ©AGS third-party notice.** The `.wasm` **embeds the AGS4 dictionary** (the
reference leaf's `include_str!`), so the notice has to ride with it exactly as it
does with each `.node`. wasm-pack does not copy a LICENSE — it warns *"License key
is set in Cargo.toml but no LICENSE file(s) were found"* — so
`prepare-wasm-package.sh` stages it and `tools/release/verify-npm-notice.sh`
proves it by looking inside the tarball npm would actually upload. The published
0.7.0 npm packages shipped verbatim ©AGS text under a bare `"license": "MIT"`
with no notice at all; this is that bug's wasm twin, closed before it could ship.

## Rehearsing without a tag

Both scripts are files, so the whole path runs at a terminal — the lesson from
the 0.8.0 npm release, whose notice guard was inline bash reachable only by
pushing a real tag and took three attempts and three tag moves to get right:

```bash
wasm-pack build rust-packages/laterite-ags4-wasm --target web --release \
  --out-dir /tmp/wasm-pkg -- --no-default-features
node tools/release/check-wasm-slim.mjs /tmp/wasm-pkg
tools/release/prepare-wasm-package.sh /tmp/wasm-pkg
tools/release/verify-npm-notice.sh /tmp/wasm-pkg
(cd /tmp/wasm-pkg && npm pack --dry-run)
```

`workflow_dispatch` also runs `build-wasm` + `wasm-verify` on any ref and stops
short of publishing.

## Package facts worth knowing

- **`--target web`** — ESM with an explicit `init()`, the shape the crate README
  documents and the web app already consumes. It works both under a bundler and
  from a plain module script, unlike `--target bundler`.
- **Size**: the `.wasm` is **1.8 MiB** (749 KiB gzipped), re-measured 2026-08-16.
  It was ~7.1 MB until #330 gated `excel`/`arrow`/`certify`/`diff`/`merge`/
  `censor` out of the published build; a source build with `default = full` is
  still 5.1 MiB. Consumers should lazy-load it rather than putting it in a
  critical bundle. `check-wasm-slim.mjs` holds both figures — a gzip ceiling and
  a raw one — so if either jumps back, the publish built the wrong shape.
- The package name is **rewritten** from wasm-pack's crate-derived
  `laterite-ags4-wasm`; `--scope` would give `@laterite/laterite-ags4-wasm`, which
  is not the published name.
