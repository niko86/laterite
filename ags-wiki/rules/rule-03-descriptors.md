---
type: rule
title: Rule 3 — descriptors
status: drafted
tags: [rule]
rule_number: 3
rule_sub: ""
rule_family: line
varies_between_editions: false
divergences: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  impl: "repo:rust-packages/laterite-ags4-validator/src/rules/line_format.rs"
  fixtures: "repo:rust-packages/laterite-ags4-validator/tests/fixtures/rule3_bad_descriptor.ags"
  regression: "repo:rust-packages/laterite-ags4-validator/tests/regression.rs::rule3_bad_descriptor_flagged_at_its_line"
  spec: "spec:AGS4-4.2-2025.pdf §4.1.1 Rule 3"
related: [rule-families, traceability-chain, parity-model]
sources: [spec-4.2]
---
# Rule 3 — descriptors

## Statement
> [!quote] AGS4 Rule 03 — `spec:AGS4-4.2-2025.pdf §4.1.1 Rule 03`
> Each row in the data file must start with a DATA DESCRIPTOR that defines the contents of that row. The following Data Descriptors are used as described below:
> - Each GROUP row shall be preceded by the "GROUP" Data Descriptor.
> - Each HEADING row shall be preceded by the "HEADING" Data Descriptor.
> - Each UNIT row shall be preceded by the "UNIT" Data Descriptor.
> - Each TYPE row shall be preceded by the "TYPE" Data Descriptor.
> - Each DATA row shall be preceded by the "DATA" Data Descriptor.

Rule **normative content is unchanged across AGS 4.0.3 → 4.2** — verified by reading §8.1 (4.0.3/4.0.4 prose) and §4.1.1 (4.1/4.1.1/4.2 table) of *all five* PDFs, not by trusting a foreword. The text is *not* byte-identical: 4.1 reorganised prose→table, dropped Section-cross-ref parentheticals (Rules 7/10c/11), and changed Rule 15's example `ERES_RUNI`→`ELRG_RUNI` tracking the dictionary's ERES→ELRG replacement. Cross-edition rule variation is thus a *presentation + interpretation/implementation* axis, not a normative-text axis — see [[ags4-rules-frozen-dictionary-evolves]] and [[rule15-example-tracks-eres-elrg-removal]].

## Rule family
`line` — implemented in `repo:rust-packages/laterite-ags4-validator/src/rules/line_format.rs`. See [[rule-families]].

## Implementation (this repo)
> [!quote] Implemented in `repo:rust-packages/laterite-ags4-validator/src/rules/line_format.rs::rule_3`

Row must start with one of the 5 descriptors GROUP/HEADING/UNIT/TYPE/DATA.

*Clean-room: rule logic derived from the spec; python-ags4 (LGPL) read only for behavioural parity, never copied (see the module header).*

## Traceability chain

```mermaid
flowchart LR
  R["Rule 3"] --> I["line_format.rs"] --> F["1 fixture(s)"] --> T["1 test(s)"] --> O["O-N (linked at Ingest)"]
```

- Fixtures: `rule3_bad_descriptor.ags`
- Regression: `rule3_bad_descriptor_flagged_at_its_line`

## Variations
> [!note] **Rule prose is frozen across editions.** The 4.2 Foreword states the AGS 4 Rules are unchanged and live in §4.1.1 (`spec:AGS4-4.2-2025.pdf §4.1.1`). So a rule's *spec text* does not vary 4.0.3→4.2 — cross-edition variation enters via the **Data Dictionary** (groups/types this rule operates over) and via **implementation/interpretation** (the Rust↔python axis, wired from Phase B/C as `[[O-NN]]`).

```mermaid
timeline
  title Rule text across editions (constant)
  4.0.3 : Rule 03 (same)
  4.0.4 : Rule 03 (same)
  4.1   : Rule 03 (same)
  4.1.1 : Rule 03 (same)
  4.2   : Rule 03 (same)
```

- Edition deltas (spec text): **none** — see [[ags4-rules-frozen-dictionary-evolves]].
- Divergence (Rust↔python): wired in Phase B/C — `[[O-NN]]` or _none_.

## Related
[[rule-families]] · [[traceability-chain]] · [[parity-model]] · [[ags-4.2]]
