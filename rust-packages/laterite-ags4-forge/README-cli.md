# laterite-ags4-forge

Evolutionary AGS4 dual-validation dogfood generator. It **generates**
AGS4 files and runs each through the in-process Rust validator and
(when available) the official `python-ags4`, classifying every result
with the shared `laterite-ags4-parity` model (the same `classify`/`reconcile`
+ O-2/O-3/O-26/O-30/O-34 arms `laterite-ags4-corpus-qa` uses). A declarative
strategy file drives the loop; this binary is the deterministic
mutate/validate/report muscle and embeds **no LLM**.

> Status: **complete (P0–P5)** — `check`, `gen`, `run` (the
> evolutionary loop), `minimize` (ddmin), `strategy
> new|validate|explain`, `confidence show|reset|export`, `seed
> vendor`; the persistent adaptive parity-confidence ledger; first
> finding ratified ([[strat-forge-rule10a-relational]] retired the
> Rule-10a parity blind spot; the O-35/O-3-narrow cascade was
> reproduced, not re-opened). See `ags-wiki/tools/laterite-ags4-forge.md` and
> `ags-wiki/concepts/evolutionary-dogfooding.md`.

## Commands

```
laterite-ags4-forge check <file.ags> [--no-oracle] [--timeout S]
laterite-ags4-forge gen [--scaffold minimal|loca-samp] [--inject TOK]... \
               [--validate] [--no-oracle] [--out-dir D]
```

- **check** — dual-validate one existing file; print the Rust rule
  set, the python rule set, and the parity verdict.
- **gen** — synthesize a spec-valid base (`loca-samp` adds a real
  LOCA→SAMP + ABBR relational scaffold, which is what makes the
  Rule 10a/10c/16 parity blind spots single-rule-injectable), apply
  each `--inject` (`none|rule10a|rule10c|rule8|rule5|rule19|rule13`),
  write the `.ags` candidates under `<out>/runs/<id>/`, and
  (`--validate`) dual-validate each.

## Conventions (shared CLI contract)

Results to **stdout** in the resolved mode (`--output table|json|ndjson`,
`--json`; ndjson auto when piped); progress/hints to **stderr**;
`--quiet`, `--no-color`, `--no-input`; `--dry-run` mutates nothing;
`--compact` is the token-lean agent view (drops the per-candidate
array). `--readme` prints this guide.

## Exit codes

```
0  success / no parity action (clean or documented divergence)
1  parity action — a real Rust↔python divergence to triage
3  I/O — file not found / out-dir unwritable
5  bad args (unknown --scaffold / --inject token)
```

## Guarantees

- Generated files are written only under the run dir; a `--out-dir`
  resolving inside `laterite-ags4-validator/tests/fixtures/` is refused
  (that tree is asserted hard-error-free). Confirmed reproducers are
  promoted to `ags-wiki/.bootstrap/probes/`, never `tests/fixtures/`.
- The synthetic base is clean **by construction** from the validator's
  bundled v4.2 standard dictionary; a test asserts `RustResult::Clean`
  on the un-injected base, so a generator bug is a test failure, never
  a reported "finding".
