# Releasing laterite

`laterite` and `laterite-ags5` are versioned **in lockstep** (always the same
version — they ship together; the `[ags5]` extra pulls the companion) and follow
**semantic versioning** with a pre-1.0 convention.

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
# 3. Push the branch + the tag (the tag triggers release.yml → wheels → PyPI):
git push && git push --tags
```

`bump-my-version` updates, atomically: both wheel `pyproject.toml`s + the root
umbrella, `compat.py`'s `__version__` base + Checker banner (preserving the
`+compat.python-ags4.<pin>` pin), the cross-wheel dependency floors, and rolls
`CHANGELOG.md`'s `[Unreleased]` into the new dated section. Run `uv lock`
afterwards if the lockfile needs refreshing.

### Pre-releases (RC / dev)

The `.ags5db` format may still change pre-1.0, so cut a release candidate first
when in doubt. Use the **explicit** form (PEP 440 canonical — no hyphen):

```bash
uv run bump-my-version bump --new-version 0.2.0rc1     # then rc2, rc3, ...
uv run bump-my-version bump --new-version 0.2.0         # promote to final
```

## What keeps it honest

- **Guard test** `test_compat_python_ags4_pin_stays_in_sync`: fails CI if the
  shipped version carries a PEP 440 `+local` segment (PyPI rejects those), if
  `compat.__version__`'s prefix drifts from the shipped version, or if the
  python-ags4 pin desyncs. It's phase-aware (see below).
- **`release.yml` tag-check**: on a `v*` tag, asserts the tag matches the
  pyproject version before building — so you can't tag `v0.2.0` against a stale
  `0.1.0` source.

## PyPI projects + trusted publishers

Two **separate** PyPI projects ship from this repo:

| project | what | install |
|---|---|---|
| `laterite` | base AGS4 toolkit | `pip install laterite` |
| `laterite-ags5` | the `.ags5db` companion (the `[ags5]` extra) | pulled by `pip install laterite[ags5]` |

`release.yml`'s `pypi-publish` job uploads **both** wheels under PyPI trusted
publishing (no API token). They do **not** share a publisher — each PyPI project
needs its own, configured on PyPI → *Publishing* (or as a pending publisher)
with: **owner/repo** `niko86/laterite` · **workflow** `release.yml` ·
**environment** `pypi`.

> **`laterite-ags5` first publish (one-time owner action):** it isn't on PyPI
> yet, so add it as a **pending publisher** (PyPI → *Account settings* →
> *Publishing* → *Add a pending publisher*) — project name `laterite-ags5`, the
> values above — **before** the next `v*` release. On first publish PyPI creates
> the project and binds the publisher. Without it, the `pypi-publish` job fails
> on the `laterite-ags5` wheel and `pip install laterite[ags5]` stays
> unresolvable from PyPI.

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
