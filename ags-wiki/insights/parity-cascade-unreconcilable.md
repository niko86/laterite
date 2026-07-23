---
type: insight
title: "Presence-only reconcile() cannot whittle a python parse-layer cascade → false ACTION noise"
status: ratified
tags: [insight]
gap_kind: rust-vs-python
severity: med
editions_affected: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
rules: [rule-06-comma-no-embedded-crlf, rule-01-ascii, rule-05-quoting]
proposes_observation: true
feeds_strategy: [strat-rule6-embedded-cr, strat-parity-matrix]
feeds_ags5_req: []
discovered_phase: D
related: [parity-model, strat-rule6-embedded-cr, strat-parity-matrix, O-35, O-02, O-01, O-03]
sources: []
---
# Parity cascades are unreconcilable by presence-only diffing

## Claim
> [!divergence] python-ags4's **parsing layer** turns one malformed
> construct into a *multi-rule cascade*: a lone CR → universal-newline
> record split → Rule 2a+3+5 ([[strat-rule6-embedded-cr]]); a valid
> extended char → Rust FYI-only / python silent ([[O-01]]); an
> unquoted field → python Rule **3** (or 4 by position) vs Rust Rule 5
> ([[O-03]]). `repo:rust-packages/ags4-corpus-qa/src/parity.rs`
> `reconcile()` whittles only **single documented rule-swaps**
> (O-2/O-3/O-26/O-27) and only when the *entire* symmetric diff is
> consumed — so a cascade leaves residue and a *known* root cause
> classifies as a **false `RUST_ONLY`/`PYTHON_ONLY` ACTION**.

## Evidence (probe + matrix, both run)
- `ags-wiki/.bootstrap/probes/RESULTS.md`: embedded-CR → `rust={6}` vs
  `py={2a,3,5}` → unreconciled.
- `ags-wiki/.bootstrap/probes/parity-matrix.md`: Rule 1-valid-extended
  → `RUST_ONLY {FYI Rule 1}` (no O-1 arm); Rule 5-unquoted →
  `RUST_ONLY` (`py={Rule 3}`, outside O-3's Rule-4-only arm).

## Why it matters
The dogfood ACTION list is the triage signal. Polluting it with
*known* cascades (every embedded-CR / valid-extended / unquoted-field
file) trains reviewers to ignore it — the precise way a real
divergence gets missed. Generic widening is **not** the fix: Rules
2a/3/5/9/18 fire for many legitimate reasons; absorbing them whenever
Rust has Rule 6 would mask genuine clean-room failures. The safe
direction is **signature-narrow** reconcile arms (à la the O-34
triple-guard), proposed below for ratification — never a silent broad
widening of the clean-room reconcile.

## OBSERVATIONS entry — **ratified as [[O-35]]**
> [!spec] **[NOTE] O-35: python parse-layer cascades are unreconcilable
> by presence-only parity.** A lone embedded CR, a valid extended
> char, or an unquoted field each makes python emit a *set* of rules
> (2a/3/5; ∅; 3) where Rust emits one (6; FYI-1; 5). Recommend
> documenting that these classify as ACTION today, and adding
> **signature-narrow** arms: `rust=={Rule 6} ∧ py⊆{2a,3,5} → O-2`;
> `rust=={FYI Rule 1} ∧ py==∅ → O-1`; `rust⊇{Rule 5} ∧ py⊇{Rule 3}
> (unquoted) → O-3`. Each bounded like the O-34 guard.
>
> **Ratified**: written to
> `repo:OBSERVATIONS.md#o-35`
> ([NOTE], in the Post-V8 section) with the signature-narrow follow-up
> arms as the sanctioned next step.

## Related
[[parity-model]] · [[strat-rule6-embedded-cr]] · [[strat-parity-matrix]] · [[O-35]] · [[O-02]] · [[O-01]] · [[O-03]]
