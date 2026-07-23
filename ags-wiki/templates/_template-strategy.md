---
type: strategy
title: "<test-strategy one-liner>"
status: proposed              # proposed | probed | confirmed
tags: [strategy]
targets: []                   # [[rule-…]] under test
divergence_hypothesis: ""     # where/why Rust and python-ags4 might disagree
probe_files: []               # ags-wiki/.bootstrap/probes/<name>.ags
expected_rust: ""
expected_python: ""
evidence: ""                  # probe output ref once run
feeds_ags5_req: []
related: []
sources: []
---

# <test-strategy one-liner>

## Hypothesis
> [!divergence] Where Rust ↔ python-ags4 could disagree, and why
> (cite the edition delta / impl / O-N that motivates it).

## Probe design
- Fixture(s): `ags-wiki/.bootstrap/probes/<name>.ags` (NEVER
  `laterite-ags4-validator/tests/fixtures/` — corpus-qa e2e asserts that
  dir hard-error-free).
- Run: `lat validate <probe>` and `uv run python tools/py_ags4_check_json.py <probe>`.

## Expected vs observed

| | Rust `lat` | python-ags4 |
|---|---|---|
| expected | <!-- --> | <!-- --> |
| observed (probe) | <!-- status:probed+ --> | <!-- --> |

## Verdict
<!-- AGREE / divergence confirmed → which [[insight]] it proves; status -->

## Related
<!-- [[rule-…]] · [[parity-model]] · [[insights/…]] -->
