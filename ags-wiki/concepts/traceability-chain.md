---
type: concept
title: traceability chain
status: drafted
tags: [concept]
ags_editions: []
repo_refs: {}
related: [start-here]
sources: []
---
# traceability chain

## Definition
> [!quote] Every rule traces: spec §4.1.1 → src/rules/<family>.rs::rule_N → tests/fixtures/ruleN_*.ags → tests/regression.rs::ruleN_*_flagged → OBSERVATIONS O-N. Lint enforces the triad; a missing leg is a gap.

## Why it matters
Load-bearing for the test strategy: this is how the validator (or the parity harness) actually behaves — gaps surface as deltas against the spec (Phase A) and python-ags4 (Phase C/D).

## Diagram

```mermaid
flowchart LR
  S["spec: AGS4-4.2 §4.1.1 Rule N"] --> I["src/rules/<family>.rs"]
  I --> F["tests/fixtures/ruleN_*.ags"]
  F --> T["tests/regression.rs::ruleN_*"]
  T --> O["OBSERVATIONS.md O-N"]
```

## Where it shows up
Load-bearing across the rule families that depend on it — followed end-to-end by the [[traceability-chain]] and surfaced as deltas in [[parity-model]].

## Related
[[start-here]] · [[parity-model]] · [[rule-families]]
