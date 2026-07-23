---
type: rule
title: Rule 19b — heading prefix
status: drafted
tags: [rule]
rule_number: 19
rule_sub: b
rule_family: naming
varies_between_editions: false
divergences: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  impl: "repo:rust-packages/laterite-ags4-validator/src/rules/naming.rs"
  fixtures: "repo:rust-packages/laterite-ags4-validator/tests/fixtures/rule19b_bad_prefix.ags repo:rust-packages/laterite-ags4-validator/tests/fixtures/rule19b_borrowed_bad.ags"
  regression: "repo:rust-packages/laterite-ags4-validator/tests/regression.rs::rule19b_bad_prefix_flagged repo:rust-packages/laterite-ags4-validator/tests/regression.rs::rule19b_unknown_borrowed_prefix_flagged"
  spec: "spec:AGS4-4.2-2025.pdf §4.1.1 Rule 19b"
related: [rule-families, traceability-chain, parity-model]
sources: [spec-4.2]
---
# Rule 19b — heading prefix

## Statement
> [!quote] AGS4 Rule 19b — `spec:AGS4-4.2-2025.pdf §4.1.1 Rule 19b`
> HEADING names shall start with the GROUP name followed by an underscore character e.g. "NGRP_HED1". Where a HEADING refers to an existing HEADING within another GROUP, the HEADING name added to the group shall bear the same name. e.g. "CMPG_TESN" in the "CMPT" GROUP.

Rule **normative content is unchanged across AGS 4.0.3 → 4.2** — verified by reading §8.1 (4.0.3/4.0.4 prose) and §4.1.1 (4.1/4.1.1/4.2 table) of *all five* PDFs, not by trusting a foreword. The text is *not* byte-identical: 4.1 reorganised prose→table, dropped Section-cross-ref parentheticals (Rules 7/10c/11), and changed Rule 15's example `ERES_RUNI`→`ELRG_RUNI` tracking the dictionary's ERES→ELRG replacement. Cross-edition rule variation is thus a *presentation + interpretation/implementation* axis, not a normative-text axis — see [[ags4-rules-frozen-dictionary-evolves]] and [[rule15-example-tracks-eres-elrg-removal]].

## Rule family
`naming` — implemented in `repo:rust-packages/laterite-ags4-validator/src/rules/naming.rs`. See [[rule-families]].

## Implementation (this repo)
> [!quote] Implemented in `repo:rust-packages/laterite-ags4-validator/src/rules/naming.rs::rule_19b + references.rs`

Structural AAAA_BBBB, field part 1–4 chars (stricter than prose — O-7). Dict-aware borrowed-heading parts (19b_2/19b_3) NOT re-reported (python triple-reports — O-26).

*Clean-room: rule logic derived from the spec; python-ags4 (LGPL) read only for behavioural parity, never copied (see the module header).*

## Traceability chain

```mermaid
flowchart LR
  R["Rule 19b"] --> I["naming.rs"] --> F["2 fixture(s)"] --> T["2 test(s)"] --> O["O-N (linked at Ingest)"]
```

- Fixtures: `rule19b_bad_prefix.ags`, `rule19b_borrowed_bad.ags`
- Regression: `rule19b_bad_prefix_flagged`, `rule19b_unknown_borrowed_prefix_flagged`

## Variations
> [!note] **Rule prose is frozen across editions.** The 4.2 Foreword states the AGS 4 Rules are unchanged and live in §4.1.1 (`spec:AGS4-4.2-2025.pdf §4.1.1`). So a rule's *spec text* does not vary 4.0.3→4.2 — cross-edition variation enters via the **Data Dictionary** (groups/types this rule operates over) and via **implementation/interpretation** (the Rust↔python axis, wired from Phase B/C as `[[O-NN]]`).

```mermaid
timeline
  title Rule text across editions (constant)
  4.0.3 : Rule 19b (same)
  4.0.4 : Rule 19b (same)
  4.1   : Rule 19b (same)
  4.1.1 : Rule 19b (same)
  4.2   : Rule 19b (same)
```

- Edition deltas (spec text): **none** — see [[ags4-rules-frozen-dictionary-evolves]].
- Divergence (Rust↔python): wired in Phase B/C — `[[O-NN]]` or _none_.

## Related
[[rule-families]] · [[traceability-chain]] · [[parity-model]] · [[ags-4.2]]
