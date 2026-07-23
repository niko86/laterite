---
type: insight
title: "Default --parity-sample 0 cross-checks only triage files — confidently-wrong files never reach the oracle"
status: ratified
tags: [insight]
gap_kind: rust-vs-python
severity: high
editions_affected: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
rules: []
proposes_observation: true
feeds_strategy: [strat-parity-matrix]
feeds_ags5_req: []
discovered_phase: D
related: [parity-model, strat-parity-matrix, oracle-drift-pin, rust-vs-python-ags4-parity, O-36]
sources: []
---
# The parity differential is structurally biased to triage files

## Claim
> [!divergence] `repo:rust-packages/ags4-corpus-qa/src/parity.rs`
> builds the parity set as **triage ∪ a reservoir sample of size
> `--parity-sample`**, and `--parity-sample` **defaults to 0**
> (`repo:rust-packages/ags4-corpus-qa/src/cli.rs`). So by default the
> only files cross-checked against python-ags4 are those the Rust side
> *already* flagged odd (HardError / Panic / `surprising`). A file the
> Rust validator handles **confidently but wrongly** — plausible
> `Findings`, not surprising — is **never** sent to the oracle. Silent
> agreement on a wrong verdict is invisible: the exact failure a
> clean-room cross-check exists to catch.

## Evidence
- Parity set construction: `parity.rs` `run()` —
  `triage ∪ reservoir(rest, args.parity_sample)`; `--parity-sample`
  default `0` (`cli.rs` `ParityArgs`).
- Corollary: "12.5k corpus mostly AGREEs" is uninformative about
  untested rules — agreement is dominated by clean files; whole rules
  (e.g. 11c) can have *zero* differential evidence yet hide inside the
  AGREE bucket (the [[strat-parity-matrix]] blind-spot list quantifies
  this complement: 13 rules with no differential coverage at all).

## Why it matters
The headline trust statement ("dogfooded against python-ags4 at
scale") is, by default, "dogfooded against python-ags4 on the files
Rust already found suspicious." That is the opposite sampling of what
builds confidence. Mitigations are cheap and already partly present
(`--parity-sample N` exists) — the gap is the **default** and the
absence of a per-rule coverage report.

## OBSERVATIONS entry — **ratified as [[O-36]]**
> [!spec] **[NOTE] O-36: parity sampling is triage-biased by default.**
> `--parity-sample 0` means confidently-wrong (non-surprising
> `Findings`) files are never oracle-checked. Sanctioned follow-ups:
> (a) a non-zero default sample (or an explicit "differential is
> triage-only" banner), and (b) a per-rule "rules with zero parity
> evidence across this run" report so the AGREE bucket cannot hide
> untested rules.
>
> **Ratified**: written to
> `repo:OBSERVATIONS.md#o-36`
> ([NOTE], Post-V8). The per-rule matrix (`parity_matrix_dogfood`) is
> the first instalment of (b).

## Related
[[parity-model]] · [[strat-parity-matrix]] · [[oracle-drift-pin]] · [[rust-vs-python-ags4-parity]] · [[O-36]]
