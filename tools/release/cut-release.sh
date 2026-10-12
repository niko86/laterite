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
# WHAT IT DOES NOT DO — same as `bump-version.sh`, for the same reasons:
#   * No tag, no push. The bump lands on `main` via a reviewed release PR; the
#     publish tags are cut after it merges. See RELEASING.md.
#   * No publish. The registries are append-only and the publishes are
#     environment-gated on purpose. This prints the sequence; a human runs it.
#   * It does not touch `laterite-duckdb`. Separate repo, downstream of the
#     crates.io index; printed as a follow-up, never done.
#
# Usage:
#   tools/release/cut-release.sh 0.12.0           # stamp the product
#   tools/release/cut-release.sh 0.12.0 --plan    # print the plan and stop
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

die() {
  echo "cut-release: $*" >&2
  exit 1
}

[ "$#" -ge 1 ] || die "usage: cut-release.sh <version> [--plan]  (see header)"

VERSION="$1"; shift
PLAN_ONLY=0
while [ "$#" -gt 0 ]; do
  case "$1" in
    --plan) PLAN_ONLY=1 ;;
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

=== engine crates ===

  The engine cut above stamped every crate owed a bump. After the release PR
  merges, each crate (and the facade) ahead of the registry is published by the
  nightly's publish dispatch (#1073), or at once with:
    gh workflow run publish-crates.yml --ref main -f execute=true

EOF
