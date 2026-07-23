---
type: group
title: CORE — Coring Information
status: drafted
tags: [group]
group_code: CORE
parent: LOCA
is_high_volume: false
varies_between_editions: false
key_headings: [LOCA_ID, CORE_TOP, CORE_BASE]
required_headings: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=CORE]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, LOCA]
sources: []
---
# CORE — Coring Information

## Purpose
> [!quote] The **CORE** group — Coring Information. It is a **child of [[LOCA]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  LOCA ||--o{ CORE : has
  CORE {
    KEY LOCA_ID
    KEY CORE_TOP
    KEY CORE_BASE
  }
```

- Parent: [[LOCA]]
- Children: _none_
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=CORE]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

10 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `CORE_TOP` | **KEY** | `2DP` | Depth to top of core run |
| `CORE_BASE` | **KEY** | `2DP` | Depth to base of core run |
| `CORE_PREC` | OTHER | `0DP` | Percentage of core recovered in core run (TCR) |
| `CORE_SREC` | OTHER | `0DP` | Percentage of solid core recovered in core run (SCR) |
| `CORE_RQD` | OTHER | `0DP` | Rock Quality Designation for core run (RQD) |
| `CORE_DIAM` | OTHER | `0DP` | Core diameter |
| `CORE_DURN` | OTHER | `T` | Time taken to drill core run |
| `CORE_REM` | OTHER | `X` | Remarks |
| `FILE_FSET` | OTHER | `X` | Associated file reference (e.g. photographs of rock cores) |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `CORE_TOP`, `CORE_BASE`. Children (0): _none_. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[LOCA]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[LOCA]]
