---
type: group
title: BKFL — Exploratory Hole Backfill Details
status: drafted
tags: [group]
group_code: BKFL
parent: LOCA
is_high_volume: false
varies_between_editions: false
key_headings: [LOCA_ID, BKFL_TOP]
required_headings: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=BKFL]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, LOCA]
sources: []
---
# BKFL — Exploratory Hole Backfill Details

## Purpose
> [!quote] The **BKFL** group — Exploratory Hole Backfill Details. It is a **child of [[LOCA]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  LOCA ||--o{ BKFL : has
  BKFL {
    KEY LOCA_ID
    KEY BKFL_TOP
  }
```

- Parent: [[LOCA]]
- Children: _none_
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=BKFL]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

8 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `BKFL_TOP` | **KEY** | `2DP` | Depth to top of section |
| `BKFL_BASE` | OTHER | `2DP` | Depth to base of section |
| `BKFL_DESC` | OTHER | `X` | Backfill description |
| `BKFL_LEG` | OTHER | `PA` | Backfill legend abbreviation |
| `BKFL_DATE` | OTHER | `DT` | Date of completion of backfill |
| `BKFL_REM` | OTHER | `X` | Backfill remarks including how it was placed |
| `FILE_FSET` | OTHER | `X` | Associated file reference (e.g. drilling journals) |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `BKFL_TOP`. Children (0): _none_. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[LOCA]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[LOCA]]
