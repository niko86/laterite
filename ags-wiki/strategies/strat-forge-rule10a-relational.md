---
type: strategy
title: "Probe: forge-synthesized LOCA→SAMP duplicate KEY tuple (Rule 10a blind spot)"
status: confirmed
tags: [strategy]
targets: [rule-10a-key-uniqueness]
divergence_hypothesis: "Rule 10a has zero differential parity evidence because clean_minimal lacks a relational base; a forge Mode-B synthesized LOCA→SAMP scaffold with one duplicated KEY tuple may expose a Rust↔python disagreement (or prove AGREE and retire a blind spot)."
probe_files: [ags-wiki/.bootstrap/probes/probe-forge-rule10a-relational.ags]
expected_rust: "Rule 10a (duplicate KEY tuple within SAMP under one LOCA)"
expected_python: "Rule 10a (rule_10a duplicate key) — expected AGREE; to be confirmed by probe"
evidence: "laterite-ags4-forge gen(loca-samp,inject rule10a) → ddmin → probe-forge-rule10a-relational.ags; reproduced via lat (exit 1, Rule 10a) AND uv run tools/py_ags4_check_json.py (Rule 10a fired), cwd repo root → forge classify = AGREE"
related: [rule-10a-key-uniqueness, parity-model, evolutionary-dogfooding, laterite-ags4-forge, strat-parity-matrix, O-35, O-03]
sources: []
---
# Probe: forge-synthesized LOCA→SAMP duplicate KEY tuple (Rule 10a blind spot)

## Hypothesis
> [!divergence] Rule 10a is one of the **13 zero-evidence blind
> spots** in [[strat-parity-matrix]]: the minimal fixtures have no
> parent→child relational base, so single-rule-isolable 10a evidence
> never exists. [[laterite-ags4-forge]] Mode-B synthesizes a spec-valid
> `LOCA→SAMP` scaffold (real KEY tuples), dual-validates the
> *un-injected* baseline clean in both validators, then injects
> exactly one duplicated SAMP KEY tuple. Outcome is either a confirmed
> Rust↔python divergence *or* a proven AGREE that retires the blind
> spot — both are wins. The executable twin is the first
> `strategy.toml` (target `rules=["10a"]`, `relational_scaffold =
> ["LOCA->SAMP"]`).

## Probe design
- Fixture(s): `ags-wiki/.bootstrap/probes/probe-forge-rule10a-*.ags`
  (the ddmin-minimized reproducer, written **only** after the run —
  under `.bootstrap/probes/`, NEVER
  `laterite-ags4-validator/tests/fixtures/`).
- Run: `lat validate <probe>` and
  `uv run python tools/py_ags4_check_json.py <probe>` (cwd = repo
  root), i.e. the same dual path [[laterite-ags4-forge]] automates via
  [[laterite-ags4-parity]].

## Expected vs observed

| | Rust `lat` | python-ags4 |
|---|---|---|
| expected | Rule 10a (duplicate KEY tuple) | Rule 10a (`rule_10a`) — expected AGREE |
| observed (probe) | findings incl. **Rule 10a** (+ Rule 8), exit 1 | **Rule 10a** fired (+ Rule 8) — same rule set |

## Verdict
> [!note] **CONFIRMED — AGREE. The Rule-10a differential blind spot is
> RETIRED.** [[laterite-ags4-forge]] Mode-B synthesized the `LOCA→SAMP`+`ABBR`
> relational base the [[strat-parity-matrix]] PROJ/TRAN/UNIT/TYPE base
> structurally *could not* provide; the un-injected base validates
> clean in both validators (P2), and the duplicated SAMP KEY tuple
> makes **both** `lat` and python-ags4 fire `AGS Format Rule
> 10a` (presence sets match → `classify` = `AGREE`, not an ACTION).
> So Rule 10a now has real differential evidence and the two AGREE —
> no divergence, **no O-N opened**.
>
> Separately, the same evolutionary `run` independently reproduced the
> **already-ratified** [[O-35]] presence-only cascade / [[O-03]]-narrow
> via the `rule5:unquoted` injector (Rust → Rule 5; python parse-layer
> cascades → Rule 3-by-position; `reconcile()`'s O-3 arm covers only
> the Rule-4 variant) — surfaced as `RUST_ONLY_RULES`. This is
> **recognised as the known O-35, NOT a new observation**: opening an
> O-N here would be gratuitous churn of the canonical authority
> (`AGS-WIKI.md` §12.5). `OBSERVATIONS.md` is therefore **unchanged**;
> forge's value here is the *independent reproduction* of O-35.

## Related
[[rule-10a-key-uniqueness]] · [[parity-model]] · [[evolutionary-dogfooding]] · [[laterite-ags4-forge]] · [[strat-parity-matrix]] · [[O-35]] · [[O-03]]
