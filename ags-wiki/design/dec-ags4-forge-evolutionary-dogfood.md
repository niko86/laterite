---
type: decision
title: "Evolutionary dual-validation dogfood tool (laterite-ags4-forge)"
status: accepted
tags: [design, decision]
decided: 2026-05-17
supersedes: []
from_gap: [parity-triage-sampling-bias, parity-cascade-unreconcilable]
related: [parity-triage-sampling-bias, parity-cascade-unreconcilable, laterite-ags4-forge, laterite-ags4-parity, agent-first-cli-contract, evolutionary-dogfooding, parity-confidence-model, dec-forge-audience-boundary, dec-forge-type-axis-instrument]
sources: []
---
# Evolutionary dual-validation dogfood tool (laterite-ags4-forge)

## Context
The clean-room "Rust ≡ python except enumerated O-Ns" claim
([[parity-model]]) — which AGS5 conformance will inherit via
req-reproducible-conformance-corpus — is only ever exercised
against *found* corpora. [[strat-parity-matrix]] proves **13 rules
have zero differential evidence**; [[parity-triage-sampling-bias]]
([[O-36]]) and [[parity-cascade-unreconcilable]] ([[O-35]]) explain
why a crawl cannot reach them. We need *manufactured, proven*
divergences, with the strategy chosen by the agent+wiki, not brute force.

## Options considered
1. Extend [[laterite-ags4-corpus-qa]] with a generator — conflates a found-corpus
   crawler with a synthesizer; bloats one binary.
2. **New `laterite-ags4-forge` crate + extract a shared `laterite-ags4-parity` crate**
   (classify/reconcile/PyOracle/Rng) consumed by both; fold
   `Ctx/Report/emit/Plan` into [[laterite-cliutil]].
3. LLM-in-the-binary vs a **declarative strategy file** the agent authors.
4. Fixed oracle sampling vs an **adaptive confidence model** with a floor.

## Decision
Option 2 + declarative TOML strategy + the
[[parity-confidence-model]]; the binary embeds **no LLM**;
[[evolutionary-dogfooding]] is the loop; wiki design-capture is the
gating first step (this page set). Naming/UX follow the
[[agent-first-cli-contract]].

## Why
Extract-over-duplicate is the standing stance that created
[[laterite-cliutil]]; reusing the *exact* `classify`/`reconcile` keeps
forge's "real divergence" definitionally identical to the established
[[parity-model]] (a `KnownDivergence{O-N}` is by construction never a
finding). A declarative strategy keeps the binary deterministic and
reproducible; the confidence model spends the scarce python oracle
where it informs while a floor + fingerprint-reset keep the trust
honest.

## Consequences
Commits to: a 5th + 6th workspace crate (`laterite-ags4-parity`, `laterite-ags4-forge`)
and a `report` module in [[laterite-cliutil]]; a behaviour-neutral
[[laterite-ags4-corpus-qa]] refactor (its parity unit tests move and must stay
green); the validator library's lean dep-graph stays intact (parity
depends on the validator, never the reverse); confirmed findings flow
through the §12.5 insight→`OBSERVATIONS.md` O-N path; generated
reproducers live in `ags-wiki/.bootstrap/probes/`, **never**
`laterite-ags4-validator/tests/fixtures/`.

## Related
[[parity-triage-sampling-bias]] · [[parity-cascade-unreconcilable]] · req-reproducible-conformance-corpus · [[laterite-ags4-forge]] · [[laterite-ags4-parity]] · [[agent-first-cli-contract]] · [[evolutionary-dogfooding]] · [[parity-confidence-model]] · [[design/_README\|AGS5 register]]
