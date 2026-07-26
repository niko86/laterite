---
type: rule
title: Rule 15 — unit group
status: drafted
tags: [rule]
rule_number: 15
rule_sub: ""
rule_family: groups
varies_between_editions: false
divergences: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  impl: "repo:rust-packages/laterite-ags4-validator/src/rules/groups.rs"
  fixtures: "repo:rust-packages/laterite-ags4-validator/tests/fixtures/rule15_unit_undef.ags"
  regression: "repo:rust-packages/laterite-ags4-validator/tests/regression.rs::rule15_undefined_unit_flagged"
  spec: "spec:AGS4-4.2-2025.pdf §4.1.1 Rule 15"
related: [rule-families, traceability-chain, parity-model, rule15-example-tracks-eres-elrg-removal]
sources: [spec-4.2]
---
# Rule 15 — unit group

## Statement
> [!quote] AGS4 Rule 15 — `spec:AGS4-4.2-2025.pdf §4.1.1 Rule 15`
> Each data file shall contain the UNIT GROUP to list all units used within the data file. Every unit of measurement entered in the UNIT row of a GROUP or data entered in a FIELD where the field TYPE is defined as "PU" (for example ELRG_RUNI, GCHM_UNIT or MOND_UNIT FIELDs) shall be listed and defined in the UNIT GROUP.

Rule **normative content is unchanged across AGS 4.0.3 → 4.2** — verified by reading §8.1 (4.0.3/4.0.4 prose) and §4.1.1 (4.1/4.1.1/4.2 table) of *all five* PDFs, not by trusting a foreword. The text is *not* byte-identical: 4.1 reorganised prose→table, dropped Section-cross-ref parentheticals (Rules 7/10c/11), and changed Rule 15's example `ERES_RUNI`→`ELRG_RUNI` tracking the dictionary's ERES→ELRG replacement. Cross-edition rule variation is thus a *presentation + interpretation/implementation* axis, not a normative-text axis — see [[ags4-rules-frozen-dictionary-evolves]] and [[rule15-example-tracks-eres-elrg-removal]].

## Rule family
`groups` — implemented in `repo:rust-packages/laterite-ags4-validator/src/rules/groups.rs`. See [[rule-families]].

## Implementation (this repo)
> [!quote] Implemented in `repo:rust-packages/laterite-ags4-validator/src/rules/groups.rs`

[[UNIT]] group lists all used units + PU-typed field values. Spec example was ERES_RUNI→ELRG_RUNI@4.1 — see [[rule15-example-tracks-eres-elrg-removal]].

*Clean-room: rule logic derived from the spec; python-ags4 (LGPL) read only for behavioural parity, never copied (see the module header).*

## Traceability chain

```mermaid
flowchart LR
  R["Rule 15"] --> I["groups.rs"] --> F["1 fixture(s)"] --> T["1 test(s)"] --> O["O-N (linked at Ingest)"]
```

- Fixtures: `rule15_unit_undef.ags`
- Regression: `rule15_undefined_unit_flagged`

## Variations
> [!note] **Rule prose is frozen across editions.** The 4.2 Foreword states the AGS 4 Rules are unchanged and live in §4.1.1 (`spec:AGS4-4.2-2025.pdf §4.1.1`). So a rule's *spec text* does not vary 4.0.3→4.2 — cross-edition variation enters via the **Data Dictionary** (groups/types this rule operates over) and via **implementation/interpretation** (the Rust↔python axis, wired from Phase B/C as `[[O-NN]]`).

```mermaid
timeline
  title Rule text across editions (constant)
  4.0.3 : Rule 15 (same)
  4.0.4 : Rule 15 (same)
  4.1   : Rule 15 (same)
  4.1.1 : Rule 15 (same)
  4.2   : Rule 15 (same)
```

- Edition deltas (spec text): **none** — see [[ags4-rules-frozen-dictionary-evolves]].
- Known/!suspected gap: [[rule15-example-tracks-eres-elrg-removal]] (Phase A spec-vs-impl candidate).
- Divergence (Rust↔python): wired in Phase B/C — `[[O-NN]]` or _none_.

## Related
[[rule-families]] · [[traceability-chain]] · [[parity-model]] · [[ags-4.2]]
