#!/usr/bin/env bash
#
# Validate the python-ags4 drop-in CLAIMS with evidence.
#
# Runs python-ags4 1.2.0's OWN test suite through `laterite.compat` (the
# Rust-backed drop-in) and reports BOTH:
#   * PARITY  — how many of python-ags4's tests pass against laterite
#               (the README's "121 / 131" claim), and
#   * COVERAGE — how much of `laterite.compat` (and laterite overall) that
#                external suite actually exercises.
# Together: the drop-in is faithful (tests pass) AND genuinely covered (the
# claimed surface is really run), not just asserted.
#
# Reuses tools/run_python_ags4_tests.sh for the clone-shim-run plumbing (it
# shims `python_ags4` → `laterite.compat` via a generated conftest and forwards
# "$@" to pytest). This wrapper auto-clones the pinned sibling and layers --cov.
#
# Usage:  ./tools/parity-coverage.sh            # full suite + coverage
#         PYTHON_AGS4_VERSION=1.2.0 PARITY_MIN_PASSING=122 ./tools/parity-coverage.sh
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SIBLING="$REPO_ROOT/../ags-python-library"
PY_AGS4_VERSION="${PYTHON_AGS4_VERSION:-1.2.0}"
# The floor is 121, not 122: O-47 made test_rule_6_2 a CORRECT divergence and
# lowered it. parity.yml was updated at the time; this default was not, and
# nor was any published claim (laterite-dev#556). The SET is now the contract —
# parity-known-failures.json + tools/check_parity.py; this stays a count only
# because this script is a local coverage helper, not the gate.
PARITY_MIN="${PARITY_MIN_PASSING:-121}"
OUT="$REPO_ROOT/output"            # gitignored working space
LOG="$OUT/parity-coverage.log"
XML="$OUT/parity-coverage.xml"
mkdir -p "$OUT"

# 1. Clone (or reuse) python-ags4 at the pinned tag — its wheel doesn't ship
#    tests/, so we need the source. Shallow + single-tag is enough.
if [ ! -d "$SIBLING/tests" ]; then
    echo "==> cloning python-ags4 $PY_AGS4_VERSION -> $SIBLING"
    # python-ags4's tags have NO `v` prefix (the tag is `1.2.0`, not `v1.2.0`).
    git clone --depth 1 --branch "$PY_AGS4_VERSION" \
        https://gitlab.com/ags-data-format-wg/ags-python-library "$SIBLING"
else
    echo "==> reusing python-ags4 sibling at $SIBLING ($(cd "$SIBLING" && git describe --tags 2>/dev/null || echo '?'))"
fi

# 2. Run their suite via laterite.compat WITH coverage of laterite. The runner
#    forwards these flags to pytest; the shim makes `import python_ags4` resolve
#    to laterite.compat, so --cov=laterite captures what the external suite drives.
echo "==> running python-ags4's suite through laterite.compat (+ --cov=laterite)"
set +e
"$REPO_ROOT/tools/run_python_ags4_tests.sh" \
    --cov=laterite --cov-report=term-missing "--cov-report=xml:$XML" \
    2>&1 | tee "$LOG"
set -e

# 3. Extract parity counts + coverage from the captured output.
passed=$(grep -Eo '[0-9]+ passed' "$LOG" | tail -n1 | awk '{print $1}'); passed=${passed:-0}
failed=$(grep -Eo '[0-9]+ failed' "$LOG" | tail -n1 | awk '{print $1}'); failed=${failed:-0}
total=$((passed + failed))
# compat.py row + the TOTAL row from the term-missing table (last % on each line).
compat_cov=$(grep -E 'laterite/compat\.py' "$LOG" | grep -Eo '[0-9]+%' | tail -n1)
total_cov=$(grep -E '^TOTAL'              "$LOG" | grep -Eo '[0-9]+%' | tail -n1)

echo
echo "================ parity + compat-coverage ================"
printf '  parity:            %s passed / %s failed  (of %s; claim >= %s)\n' \
    "$passed" "$failed" "$total" "$PARITY_MIN"
printf '  laterite.compat:   %s line coverage driven by the python-ags4 suite\n' \
    "${compat_cov:-n/a}"
printf '  laterite (total):  %s line coverage driven by the python-ags4 suite\n' \
    "${total_cov:-n/a}"
echo "  (xml: $XML  |  log: $LOG)"
echo "=========================================================="

# 4. Gate on parity (coverage is reported, not gated — it's evidence, not a floor).
if [ "$passed" -lt "$PARITY_MIN" ]; then
    echo "::error::parity regressed: $passed passing < $PARITY_MIN required" >&2
    exit 1
fi
echo "parity claim upheld ($passed >= $PARITY_MIN)."
