---
type: insight
title: "Rule 1 says 'entirely ASCII' (0-127) but 128-255 is tolerated (O-1/O-32)"
status: hypothesis
tags: [insight]
gap_kind: spec-ambiguity
severity: med
editions_affected: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
rules: [rule-01-ascii]
proposes_observation: false
feeds_strategy: []
discovered_phase: A
related: [rule-01-ascii, O-01, O-32]
sources: [spec-4.2]
---
# Rule 1: "entirely ASCII" vs tolerated extended ASCII

## Claim
> [!spec-ambiguity] `spec:AGS4-4.2-2025.pdf §4.1.1 Rule 1`: "The data file shall be entirely
> composed of ASCII characters." ASCII strictly = 0–127, but
> [[O-01]]/[[O-32]] show both validators tolerating 128–255
> (extended/Latin-1) as FYI/lossy, not a hard fail. The spec word
> "ASCII" is ambiguous vs ubiquitous real-world cp1252 data.

## Evidence
- Spec: `spec:AGS4-4.2-2025.pdf §4.1.1 Rule 1` (verbatim on [[rule-01-ascii]]).
- Prior art: [[O-01]] (0–255 tolerance), [[O-32]] (lossy decode →
  Rule 1 finding, mirrors python `errors="replace"`).
- `hypothesis` until Phase D probes the 0–127 vs 128–255 vs >255
  boundary through both validators.

## Why it matters
Real AGS deliveries are endemically cp1252. A strict reading would
hard-reject most of the corpus; the lenient reading (both validators)
is pragmatic but *undocumented in the spec* — a concrete AGS-DFWG
upstream candidate and an AGS5 encoding-policy requirement.

## Related
[[rule-01-ascii]] · [[O-01]] · [[O-32]] · [[upstream-reporting]]
