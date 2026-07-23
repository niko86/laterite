---
type: type
title: TYPE nSCI
status: drafted
tags: [type]
type_code: nSCI
parametric: true
subtypes: [1SCI, 2SCI, 3SCI]
canonical_type: decimal
varies_between_editions: false
divergences: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  classifier: "repo:packages/laterite/python/laterite/ags_types.py::canonical_type"
  rule8_impl: "repo:rust-packages/laterite-ags4-validator/src/rules/typed_values.rs"
related: [rule-08-typed-values, heading-status-vocabulary, O-49, numeric-type-count-uncapped-format-width]
sources: [AGS4-4.2-2025.pdf]
---
# TYPE nSCI

## Definition
> [!quote] AGS4 TYPE `nSCI` — `spec:AGS4-4.2-2025.pdf §3.3`
> Scientific notation with required number of decimal places. e.g. 73100 as 2SCI = 7.31E4; as 1SCI = 7.3E4.

## Canonical mapping
Maps to `CanonicalType.decimal` — cite `repo:packages/laterite/python/laterite/ags_types.py::canonical_type`. Parametric family; subtypes in frontmatter.

## Validation (Rule 8)
> [!quote] Rule 8 enforces scientific notation with n decimal places (2SCI→'7.31E4'); same exact-classification robustness note as nDP (O-13).

See [[rule-08-typed-values]] · [[parity-model]].

## Variations
> [!todo] Ingest: range/format edge cases + Rust↔python-ags4 divergence (`> [!divergence]` [[O-12]]/[[O-31]]/[[O-33]] for DT, else "none").

## Security note
> [!variance] The `n` in `nSCI` is read uncapped from the file's TYPE row and
> feeds a format width — a crafted count OOM'd both validators; see [[O-49]] /
> [[numeric-type-count-uncapped-format-width]].

## Related
[[rule-08-typed-values]] · [[heading-status-vocabulary]] · [[O-49]] · [[numeric-type-count-uncapped-format-width]]
