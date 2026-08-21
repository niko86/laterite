---
type: insight
title: "PA abbreviations NOT case sensitive but PT/PU ARE — asymmetric, easy to mis-validate"
status: hypothesis
tags: [insight]
gap_kind: rule-weakness
severity: med
editions_affected: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
rules: [rule-16-abbr-group, rule-15-unit-group, rule-17-type-group]
proposes_observation: false
feeds_strategy: []
discovered_phase: A
related: [start-here]
sources: [spec-4.2]
---
# PA abbreviations NOT case sensitive but PT/PU ARE — asymmetric, easy to mis-validate

## Claim
> [!note] §3.3: [[PA]] (ABBR) is *not* case sensitive; [[PT]] (TYPE) and [[PU]] (UNIT) *are*. A validator must case-fold PA lookups but not PT/PU. This asymmetry is a latent Rust↔python divergence candidate (does each side case-fold PA consistently?) — flag for Phase B/D probing. hypothesis.

## Evidence
- Spec: `spec:AGS4-4.2-2025.pdf §3.x` (see [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[start-here]] · [[ags4-rules-frozen-dictionary-evolves]]
