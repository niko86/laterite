#!/usr/bin/env bash
# Cut a PRODUCT release, with the plan printed before anything is stamped.
#
# This used to cut BOTH tiers at one number (#716): the engine was ten crates in
# lockstep, and the second of two bump decisions was easy to not make. #781
# retired the lockstep (2026-08-30) — each published engine crate versions
# per-crate, bumped by `tools/release/bump_crate.py` when IT changes, published
# whenever a bump lands. So there is no engine tier left for this script to
# stamp, and no `--skip` to choose: a product cut is the only cut there is.
#
# It does apply the ENGINE CUT first, though: the owed per-crate bumps and the
# coherence cascade, exactly as the nightly does. The surfaces compile engine
# source, but the crates.io facade this script stamps resolves its engine deps
# from the REGISTRY — so a product cut over an engine fix that was never bumped
# ships a facade that cannot see the fix. Leaving those bumps to a later nightly
# made every release two PRs and a day long, and was easy to forget.
#
# What survives from #716 is the shape a release decision should have: the
# release_status report in front of the stamp, the guardrails, and the ordered
# follow-ups. What does NOT survive is worth naming: this script used to print
# "the products will ship the engine already on the registry" for a skipped
# engine, which was never true — the surfaces compile engine SOURCE from the
# tree (bare `path` deps, no version), and the registry is only what
# `laterite-duckdb` and direct Rust consumers see.
#
# TWO STEPS, because crates.io and the tags can only take what is on `main`:
#   1. On a release branch: the engine cut + the product stamp, as commits for a
#      reviewed release PR. No tag, no push.
#   2. After that PR merges, on `main` with `--ship`: every product tag, on the
#      release commit. The tags start every publish: `v*` builds PyPI and
#      dispatches crates.io (the engine crates the cut stamped + the facade),
#      `node-v*` / `wasm-v*` build npm — all in release.yml. A tag that already
#      exists is skipped, so a re-run after a partial failure finishes the job.
#
# WHAT IT STILL DOES NOT DO:
#   * Approve anything. The `pypi` / `npm` environments stay a human's click.
#   * Touch `laterite-duckdb`. Separate repo, downstream of the crates.io index;
#     printed as a follow-up, never done.
#
# Usage:
#   tools/release/cut-release.sh 0.12.0           # step 1: stamp (release branch)
#   tools/release/cut-release.sh 0.12.0 --plan    # print the plan and stop
#   tools/release/cut-release.sh 0.12.0 --ship    # step 2: every tag (merged main)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

die() {
  echo "cut-release: $*" >&2
  exit 1
}

[ "$#" -ge 1 ] || die "usage: cut-release.sh <version> [--plan|--ship]  (see header)"

VERSION="$1"; shift
PLAN_ONLY=0
SHIP=0
while [ "$#" -gt 0 ]; do
  case "$1" in
    --plan) PLAN_ONLY=1 ;;
    --ship) SHIP=1 ;;
    --skip) die "--skip is gone with the engine tier (#781) — engine crates bump per-crate via bump_crate.py" ;;
    *) die "unknown argument '$1'" ;;
  esac
  shift
done

# A release number, not a part. The part is a decision this script reports on
# (see release_status.py) but never makes silently — picking `minor` for someone
# is exactly the kind of help that ships the wrong number.
echo "$VERSION" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+([a-z0-9.]*)?$' \
  || die "'$VERSION' does not look like a version (expected e.g. 0.12.0)"

branch="$(git branch --show-current)"

