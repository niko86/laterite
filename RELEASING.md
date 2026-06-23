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

The version lives in one logical place, stamped everywhere by
[`bump-my-version`](https://callowayproject.github.io/bump-my-version/) (config
in the root `pyproject.toml` `[tool.bumpversion]`). **Never hand-edit a version
string** — the guard test + the release tag-check will catch drift.

```bash
# 1. Pick the bump per the policy above (preview first):
uv run bump-my-version bump minor --dry-run -v     # or: patch
# 2. Apply it — stamps all sites, rolls CHANGELOG, commits "release: X", tags vX:
uv run bump-my-version bump minor
# 3. Push the branch + the tag (the tag triggers release.yml → wheels + sdist → PyPI):
git push && git push --tags
```

`bump-my-version` updates, atomically: the `laterite` wheel `pyproject.toml` + the
root umbrella, `compat.py`'s `__version__` base + Checker banner (preserving the
`+compat.python-ags4.<pin>` pin), the Rust workspace version, and rolls
`CHANGELOG.md`'s `[Unreleased]` into the new dated section. Run `uv lock`
afterwards if the lockfile needs refreshing.

### Pre-releases (RC / dev)

The API may still change pre-1.0, so cut a release candidate first when in doubt.
Use the **explicit** form (PEP 440 canonical — no hyphen):

```bash
uv run bump-my-version bump --new-version 0.2.0rc1     # then rc2, rc3, ...
uv run bump-my-version bump --new-version 0.2.0         # promote to final
```

### The npm `laterite` (node) track — separate, manual

The npm `laterite` package (+ the three `@laterite/native-*` platform packages) is
versioned on its **own `node-v*` tag track**, *independent* of the Python wheel.
`bump-my-version` does **not** touch `rust-packages/laterite-node/package.json` (the
napi crate's *Cargo* version rides the workspace, but the published npm version does
not), so a Python `v0.5.0` release leaves npm `laterite` at its current version until a
node release is cut separately. To cut one:

```bash
# 1. Bump package.json "version" AND the three optionalDependencies pins
#    (@laterite/native-*) to the new node version — they must match.
# 2. Commit, then tag node-v<version> and push the tag (triggers release.yml's
#    npm publish, which checks the tag == package.json version):
git tag node-v0.5.0 && git push origin node-v0.5.0
```

Keep the npm version aligned with the wheels when practical (the last alignment was a
deliberate manual commit), but a Python release does **not** require a simultaneous node
release.

## What keeps it honest

- **Guard test** `test_compat_python_ags4_pin_stays_in_sync`: fails CI if the
  shipped version carries a PEP 440 `+local` segment (PyPI rejects those), if
  `compat.__version__`'s prefix drifts from the shipped version, or if the
  python-ags4 pin desyncs. It's phase-aware (see below).
- **`release.yml` tag-check**: on a `v*` tag, asserts the tag matches the
  pyproject version before building — so you can't tag `v0.2.0` against a stale
  `0.1.0` source.

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
