---
type: insight
title: "O-8's rule_7_2 IndexError is effectively unreachable under default rename_duplicate_headers=True"
status: ratified
tags: [insight]
gap_kind: rust-vs-python
severity: low
editions_affected: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
rules: [rule-07-heading-order]
proposes_observation: true
feeds_strategy: [strat-o8-dup-heading-refuted]
feeds_ags5_req: []
discovered_phase: D
related: [O-08, strat-o8-dup-heading-refuted, rule-07-heading-order, rule-09-unknown-headings]
sources: []
---
# O-8 is shielded by python-ags4's own default

## Claim
> [!divergence] [[O-08]] [BUG] documents python `rule_7_2` indexing
> `temp[i]` unguarded → `IndexError` on a trailing duplicate in-order
> HEADING. A probe ([[strat-o8-dup-heading-refuted]]) shows it does
> **not** fire: `AGS4.check_file(..., rename_duplicate_headers=True)`
> — the **default**, and exactly what `ags4 check` / our wrapper does
> — renames the 2nd `PROJ_NAME` → `PROJ_NAME_1` before `rule_7_2`, so
> `set(headings).issubset(reference)` fails first and `temp[i]` is
> never reached (python instead emits Rule 7+9+18). The crash is
> reachable only with the non-default `rename_duplicate_headers=False`.

## Evidence
- Probe: `ags-wiki/.bootstrap/probes/probe-o8-dup-heading.ags` →
  RESULTS.md (python: Rule 7+9+18, **no crash**).
- `ext:ags-python-library:python_ags4/check.py` `rule_7_2`
  (`temp[i]`); `check_file(..., rename_duplicate_headers=True)` default
  (verified via `inspect.signature`).
- Decision in `repo:rust-packages/ags4-corpus-qa/src/parity.rs`
  `classify()` comment: no speculative O-8 crash arm (would
  over-claim); generic `PythonError` short-circuit is adequate.

## Why it matters
Keeps the catalogue honest: O-8's prose ("can crash the validator")
overstates real-world reachability under default python-ags4 1.2.0.
Rust's bounds-guard is still correct defensive coding, but the
*divergence* it documents is narrower than written. Also tightly
[[oracle-drift-pin|coupled to the pinned oracle]]: a future
python-ags4 that changes the rename default would change this.

## OBSERVATIONS entry — **ratified (O-8 refined)**
> [!spec] **[[O-08]] refined.** Its Assessment now states the
> precondition: the `rule_7_2` IndexError requires
> `rename_duplicate_headers=False` (non-default); under default
> python-ags4 1.2.0 a duplicate HEADING yields Rule 7+9+18, not a
> crash. The [BUG] tag is kept (latent — a `False` toggle exposes
> it) but it is no longer cited as a routinely-reachable crash.
>
> **Ratified**: written into
> `repo:OBSERVATIONS.md#o-8`
> (Assessment + Upstream-reportable + Our-decision bullets).

## Related
[[O-08]] · [[strat-o8-dup-heading-refuted]] · [[rule-07-heading-order]] · [[rule-09-unknown-headings]]
