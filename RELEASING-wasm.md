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

**Add a `wasm-v*` tag rule to the `npm` GitHub environment.** `wasm-publish` is
bound to `environment: npm`, whose `deployment_branch_policy` currently allows
only `node-v*` refs — so a `wasm-v*` tag would be rejected before a step runs.
Environments take multiple rules (the `pypi` env already has `branch: main` *and*
`tag: v*`), so this is one added rule, not a second environment:

> repo Settings → Environments → `npm` → Deployment branches and tags →
> **Add rule** → `wasm-v*`

Or:

```bash
gh api -X POST repos/niko86/laterite/environments/npm/deployment-branch-policies \
  -f name='wasm-v*' -f type='tag'
```

**After the first publish**, add the trusted publisher so npm switches to OIDC:
package Settings → *Trusted Publisher* → GitHub Actions, Repository = this repo,
Workflow = `release.yml`, Environment = `npm`. Until then the publish
authenticates with the existing `NPM_TOKEN` secret — npm cannot pre-configure a
trusted publisher for a package that does not yet exist, the same bootstrap
problem RELEASING-node.md documents.

## What a `wasm-v*` tag triggers

- **`build-wasm`** — `wasm-pack build --target web --release`, then
  `tools/release/prepare-wasm-package.sh`, uploaded as the `wasm-pkg` artifact.
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

**`E402` — scoped packages default to private.** `@laterite/ags4-wasm` is scoped,
and publishing a scoped package without `publishConfig.access = "public"` fails
with *"you must sign up for private packages"*. That is how the 0.1.0 npm
shakedown failed for the `@laterite/native-*` packages. `prepare-wasm-package.sh`
sets it and `wasm-verify` asserts it.

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
wasm-pack build rust-packages/laterite-ags4-wasm --target web --release --out-dir /tmp/wasm-pkg
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
- **Size**: ~2.0 MB packed, ~7.2 MB unpacked (the `.wasm` is ~7.1 MB). Consumers
  should lazy-load it rather than putting it in a critical bundle.
- The package name is **rewritten** from wasm-pack's crate-derived
  `laterite-ags4-wasm`; `--scope` would give `@laterite/laterite-ags4-wasm`, which
  is not the published name.
