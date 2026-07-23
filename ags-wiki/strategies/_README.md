---
type: concept
title: "Test Strategies — register"
status: drafted
tags: [moc, register]
related: [start-here, parity-model, evolutionary-dogfooding]
sources: []
repo_refs: {}
---

# Test Strategies — register

Concrete Rust↔python-ags4 cross-check probes. Each page is one
hypothesis about where `lat` and `tools/py_ags4_check_json.py`
could disagree on a rule, plus the crafted probe to settle it. Probe
fixtures live in `.bootstrap/probes/` — **never**
`laterite-ags4-validator/tests/fixtures/` (corpus-qa e2e asserts that dir
hard-error-free). `proposed` → `probed` → `confirmed` (with recorded
`evidence`).

A `strat-forge-*` page is the human-readable twin of an executable
[[laterite-ags4-forge]] `strategy.toml`: the agent authors both from this wiki,
the CLI evolves/minimizes, and a confirmed divergence flips the page
`proposed`→`probed`→`confirmed` with the minimized probe as
`evidence` ([[evolutionary-dogfooding]]).

## All strategies

```dataview
TABLE targets, status, divergence_hypothesis FROM "strategies"
WHERE type = "strategy" SORT status, file.name
```

## Confirmed divergences (→ which insight they prove)

```dataview
TABLE targets, evidence FROM "strategies" WHERE status = "confirmed"
```

## Related
[[start-here]] · [[parity-model]] · [[evolutionary-dogfooding]] · [[laterite-ags4-forge]] · [[insights/_README|insights register]]
