#!/usr/bin/env bash
###############################################################################
# prepare-wasm-package.sh — turn a `wasm-pack build` output directory into the
# package we actually publish as `@laterite/ags4-wasm`.
#
# WHY THIS IS A SCRIPT
#   The npm notice guard used to be inline bash in release.yml, reachable only
#   by pushing a real tag — so its first execution was a live release, and it
#   took three attempts and three tag moves to get right (see
#   verify-npm-notice.sh's header). Everything on the publish path is a file now
#   so it can be run, and failed, at a terminal instead.
#
# WHAT IT CHANGES, AND WHY EACH ONE
#   1. `name` → `@laterite/ags4-wasm`. wasm-pack derives the package name from
#      the CRATE name (`laterite-ags4-wasm`), and its `--scope` flag would give
#      `@laterite/laterite-ags4-wasm`. Neither is the published name, so the
#      generated manifest is rewritten rather than fought with.
#   2. `publishConfig.access = "public"`. Scoped packages default to PRIVATE, a
#      paid feature — publishing one without this fails with `E402 you must sign
#      up for private packages`. That is exactly how the 0.1.0 npm shakedown
#      failed for the `@laterite/native-*` packages (RELEASING-node.md).
#   3. A `LICENSE` alongside. The wasm binary EMBEDS the AGS4 dictionary (the
#      reference leaf's `include_str!`), so the ©AGS third-party notice has to
#      ride with it exactly as it does with each `.node`. The published 0.7.0
#      npm packages shipped verbatim ©AGS text under a bare `"license": "MIT"`
#      with no notice at all; this is that bug's wasm twin, fixed before it can
#      ship rather than after. npm includes a LICENSE regardless of the `files`
#      array, so placing the file is enough — but `verify-npm-notice.sh` is what
#      proves it, by looking inside the tarball npm would upload.
#   4. `repository` / `homepage` / `bugs`, if absent — wasm-pack does not derive
#      them, and a published package with no repository link is a dead end.
#
# USAGE
#   tools/release/prepare-wasm-package.sh <pkg-dir> [<license-file>]
#   <license-file> defaults to LICENSE at the repo root.
#   Exit 0 = the directory is ready to `npm publish`; 1 = it is not, and why.
###############################################################################
set -euo pipefail

pkg="${1:?usage: prepare-wasm-package.sh <pkg-dir> [<license-file>]}"
here="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
license="${2:-$here/LICENSE}"

PUBLISHED_NAME="@laterite/ags4-wasm"

if [[ ! -f "$pkg/package.json" ]]; then
  echo "  FAIL $pkg has no package.json — did wasm-pack build succeed?" >&2
  exit 1
fi
if [[ ! -f "$license" ]]; then
  echo "  FAIL no license file at $license" >&2
  exit 1
fi

# The .wasm is the whole point of the package; a manifest without it is a
# packaging fault that would otherwise publish an empty shell.
if ! ls "$pkg"/*.wasm >/dev/null 2>&1; then
  echo "  FAIL $pkg contains no .wasm — refusing to prepare an empty package" >&2
  exit 1
fi

cp "$license" "$pkg/LICENSE"

PUBLISHED_NAME="$PUBLISHED_NAME" node -e '
const fs = require("fs");
const p = process.argv[1] + "/package.json";
const m = JSON.parse(fs.readFileSync(p, "utf8"));

m.name = process.env.PUBLISHED_NAME;
m.publishConfig = { ...(m.publishConfig || {}), access: "public" };
m.repository = m.repository || { type: "git", url: "git+https://github.com/niko86/laterite.git" };
m.homepage = m.homepage || "https://niko86.github.io/laterite/";
m.bugs = m.bugs || { url: "https://github.com/niko86/laterite/issues" };

fs.writeFileSync(p, JSON.stringify(m, null, 2) + "\n");
console.log(`  ok   ${m.name}@${m.version} prepared (access=${m.publishConfig.access})`);
' "$pkg"
