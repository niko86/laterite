---
type: strategy
title: "cp1252 input → both emit Rule 1 (O-32, dogfood-proven)"
status: confirmed
tags: [strategy]
targets: [rule-01-ascii]
divergence_hypothesis: "see body"
probe_files: []
expected_rust: "Rule 1"
expected_python: "Rule 1"
evidence: "probe-run / prior dogfood — see body"
related: [rule-01-ascii, O-32, O-01]
sources: []
---
# cp1252 input → both emit Rule 1 (O-32, dogfood-proven)

## Hypothesis
> [!divergence] Invalid-UTF-8 (cp1252 °) → Rust from_utf8_lossy U+FFFD → Rule 1; python errors='replace' U+FFFD → Rule 1.

## Probe design
- Fixture: `(proven at scale by the O-32 12,503-file dogfood, not a synthetic probe)` (under `.bootstrap/probes/` — NEVER `laterite-ags4-validator/tests/fixtures/`).
- Run: `lat validate <probe>` and `uv run python tools/py_ags4_check_json.py <probe>`.

## Expected vs observed

| | Rust `lat` | python-ags4 |
|---|---|---|
| expected | Rule 1 | Rule 1 |
| observed | Rule 1 (`from_utf8_lossy`→U+FFFD) | Rule 1 (`errors='replace'`→U+FFFD) |

> [!note] Provenance: proven at **scale** by the O-32 12,503-file
> dogfood (12 cp1252 files moved VALIDITY_DISAGREE→AGREE on Rule 1),
> not a minimal re-runnable probe. The per-rule minimal reproduction
> is the `Rule 1 (invalid byte)` row of [[strat-parity-matrix]]
> (raw `0xB0` → both AGREE on Rule 1).

## Verdict
> [!note] **CONFIRMED.** CONFIRMED by the O-32 real-data proof: 12 cp1252 files moved VALIDITY_DISAGREE→AGREE on Rule 1.

## Related
[[rule-01-ascii]] · [[O-32]] · [[O-01]]
