---
type: strategy
title: "Probe: ELRG group — does Rust mis-flag Rule 9? (tests proposed O-35)"
status: confirmed
tags: [strategy]
targets: [rule-09-unknown-headings, rule-15-unit-group]
divergence_hypothesis: "see body"
probe_files: [ags-wiki/.bootstrap/probes/probe-elrg.ags]
expected_rust: "(hypothesised) Rule 9 on ELRG_*"
expected_python: "no Rule 9 on ELRG"
evidence: "probe-run / prior dogfood — see body"
feeds_ags5_req: []
related: [rule-09-unknown-headings, elrg-not-in-repo-dictionary, rule15-example-tracks-eres-elrg-removal]
sources: []
---
# Probe: ELRG group — does Rust mis-flag Rule 9? (tests proposed O-35)

## Hypothesis
> [!divergence] If the Rust validator used ags5_dictionary.json (which lacks ELRG) it would Rule-9 ELRG_* headings while python wouldn't.

## Probe design
- Fixture: `ags-wiki/.bootstrap/probes/probe-elrg.ags` (under `.bootstrap/probes/` — NEVER `laterite-ags4-validator/tests/fixtures/`).
- Run: `lat validate <probe>` and `uv run python tools/py_ags4_check_json.py <probe>`.

## Expected vs observed

| | Rust `lat` | python-ags4 |
|---|---|---|
| expected | (hypothesised) Rule 9 on ELRG_* | no Rule 9 on ELRG |
| observed | Rule 10a/10b/10c on ELRG (lists ELRG_RUNI/ELRG_METH/parent SAMP) — NO Rule 9. Rust's bundled dictionary KNOWS ELRG. | Rule 10a/10b/10c — largely AGREE on the relational shape. |

## Verdict
> [!note] **CONFIRMED.** probe run: Rust gave Rule 10a/10b/10c and NO Rule 9 — it recognises ELRG. Hypothesis REFUTED: laterite-ags4-validator bundles its own dicts, independent of ags5_dictionary.json.

## Related
[[rule-09-unknown-headings]] · [[elrg-not-in-repo-dictionary]] · [[rule15-example-tracks-eres-elrg-removal]]
