---
type: strategy
title: "Out-of-pandas-range date → both flag Rule 8 (O-33, dogfood-proven)"
status: confirmed
tags: [strategy]
targets: [rule-08-typed-values]
divergence_hypothesis: "see body"
probe_files: []
expected_rust: "Rule 8"
expected_python: "Rule 8"
evidence: "probe-run / prior dogfood — see body"
related: [rule-08-typed-values, O-33, O-12]
sources: []
---
# Out-of-pandas-range date → both flag Rule 8 (O-33, dogfood-proven)

## Hypothesis
> [!divergence] A spec-valid but pre-1678/post-2262 date: chrono accepted (Rust clean) vs pandas NaT (python Rule 8). Bounded in O-33 → both Rule 8.

## Probe design
- Fixture: `(proven by the O-33 5,492-file corpus: 8 PYTHON_ONLY→AGREE)` (under `.bootstrap/probes/` — NEVER `laterite-ags4-validator/tests/fixtures/`).
- Run: `lat validate <probe>` and `uv run python tools/py_ags4_check_json.py <probe>`.

## Expected vs observed

| | Rust `lat` | python-ags4 |
|---|---|---|
| expected | Rule 8 | Rule 8 |
| observed | Rule 8 (chrono, bounded to pandas range — O-33) | Rule 8 (pandas NaT, pre-1678/post-2262) |

> [!note] Provenance: proven by the O-33 5,492-file corpus run
> (8 PYTHON_ONLY→AGREE), not a minimal re-runnable probe. The general
> Rule-8 minimal reproduction is the `Rule 8` row of
> [[strat-parity-matrix]] (non-date in a DT column → both AGREE).

## Verdict
> [!note] **CONFIRMED.** CONFIRMED by the O-33 real-data proof.

## Related
[[rule-08-typed-values]] · [[O-33]] · [[O-12]]
