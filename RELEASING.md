# Releasing laterite

`laterite` is the single shipped wheel and follows **semantic versioning** with a
pre-1.0 convention.

## Version policy — the axis is *compatibility*, not change size

| Bump | The test | Examples |
|---|---|---|
| **PATCH** (`0.1.x`) | existing behaviour unchanged | bug fix, doc tweak, perf win, internal refactor, a batch of small fixes |
| **MINOR** (`0.x.0`) | **new** behaviour, old code still works (additive) | new function / CLI flag / output option, new optional arg — *even a whole new feature*, as long as nothing existing changes |
| **MAJOR** (`x.0.0`) | **breaks** existing callers | removed/renamed API, changed default, changed output/file format, dropped a Python version |

A big *additive* feature is **MINOR**, not major — size is irrelevant, compatibility is everything.

**Pre-1.0 convention (we are here):** while at `0.x`, a **breaking** change bumps
the **MINOR** (`0.1 → 0.2`); features and fixes bump **PATCH**. We save `1.0.0`
for when we deliberately want to signal "stable — I'll keep compatibility."

## Cutting a release

**There is one shared PRODUCT number, and a version per published crate**
(#153 split product from engine 2026-08-01; #781 retired the engine lockstep
2026-08-30):

| | Covers | Resolved by | Bumped with |
|---|---|---|---|
| **product** | the Python wheel, the npm `laterite` package + its `@laterite/native-*` addons, the browser package, the `lat` binary, the DuckDB extension | `pip install laterite` · `npm i laterite` | `bump-version.sh product` |
| **each engine crate** | itself — ten crates.io engine crates + the `laterite` facade, each on its own line | `cargo add laterite-ags4-validator` | `bump_crate.py <crate> <part>` |

Every **product** still shares one number, so `pip install laterite==X` and
`npm i laterite@X` are the same release. Each engine crate moves when IT
changes: `laterite-ags4-excel` changing rarely no longer drags
`laterite-ags4-validator`'s cadence, heterogeneous numbers across the set are
expected, and a version bump means something happened in that crate.

> [!IMPORTANT] **A bump and a release are the same act.** If you stamp a product
> version, cut *every* product tag — not just the surfaces that changed. They
> share one number, so a product left un-cut leaves a version that exists in this
> tree and on no registry. That is not hypothetical: 0.8.1 and 0.8.2 were stamped
> for a browser-only fix and tagged `wasm-v*` alone, so PyPI went 0.8.0 → 0.9.0
> and wheel 0.8.1/0.8.2 were never published at all. Re-shipping an unchanged
> product is the cost of the shared number.

### Start here: is a release owed, and what part?

```bash
uv run --no-sync python tools/release/release_status.py
```

Prints every published crate (own version, API delta, derived part) and the
product tier. Each crate's verdict comes from ITS committed `cargo-public-api`
snapshot in `tools/release/public-api/` — the only source that can see an
addition, since `cargo semver-checks` has no `function_added` lint and skips
every `minor` lint between releases. The product verdict comes from the
`changelog.json` sections every PR is already forced to fill in.

