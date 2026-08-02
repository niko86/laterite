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
# TWO NUMBERS, and you must say which (since 2026-08-01).
#
#   product  the wheel, the npm package, `lat`, the browser package. What
#            `pip install laterite` and `npm i laterite` resolve.
#   engine   the Rust workspace and the eight crates.io crates. What
#            `cargo add laterite-ags4-validator` resolves.
#
# They were one number, and the coupling had a cost with a name.
# `laterite-ags4-wasm` is `version.workspace = true`, so shipping a browser-only
# patch meant bumping the number every surface shared. It happened at 0.8.1 and
# again at 0.8.2 — and because the advice below used to be "tag only what
# changed", PyPI never saw either. Wheel versions 0.8.1 and 0.8.2 exist in git
# history and on no registry.
#
# THE RULE THAT REPLACED IT: a bump and a release are the same act. If you stamp
# a product version, every product ships at it — including the ones whose bytes
# did not change. That is what the shared number costs, and it is what buys
# `pip install laterite==X` and `npm i laterite@X` meaning the same thing.
#
# Usage:
#   tools/release/bump-version.sh product minor              # 0.5.1 -> 0.6.0
#   tools/release/bump-version.sh product patch              # 0.6.0 -> 0.6.1
#   tools/release/bump-version.sh engine  minor              # the crates.io tier
#   tools/release/bump-version.sh product --new-version 0.6.0rc1
#   DRY_RUN=1 tools/release/bump-version.sh product minor    # stamp + regen, no commit
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

die() {
  echo "bump-version: $*" >&2
  exit 1
}

[ "$#" -ge 2 ] || die "usage: bump-version.sh <product|engine> <part|--new-version X>  (see header)"

# The target is required rather than defaulted. A default here would be a way to
# bump the wrong tier by omission, and the whole point of the split is that the
# two are not interchangeable.
target="$1"; shift
#
# The expansions below are the guarded `${CONFIG[@]+...}` form rather than a
# plain quoted one. Under `set -u`, bash 3.2 -- which is what macOS still ships
# -- treats an EMPTY array's `[@]` expansion as an unbound variable and aborts.
# The product tier is exactly the empty case (it uses bump-my-version's default
# config), so the plain form made `bump-version.sh product` die on macOS from the
# moment the two-tier split introduced the array, while `engine` kept working
# because its array is never empty.
case "$target" in
  product) CONFIG=(); SURFACES="wheel + umbrella + compat + npm + lat" ;;
  engine)  CONFIG=(--config-file tools/release/engine-version.toml)
           SURFACES="Rust workspace + [workspace.dependencies] (the crates.io tier)" ;;
  *) die "unknown target '$target' — expected 'product' or 'engine'" ;;
esac

# --- guardrails: a release bump belongs on a clean release branch, not the trunk.
branch="$(git branch --show-current)"
[ "$branch" != "main" ] || die "refuse to bump on main — cut a release/X branch first"
[ -z "$(git status --porcelain)" ] || die "working tree is dirty — commit or stash first"

old="$(uv run --no-sync bump-my-version show ${CONFIG[@]+"${CONFIG[@]}"} current_version)"

# --- 1. stamp the tracked version strings for THIS tier.
#         --no-commit/--no-tag: we fold the lock regen into one commit, and the
#         publish tags come after the release PR merges.
uv run --no-sync bump-my-version bump ${CONFIG[@]+"${CONFIG[@]}"} --no-commit --no-tag "$@"
new="$(uv run --no-sync bump-my-version show ${CONFIG[@]+"${CONFIG[@]}"} current_version)"
[ "$new" != "$old" ] || die "version did not change ($old) — nothing to do"
echo "bump-version: $target $old -> $new  ($SURFACES)"

if [ "$target" = "product" ]; then
  # --- 1b. stamp the generated napi loader (see header). Every literal is the
  #         version; the node CI drift guard verifies this matches a fresh build.
  #         Product-only: the loader carries the npm package's version.
  old_re="${old//./\\.}"
  sed -i.bak "s/${old_re}/${new}/g" rust-packages/laterite-node/index.js
  rm -f rust-packages/laterite-node/index.js.bak

  # --- 1c. roll the CHANGELOG. It is generated from changelog.json (the SSOT):
  #         `--release` moves [Unreleased] into a dated `[$new]` section in the
  #         JSON and regenerates CHANGELOG.md. Refuses if [Unreleased] is empty
  #         (nothing to release) and runs the leak-gate over every entry.
  #
  #         Product-only, deliberately. The changelog is the record of what
  #         REACHED someone, and an engine bump on its own reaches nobody — the
  #         crates are the substrate every product is rebuilt from. Engine work
  #         stays under [Unreleased] and rolls with the product release that
  #         actually ships it, which is the release a reader can install.
  echo "bump-version: rolling the changelog ([Unreleased] -> $new)…"
  uv run --no-project python tools/gen_changelog.py --release "$new"
fi

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

if [ "$target" = "engine" ]; then
cat <<EOF
bump-version: committed 'release: engine $new' on branch '$branch'.
Next (see RELEASING.md):
  1. Open a PR into main and merge it.
  2. Publish the crates.io tier from the merged main:
       uv run --no-sync python tools/publish_crates.py            # dry run first
       uv run --no-sync python tools/publish_crates.py --execute
  3. An engine bump reaches NOBODY on its own — every product is built from these
     crates and still ships the previous engine until it is rebuilt. Follow with:
       tools/release/bump-version.sh product <part>
     Engine changelog entries stay under [Unreleased] until that product release
     rolls them, which is the release a reader can actually install.
EOF
else
cat <<EOF
bump-version: committed 'release: $new' on branch '$branch'.
Next (see RELEASING.md):
  1. Open a PR into main and merge it (this also republishes the docs at
     /laterite/docs/ with $new + its notes — derived at build, nothing to run).
  2. On the merged main, cut EVERY product tag. Not "the ones that changed" —
     they share one number, so the number has to mean the same thing everywhere,
     and a product left un-cut leaves a version that exists in this tree and on
     no registry. That is not hypothetical: 0.8.1 and 0.8.2 were stamped for a
     browser-only fix and tagged \`wasm-v*\` alone, so PyPI went 0.8.0 -> 0.9.0 and
     wheel 0.8.1/0.8.2 were never published at all.
       gh release create v$new       --title v$new       --generate-notes  # wheels -> PyPI, CLI -> GH release
       gh release create node-v$new  --title node-v$new  --generate-notes  # npm 'laterite' + @laterite/native-*
       git tag --no-sign wasm-v$new && git push origin wasm-v$new           # npm '@laterite/ags4-wasm' (browser)
     (--no-sign because tag.gpgsign is on here: a bare 'git tag' implies a
      SIGNED ANNOTATED tag, dies with 'fatal: no tag message?', and would not
      match v*/node-v* anyway — those are lightweight, cut server-side by
      'gh release create'.)
     Re-shipping a product whose bytes did not change is the cost of the shared
     number, and it is the thing that makes \`pip install laterite==$new\` and
     \`npm i laterite@$new\` the same release. If you want one product to ship
     alone, that is a different versioning scheme — see the design page, not a
     shortcut here.
  3. Approve the 'pypi' / 'npm' environments on the resulting Actions runs.
     (The 'npm' env gates node-v* AND wasm-v*; see RELEASING-wasm.md.)
  4. Cut the DuckDB extension at the SAME version (its own repo):
       cd <niko86/laterite-duckdb> && bash scripts/release.sh $new
EOF
fi
