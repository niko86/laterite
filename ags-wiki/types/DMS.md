---
type: type
title: TYPE DMS
status: drafted
tags: [type]
type_code: DMS
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
# TYPE DMS

## Definition
> [!quote] AGS4 TYPE `DMS` — `spec:AGS4-4.2-2025.pdf §3.3`
> Degrees:Minutes:Seconds. e.g. 51:28:52.498.

## Canonical mapping
Maps to `CanonicalType.string` — cite `repo:packages/laterite/python/laterite/ags_types.py::canonical_type`.

## Validation (Rule 8)
> [!quote] Degrees:Minutes:Seconds — structural shape check (is_dms); minutes/seconds range-validated.

See [[rule-08-typed-values]] · [[parity-model]].

## Variations
> [!todo] Ingest: range/format edge cases + Rust↔python-ags4 divergence (`> [!divergence]` [[O-12]]/[[O-31]]/[[O-33]] for DT, else "none").

## Related
[[rule-08-typed-values]] · [[heading-status-vocabulary]]
