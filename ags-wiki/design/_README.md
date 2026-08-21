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
DuckDB extension, the monorepo structure). One page kind:

- **decision** — a design choice: context / options / decision / why
  / consequences (links the gaps it answers). Covers both shipped-AGS4
  architecture and AGS5 design.

**`requirement` and `experiment` were the other two, retired in #500.** Both
were AGS5-strand classes and neither ever had a live page here, so the strand's
gap → requirement → decision → experiment spine existed only as a description
of itself. Worth recording rather than deleting quietly: this README claimed
*"Lint flags any requirement without `from_gap`"* and *"Lint enforces this"*,
and `lint.py` has never contained the string `from_gap`. A stated safety that
does not exist is worse than none — a reader deciding whether the traceability
could rot got a confident yes. `tests/test_self_named_gates.py` catches this
shape only when the gate is named as a `tests/test_*.py` file; "Lint enforces
this" names one in prose and reaches no gate at all.

## Decisions

```dataview
TABLE status, decided, from_gap FROM "design"
WHERE type = "decision" SORT decided DESC
```

## Related
[[start-here]] · [[insights/_README|insights register]] · ags4-vs-ags5
