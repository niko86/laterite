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
> [!note] Every O-N entry by tag and upstream-reportability — the spine of the 4.2-improvement / AGS-DFWG list. The two lists below are **generated** from `observations.json`, the catalogue's SSOT, so this page cannot fall behind it again: it had frozen at "all 39 O-N entries" (a Phase-I snapshot) while the catalogue reached 50, and its upstream set had drifted from the one `OBSERVATIONS.md` actually renders.

<!-- BEGIN GENERATED: observations-coverage — from observations.json; regenerate with `uv run --no-sync python tools/gen_observations.py` (DO NOT EDIT THE LISTS BY HAND) -->

## By tag
- **VARIANCE** (19): [[O-01]], [[O-10]], [[O-12]], [[O-20]], [[O-25]], [[O-28]], [[O-30]], [[O-31]], [[O-32]], [[O-33]], [[O-34]], [[O-37]], [[O-41]], [[O-42]], [[O-43]], [[O-44]], [[O-45]], [[O-49]], [[O-50]]
- **SPEC** (8): [[O-04]], [[O-06]], [[O-07]], [[O-11]], [[O-17]], [[O-21]], [[O-38]], [[O-39]]
- **BUG** (2): [[O-02]], [[O-08]]
- **NOTE** (21): [[O-03]], [[O-05]], [[O-09]], [[O-13]], [[O-14]], [[O-15]], [[O-16]], [[O-18]], [[O-19]], [[O-22]], [[O-23]], [[O-24]], [[O-26]], [[O-27]], [[O-29]], [[O-35]], [[O-36]], [[O-40]], [[O-46]], [[O-47]], [[O-48]]

## Upstream-reportable (16)
[[O-01]], [[O-02]], [[O-04]], [[O-06]], [[O-07]], [[O-08]], [[O-11]], [[O-17]], [[O-21]], [[O-30]], [[O-31]], [[O-32]], [[O-38]], [[O-39]], [[O-42]], [[O-49]]

<!-- END GENERATED: observations-coverage -->

> [!divergence] The AGS-DFWG candidate list. The SPEC/BUG ones (plus the spec-ambiguity [[O-01]]) are the strongest 4.2-improvement proposals; the VARIANCE ones document our deliberate, data-driven divergences. [[strat-ags-dfwg-upstream-list]] turns this set into the actual proposals, tiered.

## Why it matters
The SPEC/BUG entries are concrete AGS 4.2 rule-improvement proposals (prose⇄dictionary mismatches O-6/O-7, Rule 6 no-op O-2, rule_7_2 crash O-8, attribution O-4/O-11/O-17, hardcoded parentless O-21). The VARIANCE entries document our deliberate, data-driven divergences — including the two that are only *reachable* upstream because the AGS4 spec is silent on them (O-30/O-42, edition resolution). Downstream of this page: the test-strategy register and [[strat-ags-dfwg-upstream-list]].

## Related
[[upstream-reporting]] · [[parity-model]] · [[ags4-rules-frozen-dictionary-evolves]]
