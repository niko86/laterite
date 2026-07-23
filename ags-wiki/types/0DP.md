---
type: type
title: TYPE 0DP
status: drafted
tags: [type]
type_code: 0DP
parametric: false
subtypes: []
canonical_type: integer
varies_between_editions: false
divergences: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  classifier: "repo:packages/laterite/python/laterite/ags_types.py::canonical_type"
  rule8_impl: "repo:rust-packages/laterite-ags4-validator/src/rules/typed_values.rs"
related: [rule-08-typed-values, heading-status-vocabulary]
sources: [AGS4-4.2-2025.pdf]
---
# TYPE 0DP

## Definition
> [!quote] AGS4 TYPE `0DP` — `spec:AGS4-4.2-2025.pdf §3.3`
> The n=0 instance of the nDP family — a value with zero decimal places (integer presentation). Spec §3.3 defines this parametrically under nDP, not as a separate type token.

## Canonical mapping
Maps to `CanonicalType.integer` — cite `repo:packages/laterite/python/laterite/ags_types.py::canonical_type`.

## Validation (Rule 8)
> [!quote] The n=0 instance of nDP — integer presentation, zero decimal places. Same Rule 8 engine as nDP.

See [[rule-08-typed-values]] · [[parity-model]].

## Variations
> [!todo] Ingest: range/format edge cases + Rust↔python-ags4 divergence (`> [!divergence]` [[O-12]]/[[O-31]]/[[O-33]] for DT, else "none").

## Related
[[rule-08-typed-values]] · [[heading-status-vocabulary]]
