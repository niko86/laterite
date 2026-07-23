---
type: type
title: TYPE ID
status: drafted
tags: [type]
type_code: ID
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
# TYPE ID

## Definition
> [!quote] AGS4 TYPE `ID` — `spec:AGS4-4.2-2025.pdf §3.3`
> Unique Identifier. Unique within the parent group; may repeat in child groups (e.g. SAMP parent vs ELRG child; LOCA parent vs SAMP child).

## Canonical mapping
Maps to `CanonicalType.string` — cite `repo:packages/laterite/python/laterite/ags_types.py::canonical_type`.

## Validation (Rule 8)
> [!quote] Not a Rule 8 concern per se — ID is text. python-ags4 additionally folds group-ID *uniqueness* into Rule 8 (O-11); the spec-correct home is Rule 10a. Rust mirrors the attribution for parity, re-detects under 10a.

See [[rule-08-typed-values]] · [[parity-model]].

## Variations
> [!todo] Ingest: range/format edge cases + Rust↔python-ags4 divergence (`> [!divergence]` [[O-12]]/[[O-31]]/[[O-33]] for DT, else "none").

## Related
[[rule-08-typed-values]] · [[heading-status-vocabulary]]
