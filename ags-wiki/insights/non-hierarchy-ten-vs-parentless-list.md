---
type: insight
title: "Spec lists 10 non-hierarchy groups; O-21 hardcodes 11 (incl. LOCA)"
status: confirmed
tags: [insight]
gap_kind: spec-vs-rust
severity: med
editions_affected: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
rules: [rule-10c-parent-child]
proposes_observation: false
feeds_strategy: []
discovered_phase: A
related: [O-21]
sources: [spec-4.2]
---
# Spec lists 10 non-hierarchy groups; O-21 hardcodes 11 (incl. LOCA)

## Claim
> [!divergence] §3.1 enumerates exactly TEN non-hierarchy groups: [[PROJ]], [[TRAN]], [[ABBR]], [[DICT]], [[TYPE]], [[FILE]], [[UNIT]], [[LBSG]], [[PREM]], [[STND]]. [[O-21]] records the validator hardcoding a parentless list that ALSO includes [[LOCA]] — but §3.1 places LOCA *in* the hierarchy (immediately below PROJ). LOCA is parentless-in-practice (its parent PROJ is the root) yet the spec doesn't call it a non-hierarchy group. Phase B must reconcile the validator's Rule 10c parentless set against this exact §3.1 list. hypothesis.

## Evidence
- Spec: `spec:AGS4-4.2-2025.pdf §3.x` (see [[ags4-rules-frozen-dictionary-evolves]]).
- Prior art: [[O-21]].

## Related
[[O-21]] · [[ags4-rules-frozen-dictionary-evolves]]

## Phase B verification
> [!note] **confirmed** (code-read). CODE-VERIFIED: relational.rs:66-68 PARENTLESS = 11 groups INCLUDING LOCA (PROJ,TRAN,ABBR,DICT,UNIT,TYPE,LOCA,FILE,LBSG,PREM,STND); spec §3.1 lists 10 (no LOCA). NOT a defect — relational.rs:62-65 explains LOCA is parentless-for-Rule-10c (LOCA rows carry no PROJ key, so the repeated-KEY check can't run) though hierarchically under PROJ. Reclassified: spec UNDER-SPECIFICATION — §3.1 conflates 'hierarchy position' with '10c-checkable linkage'. AGS5 should separate the two. python-ags4 hardcodes the same 11 (O-21).
