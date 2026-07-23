---
type: strategy
title: "Probe: digit-bearing GROUP name — the gap is the 4.2 prose, not the validator"
status: confirmed
tags: [strategy]
targets: [rule-19-group-name-format]
divergence_hypothesis: "see body"
probe_files: [ags-wiki/.bootstrap/probes/probe-rule19-digit.ags]
expected_rust: "Rule 19 (de-facto exactly-4-letters — correct vs the dictionary)"
expected_python: "no Rule 19 (python's len==4 & isupper() both pass 'AB12' — a narrower de-facto check)"
evidence: "probe-run / prior dogfood — see body"
feeds_ags5_req: []
related: [rule-19-group-name-format, rule19-spec-allows-numbers-validator-may-not, O-06, O-07]
sources: []
---
# Probe: digit-bearing GROUP name — the gap is the 4.2 prose, not the validator

## Hypothesis
> [!divergence] 4.2 Rule 19 prose permits 'uppercase letters AND numbers, <=4'. The DELIBERATE design (user-confirmed): Rust enforces the format's real, universal convention — GROUP = exactly 4 uppercase letters, HEADING = AAAA_BBBB (4 letters + _ + <=4) — because (a) the python library effectively limits it and (b) 0/319 standard groups & 0/4199 headings deviate (O-6/O-7). Probe 'AB12' shows where the loose prose and the real convention part.

## Probe design
- Fixture: `ags-wiki/.bootstrap/probes/probe-rule19-digit.ags` (under `.bootstrap/probes/` — NEVER `laterite-ags4-validator/tests/fixtures/`).
- Run: `lat validate <probe>` and `uv run python tools/py_ags4_check_json.py <probe>`.

## Expected vs observed

| | Rust `lat` | python-ags4 |
|---|---|---|
| expected | Rule 19 (de-facto exactly-4-letters — correct vs the dictionary) | no Rule 19 (python's len==4 & isupper() both pass 'AB12' — a narrower de-facto check) |
| observed | Rule 19 flagged: 'GROUP name must be exactly 4 uppercase letters (A-Z)' (+7,9,18,19b,10b,10c) | Rule 7,9,10b,10c,18 — NO Rule 19 (len('AB12')==4 and 'AB12'.isupper() is True) |

## Verdict
> [!note] **CONFIRMED.** probe run: Rust=Rule19 present; python=Rule19 absent. CONFIRMED — but this is NOT a Rust over-strictness defect: Rust is the MORE-correct side vs the AGS dictionary's universal convention; the actionable gap is the AGS 4.2 Rule 19 PROSE being looser than the format's own reality (dead 'letters and numbers' allowance — 0/319 use it). python's isupper() check merely happens not to catch the digit case. Upstream-reportable as a SPEC defect (O-6), not a validator bug.

## Related
[[rule-19-group-name-format]] · [[rule19-spec-allows-numbers-validator-may-not]] · [[O-06]] · [[O-07]]
