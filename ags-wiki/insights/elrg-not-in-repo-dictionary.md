---
type: insight
title: "Repo dictionary lacks ELRG despite declaring edition 4.1 (ERES→ELRG untracked)"
status: refuted
tags: [insight]
gap_kind: rust-vs-python
severity: high
editions_affected: [4.1, 4.1.1, 4.2]
rules: [rule-15-unit-group, rule-09-unknown-headings]
proposes_observation: false
feeds_strategy: []
discovered_phase: B
related: [rule-15-unit-group]
sources: [spec-4.2]
---
# Repo dictionary lacks ELRG despite declaring edition 4.1 (ERES→ELRG untracked)

## Claim
> [!divergence] ags5_dictionary.json (ags_edition:4.1) contains ERES, lacks ELRG — yet 4.1 replaced ERES with ELRG. The Rust validator's effective dictionary therefore cannot recognise ELRG_* headings: a conformant 4.1/4.2 file using ELRG would trip Rule 9 (unknown heading) in Rust, while python-ags4 (using its own up-to-date dict) would not — a likely Rust-only false-positive. PROPOSED O-35 (await ratification): document the repo-dictionary staleness + decide refresh. MUST be empirically probed in Phase D (craft an ELRG file, run both validators) before status→confirmed.

## Proposed OBSERVATIONS entry
> [!note] Awaiting user ratification — the agent never writes OBSERVATIONS.md (AGS-WIKI §12.5).

```
### O-35 [VARIANCE] Repo dictionary lacks ELRG despite declaring edition 4.1 (ERES→ELRG untracked)
- **Observed**: …
- **Spec**: …
- **Assessment**: …
- **Upstream-reportable**: …
- **Our decision**: …
```

## Related
[[rule-15-unit-group]] · [[rule-09-unknown-headings]] · [[rule15-example-tracks-eres-elrg-removal]] · [[ags4-rules-frozen-dictionary-evolves]]

## Phase D probe outcome
> [!divergence] **REFUTED.** PROBE-REFUTED (probe-elrg). The proposed O-35 conflated two dictionaries: the Rust laterite-ags4-validator bundles its OWN AGS dicts (build.rs, all 5 editions) and DOES recognise ELRG (probe: Rule 10a/10b/10c, no Rule 9). At the time of this probe, the `ags5_dictionary.json` (then the Python `ags5-models` registry) genuinely lacked ELRG, but that was a Python-model data gap with NO effect on the Rust validator — it did not cause a Rust Rule-9 false-positive. proposes_observation withdrawn (no O-35 as framed). The narrower true finding: the Python `ags5-models` registry was stale re: ERES→ELRG (an AGS5-model bookkeeping input) — moot now that `ags5-models` was deleted entirely (F2c arc); the AGS5 dict now lives with the dormant AGS5 strand, outside this tree, unaffected by this finding. <!-- historical -->
