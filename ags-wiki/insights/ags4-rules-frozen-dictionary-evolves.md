---
type: insight
title: "AGS4 Rules frozen since 4.0.x while the data model gained ~20+ groups"
status: confirmed
tags: [insight]
gap_kind: rule-weakness
severity: med
editions_affected: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
rules: []
proposes_observation: false
feeds_strategy: []
feeds_ags5_req: []
discovered_phase: A
related: [start-here, edition-resolution, rule15-example-tracks-eres-elrg-removal]
sources: [spec-4.2]
---
# AGS4 Rules normatively frozen 4.0.3→4.2 while the data model kept growing

## Claim
> [!note] The 20 Rules (+subrules) are **normatively unchanged across
> 4.0.3 → 4.2** — but the rule *text is not byte-identical*. Verified
> by reading every edition's rules section directly (4.0.3 §8.1,
> 4.0.4 §8.1, 4.1/4.1.1/4.2 §4.1.1), NOT by trusting a foreword.
> Three edition text-deltas exist: (1) 4.1 reorganised prose §8.1 →
> tabulated §4.1.1; (2) 4.1 dropped the "(Section 8)/(Section 7.3)"
> cross-refs in Rules 7/10c/11; (3) **Rule 15's example changed
> `ERES_RUNI`→`ELRG_RUNI` at 4.1** as the dictionary replaced ERES
> with ELRG. Meanwhile the Data Dictionary expanded substantially
> (4.2 alone: +PMMx/DMDx/DMTx/ISTx/MONS/ITCH/CBRP, ERES & IPRG/IPRT
> removed, CTRx/RESx deprecated).

## Evidence (primary, all 5 PDFs)
- 4.0.3 §8.1 / 4.0.4 §8.1 (prose) — read; Rules 1-20 verbatim equal;
  Rule 15 ex = `ERES_RUNI`; Rule 7/10c/11 carry the Section refs.
- 4.1 §4.1.1 / 4.1.1 §4.1.1 / 4.2 §4.1.1 (table) — read; Rules 1-20
  verbatim equal; Rule 15 ex = `ELRG_RUNI`; Section refs dropped.
- 4.1 Foreword: "The Rules have not changed since version 4.0.4, but
  they have been moved into a new section (4.1.1) and tabulated";
  4.1.1 Foreword: "very minor update… Rules remain unchanged"; 4.2
  Foreword: "The AGS 4 Rules remain unchanged".
- Concrete proof: [[rule15-example-tracks-eres-elrg-removal]].

## Why it matters
A normatively frozen rule set over a growing model means newer
constructs (PMMx, MONS, IST*, numbered groups) are governed only by
generic rules (10a/10c/11/19/19b) never re-examined for them — and
Rule 15 *visibly went stale* (`ERES_RUNI`) until patched. This is the
**central thesis for the test strategy and AGS5**: real 4.2 weakness
lives in interpretation/implementation drift and rules-vs-model lag,
not in normative-text edits. AGS5 should co-evolve rules with the
model and not embed model-specific examples in rule text.

## Related
[[start-here]] · [[rule15-example-tracks-eres-elrg-removal]] · [[edition-resolution]] · [[ags-4.2]] · ags4-vs-ags5 · [[parity-model]]
