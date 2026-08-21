---
type: concept
title: "Insights & Gaps — register"
status: drafted
tags: [moc, register]
related: [start-here, parity-model, upstream-reporting]
sources: []
repo_refs: {}
---

# Insights & Gaps — register

The campaign's gap register (AGS-WIKI.md §12.5). Each page is one gap:
spec ambiguity/contradiction, a cross-edition regression, a spec↔Rust
or Rust↔python divergence, or a 4.2 rule weakness. A gap is
`hypothesis` until **empirically probed** through both validators
(then `confirmed`); `proposes_observation: true` carries a drafted
`O-N` which the agent writes into `OBSERVATIONS.md` directly
(canonical authority — deliberate, house-style edits), then sets the
page `status: ratified`.

## By kind & severity

```dataview
TABLE gap_kind, severity, status, editions_affected FROM "insights"
WHERE type = "insight" SORT severity DESC, status
```

## Proposed OBSERVATIONS entries (await ratification)

```dataview
TABLE severity, rules FROM "insights"
WHERE proposes_observation = true AND status != "ratified"
```

## Confirmed gaps feeding the test strategy

```dataview
TABLE feeds_strategy FROM "insights"
WHERE status = "confirmed" OR status = "ratified"
```

## Related
[[start-here]] · [[parity-model]] · [[upstream-reporting]] · [[strategies/_README|strategies register]] · [[design/_README|design register]]
