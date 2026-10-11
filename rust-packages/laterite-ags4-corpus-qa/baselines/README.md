# Finding-value drift baselines

A committed baseline here freezes the **exact findings** the clean-room Rust
validator produces over a fixed corpus — the structural tuple
`(rule, line, group, field_index, severity)` per file, keyed by content
`sha256`. It pins the engine's absolute finding *values*: a shared-engine
change that skews findings *consistently* across every surface (so no
cross-surface comparison can see it) surfaces here as drift.

The baseline carries **no paths, filenames, or finding text** (see
`src/baseline.rs` — it stores only the structural tuple, sha-keyed), so it is
safe to commit even though the corpus itself is private.

## `pyags4-vendor.json`

The validator's findings over the vendored python-ags4 fixture corpus
(`../../laterite-ags4-forge/vendor/pyags4-tests`, 83 files by content). Invariants:
**warnings ON, FYI ON, `check_files` OFF, dict auto**.

### Check it

CI runs this check as the **`corpus-drift`** job in
`.github/workflows/ci.yml`, on every pull request whose paths can move it: the
validator and its parse / reference / types deps, `laterite-ags4-hostopts`,
this crate (baseline included), the vendored fixtures,
`rust-packages/Cargo.lock`, the job's script and the workflow itself (the
`corpus` filter in the `changes` job). A PR that touches none of those skips
the job, which reports success; a scheduled or manually dispatched run of the
workflow always runs it. The job crawls the fixtures, runs the check
below, and hands both JSON outputs to `tools/check_corpus_drift.py`, which
prints one summary line on every run (files checked, walk-skipped, not
selected, only in the baseline, only in the crawl) and, on drift, each moved
fixture by name with its `added`/`removed` findings and the re-freeze command.

To run the same check locally, from the repo root:

```bash
FIX=rust-packages/laterite-ags4-forge/vendor/pyags4-tests
QA=output/qa-drift              # gitignored working space
cargo run --manifest-path rust-packages/Cargo.toml -p laterite-ags4-corpus-qa -- \
  crawl --root "$FIX" --all --corpus-dir "$QA" --no-input -q
cargo run --manifest-path rust-packages/Cargo.toml -p laterite-ags4-corpus-qa -- \
  baseline --check rust-packages/laterite-ags4-corpus-qa/baselines/pyags4-vendor.json \
  --show-warnings --show-fyi --no-check-files --corpus-dir "$QA" --no-input
```

`--check` exits non-zero on any drift and prints the per-file `added`/`removed`
findings.

### Re-freeze it (ratify an intended change)

When a validator change *intentionally* moves findings, re-freeze — swap
`--check <file>` for `--out <file>` in the second command — then commit the new
baseline **in the same PR as the behaviour change**, so the diff shows exactly
which findings moved. An unratified drift is a regression; a ratified one that
changes cross-validator behaviour also wants an `O-N` in `OBSERVATIONS.md`.
