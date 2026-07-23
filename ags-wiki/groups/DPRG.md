---
type: group
title: DPRG — Dynamic Probe Tests - General
status: drafted
tags: [group]
group_code: DPRG
parent: LOCA
is_high_volume: false
varies_between_editions: false
key_headings: [LOCA_ID, DPRG_TESN]
required_headings: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=DPRG]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, LOCA, DPRB]
sources: []
---
# DPRG — Dynamic Probe Tests - General

## Purpose
> [!quote] The **DPRG** group — Dynamic Probe Tests - General. It is a **child of [[LOCA]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  LOCA ||--o{ DPRG : has
  DPRG ||--o{ DPRB : has
  DPRG {
    KEY LOCA_ID
    KEY DPRG_TESN
  }
```

- Parent: [[LOCA]]
- Children: [[DPRB]]
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=DPRG]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

26 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `DPRG_TESN` | **KEY** | `X` | Test reference |
| `DPRG_DATE` | OTHER | `DT` | Test date |
| `DPRG_TYPE` | OTHER | `PA` | Dynamic probe type |
| `DPRG_METH` | OTHER | `X` | Test method |
| `DPRG_MASS` | OTHER | `1DP` | Hammer mass |
| `DPRG_DROP` | OTHER | `0DP` | Standard drop |
| `DPRG_CONE` | OTHER | `1DP` | Cone base diameter |
| `DPRG_ROD` | OTHER | `0DP` | Rod diameter |
| `DPRG_TANV` | OTHER | `X` | Type of anvil |
| `DPRG_DAMP` | OTHER | `X` | Type of anvil damper |
| `DPRG_TIP` | OTHER | `2DP` | Depth of cone if left in ground |
| `DPRG_REM` | OTHER | `X` | General remarks |
| `DPRG_ANG` | OTHER | `0DP` | Cone angle |
| `DPRG_RMSS` | OTHER | `1DP` | Rod mass |
| `DPRG_PARF` | OTHER | `X` | Precautions against rod friction |
| `DPRG_PDIU` | OTHER | `X` | Pre-drilling if used |
| `DPRG_BCF` | OTHER | `X` | Blow count frequency |
| `DPRG_GW` | OTHER | `2DP` | Groundwater level |
| `DPRG_REET` | OTHER | `X` | Reasons for early end of test |
| `DPRG_ENV` | OTHER | `X` | Details of weather and environmental conditions during test |
| `DPRG_CONT` | OTHER | `X` | Name of testing organization |
| `DPRG_CRED` | OTHER | `X` | Accrediting body and reference number (when appropriate) |
| `TEST_STAT` | OTHER | `X` | Test status |
| `FILE_FSET` | OTHER | `X` | Associated file reference (e.g. equipment calibrations) |
| `DPRG_OPER` | OTHER | `X` | Name of test operator |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `DPRG_TESN`. Children (1): [[DPRB]]. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[LOCA]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[LOCA]] · [[DPRB]]
