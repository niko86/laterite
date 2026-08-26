#!/usr/bin/env bash
# Cut BOTH tiers at one number, in one commit (#716).
#
# `bump-version.sh` stamps one tier and tells you to follow with the other. That
# instruction is correct and was not followed: between engine 0.9.0 (#184) and
# this being written the product shipped three times while the engine stayed
# put, so crates.io consumers — and `laterite-duckdb`, which pins the engine
# crates FROM THE REGISTRY — sat four figures of public API behind what every
# `pip install laterite` user was running.
#
# Nothing was wrong with the two-tier scheme. What was missing is that the two
# bumps were two decisions, and the second one is easy to not make. This makes
# them one.
#
# WHAT IT DOES NOT DO — the same three as `bump-version.sh`, for the same reasons:
#   * No tag, no push. The bump lands on `main` via a reviewed release PR; the
#     publish tags are cut after it merges. See RELEASING.md.
#   * No publish. crates.io is APPEND-ONLY — a published version can never be
#     withdrawn or re-cut — and the publish is environment-gated on purpose
#     (#463 was filed about doing the least reversible step by hand). This
#     prints the sequence; a human runs it.
#   * It does not touch `laterite-duckdb`. That is a separate repo which builds
#     against the engine crates from crates.io, so it can only move AFTER this
#     release is live on the index. It is printed as a follow-up, never done.
#
# TWO NUMBERS THAT USUALLY MATCH, NOT ONE NUMBER.
#
# This stamps both tiers to the same version, which is the common case and what
# makes `pip install laterite==X`, `npm i laterite@X` and
# `cargo add laterite-ags4-validator@X` mean the same release. But they stay two
# fields, and `--skip` is why: crates.io is append-only, so burning eleven crate
# versions on a browser-only fix is a cost worth being able to decline. That is
# exactly the case the 2026-08-01 split was bought with — 0.8.1 and 0.8.2 were
# browser-only. Collapsing the two into one field would take the option away
# permanently.
#
# Usage:
#   tools/release/cut-release.sh 0.12.0                 # both tiers
#   tools/release/cut-release.sh 0.12.0 --skip engine   # product only (browser-only fix)
#   tools/release/cut-release.sh 0.12.0 --skip product  # engine only (rare; see RELEASING.md)
#   tools/release/cut-release.sh 0.12.0 --plan          # print the plan and stop
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

die() {
  echo "cut-release: $*" >&2
  exit 1
}

[ "$#" -ge 1 ] || die "usage: cut-release.sh <version> [--skip engine|product] [--plan]  (see header)"

VERSION="$1"; shift
SKIP=""
PLAN_ONLY=0
while [ "$#" -gt 0 ]; do
  case "$1" in
    --skip) shift; SKIP="${1:-}"; [ -n "$SKIP" ] || die "--skip needs 'engine' or 'product'" ;;
    --plan) PLAN_ONLY=1 ;;
    *) die "unknown argument '$1'" ;;
  esac
  shift
done
case "$SKIP" in
  ""|engine|product) ;;
  *) die "--skip takes 'engine' or 'product', not '$SKIP'" ;;
esac

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
echo "=== what this will stamp ==="
tiers=()
[ "$SKIP" = "engine" ]  || tiers+=("engine")
[ "$SKIP" = "product" ] || tiers+=("product")
[ "${#tiers[@]}" -gt 0 ] || die "both tiers skipped — nothing to do"
for t in "${tiers[@]}"; do
  printf '  %-8s -> %s\n' "$t" "$VERSION"
done
if [ "$SKIP" = "engine" ]; then
  echo "  engine SKIPPED — no crates.io publish. The products will ship the engine"
  echo "  already on the registry, which is the intended use for a browser-only fix."
fi
echo

if [ "$PLAN_ONLY" -eq 1 ]; then
  echo "cut-release: --plan given, stopping before any stamp."
  exit 0
fi

# --- stamp. Each bump-version.sh call makes its own commit (and each demands a
#     clean tree, so they cannot be folded with DRY_RUN). They are squashed into
#     one below: two commits both reading `release: X` is a history that cannot
#     be read, and the release is one act.
made=0
for t in "${tiers[@]}"; do
  echo "--- bump-version.sh $t --new-version $VERSION"
  tools/release/bump-version.sh "$t" --new-version "$VERSION"
  made=$((made + 1))
done

if [ "$made" -gt 1 ]; then
  git reset --soft "HEAD~${made}"
  git commit -m "release: $VERSION (engine + product)"
  echo
  echo "cut-release: squashed $made tier bumps into one 'release: $VERSION' commit."
fi

cat <<EOF

=== next, in this order (the order is forced by what depends on what) ===

  1. Open a PR into main and merge it.

  2. Publish the ENGINE first — every product is built from these crates:
       uv run --no-sync python tools/publish_crates.py            # rehearsal
       uv run --no-sync python tools/publish_crates.py --execute
     then approve the 'crates' environment on the run.

  3. Cut EVERY product tag. Not "the ones that changed" — they share one number,
     so a product left un-cut leaves a version that exists in this tree and on no
     registry (0.8.1 and 0.8.2 are exactly that).
       gh release create v$VERSION      --title v$VERSION      --generate-notes
       gh release create node-v$VERSION --title node-v$VERSION --generate-notes
       git tag --no-sign wasm-v$VERSION && git push origin wasm-v$VERSION
     then approve the 'pypi' / 'npm' environments.

  4. The DuckDB extension is DOWNSTREAM and lives in its own repo. It pins the
     engine crates from crates.io, so it can only move once step 2 is on the
     index — and its requirement is a caret range, so it will NOT pick up a new
     minor on its own. Edit the pins, then cut it:
       cd <niko86/laterite-duckdb>
       # bump laterite-ags4-* requirements to $VERSION, then:
       bash scripts/release.sh $VERSION

EOF
