# Releasing laterite

`laterite` is the single shipped wheel and follows **semantic versioning** with a
pre-1.0 convention. (The experimental `.ags5db` companion `laterite-ags5` was
decoupled to the dormant `ags5/` holding folder in #177 — it is no longer built
or published; a future AGS5 strand will publish it separately.)

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

**Every published surface shares one version** (#372): the Python wheel, the
Rust workspace (so the `lat` binary), the npm `laterite` package with its
three `@laterite/native-*` addons, and the `laterite_ags4` DuckDB extension all
move on one number. `tools/release/bump-version.sh` drives the in-repo bump
(wrapping [`bump-my-version`](https://callowayproject.github.io/bump-my-version/),
config in the root `pyproject.toml` `[tool.bumpversion]`, plus lockfile
regeneration); the DuckDB extension lives in its own repo and takes the same
number when you cut it (below). The **docs site** carries the version too — it's
derived at build and republishes on merge. **Never hand-edit a version string**
— `test_version_faithful.py`, the compat guard, and the `release.yml` tag-check
all catch drift.

> [!IMPORTANT] **Releases PUBLISH FROM THE PUBLIC MIRROR `niko86/laterite`, NOT this
> repo.** `laterite`'s `release.yml` builds + tests the matrix, but only the
> *mirror's* `release.yml` publishes — the PyPI/npm trusted publishers are configured
> for `niko86/laterite`. **Do NOT `git push --tags` to `laterite`**: it builds, then
> the publish step fails with `invalid-publisher` (by design). The release tags go on
> the **mirror** (see step 3).

Before bumping, finish the `CHANGELOG.md` `[Unreleased]` section (the bump rolls
it into the dated release). Then:

```bash
# 1. On a release branch (the script refuses to run on master or a dirty tree),
#    bump every surface + regenerate uv.lock / Cargo.lock / package-lock.json,
#    verify the drift-gate, and make one "release: X" commit (no tag, no push):
git switch -c release/0.6.0
tools/release/bump-version.sh minor          # or: patch  ·  --new-version 0.6.0rc1
#    (DRY_RUN=1 tools/release/bump-version.sh minor  stamps + regenerates without committing)

# 2. master is PROTECTED → land the bump via a release PR (merge-commit, NOT squash):
git push -u origin release/0.6.0
gh pr create -B master -t "release: 0.6.0" -b "version bump"   # merge once CI is green

# 3. Sync the bumped tree to the mirror, then cut BOTH tags THERE:
tools/release/push-public-tree.sh --push     # carries the bump → niko86/laterite
gh release create v0.6.0      --repo niko86/laterite   # wheels + sdist → PyPI, CLI → GH release
gh release create node-v0.6.0 --repo niko86/laterite   # npm addon + @laterite/native-*
# 4. Approve the `pypi` / `npm` environments in the resulting Actions runs (the OIDC gates).

# 5. Cut the DuckDB extension at the SAME version (its own repo — see below):
cd <the niko86/laterite-duckdb checkout> && bash scripts/release.sh 0.6.0
# 6. Confirm the docs republished: /laterite/docs/ Reference → Changelog shows 0.6.0.
```

The two tags stay separate because `release.yml` has independent `v*` (Python +
CLI) and `node-v*` (npm) build/publish paths — but the **number** is now one, so
you cut `v0.6.0` and `node-v0.6.0` from the same release. A Python-only patch can
still skip the `node-v*` tag; the version simply doesn't move on npm until you do.

> Cutting a release on a **new** tag fires **two** workflow runs — a `push` (tag) run
> and a `release` run — because GitHub raises both events. That's expected: the `push`
> run builds + publishes; the `release` run no-ops (jobs skip on `release`). They no
> longer cancel each other (the concurrency key includes `github.event_name`;
> laterite#264). If you ever see the build run *cancelled* at ~3s, that regression is
> back — re-run the `push` run.

`bump-version.sh` stamps, atomically in one commit: the `laterite` wheel
`pyproject.toml` + the root umbrella, `compat.py`'s `__version__` base + Checker
banner (preserving the `+compat.python-ags4.<pin>` pin), the Rust workspace
version, the npm `package.json` version + its three `@laterite/native-*`
optionalDeps, and rolls `CHANGELOG.md`'s `[Unreleased]` into the new dated
section — then regenerates `uv.lock`, `Cargo.lock`, the npm `package-lock.json`
(the last one avoids the `EUSAGE` failure `npm ci` throws at publish against a
stale lock), and the generated napi loader `index.js` (the node CI job runs
`napi build` + `git diff --exit-code`, so the committed loader's version literals
must match a fresh build — a stale one reds that guard). `web/src/wasm/package.json`
is gitignored and wasm-pack regenerates it from the crate version; the docs
Changelog page derives its version at build. Neither needs stamping.

### Pre-releases (RC / dev)

The API may still change pre-1.0, so cut a release candidate first when in doubt.
Use the **explicit** form (PEP 440 canonical — no hyphen):

```bash
tools/release/bump-version.sh --new-version 0.6.0rc1     # then rc2, rc3, ...
tools/release/bump-version.sh --new-version 0.6.0         # promote to final
```

## The DuckDB extension (`laterite_ags4`)

The extension is a **separate repo, `niko86/laterite-duckdb`**, published through
DuckDB **community-extensions** — not from this repo and not from the mirror. As
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

The docs (`web/docs-site/`) deploy to `/laterite/docs/` on **every master push**
(`deploy-validator.yml`) — deliberately *not* gated on the release tag, so a doc
fix ships immediately. The **Changelog page** (`reference/changelog.md`, generated
by `scripts/gen_changelog.py`) renders the root `CHANGELOG.md` and stamps the
shipped version, both **derived at build**. So merging the release PR to master
republishes the docs with the new version + notes automatically — step 6 is just
a confirmation, nothing to run.

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
