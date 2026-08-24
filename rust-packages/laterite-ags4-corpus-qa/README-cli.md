# laterite-ags4-corpus-qa

Dev/QA harness that **dogfoods the clean-room Rust AGS4 validator**
against real-world `.ags` files and cross-checks it against the
reference Python library `python-ags4`.

Pipeline (each stage is also a standalone subcommand):

    crawl  →  validate  →  parity

- **crawl** — walk a (network) share, copy a subset of `.ags` files
  into a local corpus, write `manifest.json`.
- **validate** — run the **Rust** validator over them; bucket each
  file (clean / findings / hard-error / panic), record which AGS
  rules fired and how the dictionary edition was resolved, group
  files by identical rule-signature into *clusters*; write
  `report.json`.
- **parity** — take the interesting files (the *triage* set: Rust
  hard-errored / panicked / looked surprising — plus an optional
  random sample) and run them through **python-ags4** too, then
  classify each: `AGREE`, `RUST_ONLY_RULES`, `PYTHON_ONLY_RULES`
  (a Rust under-detection), `RULES_DIFFER` (both, each carrying rules
  the other lacks), `KNOWN_DIVERGENCE` (a documented
  OBSERVATIONS variance), `PYTHON_ERROR`. Writes `parity.json` + an
  ACTION list of genuine divergences worth filing. Optional QA: if
  `uv`/`python-ags4` isn't available it prints a notice and exits 0.

**Parity can be run after the fact.** `crawl`+`validate` are fast and
Python-free. Run `parity` later — even days later, or on another
machine if the corpus dir is shared — it just needs a prior
`report.json`. It defaults to the newest run; `--run-id <id>` targets
an older one. The only requirement is `uv`+`python-ags4` *at parity
time*.

## Commands

    laterite-ags4-corpus-qa crawl    --root <DIR|\\srv\share> (--all | --sample N | --pick)
    laterite-ags4-corpus-qa validate
    laterite-ags4-corpus-qa parity   [--parity-sample N]
    laterite-ags4-corpus-qa run      --root <DIR> (--all | --sample N | --pick)   # all three
    laterite-ags4-corpus-qa baseline (--out <f> | --check <f>)   # findings drift gate
    laterite-ags4-corpus-qa censor   --out-dir <DIR> [--sample N] [--redact "..."]  # anonymise
    laterite-ags4-corpus-qa --readme        # this document
    laterite-ags4-corpus-qa <cmd> --help    # full flags

`baseline` freezes (`--out`) or drift-checks (`--check`) a deterministic,
privacy-scrubbed snapshot of the validator's findings over a manifest —
keyed by content sha256, structural `(rule, line, group, field_index,
severity)` tuples only (no paths/filenames/finding-text), so it's safe to
commit. `--check` exits 1 on any drift. The parser-convergence gate.

`censor` anonymises harvested files for sharing (gather → clean → check):
per the SSOT `sensitive_headings.json` it pseudonymises IDs (refs stay
intact), sets PROJ_ID to the file hash (== the cleaned filename), blanks
coordinates, tokenises names/labs/accreditation/methods/remarks + named geological
formations (`GEOL_FORM`/`GEOL_BGS` — location-revealing offshore), strips
`[GEOLOGICAL UNITS]` from descriptions, deletes non-standard (vendor-
custom) columns/groups + their orphaned DICT/ABBR definitions, and applies
any `--redact "<substring>"` keyword safety-net. Writes hash-named files +
a source-stripped manifest (a drop-in for `validate`/`baseline`).

## Output (global flags, gogcli-style)

`-o/--output table|json|ndjson` (default: `table` on a TTY, `ndjson`
when piped — agent-friendly with no flag), `--json` (= `-o json`),
`--compact` (summary/clusters only, drop the per-file arrays),
`--no-color`, `-q/--quiet` (silence progress), `--dry-run` (mutate
nothing — print the plan and stop), `--no-input` (never prompt; fail
instead — for CI/agents). Results go to **stdout**; progress and
hints to **stderr**.

## Concurrency (which flag parallelizes which stage)

- `crawl --walk-jobs N` — parallel directory walk (default 1 =
  sequential; the dominant cost on a slow network share). Each worker
  shows the nested folder it's descending. Sampling stays
  deterministic under any value when `--seed` is given.
- `crawl --jobs N` — parallel file **copy** (default: CPU cores).
- `validate --jobs N` — parallel validation (default: CPU cores).
- `parity --parity-jobs N` — python-ags4 subprocess fan-out
  (default 2).
- `parity --parity-sample N` / `run --parity-sample N` — also
  parity-check N random non-triage files on top of the triage set
  (default 0). Deterministic with `--seed`.

## Artifacts

Each run writes `<corpus>/runs/<run-id>/{manifest,report,parity}.json`
(re-runs no longer overwrite); `<corpus>/runs/latest` points at the
newest so `validate`/`parity` need no flag. `--run-id <id>` pins a
specific run; explicit `--manifest`/`--report`/`--out` always win.
`<corpus>/harvested/` is the shared, content-addressed file cache —
it accumulates across runs (re-crawl is cheap) and is never
overwritten.

`<corpus>` defaults to `./corpus` (relative to the current directory,
created on demand); override with `--corpus-dir` or `$AGS4_CORPUS_DIR`.
The resolved location is echoed on stderr (`manifest → …`).

## Exit codes

    0  success / no triage items
    1  triage items present (validate) or parity actions (parity/run)
    3  I/O — share unreachable / manifest or report missing
    5  bad args (e.g. no selection mode, --pick without `tui`, or
       --pick with --no-input / no terminal)

## Examples

    # Preview a share without copying anything (fast, mutates nothing)
    laterite-ags4-corpus-qa crawl --root \\srv\share --all --dry-run --walk-jobs 8

    # Full dogfood run, 200-file sample, reproducible
    laterite-ags4-corpus-qa run --root \\srv\share --sample 200 --seed 1

    # Re-parity the latest run's odd files + 50 random extras, as JSON
    laterite-ags4-corpus-qa parity --parity-sample 50 --json | jq '.counts'

    # Just the high-signal summary (token-lean, for an agent)
    laterite-ags4-corpus-qa validate --compact --json

    # Freeze a findings baseline, then drift-check after a code change
    laterite-ags4-corpus-qa baseline --out baselines/corpus.json
    laterite-ags4-corpus-qa baseline --check baselines/corpus.json   # exit 1 on drift

    # Anonymise the whole corpus for sharing (+ a known client name)
    laterite-ags4-corpus-qa censor --out-dir corpus/clean --redact "Acme Geotech"
