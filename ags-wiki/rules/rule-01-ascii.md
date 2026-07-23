---
type: rule
title: Rule 1 — ascii
status: drafted
tags: [rule]
rule_number: 1
rule_sub: ""
rule_family: line
varies_between_editions: false
divergences: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  impl: "repo:rust-packages/laterite-ags4-validator/src/rules/line_format.rs"
  fixtures: "repo:rust-packages/laterite-ags4-validator/tests/fixtures/rule1_non_ascii.ags"
  regression: "repo:rust-packages/laterite-ags4-validator/tests/regression.rs::rule1_non_ascii_flagged_at_its_line"
  spec: "spec:AGS4-4.2-2025.pdf §4.1.1 Rule 1"
related: [rule-families, traceability-chain, parity-model]
sources: [spec-4.2]
---
# Rule 1 — ascii

## Statement
> [!quote] AGS4 Rule 01 — `spec:AGS4-4.2-2025.pdf §4.1.1 Rule 01`
> The data file shall be entirely composed of ASCII characters.

Rule **normative content is unchanged across AGS 4.0.3 → 4.2** — verified by reading §8.1 (4.0.3/4.0.4 prose) and §4.1.1 (4.1/4.1.1/4.2 table) of *all five* PDFs, not by trusting a foreword. The text is *not* byte-identical: 4.1 reorganised prose→table, dropped Section-cross-ref parentheticals (Rules 7/10c/11), and changed Rule 15's example `ERES_RUNI`→`ELRG_RUNI` tracking the dictionary's ERES→ELRG replacement. Cross-edition rule variation is thus a *presentation + interpretation/implementation* axis, not a normative-text axis — see [[ags4-rules-frozen-dictionary-evolves]] and [[rule15-example-tracks-eres-elrg-removal]].

## Rule family
`line` — implemented in `repo:rust-packages/laterite-ags4-validator/src/rules/line_format.rs`. See [[rule-families]].

## Implementation (this repo)
> [!quote] Implemented in `repo:rust-packages/laterite-ags4-validator/src/rules/line_format.rs::rule_1`

Scans each line's max codepoint: ≤127 clean; >255 RULE_1 error; 128–255 RULE_1_FYI (suppressed unless --show-fyi). Invalid UTF-8 decoded lossily → U+FFFD>255 → Rule 1 (O-32, mirrors python errors='replace'). Spec says 'entirely ASCII' (0–127) — both validators tolerate 128–255 (O-1).

*Clean-room: rule logic derived from the spec; python-ags4 (LGPL) read only for behavioural parity, never copied (see the module header).*

## Traceability chain

```mermaid
flowchart LR
  R["Rule 1"] --> I["line_format.rs"] --> F["1 fixture(s)"] --> T["1 test(s)"] --> O["O-N (linked at Ingest)"]
```

- Fixtures: `rule1_non_ascii.ags`
- Regression: `rule1_non_ascii_flagged_at_its_line`

## Variations
> [!note] **Rule prose is frozen across editions.** The 4.2 Foreword states the AGS 4 Rules are unchanged and live in §4.1.1 (`spec:AGS4-4.2-2025.pdf §4.1.1`). So a rule's *spec text* does not vary 4.0.3→4.2 — cross-edition variation enters via the **Data Dictionary** (groups/types this rule operates over) and via **implementation/interpretation** (the Rust↔python axis, wired from Phase B/C as `[[O-NN]]`).

```mermaid
timeline
  title Rule text across editions (constant)
  4.0.3 : Rule 01 (same)
  4.0.4 : Rule 01 (same)
  4.1   : Rule 01 (same)
  4.1.1 : Rule 01 (same)
  4.2   : Rule 01 (same)
```

- Edition deltas (spec text): **none** — see [[ags4-rules-frozen-dictionary-evolves]].
- Known/!suspected gap: [[rule1-ascii-strict-vs-extended]] (Phase A spec-vs-impl candidate).
- Divergence (Rust↔python): none — dogfood-confirmed by [[strat-cp1252-rule1]] (invalid-UTF-8/cp1252 input: both Rust and python-ags4 emit Rule 1).

## Related
[[rule-families]] · [[traceability-chain]] · [[parity-model]] · [[ags-4.2]]
