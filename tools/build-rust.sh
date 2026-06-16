#!/usr/bin/env bash
# Build the Rust read-side `lat-db` CLI in release mode.
#
# Mac/Linux sibling of build-rust.ps1. The Rust binary is the small-
# and-fast counterpart to the Python CLI (`ags5db-py`, installed via
# `uv tool install ./packages/ags5-db`). It ships at ~25-36 MB and
# starts in <100 ms, replacing the PyInstaller bundle (117.8 MB / 7.6 s
# cold) for the read-side commands.
#
# Run from repo root (or anywhere — the script resolves its own path):
#     tools/build-rust.sh
#
# Output:
#     dist/lat-db                          the canonical read-side binary
#     rust-packages/target/release/        cargo build cache (gitignored)
#
# Requires: cargo (Rust toolchain). Installed via `rustup`. The first
# build compiles bundled libduckdb from source and takes ~5-10 min;
# incremental builds are ~10 s.

set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
repo=$(cd "$script_dir/.." && pwd)

(
  cd "$repo/rust-packages"
  # `--bin lat-db` builds only the binary crate. The sibling crates
  # in the workspace aren't needed for shipping the read-side CLI.
  cargo build --release --bin lat-db
)

src="$repo/rust-packages/target/release/lat-db"
dst="$repo/dist/lat-db"
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