# --- step 2: ship the merged release. ----------------------------------------
if [ "$SHIP" -eq 1 ]; then
  [ "$branch" = "main" ] || die "--ship runs on main, after the release PR merges"
  [ -z "$(git status --porcelain)" ] || die "working tree is dirty — commit or stash first"
  git fetch -q origin main
  head="$(git rev-parse HEAD)"
  [ "$head" = "$(git rev-parse origin/main)" ] \
    || die "main is not origin/main — 'git pull --ff-only' first"
  # The tags must sit on the release commit itself, never on a later merge.
  case "$(git log -1 --format=%s)" in
    "release: $VERSION"*) ;;
    *) die "main's head is not the merged 'release: $VERSION' commit — tag that commit by hand, or ship before anything else merges" ;;
  esac
  facade="$(sed -n 's/^version = "\(.*\)"/\1/p' rust-packages/laterite/Cargo.toml | head -1)"
  [ "$facade" = "$VERSION" ] || die "the facade is stamped $facade, not $VERSION"

  for tag in "v$VERSION" "node-v$VERSION" "wasm-v$VERSION"; do
    if git ls-remote --exit-code --tags origin "refs/tags/$tag" >/dev/null; then
      echo "tag $tag: already on origin, skipped"
      continue
    fi
    case "$tag" in
      # v* / node-v* are releases, cut server-side (lightweight, so they match
      # the release.yml triggers). The target is the full SHA: the API refuses
      # an abbreviated one.
      v*|node-v*) gh release create "$tag" --target "$head" --title "$tag" --generate-notes ;;
      # --no-sign: tag.gpgsign would make a bare 'git tag' a signed annotated tag.
      wasm-v*) git tag --no-sign "$tag" "$head" && git push origin "$tag" ;;
    esac
    echo "tag $tag: cut on $head"
  done

  echo
  echo "=== left for a human ==="
  echo "  1. Approve the 'pypi' / 'npm' environments on the release runs:"
  echo "       gh run list --workflow release.yml --limit 6"
  echo "  2. The DuckDB extension (its own repo) pins ENGINE crates — update its"
  echo "     pins once crates.io shows the new versions, then cut it on its own number."
  exit 0
fi

[ "$branch" != "main" ] || die "refuse to cut on main — 'git switch -c release/$VERSION' first"
[ -z "$(git status --porcelain)" ] || die "working tree is dirty — commit or stash first"

# --- the plan, before anything is stamped. -----------------------------------
echo "=== what is unreleased ==="
uv run --no-sync python tools/release/release_status.py
echo
# The same reading the nightly cut takes (nightly.yml), so the two cannot
# disagree about what is owed.
status_json="$(mktemp)"; bumps_txt="$(mktemp)"
trap 'rm -f "$status_json" "$bumps_txt"' EXIT
uv run --no-sync python tools/release/release_status.py --json > "$status_json"
uv run --no-sync python tools/release/engine_cut.py --bumps "$status_json" > "$bumps_txt"

echo "=== what this will stamp ==="
printf '  %-8s -> %s\n' "product" "$VERSION"
if [ -s "$bumps_txt" ]; then
  while read -r crate part; do
    printf '  %-8s -> %s %s\n' "engine" "$crate" "$part"
  done < "$bumps_txt"
  echo "  (then the coherence cascade, which may bump the crates pinning those)"
else
  echo "  (no engine bump owed; the coherence cascade still runs)"
fi
echo

if [ "$PLAN_ONLY" -eq 1 ]; then
  echo "cut-release: --plan given, stopping before any stamp."
  exit 0
fi

# --- the engine cut, committed on its own so bump-version.sh (which refuses a
#     dirty tree) stamps the product on top of it.
while read -r crate part; do
  echo "--- bump_crate.py $crate $part"
  uv run --no-sync python tools/release/bump_crate.py "$crate" "$part"
done < "$bumps_txt"
echo "--- engine_cut.py --cascade"
uv run --no-sync python tools/release/engine_cut.py --cascade
if git diff --quiet -- rust-packages; then
  echo "engine cut: nothing owed"
else
  git add -A
  git commit -m "release: engine cut for $VERSION"
fi

# --- stamp. bump-version.sh owns the commit and prints the tag/publish
#     follow-ups (every product tag, the environment approvals, the duckdb
#     downstream note).
echo "--- bump-version.sh product --new-version $VERSION"
tools/release/bump-version.sh product --new-version "$VERSION"

cat <<EOF

=== after the release PR merges ===

  On the merged main, one command cuts every product tag; the tags publish
  PyPI, npm and crates.io (the engine crates and the facade):
    git switch main && git pull --ff-only
    tools/release/cut-release.sh $VERSION --ship

EOF
