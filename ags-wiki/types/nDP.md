---
type: type
title: TYPE nDP
status: drafted
tags: [type]
type_code: nDP
parametric: true
subtypes: [1DP, 2DP, 3DP, 4DP, 5DP, 6DP]
canonical_type: decimal
varies_between_editions: false
divergences: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  classifier: "repo:packages/laterite/python/laterite/ags_types.py::canonical_type"
  rule8_impl: "repo:rust-packages/laterite-ags4-validator/src/rules/typed_values.rs"
related: [rule-08-typed-values, heading-status-vocabulary, O-49, O-50, numeric-type-count-uncapped-format-width, 0dp-integer-conversion-precision-loss]
sources: [AGS4-4.2-2025.pdf]
---
# TYPE nDP

## Definition
> [!quote] AGS4 TYPE `nDP` — `spec:AGS4-4.2-2025.pdf §3.3`
> Value with required number of decimal places. e.g. 2DP = 2 decimal places = 2.34. (0DP = integer-valued instance.)

## Canonical mapping
Maps to `CanonicalType.decimal` — cite `repo:packages/laterite/python/laterite/ags_types.py::canonical_type`. Parametric family; subtypes in frontmatter.

## Validation (Rule 8)
> [!quote] Rule 8 enforces EXACTLY n decimal places (2DP→'2.34', not '2.3'/'2.340'); python uses substring 'DP' dispatch (O-13), Rust exact prefix+suffix — equivalent on valid codes, Rust more robust to malformed ones.

See [[rule-08-typed-values]] · [[parity-model]].

## Variations
> [!todo] Ingest: range/format edge cases + Rust↔python-ags4 divergence (`> [!divergence]` [[O-12]]/[[O-31]]/[[O-33]] for DT, else "none").

## Security note
> [!variance] The `n` in `nDP` is read uncapped from the file's TYPE row and
> feeds a format width — a crafted count OOM'd both validators; see [[O-49]] /
> [[numeric-type-count-uncapped-format-width]].

## `0DP` (n=0) conversion note
> [!variance] `0DP` is the integer-valued instance of this family (`n=0`).
> Rule 8's grammar check already flags a cell that can't be a clean in-range
> `i64` on both validators — but the separate string→number CONVERSION of
> that cell diverged: laterite's pre-#611 `f as i64` silently saturated an
> out-of-range value to a fabricated `i64::MAX`, where python-ags4's
> `int(float(s))` keeps full precision. #611's `parse_ags_integer` range-
> guards the conversion to Null instead. See [[O-50]] /
> [[0dp-integer-conversion-precision-loss]].

## Related
[[rule-08-typed-values]] · [[heading-status-vocabulary]] · [[O-49]] · [[O-50]] · [[numeric-type-count-uncapped-format-width]] · [[0dp-integer-conversion-precision-loss]]
