---
type: insight
title: "TYPE 'MC' retained but explicitly 'not used in AGS 4.2' (dead type)"
status: confirmed
tags: [insight]
gap_kind: rule-weakness
severity: low
editions_affected: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
rules: []
proposes_observation: false
feeds_strategy: []
discovered_phase: A
related: [start-here]
sources: [spec-4.2]
---
# TYPE 'MC' retained but explicitly 'not used in AGS 4.2' (dead type)

## Claim
> [!note] §3.3 defines [[MC]] (BS1377 moisture content) then states it is *included for legacy reasons and not used in AGS 4.2*. A dead type carried in the spec is a model-debt smell — validators must still classify it; AGS5 should drop legacy-dead types rather than carry them. Confirmed (spec-cited).

## Evidence
- Spec: `spec:AGS4-4.2-2025.pdf §3.x` (see [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[start-here]] · [[ags4-rules-frozen-dictionary-evolves]]