The engine baseline is the last **published** version — the exact commit the
crates.io tarball records in its `.cargo_vcs_info.json` — never a stamp that
went nowhere (#806). `--cut` prints the actionable view: per engine crate, the
bump / publish / needs-a-human act the nightly derives, with the exact
`bump_crate.py` commands.

Read what it says it cannot see. The product's own API surface is **not**
measured — no committed snapshot exists for the Python or Node surface — so that
verdict is a suggestion. The **engine** tier IS checked against crates.io — each
crate's stamp is looked up in the sparse index, and a stamp that never published
shows as `PUBLISH OWED` beside its bump verdict, because a bump owed and a
publish owed are different actions. A read that fails is reported as unasked and
never as unpublished, and the count of crates that went unasked prints on every
run. The **product** tier is not checked: nothing here asks PyPI or npm, so a
stamped product version whose tag was never cut is still invisible (the
0.8.1/0.8.2 failure above). `--no-registry` skips the lookup for an offline run.
The same derivation runs nightly as the **engine cut** (#806): anything owed
opens (and keeps current) an `Engine release work owed` tracking issue — an
issue a human sees, where the step summary that recorded emit 0.12.0's missing
publish was a summary nobody read — and a stamped-but-unpublished crate gets
the publish dispatched automatically, any stale queued run cancelled first.
The dispatched run executes unattended: since 2026-09-03 the `crates`
environment carries no required reviewer, so the human gate on a publish is
the PR merge that put the stamp on `main`, not a second click on the run.
Owed *bumps*: the nightly opens one cut PR for the whole owed set
(`ENGINE_CUT_MODE=pr` since 2026-09-03; a human still reviews and merges it —
that merge is the release act, and the only human step left in an engine cut).

### Cutting a product release

```bash
git switch -c release/0.12.0
tools/release/cut-release.sh 0.12.0            # prints the plan, then stamps
tools/release/cut-release.sh 0.12.0 --plan     # print the plan and stop
```

One command, one `release: X` commit, the product tier only — engine crates are
never stamped by a product cut. The old two-tier `cut-release.sh` and its
`--skip engine` escape hatch are gone with the lockstep: skipping the engine
existed to avoid burning eleven lockstep versions on a browser-only fix, and
per-crate versioning dissolves the dilemma — a browser-only fix simply bumps no
engine crate.

`tools/release/bump-version.sh product` drives the in-repo bump (wrapping
[`bump-my-version`](https://callowayproject.github.io/bump-my-version/) —
product config in the root `pyproject.toml` `[tool.bumpversion]` — plus
lockfile regeneration). The DuckDB extension lives in
its own repo and takes the **product** number when you cut it (below). The **docs
site** carries the version too — it's derived at build and republishes on merge.

The facade (`laterite`, its own `0.1.x` line) is just another per-crate line
since #781 — it was the scheme's precedent, not an exception. `laterite-cli`
carries the product number, so `lat --version` agrees with the wheel's `lat`.

**Never hand-edit a version string** — `test_version_faithful.py`, the compat
guard, and the `release.yml` tag-check all catch drift.

> [!IMPORTANT] **Releases publish from this repo.** The PyPI/npm trusted
> publishers are configured for `niko86/laterite`, so its `release.yml` builds,
> tests, *and* publishes. Cut the `v*` / `node-v*` tags here, on `main`, once the
> release PR has merged (step 3) — there is no separate publish origin.

Before bumping, finish the `[Unreleased]` section — in **`changelog.json`**, the
SSOT; `CHANGELOG.md` is generated from it and never hand-edited. Then ask which
bump the queued entries justify:

```bash
uv run --no-sync python tools/gen_changelog.py --advise
```

**A breaking entry must say so twice.** Compatibility is the axis the table above
turns on, so it is *declared* on the entry rather than inferred from its prose:

```json
{ "text": "**Breaking:** callers must pass the new flag.", "breaking": true }
```

The `breaking` flag drives `--advise`; the `**Breaking:**` marker tells the
reader. `gen_changelog.py` fails if one is present without the other, so neither
can be added or removed alone. (It used to be a `\bbreaking\b` search over the
entry text, which counted "a non-breaking change" and "this is not a breaking
change" as breaks — the flag exists because a wrong answer here becomes a wrong
version on an append-only registry.)

Then: 

```bash
# 1. On a release branch (the script refuses to run on main or a dirty tree),
#    bump every PRODUCT + regenerate uv.lock / Cargo.lock / package-lock.json,
#    verify the drift-gate, and make one "release: X" commit (no tag, no push):
git switch -c release/0.6.0
tools/release/bump-version.sh product minor  # or: patch  ·  --new-version 0.6.0rc1
#    (DRY_RUN=1 tools/release/bump-version.sh product minor  stamps + regenerates without committing)

# 2. main is PROTECTED → land the bump via a release PR (merge-commit, NOT squash):
git push -u origin release/0.6.0
gh pr create -B main -t "release: 0.6.0" -b "version bump"   # merge once CI is green

# 3. On the merged main, cut EVERY product tag — release.yml builds + publishes from them:
git switch main && git pull
gh release create v0.6.0      --title v0.6.0      --generate-notes   # wheels + sdist → PyPI, CLI → GH release
gh release create node-v0.6.0 --title node-v0.6.0 --generate-notes   # npm addon + @laterite/native-*
git tag --no-sign wasm-v0.6.0 && git push origin wasm-v0.6.0          # npm @laterite/ags4-wasm (browser)
# 4. Approve the `pypi` / `npm` environments in the resulting Actions runs (the OIDC gates).

# 5. Cut the DuckDB extension at the SAME version (its own repo — see below):
cd <the niko86/laterite-duckdb checkout> && bash scripts/release.sh 0.6.0
# 6. Confirm the docs republished (the site rebuilds on every main push).
```

The tags stay separate because `release.yml` has independent `v*` (Python + CLI),
`node-v*` (npm) and `wasm-v*` (browser) build/publish paths — but the **number**
is one, so you cut all three from the same release.

**Do not skip a tag because that surface didn't change.** The number is shared,
so a surface left un-cut leaves a version stamped in this tree that exists on no
registry — which is precisely how wheel 0.8.1 and 0.8.2 came to be published
nowhere. If skipping feels right, the thing you actually want is per-product
versions, which is a different scheme; see
`ags-wiki/design/dec-rust-api-crates-io.md` rather than skipping a tag here.

**A release that adds a platform target needs ONE manual publish first.** Each
`napi` target in `rust-packages/laterite-node/package.json` is its own npm
package (`aarch64-unknown-linux-gnu` → `@laterite/native-linux-arm64-gnu`), and
the npm job authenticates by OIDC alone — deliberately, see the comment on the
publish step. A trusted publisher is configured *per package* on npmjs.com, so a
name that has never been published cannot have one, and npm answers `PUT` with
`404 … could not be found or you do not have permission to access it`. Every
name on the registry today was created while `NODE_AUTH_TOKEN` still existed
(dropped 2026-07-30, #174); the first release to add a target after that — 0.11.0,
carrying arm64 Linux from #316 — failed on exactly this, half-published, and had
to be finished by hand.

So when a release adds a target, before cutting `node-v*`:

1. `npm login` — NOT a granular token. A token scoped to `@laterite` covers the
   platform packages and **not** the unscoped `laterite` package, which fails
   only at the last publish of the sequence, as `403 … You may not perform that
   action with these credentials`; and `laterite` may additionally carry
   *Publishing access → Require two-factor authentication and disallow tokens*,
   which no token satisfies by design. A login session is the account, so it
   clears both.
2. publish that one package from the release's own artifacts;
3. set its trusted publisher on npmjs.com to match the others;
4. `npm logout`.

`npm publish ./npm/<platform>` needs the leading `./` — `npm/<x>` is npm's
`owner/repo` shorthand, so without it npm tries to clone `github.com/npm/<x>`
and reports a git error that names neither the registry nor the directory.

It is a one-time cost per NEW package name, not per release — after step 2 the
name is on the OIDC path with its siblings. Beware the half-published state if
you find out the hard way: `napi pre-publish` aborts on the first failure, so the
targets ahead of the new one in the list are already on the registry, and npm
refuses to publish over them (`cannot publish over the previously published
versions`). Re-running the job cannot get past that — finish the remaining
packages by hand, then `napi pre-publish -t npm --skip-optional-publish` to sync
the main package's `optionalDependencies` before publishing it. Build from the
tag's artifacts (`gh run download <the failed run>`), never a fresh local build:
the published number must carry the same engine every other surface published at
that number.

> Cutting a release on a **new** tag fires **two** workflow runs — a `push` (tag) run
> and a `release` run — because GitHub raises both events. That's expected: the `push`
> run builds + publishes; the `release` run no-ops (jobs skip on `release`). They no
> longer cancel each other (the concurrency key includes `github.event_name`;
> laterite#264). If you ever see the build run *cancelled* at ~3s, that regression is
> back — re-run the `push` run.

`bump-version.sh product` stamps, atomically in one commit: the `laterite` wheel
`pyproject.toml` + the root umbrella, the compat `__version__` base + Checker
banner in `packages/laterite/python/laterite/compat/_impl.py` (preserving the
`+compat.python-ags4.<pin>` pin), `laterite-cli`'s crate
version, the npm `package.json` version, and rolls `CHANGELOG.md`'s
`[Unreleased]` into the new dated section — then regenerates `uv.lock`,
`Cargo.lock`, the npm `package-lock.json`
(the last one avoids the `EUSAGE` failure `npm ci` throws at publish against a
stale lock), and the generated napi loader `index.js` (the node CI job runs
`napi build` + `git diff --exit-code`, so the committed loader's version literals
must match a fresh build — a stale one reds that guard). `web/src/wasm/package.json`
is gitignored and wasm-pack regenerates it from the crate version; the docs
Changelog page derives its version at build. Neither needs stamping.

### Bumping and publishing engine crates (crates.io)

Per-crate since #781: bump the crate that changed, when it changes — usually in
(or right after) the PR that changed it, never as a side effect of a product
cut.

```bash
uv run --no-project python tools/release/bump_crate.py laterite-ags4-emit minor
git switch -c release/emit-0.12.0 && git add -A && git commit -m "release: laterite-ags4-emit 0.12.0"
gh pr create -B main -t "release: laterite-ags4-emit 0.12.0" -b "per-crate bump"

# once merged, publish from GitHub — never from a laptop (#463):
gh workflow run publish-crates.yml --ref main                      # rehearsal: prints the waves
gh workflow run publish-crates.yml --ref main -f execute=true      # the real thing
```

`bump_crate.py` rewrites the pair that must agree — the crate's own `version`
and its `[workspace.dependencies]` floor — regenerates `Cargo.lock`, and runs
the faithfulness gate. The publisher then publishes every crate whose version
is ahead of the registry and **skips the rest by version identity** — which
after a real bump is the correct skip, and without one is the trap #781
records: a crate whose content changed at an unchanged version silently stays
stale on the registry. `check_semver` is the other half of the discipline: a
crate LEVEL with the registry has every lint enforced, so a break demands its
bump before it merges.

Two automations stand behind the by-hand flow above (#806). The **PR gate**
(`release_status.py --check-coherence`, in ci.yml's repo-gates): a floor moved
past a pin some published crate carries fails the PR that moves it and names
the crates to bump alongside — the #809 class, which no build or test can see
because in-tree the laterite deps are path deps and always unify. The
**nightly cut** derives the owed set every night, tracks it on the
`Engine release work owed` issue, dispatches the publish for anything stamped
but absent from the registry, and opens the cut PR itself
(`ENGINE_CUT_MODE=pr` since 2026-09-03). Forgetting a bump is therefore
recoverable by morning; the by-hand flow is for not wanting to wait.

The resulting Actions run executes **unattended** — unlike `pypi` and `npm`,
the `crates` environment has carried no required reviewer since 2026-09-03
(the PR merge is the human gate; see
`ags-wiki/design/dec-engine-publish-automation.md` for the decision).

`.github/workflows/publish-crates.yml` runs `tools/publish_crates.py` — the same
script, in the one place it should run. crates.io is the only **append-only**
registry of the three: a published version can never be withdrawn or re-cut,
and there is no equivalent of the tag retarget that recovered 0.8.0. Doing the
least reversible step by hand, with no environment gate and no approval, was
what #463 was filed about.

It carries no token. crates.io Trusted Publishing validates this repo, that
workflow **filename** and the `crates` environment over OIDC — so renaming or
moving that file breaks the publish for all eleven crates at once. The
per-crate configs live under each crate's Settings → Trusted Publishing on
crates.io.

Locally the script is still the way to *look*: run it with no flags and it
performs every check and prints what it would do. It derives the dependency
waves from the manifests, waits for each wave to become *resolvable* from the
registry before starting the next, and is idempotent — a re-run after a failure
resumes rather than restarting. It refuses a dirty tree, any branch but `main`,
and any crate marked `publish = false`.

**An engine publish reaches crates.io consumers and `laterite-duckdb` only.**
Every product compiles the engine SOURCE from the tree (bare `path` deps), so
wheel/npm/CLI/wasm users get engine changes with the next product cut whether
or not the crates published. Engine changelog entries stay under `[Unreleased]`
until that product release rolls them — which is the release a reader can
actually install.

The facade crate `laterite` is published by the same tool but carries its **own**
`0.1.x`, bumped by hand in `rust-packages/laterite/Cargo.toml`. It is neither
tier until it reaches parity with the Python and Node surfaces.

**When you bump it, name that number in the changelog entry.** The facade has no
changelog of its own and is not owed one: #318 scoped *"read the changelog before
upgrading"* to the product line, because Cargo will not resolve a caret
requirement across a `0.x` minor, so a facade consumer is protected by the
resolver whether they read anything or not. Its history therefore lives in
`changelog.json` with everything else, and the only route from crates.io to
*"what moved in 0.1.3"* is finding that number written in the prose. Product
`0.10.0`'s facade entries do it — *"functional at 0.1.0"*, *"in 0.1.1"*,
*"0.1.2"*. `0.11.0`'s do not, and that is the whole of #319: a route that worked
stopped working, silently, at the last release.

Nothing needs backfilling. The next facade cut carries everything landed since
`0.1.2`, so its entry names that version and what went into it. And the
convention retires itself — at `ags-wiki/design/dec-facade-parity.md` phase 8 the
facade joins the product number and there is only one version left to name.

### Pre-releases (RC / dev)

The API may still change pre-1.0, so cut a release candidate first when in doubt.
Use the **explicit** form (PEP 440 canonical — no hyphen):

```bash
tools/release/bump-version.sh product --new-version 0.6.0rc1     # then rc2, rc3, ...
tools/release/bump-version.sh product --new-version 0.6.0         # promote to final
```

## The DuckDB extension (`laterite_ags4`)

The extension is a **separate repo, `niko86/laterite-duckdb`**, published through
DuckDB **community-extensions** — not from this repo. As
of #372 it **tracks the laterite version**: cut it at the same number, as step 5
above. Its own `.github/workflows/release.yml` fires on a `v*` tag to build +
test the release artifact and pin the community descriptor to the tag's commit;
`scripts/release.sh <version>` (version is a **required** arg) stamps its
`Cargo.toml` + `description.yml`, tags, and updates the community PR. That repo's
CI asserts `Cargo.toml` == `description.yml`; the number-tracks-laterite part is
a convention this runbook enforces (a laterite drift-gate can't reach a
foreign repo, and the extension legitimately lags in the window between a
laterite release and its own). A laterite bump with no extension-relevant change
doesn't force an extension release — cut one when the extension actually changes,
at whatever laterite version is then current.

## The docs site

The docs (`web/docs-site/`) deploy to `/laterite/docs/` on **every main push**
(`deploy-validator.yml`) — deliberately *not* gated on the release tag, so a doc
fix ships immediately. The **Changelog page** (`web/docs-site/docs/reference/changelog.md`)
is a short hand-written stub that points readers at root `CHANGELOG.md` and its
breaking-changes index, with the GitHub Releases feed second. It does NOT render
`CHANGELOG.md` and carries no version number — so there is nothing about it to
confirm after a release, and step 6 above says so.

## What keeps it honest

- **Drift-gate** `test_version_faithful.py`: reads the source files (no build)
  and fails CI if the shipped wheel version, the Rust workspace version, the npm
  `package.json` version, its three `@laterite/native-*` pins, or the compat
  `__version__` prefix in
  `packages/laterite/python/laterite/compat/_impl.py` ever disagree — the net
  for a bump that missed a surface.
- **Guard test** `test_compat_python_ags4_pin_stays_in_sync`: fails CI if the
  shipped version carries a PEP 440 `+local` segment (PyPI rejects those), if
  `compat.__version__`'s prefix drifts from the shipped version, or if the
  python-ags4 pin desyncs. It's phase-aware (see below).
- **`release.yml` tag-check**: on a `v*` / `node-v*` tag, asserts the tag matches
  the pyproject / `package.json` version before building — so you can't tag
  `v0.6.0` against a stale `0.5.1` source.

## PyPI project + trusted publisher

One PyPI project ships from this repo:

| project | what | install |
|---|---|---|
| `laterite` | base AGS4 toolkit | `pip install laterite` |

`release.yml`'s `pypi-publish` job uploads the per-platform **wheels** plus a
**source distribution** (built once on the Linux leg) under PyPI trusted
publishing (no API token). The publisher is configured on PyPI → *Publishing*
with: **owner/repo** `niko86/laterite` · **workflow** `release.yml` ·
**environment** `pypi`.

The sdist is the install fallback for any platform without a published wheel
(e.g. ARM Linux, dropped from the wheel matrix at the self-hosted cutover):
`pip install laterite` builds from source. The sdist vendors all six path-dep
Rust crates (validator/core/emit/excel/types/py) and is verified
buildable + installable in isolation (#175).

## The python-ags4 compat pin

`compat.__version__` is `<version>+compat.python-ags4.<PYTHON_AGS4_COMPAT>` — a
PEP 440 *local* identity string (runtime only; **never** the distribution
version, which stays clean for PyPI). It exists so callers reaching
`compat.__version__` / the shimmed `python_ags4.__version__` aren't fooled into
thinking they're on real python-ags4. The pin is **exact** (a python-ags4 minor
can change behaviour silently). See `COMPAT.md` for the phase 1 → 2 → 3
migration; the guard test accepts either phase 1 (pin present) or phase 2 (pin
dropped from `__version__`, constant retained), so the migration won't need a
test edit.
