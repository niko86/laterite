---
type: rule
title: Rule 7 — heading order
status: drafted
tags: [rule]
rule_number: 7
rule_sub: ""
rule_family: dictionary
varies_between_editions: false
divergences: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  impl: "repo:rust-packages/laterite-ags4-validator/src/rules/dictionary.rs"
  fixtures: "repo:rust-packages/laterite-ags4-validator/tests/fixtures/rule7_out_of_order.ags"
  regression: "repo:rust-packages/laterite-ags4-validator/tests/regression.rs::rule7_headings_out_of_dict_order_flagged"
  spec: "spec:AGS4-4.2-2025.pdf §4.1.1 Rule 7"
related: [rule-families, traceability-chain, parity-model]
sources: [spec-4.2]
---
# Rule 7 — heading order

## Statement
> [!quote] AGS4 Rule 07 — `spec:AGS4-4.2-2025.pdf §4.1.1 Rule 07`
> The order of data FIELDs in each line within a GROUP is defined at the start of each GROUP in the HEADING row. HEADINGs shall be in the order described in the AGS FORMAT DATA DICTIONARY.

Rule **normative content is unchanged across AGS 4.0.3 → 4.2** — verified by reading §8.1 (4.0.3/4.0.4 prose) and §4.1.1 (4.1/4.1.1/4.2 table) of *all five* PDFs, not by trusting a foreword. The text is *not* byte-identical: 4.1 reorganised prose→table, dropped Section-cross-ref parentheticals (Rules 7/10c/11), and changed Rule 15's example `ERES_RUNI`→`ELRG_RUNI` tracking the dictionary's ERES→ELRG replacement. Cross-edition rule variation is thus a *presentation + interpretation/implementation* axis, not a normative-text axis — see [[ags4-rules-frozen-dictionary-evolves]] and [[rule15-example-tracks-eres-elrg-removal]].

## Rule family
`dictionary` — implemented in `repo:rust-packages/laterite-ags4-validator/src/rules/dictionary.rs`. See [[rule-families]].

## Implementation (this repo)
> [!quote] Implemented in `repo:rust-packages/laterite-ags4-validator/src/rules/dictionary.rs`

HEADING order must match the effective dictionary; 'no duplicate headings' is inferred (not in prose — O-9); python's rule_7_2 can IndexError on dup headings (O-8).

*Clean-room: rule logic derived from the spec; python-ags4 (LGPL) read only for behavioural parity, never copied (see the module header).*

## Traceability chain

```mermaid
flowchart LR
  R["Rule 7"] --> I["dictionary.rs"] --> F["1 fixture(s)"] --> T["1 test(s)"] --> O["O-N (linked at Ingest)"]
```

- Fixtures: `rule7_out_of_order.ags`
- Regression: `rule7_headings_out_of_dict_order_flagged`

## Variations
> [!note] **Rule prose is frozen across editions.** The 4.2 Foreword states the AGS 4 Rules are unchanged and live in §4.1.1 (`spec:AGS4-4.2-2025.pdf §4.1.1`). So a rule's *spec text* does not vary 4.0.3→4.2 — cross-edition variation enters via the **Data Dictionary** (groups/types this rule operates over) and via **implementation/interpretation** (the Rust↔python axis, wired from Phase B/C as `[[O-NN]]`).

```mermaid
timeline
  title Rule text across editions (constant)
  4.0.3 : Rule 07 (same)
  4.0.4 : Rule 07 (same)
  4.1   : Rule 07 (same)
  4.1.1 : Rule 07 (same)
  4.2   : Rule 07 (same)
```

- Edition deltas (spec text): **none** — see [[ags4-rules-frozen-dictionary-evolves]].
- Divergence (Rust↔python): wired in Phase B/C — `[[O-NN]]` or _none_.

## Related
[[rule-families]] · [[traceability-chain]] · [[parity-model]] · [[ags-4.2]]
