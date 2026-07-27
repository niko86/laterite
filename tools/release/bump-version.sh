#!/usr/bin/env bash
# One-command version bump across every published surface (#372).
#
# Before this, a bump was a split ritual: `bump-my-version` stamped the Python
# wheel + Rust workspace + CHANGELOG, but the npm `package.json` version and its
# three `@laterite/native-*` optionalDeps were hand-edited on a separate track,
# and nobody regenerated the lockfiles — so a release could ship with the wheel
# at X and the addon resolving a stale native at X-1. This drives the whole
# bump from one place: bump-my-version (now configured to stamp the node
# manifest too, see [tool.bumpversion] in pyproject.toml) followed by
# regenerating every lockfile, folded into one `release: X` commit.
#
# What it does NOT do — on purpose:
#   * No tag. The publish tags (`vX`, `node-vX`) are cut after the release PR
#     merges to `main` — see RELEASING.md. They fire `release.yml`, which builds
#     and publishes under this repo's PyPI/npm trusted publishers.
#   * No push. The bump lands on `main` via a reviewed release PR.
#
# It DOES stamp the generated napi loader `index.js`: the node CI job runs
# `napi build` then `git diff --exit-code`, so the committed loader must match a
# fresh build's version literals — a stale one turns that drift guard red. We
# sed the version (a version-only change is exactly what `napi build` would
# reproduce); the CI drift guard is the backstop if napi's template ever shifts.
#
# Usage:
#   tools/release/bump-version.sh minor              # 0.5.1 -> 0.6.0
#   tools/release/bump-version.sh patch              # 0.6.0 -> 0.6.1
#   tools/release/bump-version.sh --new-version 0.6.0rc1   # explicit pre-release
#   DRY_RUN=1 tools/release/bump-version.sh minor     # stamp + regen, no commit
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

die() {
  echo "bump-version: $*" >&2
  exit 1
}

[ "$#" -ge 1 ] || die "usage: bump-version.sh <part|--new-version X>  (see header)"

# --- guardrails: a release bump belongs on a clean release branch, not the trunk.
branch="$(git branch --show-current)"
[ "$branch" != "main" ] || die "refuse to bump on main — cut a release/X branch first"
[ -z "$(git status --porcelain)" ] || die "working tree is dirty — commit or stash first"

old="$(uv run --no-sync bump-my-version show current_version)"

# --- 1. stamp every tracked version string (Python wheel + umbrella + compat
#         prefix + Rust workspace + node package.json/optionalDeps + CHANGELOG
#         roll). --no-commit/--no-tag: we fold the lock regen into one commit,
#         and the publish tags come after the release PR merges.
uv run --no-sync bump-my-version bump --no-commit --no-tag "$@"
new="$(uv run --no-sync bump-my-version show current_version)"
[ "$new" != "$old" ] || die "version did not change ($old) — nothing to do"
echo "bump-version: $old -> $new"

# --- 1b. stamp the generated napi loader (see header). Every literal is the
#         version; the node CI drift guard verifies this matches a fresh build.
old_re="${old//./\\.}"
sed -i.bak "s/${old_re}/${new}/g" rust-packages/laterite-node/index.js
rm -f rust-packages/laterite-node/index.js.bak

# --- 1c. roll the CHANGELOG. It is generated from changelog.json (the SSOT):
#         `--release` moves [Unreleased] into a dated `[$new]` section in the
#         JSON and regenerates CHANGELOG.md. Refuses if [Unreleased] is empty
#         (nothing to release) and runs the leak-gate over every entry. Replaces
#         the old bump-my-version text-substitution (removed from [tool.bumpversion]).
echo "bump-version: rolling the changelog ([Unreleased] -> $new)…"
uv run --no-project python tools/gen_changelog.py --release "$new"

# --- 2. regenerate every lockfile so it carries the new version.
echo "bump-version: regenerating lockfiles…"
uv lock --quiet                                   # uv.lock (workspace wheel version)
( cd rust-packages && cargo update --workspace --quiet )   # Cargo.lock workspace members
( cd rust-packages/laterite-node && npm install --package-lock-only --ignore-scripts --silent )  # package-lock.json

# --- 3. sanity: the cross-surface drift-gate must pass on the freshly bumped tree.
echo "bump-version: verifying version faithfulness across surfaces…"
uv run --no-sync pytest tests/test_version_faithful.py -q

if [ "${DRY_RUN:-}" = "1" ]; then
  echo "bump-version: DRY_RUN — changes staged, not committed. Review with 'git diff', then"
  echo "              commit yourself or 'git checkout .' to discard."
  exit 0
fi

# --- 4. one reviewed-release commit. No tag, no push — the release PR + tags follow.
git add -A
git commit -m "release: $new"

cat <<EOF
bump-version: committed 'release: $new' on branch '$branch'.
Next (see RELEASING.md):
  1. Open a PR into main and merge it (this also republishes the docs at
     /laterite/docs/ with $new + its notes — derived at build, nothing to run).
  2. On the merged main, cut BOTH tags to publish:
       gh release create v$new       --title v$new       --generate-notes  # wheels -> PyPI, CLI -> GH release
       gh release create node-v$new  --title node-v$new  --generate-notes  # npm addon + @laterite/native-*
  3. Approve the 'pypi' / 'npm' environments on the resulting Actions runs.
  4. Cut the DuckDB extension at the SAME version (its own repo):
       cd <niko86/laterite-duckdb> && bash scripts/release.sh $new
EOF
