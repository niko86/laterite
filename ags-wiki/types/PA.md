---
type: type
title: TYPE PA
status: drafted
tags: [type]
type_code: PA
parametric: false
subtypes: []
canonical_type: enum
varies_between_editions: false
divergences: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  classifier: "repo:packages/laterite/python/laterite/ags_types.py::canonical_type"
  rule8_impl: "repo:rust-packages/laterite-ags4-validator/src/rules/typed_values.rs"
related: [rule-08-typed-values, heading-status-vocabulary]
sources: [AGS4-4.2-2025.pdf]
---
# TYPE PA

## Definition
> [!quote] AGS4 TYPE `PA` — `spec:AGS4-4.2-2025.pdf §3.3`
> Text listed in the ABBR Group (Rule 16). Standard abbreviations are published on the AGS website and shall not be redefined; project abbreviations must not impersonate standard ones. Multiple joined via TRAN_RCON (default "+"). NOT case sensitive.

## Canonical mapping
Maps to `CanonicalType.enum` — cite `repo:packages/laterite/python/laterite/ags_types.py::canonical_type`.

## Validation (Rule 8)
> [!quote] Rule 8 only checks the value is text; abbreviation *membership* is Rule 16 (must be defined in ABBR). PA is NOT case-sensitive (§3.3) — a latent Rust↔python case-fold divergence ([[pa-not-case-sensitive-pt-pu-are]]).

See [[rule-08-typed-values]] · [[parity-model]].

## Variations
> [!todo] Ingest: range/format edge cases + Rust↔python-ags4 divergence (`> [!divergence]` [[O-12]]/[[O-31]]/[[O-33]] for DT, else "none").

## Related
[[rule-08-typed-values]] · [[heading-status-vocabulary]]
