---
type: insight
title: "<gap one-liner>"
status: hypothesis            # hypothesis | probed | confirmed | ratified | refuted
tags: [insight]
gap_kind: ""                  # spec-ambiguity | spec-contradiction | cross-edition-regression | spec-vs-rust | rust-vs-python | rule-weakness
severity: ""                  # low | med | high
editions_affected: []         # subset of [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
rules: []                     # [[rule-…]] stems
proposes_observation: false   # true => the block below is a drafted O-N for user ratification
feeds_strategy: []            # [[strategies/…]]
discovered_phase: ""          # A | B | C | D
related: []
sources: []
---

# <gap one-liner>

## Claim
> [!{spec-ambiguity|divergence|variance}] One-sentence gap statement.

## Evidence
- Spec: `spec:AGS4-<ed>-*.pdf §…`  (cite, never paste)
- Code: `repo:…`
- Probe (if `status: confirmed`): command + the Rust vs python outputs.

## Why it matters
<!-- impact on validation correctness / interoperability / AGS5 -->

## Proposed OBSERVATIONS entry
> [!note] Only if `proposes_observation: true`. Draft the entry here,
> then write it into `OBSERVATIONS.md`
> (canonical authority — 5-field house style, next free O-N,
> clean-room) and set this page `status: ratified`.

```
### O-NN [TAG] <title>
- **Observed**: …
- **Spec**: …
- **Assessment**: …
- **Upstream-reportable**: …
- **Our decision**: …
```

## Related
<!-- [[rule-…]] · [[strategies/…]] · [[design/…]] · [[O-NN]] -->
