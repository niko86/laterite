---
type: rule
title: Rule 11a — tran dlim
status: drafted
tags: [rule]
rule_number: 11
rule_sub: a
rule_family: relational
varies_between_editions: false
divergences: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  impl: "repo:rust-packages/laterite-ags4-validator/src/rules/relational.rs"
  fixtures: ""
  regression: ""
  spec: "spec:AGS4-4.2-2025.pdf §4.1.1 Rule 11a"
related: [rule-families, traceability-chain, parity-model]
sources: [spec-4.2]
---
# Rule 11a — tran dlim

## Statement
> [!quote] AGS4 Rule 11a — `spec:AGS4-4.2-2025.pdf §4.1.1 Rule 11a`
> Each GROUP/KEY FIELD shall be separated by a delimiter character. This single delimiter character shall be defined in TRAN_DLIM. The default being "|" (ASCII character 124).

Rule **normative content is unchanged across AGS 4.0.3 → 4.2** — verified by reading §8.1 (4.0.3/4.0.4 prose) and §4.1.1 (4.1/4.1.1/4.2 table) of *all five* PDFs, not by trusting a foreword. The text is *not* byte-identical: 4.1 reorganised prose→table, dropped Section-cross-ref parentheticals (Rules 7/10c/11), and changed Rule 15's example `ERES_RUNI`→`ELRG_RUNI` tracking the dictionary's ERES→ELRG replacement. Cross-edition rule variation is thus a *presentation + interpretation/implementation* axis, not a normative-text axis — see [[ags4-rules-frozen-dictionary-evolves]] and [[rule15-example-tracks-eres-elrg-removal]].

## Rule family
`relational` — implemented in `repo:rust-packages/laterite-ags4-validator/src/rules/relational.rs`. See [[rule-families]]. Sibling sub-rules: [[rule-11b-tran-rcon]] (concatenation character) · [[rule-11c-record-links]] (RL record-link format).

## Implementation (this repo)
> [!quote] Implemented in `repo:rust-packages/laterite-ags4-validator/src/rules/relational.rs`

GROUP/KEY separated by TRAN_DLIM (default '|').

*Clean-room: rule logic derived from the spec; python-ags4 (LGPL) read only for behavioural parity, never copied (see the module header).*

## Traceability chain

```mermaid
flowchart LR
  R["Rule 11a"] --> I["relational.rs"] --> F["0 fixture(s)"] --> T["0 test(s)"] --> O["O-N (linked at Ingest)"]
```

- Fixtures: _none yet — Lint gap_
- Regression: _none yet — Lint gap_

## Variations
> [!note] **Rule prose is frozen across editions.** The 4.2 Foreword states the AGS 4 Rules are unchanged and live in §4.1.1 (`spec:AGS4-4.2-2025.pdf §4.1.1`). So a rule's *spec text* does not vary 4.0.3→4.2 — cross-edition variation enters via the **Data Dictionary** (groups/types this rule operates over) and via **implementation/interpretation** (the Rust↔python axis, wired from Phase B/C as `[[O-NN]]`).

```mermaid
timeline
  title Rule text across editions (constant)
  4.0.3 : Rule 11a (same)
  4.0.4 : Rule 11a (same)
  4.1   : Rule 11a (same)
  4.1.1 : Rule 11a (same)
  4.2   : Rule 11a (same)
```

- Edition deltas (spec text): **none** — see [[ags4-rules-frozen-dictionary-evolves]].
- Divergence (Rust↔python): wired in Phase B/C — `[[O-NN]]` or _none_.

## Related
[[rule-families]] · [[traceability-chain]] · [[parity-model]] · [[ags-4.2]]
