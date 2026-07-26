---
type: rule
title: Rule 2 — group has data
status: drafted
tags: [rule]
rule_number: 2
rule_sub: ""
rule_family: structure
varies_between_editions: false
divergences: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  impl: "repo:rust-packages/laterite-ags4-validator/src/rules/structure.rs"
  fixtures: "repo:rust-packages/laterite-ags4-validator/tests/fixtures/rule2_no_data_rows.ags"
  regression: "repo:rust-packages/laterite-ags4-validator/tests/regression.rs::rule2_group_without_data_rows_flagged"
  spec: "spec:AGS4-4.2-2025.pdf §4.1.1 Rule 2"
related: [rule-families, traceability-chain, parity-model, O-41]
sources: [spec-4.2]
---
# Rule 2 — group has data

## Statement
> [!quote] AGS4 Rule 02 — `spec:AGS4-4.2-2025.pdf §4.1.1 Rule 02`
> Each data file shall contain one or more data GROUPs. Each data GROUP shall comprise a number of GROUP HEADER rows and must have one or more DATA rows.

Rule **normative content is unchanged across AGS 4.0.3 → 4.2** — verified by reading §8.1 (4.0.3/4.0.4 prose) and §4.1.1 (4.1/4.1.1/4.2 table) of *all five* PDFs, not by trusting a foreword. The text is *not* byte-identical: 4.1 reorganised prose→table, dropped Section-cross-ref parentheticals (Rules 7/10c/11), and changed Rule 15's example `ERES_RUNI`→`ELRG_RUNI` tracking the dictionary's ERES→ELRG replacement. Cross-edition rule variation is thus a *presentation + interpretation/implementation* axis, not a normative-text axis — see [[ags4-rules-frozen-dictionary-evolves]] and [[rule15-example-tracks-eres-elrg-removal]].

## Rule family
`structure` — implemented in `repo:rust-packages/laterite-ags4-validator/src/rules/structure.rs`. See [[rule-families]]. Sub-rules: [[rule-02a-crlf-line-terminator]] (line terminator) · [[rule-02b-header-rows-order]] (GROUP HEADER row order).

## Implementation (this repo)
> [!quote] Implemented in `repo:rust-packages/laterite-ags4-validator/src/rules/structure.rs`

GROUP must have ≥1 DATA row; zero-row PROJ/TRAN double-reported with Rule 13/14 for python parity (O-16).

*Clean-room: rule logic derived from the spec; python-ags4 (LGPL) read only for behavioural parity, never copied (see the module header).*

## Traceability chain

```mermaid
flowchart LR
  R["Rule 2"] --> I["structure.rs"] --> F["1 fixture(s)"] --> T["1 test(s)"] --> O["O-N (linked at Ingest)"]
```

- Fixtures: `rule2_no_data_rows.ags`
- Regression: `rule2_group_without_data_rows_flagged`

## Variations
> [!note] **Rule prose is frozen across editions.** The 4.2 Foreword states the AGS 4 Rules are unchanged and live in §4.1.1 (`spec:AGS4-4.2-2025.pdf §4.1.1`). So a rule's *spec text* does not vary 4.0.3→4.2 — cross-edition variation enters via the **Data Dictionary** (groups/types this rule operates over) and via **implementation/interpretation** (the Rust↔python axis, wired from Phase B/C as `[[O-NN]]`).

```mermaid
timeline
  title Rule text across editions (constant)
  4.0.3 : Rule 02 (same)
  4.0.4 : Rule 02 (same)
  4.1   : Rule 02 (same)
  4.1.1 : Rule 02 (same)
  4.2   : Rule 02 (same)
```

- Edition deltas (spec text): **none** — see [[ags4-rules-frozen-dictionary-evolves]].
- Divergence (Rust↔python): [[O-41]] — a HEADING/UNIT/TYPE/DATA row before any GROUP is reported as a Rule 2 finding by laterite; python-ags4's parser hard-fails on the same row instead (no report emitted).

## Related
[[rule-families]] · [[traceability-chain]] · [[parity-model]] · [[ags-4.2]]
