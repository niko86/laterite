---
type: rule
title: "Rule <NN><sub> — <short name>"
status: stub
tags: [rule]
rule_number: <NN>
rule_sub: ""              # "" | a | b | c | b_1 | b_2 | b_3
rule_family: ""           # line|structure|naming|dictionary|typed|groups|relational|references
varies_between_editions: false
divergences: []           # [O-NN] this rule diverges from python-ags4 via
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  impl: ""                # repo:rust-packages/laterite-ags4-validator/src/rules/<file>.rs::<fn>
  fixtures: []            # [repo:rust-packages/laterite-ags4-validator/tests/fixtures/ruleNN_*.ags]
  regression: []          # [repo:rust-packages/laterite-ags4-validator/tests/regression.rs::ruleNN_*]
  spec: ""                # spec:AGS4-4.2-2025.pdf §4.1.1 Rule NN
related: []
sources: [AGS4-4.2-2025.pdf]
---

# Rule <NN><sub> — <short name>

## Statement
> [!todo] Ingest: one-line normative statement. Cite `spec:` — never paste spec prose.

## Rule Family
<!-- TODO: which of the 8 families + why. Link [[rule-families]]. -->

## Implementation (this repo)
<!-- TODO: cite repo: impl symbol; summarise, don't duplicate logic. -->

## Traceability Chain

```mermaid
flowchart LR
  R["Rule <NN><sub>"] --> I["impl: <file>.rs"]
  I --> F["fixture(s)"]
  F --> T["regression test"]
  T --> O["[[O-NN]] observations"]
```

## Variations
> [!note] Cross-edition deltas (4.0.3 → 4.2) and Rust↔python-ags4 divergence.

```mermaid
%% TODO (Ingest): edition-delta / divergence diagram
flowchart LR
  todo[fill at Ingest]
```

- Edition deltas: <!-- TODO -->
- Divergence: <!-- > [!divergence] link [[O-NN]] or "none" -->

## Related
<!-- [[rule-families]] · [[traceability-chain]] · [[O-NN]] · [[<GROUP>]] · [[ags-4.2]] -->
