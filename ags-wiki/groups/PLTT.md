---
type: group
title: PLTT — Plate Loading Tests - Data
status: drafted
tags: [group]
group_code: PLTT
parent: PLTG
is_high_volume: false
varies_between_editions: false
key_headings: [LOCA_ID, PLTG_DPTH, PLTG_TESN, PLTG_CYC, PLTT_STG, PLTT_TIME]
required_headings: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=PLTT]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, PLTG]
sources: []
---
# PLTT — Plate Loading Tests - Data

## Purpose
> [!quote] The **PLTT** group — Plate Loading Tests - Data. It is a **child of [[PLTG]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  PLTG ||--o{ PLTT : has
  PLTT {
    KEY LOCA_ID
    KEY PLTG_DPTH
    KEY PLTG_TESN
    KEY PLTG_CYC
    KEY PLTT_STG
    KEY PLTT_TIME
  }
```

- Parent: [[PLTG]]
- Children: _none_
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=PLTT]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

13 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `PLTG_DPTH` | **KEY** | `2DP` | Test depth |
| `PLTG_TESN` | **KEY** | `X` | Test reference |
| `PLTG_CYC` | **KEY** | `X` | Load cycle |
| `PLTT_STG` | **KEY** | `X` | Load stage |
| `PLTT_TIME` | **KEY** | `1DP` | Stage elapsed time |
| `PLTT_LOAD` | OTHER | `1DP` | Applied load |
| `PLTT_SET1` | OTHER | `2DP` | Settlement Gauge 1 |
| `PLTT_SET2` | OTHER | `2DP` | Settlement Gauge 2 |
| `PLTT_SET3` | OTHER | `2DP` | Settlement Gauge 3 |
| `PLTT_SET4` | OTHER | `2DP` | Settlement Gauge 4 |
| `PLTT_REM` | OTHER | `X` | Comments on reading |
| `FILE_FSET` | OTHER | `X` | Associated file reference (e.g. test result sheets) |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `PLTG_DPTH`, `PLTG_TESN`, `PLTG_CYC`, `PLTT_STG`, `PLTT_TIME`. Children (0): _none_. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[PLTG]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[PLTG]]
