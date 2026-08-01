#!/usr/bin/env bash
# Build the Rust `lat` AGS4 CLI in release mode.
#
# Mac/Linux sibling of build-rust.ps1. `lat` is the shipped standalone binary —
# the small, fast AGS4 validator/reader (validate, read, fix, diff, merge,
# certify, pack/lock, …). The Python `laterite` wheel is the primary library
# surface; this script is for the CLI on its own.
#
# Run from repo root (or anywhere — the script resolves its own path):
#     tools/build-rust.sh
#
# Output:
#     dist/lat                             the AGS4 CLI binary
#     rust-packages/target/release/        cargo build cache (gitignored)
#
# Requires: cargo (Rust toolchain, install via `rustup`). The CLI is DuckDB-free,
# so a cold build is a couple of minutes; incremental builds are ~10 s.

set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo=$(cd "$script_dir/.." && pwd)

(
  cd "$repo/rust-packages"
  # `-p laterite-cli` builds the crate that carries the `lat` binary.
  cargo build --release -p laterite-cli
)

src="$repo/rust-packages/target/release/lat"
dst="$repo/dist/lat"
if [[ ! -f $src ]]; then
  echo "Build reported success but $src is missing." >&2
  exit 1
fi
mkdir -p "$repo/dist"
cp -f "$src" "$dst"

size_mb=$(awk -v b="$(wc -c < "$dst")" 'BEGIN{printf "%.2f", b/1024/1024}')
echo
echo "Built $dst ($size_mb MB)"
echo "Smoke test:  $dst --version"
