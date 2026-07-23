---
type: type
title: TYPE T
status: drafted
tags: [type]
type_code: T
parametric: false
subtypes: []
canonical_type: time
varies_between_editions: false
divergences: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  classifier: "repo:packages/laterite/python/laterite/ags_types.py::canonical_type"
  rule8_impl: "repo:rust-packages/laterite-ags4-validator/src/rules/typed_values.rs"
related: [rule-08-typed-values, heading-status-vocabulary]
sources: [AGS4-4.2-2025.pdf]
---
# TYPE T

## Definition
> [!quote] AGS4 TYPE `T` — `spec:AGS4-4.2-2025.pdf §3.3`
> Elapsed Time. e.g. hh:mm:ss.

## Canonical mapping
Maps to `CanonicalType.time` — cite `repo:packages/laterite/python/laterite/ags_types.py::canonical_type`.

## Validation (Rule 8)
> [!quote] Elapsed time hh:mm:ss — structural per-char match; time-only, so the pandas/chrono date-range bound (O-33) does NOT apply.

See [[rule-08-typed-values]] · [[parity-model]].

## Variations
> [!todo] Ingest: range/format edge cases + Rust↔python-ags4 divergence (`> [!divergence]` [[O-12]]/[[O-31]]/[[O-33]] for DT, else "none").

## Related
[[rule-08-typed-values]] · [[heading-status-vocabulary]]
