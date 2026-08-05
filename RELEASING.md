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

**There are two version numbers** (#153, split 2026-08-01):

| | Covers | Resolved by | Bumped with |
|---|---|---|---|
| **product** | the Python wheel, the npm `laterite` package + its `@laterite/native-*` addons, the browser package, the `lat` binary, the DuckDB extension | `pip install laterite` · `npm i laterite` | `bump-version.sh product` |
| **engine** | the Rust workspace and the ten crates.io engine crates | `cargo add laterite-ags4-validator` | `bump-version.sh engine` |

Every **product** still shares one number, so `pip install laterite==X` and
`npm i laterite@X` are the same release. The engine moves on its own, because it
answers to a different audience and a different registry.

> [!IMPORTANT] **A bump and a release are the same act.** If you stamp a product
> version, cut *every* product tag — not just the surfaces that changed. They
> share one number, so a product left un-cut leaves a version that exists in this
> tree and on no registry. That is not hypothetical: 0.8.1 and 0.8.2 were stamped
> for a browser-only fix and tagged `wasm-v*` alone, so PyPI went 0.8.0 → 0.9.0
> and wheel 0.8.1/0.8.2 were never published at all. Re-shipping an unchanged
> product is the cost of the shared number.

`tools/release/bump-version.sh <product|engine>` drives the in-repo bump
(wrapping [`bump-my-version`](https://callowayproject.github.io/bump-my-version/)
— product config in the root `pyproject.toml` `[tool.bumpversion]`, engine config
in `tools/release/engine-version.toml` — plus lockfile regeneration). The target
is required; there is no default, because bumping the wrong tier by omission is
exactly the failure the split exists to prevent. The DuckDB extension lives in
its own repo and takes the **product** number when you cut it (below). The **docs
site** carries the version too — it's derived at build and republishes on merge.

Two crates sit outside both tiers on purpose: `laterite` (its own `0.1.x` until
it reaches feature parity with the Python and Node surfaces, then it joins the
product line) and `laterite-cli` (the product number, so `lat --version` agrees
with the wheel's `lat`).

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

> Cutting a release on a **new** tag fires **two** workflow runs — a `push` (tag) run
> and a `release` run — because GitHub raises both events. That's expected: the `push`
> run builds + publishes; the `release` run no-ops (jobs skip on `release`). They no
> longer cancel each other (the concurrency key includes `github.event_name`;
> laterite#264). If you ever see the build run *cancelled* at ~3s, that regression is
> back — re-run the `push` run.

`bump-version.sh product` stamps, atomically in one commit: the `laterite` wheel
`pyproject.toml` + the root umbrella, `compat.py`'s `__version__` base + Checker
banner (preserving the `+compat.python-ags4.<pin>` pin), `laterite-cli`'s crate
version, the npm `package.json` version, and rolls `CHANGELOG.md`'s
`[Unreleased]` into the new dated section — then regenerates `uv.lock`,
`Cargo.lock`, the npm `package-lock.json`
(the last one avoids the `EUSAGE` failure `npm ci` throws at publish against a
stale lock), and the generated napi loader `index.js` (the node CI job runs
`napi build` + `git diff --exit-code`, so the committed loader's version literals
must match a fresh build — a stale one reds that guard). `web/src/wasm/package.json`
is gitignored and wasm-pack regenerates it from the crate version; the docs
Changelog page derives its version at build. Neither needs stamping.

### Cutting an engine release (crates.io)

Separate from the product flow above, and usually *before* it: the engine is the
substrate every product is rebuilt from.

```bash
git switch -c release/engine-0.10.0
tools/release/bump-version.sh engine minor   # stamps rust-packages/Cargo.toml only
gh pr create -B main -t "release: engine 0.10.0" -b "engine version bump"

# once merged, from main:
uv run --no-sync python tools/publish_crates.py             # dry run — prints the waves
uv run --no-sync python tools/publish_crates.py --execute
```

`publish_crates.py` derives the dependency waves from the manifests, waits for
each wave to become *resolvable* from the registry before starting the next, and
is idempotent — a re-run after a failure resumes rather than restarting. It
refuses a dirty tree, any branch but `main`, and any crate marked
`publish = false`.

**An engine release reaches nobody on its own.** Every product is built from
these crates and keeps shipping the previous engine until it is rebuilt, so
follow with a product bump. Engine changelog entries stay under `[Unreleased]`
until that product release rolls them — which is the release a reader can
actually install.

The facade crate `laterite` is published by the same tool but carries its **own**
`0.1.x`, bumped by hand in `rust-packages/laterite/Cargo.toml`. It is neither
tier until it reaches parity with the Python and Node surfaces.

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
is a short hand-written stub that points readers at the GitHub Releases feed. It
does NOT render `CHANGELOG.md` and carries no version number — so there is
nothing about it to confirm after a release, and step 6 above says so.

## What keeps it honest

- **Drift-gate** `test_version_faithful.py`: reads the source files (no build)
  and fails CI if the shipped wheel version, the Rust workspace version, the npm
  `package.json` version, its three `@laterite/native-*` pins, or the `compat.py`
  prefix ever disagree — the net for a bump that missed a surface.
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
