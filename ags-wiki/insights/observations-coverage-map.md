---
type: insight
title: "OBSERVATIONS coverage map — the AGS-DFWG candidate register"
status: confirmed
tags: [insight, register]
gap_kind: spec-ambiguity
severity: med
editions_affected: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
rules: []
proposes_observation: false
feeds_strategy: []
feeds_ags5_req: []
discovered_phase: C
related: [upstream-reporting, parity-model, ags4-rules-frozen-dictionary-evolves]
sources: []
---
# OBSERVATIONS coverage map

## Claim
> [!note] Synthesis of all 39 O-N entries by tag and upstream-reportability — the spine of the Phase D 4.2-improvement / AGS-DFWG list. Confirmed (full read of OBSERVATIONS.md; O-35/O-36 added Phase H, O-37 added Phase I for compat parity-arc, O-38 added Phase I from DT-format dogfood probe, O-39 added Phase I for Rule 10c standalone-rows spec ambiguity, O-27 retired VARIANCE→NOTE).

## By tag
- **VARIANCE** (12): [[O-01]], [[O-10]], [[O-12]], [[O-20]], [[O-25]], [[O-28]], [[O-30]], [[O-31]], [[O-32]], [[O-33]], [[O-34]], [[O-37]]
- **SPEC** (8): [[O-04]], [[O-06]], [[O-07]], [[O-11]], [[O-17]], [[O-21]], [[O-38]], [[O-39]]
- **BUG** (2): [[O-02]], [[O-08]]
- **NOTE** (17): [[O-03]], [[O-05]], [[O-09]], [[O-13]], [[O-14]], [[O-15]], [[O-16]], [[O-18]], [[O-19]], [[O-22]], [[O-23]], [[O-24]], [[O-26]], [[O-27]], [[O-29]], [[O-35]], [[O-36]]

## Upstream-reportable (open — none yet Reported)
[[O-01]], [[O-02]], [[O-04]], [[O-06]], [[O-07]], [[O-08]], [[O-09]], [[O-11]], [[O-17]], [[O-21]], [[O-30]], [[O-31]], [[O-32]], [[O-33]], [[O-34]], [[O-38]], [[O-39]]

> [!divergence] 17 upstream-reportable O-Ns (none yet marked Reported) — the AGS-DFWG candidate list. SPEC/BUG ones (O-2,6,7,8,11,17,21,38,39 + the spec-ambiguity O-1) are the strongest 4.2-improvement proposals; VARIANCE ones (O-30..O-34) document our deliberate, data-driven divergences.

## Why it matters
The SPEC/BUG entries are concrete AGS 4.2 rule-improvement proposals (prose⇄dictionary mismatches O-6/O-7, Rule 6 no-op O-2, rule_7_2 crash O-8, attribution O-4/O-11/O-17, hardcoded parentless O-21). The VARIANCE entries document our deliberate, data-driven divergences. Phase D turns this into the test-strategy + AGS5-requirement registers.

## Related
[[upstream-reporting]] · [[parity-model]] · [[ags4-rules-frozen-dictionary-evolves]]
