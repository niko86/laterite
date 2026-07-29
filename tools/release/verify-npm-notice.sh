#!/usr/bin/env bash
###############################################################################
# verify-npm-notice.sh — assert every npm package we are about to publish
# carries the ©AGS third-party notice.
#
# WHY THIS IS A SCRIPT
#   It used to be inline bash in release.yml, reachable ONLY by pushing a real
#   `node-v*` tag. Its first ever execution was therefore during the live 0.8.0
#   release, and it took three attempts to get right — each one costing a tag
#   move and a failed release run:
#     1. it parsed `npm pack --dry-run --json` with a reader that threw on the
#        runner's npm, and reported all four packages as missing the notice;
#     2. it used `npm pack --prefix "$pkg"`, which does NOT choose the package
#        (npm reads package.json from the CWD), so it silently inspected the
#        ROOT package four times — it could only ever pass all or fail all;
#     3. it piped `tar -tzf` into `grep -q`, which exits on first match; tar
#        then took EPIPE mid-write and, under `set -o pipefail`, turned a
#        package that DOES carry the notice into a FAIL.
#   None of those needed a release to find. As a file it runs at a terminal.
#
# WHY IT VERIFIES THE TARBALL
#   The staging step that copies LICENSE into each package can silently no-op (a
#   renamed npm/ layout, a changed cwd) and still report success. So this asks
#   for the artefact npm would actually upload and looks inside it. A tarball
#   listing has no output schema to drift, unlike `--json`.
#
# USAGE
#   tools/release/verify-npm-notice.sh [<package-dir> ...]
#   Defaults to `.` plus every `npm/*/`, run from rust-packages/laterite-node.
#   Exit 0 = every package carries it; 1 = at least one does not, or a check
#   could not be run (which is reported distinctly, never as a silent pass).
###############################################################################
set -euo pipefail

pkgs=("$@")
if [[ ${#pkgs[@]} -eq 0 ]]; then
  pkgs=(.)
  for d in npm/*/; do [[ -d "$d" ]] && pkgs+=("$d"); done
fi

fail=0
dest="$(mktemp -d)"

for pkg in "${pkgs[@]}"; do
  if [[ ! -f "$pkg/package.json" ]]; then
    echo "  FAIL $pkg — no package.json (the check could not run)"
    fail=1
    continue
  fi
  # `path.resolve` rather than a hardcoded `./` prefix: the npm job passes
  # relative dirs (`.`, `npm/*/`) but the wasm job passes an absolute one, and
  # `./` + an absolute path is `.//private/...`, which does not resolve.
  name="$(node -p "require(require('path').resolve('$pkg','package.json')).name")"

  # `cd` into the package: `npm pack --prefix` does not select what is packed.
  # stderr is deliberately NOT swallowed — it reaches the job log, so a check
  # that cannot run says so rather than masquerading as a failing package.
  if ! tgz="$( cd "$pkg" && npm pack --pack-destination "$dest" | tail -1 )"; then
    echo "  FAIL $name — npm pack errored (the check could not run)"
    fail=1
    continue
  fi

  # List to a FILE, then grep the file — never `tar ... | grep -q`. See (3)
  # above: grep -q's early exit SIGPIPEs tar, and pipefail turns that into a
  # false negative once the listing outgrows the pipe buffer. `$name` is scoped
  # (@laterite/native-*), so flatten the slash for the filename.
  safe="${name//\//_}"
  if ! tar -tzf "$dest/$tgz" > "$dest/$safe.list"; then
    echo "  FAIL $name — could not read the packed tarball (the check could not run)"
    fail=1
    continue
  fi

  if grep -qx 'package/LICENSE' "$dest/$safe.list"; then
    echo "  ok   $name carries LICENSE"
  else
    echo "  FAIL $name would publish WITHOUT the ©AGS notice"
    fail=1
  fi
done

exit "$fail"
