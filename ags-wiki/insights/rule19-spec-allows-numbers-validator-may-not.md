---
type: insight
title: "Rule 19 spec permits digits in GROUP names; validator enforces letters-only (O-6)"
status: confirmed
tags: [insight]
gap_kind: spec-vs-rust
severity: med
editions_affected: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
rules: [rule-19-group-name-format]
proposes_observation: false
feeds_strategy: []
feeds_ags5_req: []
discovered_phase: A
related: [rule-19-group-name-format, O-06]
sources: [spec-4.2]
---
# Rule 19: spec permits digits; validator may enforce letters-only

## Claim
> [!divergence] `spec:AGS4-4.2-2025.pdf §4.1.1 Rule 19`: a GROUP name "shall not be more than
> 4 characters long and shall consist of **uppercase letters and
> numbers** only." Existing [[O-06]] records the Rust validator
> enforcing *exactly 4 uppercase letters* (stricter than spec). To
> confirm against actual code + python-ags4 in Phase B/D.

## Evidence
- Spec: `spec:AGS4-4.2-2025.pdf §4.1.1 Rule 19` (verbatim on [[rule-19-group-name-format]]).
- Prior art: [[O-06]] (`repo:OBSERVATIONS.md#o-6`).
- Status `hypothesis` — Phase B reads `rules/naming.rs`; Phase D
  probes a digit-bearing GROUP name through both validators.

## Why it matters
A spec-valid GROUP like `IST1` (4.2 added `ISTx` series!) would be
rejected by a letters-only check — directly interacts with the
[[ags4-rules-frozen-dictionary-evolves]] thesis (new numbered groups
vs a frozen, possibly mis-implemented Rule 19). Motivates
req-unambiguous-identifier-charset — the spec should state the
exact GROUP/HEADING charset regex, not leave letters-vs-numbers
ambiguous against the de-facto convention.

## Related
[[rule-19-group-name-format]] · [[O-06]] · [[ags4-rules-frozen-dictionary-evolves]]

## Phase B verification
> [!note] **confirmed** (code-read). CODE-VERIFIED: naming.rs:73-75 enforces exactly-4-uppercase-letters; test naming.rs:193 asserts a digit in a GROUP name IS flagged. Rust (clean-room) is deliberately stricter than 4.2 Rule 19 ('≤4, letters AND numbers'); 0/319 standard groups deviate, but a spec-valid user-defined group with a digit (Rule 18/DICT) is wrongly rejected. Practical impact: user-defined groups. python-ags4 follows the same convention. Upstream-reportable: AGS Rule 19 prose vs the de-facto dictionary convention diverge (O-6).

## Phase D probe outcome
> [!note] **CONFIRMED.** PROBE-CONFIRMED (probe-rule19-digit): Rust flags Rule 19 on 'AB12'; python-ags4 does not. CRUCIAL FRAMING (user-confirmed): the exactly-4-uppercase-letters GROUP rule (and AAAA_BBBB headings) is an **informal rule the project deliberately adopted** — driven by (a) the python library effectively limiting it and (b) the AGS dictionary universally conforming (0/319 groups, 0/4199 headings deviate — O-6/O-7). This is NOT a Rust over-strictness bug; Rust is the more-correct side vs the format's own reality. The genuine, upstream-reportable defect is the **AGS 4.2 Rule 19 prose** being looser than the format ('letters and numbers, <=4' is dead text). AGS-DFWG candidate: tighten the prose to match the informal convention everyone already enforces.
