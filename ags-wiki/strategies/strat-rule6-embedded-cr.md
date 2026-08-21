---
type: strategy
title: "Rule 6 embedded-CR: Rust catches, python no-ops + cascades (O-2)"
status: confirmed
tags: [strategy]
targets: [rule-06-comma-no-embedded-crlf]
divergence_hypothesis: "see body"
probe_files: [ags-wiki/.bootstrap/probes/probe-rule6-embedded-cr.ags]
expected_rust: "Rule 6"
expected_python: "no Rule 6 (cascades to 2a/3/5)"
evidence: "probe-run — ags-wiki/.bootstrap/probes/RESULTS.md"
related: [rule-06-comma-no-embedded-crlf, O-02, parity-model, parity-cascade-unreconcilable]
sources: []
---
# Rule 6 embedded-CR: Rust catches, python no-ops + cascades (O-2)

## Hypothesis
> [!divergence] A bare CR inside a quoted field passes Rule 5/4 but
> violates Rule 6. python `rule_6` is a literal no-op ([[O-02]]) →
> python misses it; Rust implements the independent check → Rust-only
> Rule 6.

## Probe design
- Fixture: `ags-wiki/.bootstrap/probes/probe-rule6-embedded-cr.ags` — a lone CR (`0x0D`, byte-verified, **not** followed by `0x0A`) inside the quoted `PROJ_NAME`; rows otherwise CRLF-terminated; body == clean_minimal.
- Run: `lat validate <probe>` and `uv run python tools/py_ags4_check_json.py <probe>`.

## Expected vs observed

| | Rust `lat` | python-ags4 |
|---|---|---|
| expected | Rule 6 | no Rule 6 |
| observed | **Rule 6 only** (line 5) | **Rule 2a + Rule 3 + Rule 5** — no Rule 6 |

## Verdict
> [!note] **CONFIRMED — and refined.** O-2 ("python no-ops Rule 6") is
> *necessary but not sufficient*: python's universal-newline reader
> **splits the record** on the lone CR, cascading into 2a/3/5. So the
> symmetric diff is `rust_only={Rule 6}` vs
> `py_only={Rule 2a, Rule 3, Rule 5}`. `reconcile()`'s O-2 arm strips
> only Rust-only Rule 6, leaving `py_only` non-empty → an
> embedded-CR file classifies as a **false `RUST_ONLY_RULES` ACTION**,
> *not* `KNOWN_DIVERGENCE`. The strat hypothesis ("python simply
> misses it") was too simple — the real divergence is a cascade.
> Widening `reconcile()` to swallow 2a/3/5 generically would mask
> genuine divergences, so this is **documented, not papered over** —
> see [[parity-cascade-unreconcilable]] for the proposed
> signature-narrow handling (user ratifies).

## Related
[[rule-06-comma-no-embedded-crlf]] · [[O-02]] · [[parity-model]] · [[parity-cascade-unreconcilable]]
