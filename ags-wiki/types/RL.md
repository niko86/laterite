---
type: type
title: TYPE RL
status: drafted
tags: [type]
type_code: RL
parametric: false
subtypes: []
canonical_type: string
varies_between_editions: false
divergences: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  classifier: "repo:packages/laterite/python/laterite/ags_types.py::canonical_type"
  rule8_impl: "repo:rust-packages/laterite-ags4-validator/src/rules/typed_values.rs"
related: [rule-08-typed-values, heading-status-vocabulary]
sources: [AGS4-4.2-2025.pdf]
---
# TYPE RL

## Definition
> [!quote] AGS4 TYPE `RL` — `spec:AGS4-4.2-2025.pdf §3.3`
> Record Link (Rule 11). Text in a specified format referring to one or more records in other Groups by their KEY headings.

## Canonical mapping
Maps to `CanonicalType.string` — cite `repo:packages/laterite/python/laterite/ags_types.py::canonical_type`.

## Validation (Rule 8)
> [!quote] Rule 8 does not validate the link target — that is Rule 11/11c (cross-reference resolution, positional per O-24). Rule 8 only sees it as text.

See [[rule-08-typed-values]] · [[parity-model]].

## Variations
> [!todo] Ingest: range/format edge cases + Rust↔python-ags4 divergence (`> [!divergence]` [[O-12]]/[[O-31]]/[[O-33]] for DT, else "none").

## Related
[[rule-08-typed-values]] · [[heading-status-vocabulary]]
