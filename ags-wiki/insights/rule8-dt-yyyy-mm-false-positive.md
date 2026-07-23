---
type: insight
title: "Rule 8 DT/yyyy-mm flags valid month-only dates as invalid"
status: refuted
tags: [insight, rule-08, dt-validation, resolved]
gap_kind: validator-bug
severity: medium
editions_affected: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
rules: [rule-08-typed-values]
proposes_observation: false
feeds_strategy: []
feeds_ags5_req: []
discovered_phase: I
related: [O-12, rule-08-typed-values, parity-model]
sources: []
---

> [!info] **RESOLVED Stage 7c (2026-05-26).** Added an explicit
> `if u == "yyyy-mm"` branch in
> `rust-packages/laterite-ags4-validator/src/rules/typed_values.rs::dt_semantic_ok`
> that synthesises day-01 and checks month ∈ 1..=12 before binding
> to the pandas Timestamp range. Validator test added
> (`dt_yyyy_mm_month_precision`). python-ags4 parity:
> `test_rule_8_4` line-17 LOCA_ENDD `2023-11` no longer false-flags;
> line-18 `2023-13` correctly fires.

# Rule 8 — DT validation rejects valid `yyyy-mm` values

## Claim
> [!divergence] Surfaced during Stage 6b nSF Expected work: on
> `../ags-python-library/tests/test_files/4.1-rule8-4.ags` line 17,
> the LOCA_ENDD value `2023-11` (declared TYPE `DT`, UNIT `yyyy-mm`)
> is **flagged as invalid by laterite** but accepted by python-ags4.
> Structurally `2023-11` is `YYYY-MM` and semantically month 11 is a
> valid month.

## Symptom

```python
# laterite emits:
{"rule": "AGS Format Rule 8", "line": 17, "group": "LOCA",
 "desc": "Value 2023-11 in LOCA_ENDD does not match the specified "
         "format (yyyy-mm) or is an invalid date/time."}

# python-ags4 on same fixture, same line: NO finding (accepts 2023-11)
```

## Hypothesis

`rules/typed_values.rs::structural_dt_match` or
`dt_semantic_ok` incorrectly treats `yyyy-mm` as requiring a day
(`yyyy-mm-dd`) — likely a default day appended during semantic
validation that then fails for a 2-segment date.

## Probe needed

1. Isolate `structural_dt_match("2023-11", "yyyy-mm")` — does it
   return `true`?
2. Isolate `dt_semantic_ok("2023-11", "yyyy-mm")` — what does it
   actually check? If it parses to `chrono::NaiveDate` it would
   fail (no day component) — that's a real bug.
3. Compare against python-ags4's `check.is_valid_DT_value`
   implementation (which presumably handles yyyy-mm correctly).

## Decision

Hold until probed. If confirmed validator bug, fix in Rust + add
test in `rules/typed_values.rs`. May produce a new O-N if behaviour
needs documenting (likely just a fix without O-N — bugs that we
correct don't earn observations, only variances do).

## Discovered while

Stage 6b of the python-ags4 parity arc — adding the nSF
`(Expected: NN)` suffix uncovered this on `test_rule_8_4`. The
wording divergence was fixed via translator entry; this is the
*remaining* gap.

## Related
[[O-12]] · [[rule-08-typed-values]] · [[parity-model]]
