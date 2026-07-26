---
type: concept
title: "Design — register"
status: drafted
tags: [moc, register, design]
related: [start-here]
sources: []
repo_refs: {}
---

# Design — register

Architecture & design decisions, experiments, and requirements for the whole
toolchain — the shipped **AGS4** strand and the dormant **AGS5** reimagining
alike (renamed from `ags5-design/`, which mis-implied AGS5-only when the folder
also holds AGS4 architecture calls — crate layout, the PyO3/wasm boundaries, the
DuckDB extension, the monorepo structure). Three page kinds:

- **requirement** — what a design must do. AGS5 requirements **MUST trace
  to an `insights/` AGS4 gap** (`from_gap`); Lint enforces this — the dormant
  AGS5 strand is driven by demonstrated AGS4 weaknesses, not speculation.
- **decision** — a design choice: context / options / decision / why
  / consequences (links the requirements + gaps it answers). Covers both
  shipped-AGS4 architecture and AGS5 design.
- **experiment** — what was tried, the outcome (worked/partial/
  failed), and *why*. Seeded from `docs/history/*` (past phase
  write-ups) and the current architecture.

## Requirements (each must trace to a gap)

```dataview
TABLE priority, status, from_gap FROM "design"
WHERE type = "requirement" SORT priority, status
```

## Decisions

```dataview
TABLE status, decided, from_gap FROM "design"
WHERE type = "decision" SORT decided DESC
```

## Experiments — what we tried & why it did/didn't work

```dataview
TABLE outcome, "why" , evidence FROM "design"
WHERE type = "experiment" SORT outcome
```

## Traceability — AGS4 gap → AGS5 requirement → decision/experiment

The spine of the strand: a confirmed [[insights/_README|insight]]
becomes a `requirement`, answered by a `decision`, tested by an
`experiment`. Lint flags any requirement without `from_gap` and any
decision/experiment not reachable from a requirement.

## Related
[[start-here]] · [[insights/_README|insights register]] · ags4-vs-ags5
