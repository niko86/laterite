---
type: type
title: "TYPE <code>"
status: stub
tags: [type]
type_code: "<code>"
parametric: false         # true for nDP / nSF / nSCI
subtypes: []              # e.g. [2DP, 3DP] for the nDP family
canonical_type: ""        # string|integer|decimal|datetime|date|time|bool|enum
varies_between_editions: false
divergences: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  classifier: "repo:packages/ags5-models/src/ags5_models/_types.py::canonical_type"
  display_hint: "repo:packages/ags5-models/src/ags5_models/_types.py::display_hint"
  rule8_impl: "repo:rust-packages/laterite-ags4-validator/src/rules/typed_values.rs"
related: [rule-08-typed-values]
sources: [AGS4-4.2-2025.pdf]
---

# TYPE <code>

## Definition
> [!todo] Ingest: what the AGS type means. Cite `spec:`.

## Canonical mapping
<!-- TODO: → CanonicalType.<X>; cite repo: _types.py. -->

## Validation (Rule 8)
<!-- TODO: how typed_values.rs validates it. Link [[rule-08-typed-values]]. -->

## Variations
> [!note] Range/format edge cases + Rust↔python-ags4 divergence.

- Edge cases: <!-- empty UNIT, range bounds, parametric precision -->
- Divergence: <!-- > [!divergence] [[O-NN]] (e.g. DT → [[O-12]],[[O-31]],[[O-33]]) or "none" -->

## Related
<!-- [[rule-08-typed-values]] · [[heading-status-vocabulary]] · [[O-NN]] -->
