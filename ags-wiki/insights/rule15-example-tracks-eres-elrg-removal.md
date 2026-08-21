---
type: insight
title: "Rule 15's worked example ERES_RUNI→ELRG_RUNI@4.1 — a frozen rule whose example went stale as the model moved"
status: confirmed
tags: [insight]
gap_kind: cross-edition-regression
severity: med
editions_affected: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
rules: [rule-15-unit-group]
proposes_observation: false
feeds_strategy: []
discovered_phase: A
related: [rule-15-unit-group, ags4-rules-frozen-dictionary-evolves, ERES]
sources: [spec-4.2]
---
# Rule 15's example tracked the ERES→ELRG model change (frozen rule, stale example)

## Claim
> [!variance] Rule 15 (UNIT group) is normatively frozen 4.0.3→4.2,
> yet its illustrative text was edited: "(for example **ERES_RUNI**,
> GCHM_UNIT or MOND_UNIT FIELDs)" in 4.0.3/4.0.4 became
> "**ELRG_RUNI**…" in 4.1/4.1.1/4.2 — because the dictionary replaced
> the ERES group with ELRG at 4.1. The *rule* didn't change; its
> *example referenced a group that ceased to exist*.

## Evidence (primary)
- 4.0.3 §8.1 Rule 15 / 4.0.4 §8.1 Rule 15: `ERES_RUNI` (read).
- 4.1 §4.1.1 / 4.1.1 / 4.2 Rule 15: `ELRG_RUNI` (read).
- ERES lifecycle: CNMT → **ERES new @4.0.3** (replaces CNMT, 4.0.3
  Foreword) → **`ELRG` replaces ERES @4.1** (4.1 Foreword: 24 new
  groups "replaced Groups such as ERES, IPRG and IPRT") → **ERES
  removed @4.2** (4.2 Foreword). See [[ERES]].

## Compounding repo-side finding
> [!divergence] The repo's `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json`
> declares `ags_edition: 4.1` yet still contains **ERES** and **lacks
> `ELRG`** — i.e. the repo's own model authority *also* failed to
> track the 4.1 ERES→ELRG replacement. Double bookkeeping failure:
> the spec's Rule 15 example went stale, and the downstream dictionary
> didn't apply the replacement either. (Phase B should confirm whether
> the validator can therefore even *recognise* `ELRG`.)

## Why it matters
The single clearest concrete proof of
[[ags4-rules-frozen-dictionary-evolves]]: embedding model-specific
identifiers in rule prose creates silent staleness windows (a 4.0.4
file validated against the 4.0.4 Rule 15 example would cite a group
the 4.1 dictionary no longer defines). Direct
AGS5 requirement: rules must not
hard-reference model entities; examples belong in the dictionary, not
the rule text. Test-strategy probe: does either validator recognise
`ELRG`, and special-case `ERES_RUNI` vs `ELRG_RUNI` per edition?

## Related
[[rule-15-unit-group]] · [[ags4-rules-frozen-dictionary-evolves]] · [[ERES]] · [[edition-resolution]]
