---
type: type
title: TYPE nSF
status: drafted
tags: [type]
type_code: nSF
parametric: true
subtypes: [2SF, 3SF, 4SF, 5SF]
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
# TYPE nSF

## Definition
> [!quote] AGS4 TYPE `nSF` — `spec:AGS4-4.2-2025.pdf §3.3`
> Value with required number of significant figures. e.g. 2SF = 1.2, 10.

## Canonical mapping
Maps to `CanonicalType.decimal` — cite `repo:packages/laterite/python/laterite/ags_types.py::canonical_type`. Parametric family; subtypes in frontmatter.

## Validation (Rule 8)
> [!quote] Rule 8 enforces n significant figures; the expected form is the ported MIT formatter fitted to python *output* (O-14, clean-room — never read AGS4.py's formatter).

See [[rule-08-typed-values]] · [[parity-model]].

## Variations
> [!todo] Ingest: range/format edge cases + Rust↔python-ags4 divergence (`> [!divergence]` [[O-12]]/[[O-31]]/[[O-33]] for DT, else "none").

## Security note
> [!variance] The `n` in `nSF` is read uncapped from the file's TYPE row and
> feeds a format width — a crafted `"9999999999SF"` OOM'd both validators
> (~10 GB on python-ags4); see [[O-49]] /
> [[numeric-type-count-uncapped-format-width]].

## Related
[[rule-08-typed-values]] · [[heading-status-vocabulary]] · [[O-49]] · [[numeric-type-count-uncapped-format-width]]
