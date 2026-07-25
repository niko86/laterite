#!/usr/bin/env bash
#
# Generate the criterion bench fixtures.
#
# The benches need a realistic multi-group AGS4 file, and for a long time the
# only one was `examples/output/large.ags` — a real 23 MB delivery that is
# gitignored working space. That made the single existing bench a no-op
# everywhere except the one machine that happened to have the file: it
# self-skips when the fixture is absent, so `cargo bench` reported success
# while measuring nothing. A perf gate you cannot run is not a perf gate.
#
# `forge scale` removes the dependency: it synthesises a valid AGS4 file
# calibrated to a target byte size, and same size + seed produces a
# byte-identical file. So the fixtures are reproducible on any machine and on
# CI, and they carry no real delivery data — nothing here can leak a client's
# file into a benchmark run.
#
# Output goes to `output/bench-fixtures/` (gitignored). Re-running is cheap and
# idempotent; the files are regenerated rather than cached.
#
# Usage:
#     ./tools/gen-bench-fixtures.sh          # build forge if needed, generate
#     ./tools/gen-bench-fixtures.sh --force  # regenerate even if present

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT_DIR="$REPO_ROOT/output/bench-fixtures"
FORGE="$REPO_ROOT/rust-packages/target/release/laterite-ags4-forge"

# Seed 0 everywhere: the point is a fixed file, not a varied one. A bench that
# measures a different file each run measures noise.
SEED=0
SCAFFOLD=wide

# label:size. `wide` is the ~50-group scaffold, so these exercise the
# many-groups path a real delivery has, not one enormous LOCA.
RUNGS=(
    "small:1MB"
    "medium:10MB"
    "large:25MB"
)

force=0
[ "${1:-}" = "--force" ] && force=1

if [ ! -x "$FORGE" ]; then
    echo "building laterite-ags4-forge (release) — first run only..."
    (cd "$REPO_ROOT/rust-packages" && cargo build --release -p laterite-ags4-forge)
fi

mkdir -p "$OUT_DIR"

for rung in "${RUNGS[@]}"; do
    label="${rung%%:*}"
    size="${rung##*:}"
    out="$OUT_DIR/$label.ags"
    if [ -f "$out" ] && [ "$force" -eq 0 ]; then
        echo "  $label.ags exists — skipping (use --force to regenerate)"
        continue
    fi
    "$FORGE" scale --size "$size" --scaffold "$SCAFFOLD" --seed "$SEED" --out "$out" \
        | head -n 1
done

# The DIRTY rung (T5): a `wide` base with a combined multi-fault injection, so the
# validator's error-reporting half — `findings::add`, rule 10b/11c dirty paths,
# the FYI abbr scan — actually executes under `validate/error-path`. `forge gen`
# is UNSCALED (~100 groups, a handful of findings): a size-scaled densely-dirty
# fixture needs a `forge scale` fault-density mode that does not exist yet. Seed 0
# → byte-identical. gen writes to a timestamped run dir, so copy the candidate out
# to the stable name the bench reads.
dirty_out="$OUT_DIR/dirty.ags"
if [ -f "$dirty_out" ] && [ "$force" -eq 0 ]; then
    echo "  dirty.ags exists — skipping (use --force to regenerate)"
else
    dirty_tmp="$(mktemp -d)"
    "$FORGE" gen --scaffold "$SCAFFOLD" \
        --combine rule10a,rule10c,rule8,rule5,rule19,rule13 \
        --seed "$SEED" --no-oracle --no-input --quiet --out-dir "$dirty_tmp" >/dev/null
    cp "$(find "$dirty_tmp" -name '*.ags' | head -n 1)" "$dirty_out"
    rm -rf "$dirty_tmp"
    echo "  dirty.ags generated ($(wc -c < "$dirty_out" | tr -d ' ') bytes)"
fi

echo
echo "fixtures in $OUT_DIR:"
ls -la "$OUT_DIR"/*.ags
echo
echo "run the benches with:  (cd rust-packages && cargo bench)"
