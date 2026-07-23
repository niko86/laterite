---
type: type
title: TYPE XN
status: drafted
tags: [type]
type_code: XN
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
# TYPE XN

## Definition
> [!quote] AGS4 TYPE `XN` — `spec:AGS4-4.2-2025.pdf §3.3`
> Text / numeric. Parameters typically numeric but that may validly be text (e.g. plastic limit '34' or 'NP'; water depth '2.34' or 'dry'). Text abbreviations listed in ABBR (Rule 16).

## Canonical mapping
Maps to `CanonicalType.string` — cite `repo:packages/laterite/python/laterite/ags_types.py::canonical_type`.

## Validation (Rule 8)
> [!quote] Text-or-numeric — Rule 8 accepts either form (e.g. '34' or 'NP'); no numeric coercion forced.

See [[rule-08-typed-values]] · [[parity-model]].

## Variations
> [!todo] Ingest: range/format edge cases + Rust↔python-ags4 divergence (`> [!divergence]` [[O-12]]/[[O-31]]/[[O-33]] for DT, else "none").

## Related
[[rule-08-typed-values]] · [[heading-status-vocabulary]]
