---
type: group
title: ESCG — Effective Stress Consolidation Tests - General
status: drafted
tags: [group]
group_code: ESCG
parent: SAMP
is_high_volume: false
varies_between_editions: false
key_headings: [LOCA_ID, SAMP_TOP, SAMP_REF, SAMP_TYPE, SAMP_ID, SPEC_REF, SPEC_DPTH]
required_headings: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=ESCG]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, SAMP, ESCT]
sources: []
---
# ESCG — Effective Stress Consolidation Tests - General

## Purpose
> [!quote] The **ESCG** group — Effective Stress Consolidation Tests - General. It is a **child of [[SAMP]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  SAMP ||--o{ ESCG : has
  ESCG ||--o{ ESCT : has
  ESCG {
    KEY LOCA_ID
    KEY SAMP_TOP
    KEY SAMP_REF
    KEY SAMP_TYPE
    KEY SAMP_ID
    KEY SPEC_REF
    KEY SPEC_DPTH
  }
```

- Parent: [[SAMP]]
- Children: [[ESCT]]
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=ESCG]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

48 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `SAMP_TOP` | **KEY** | `2DP` | Depth to top of sample |
| `SAMP_REF` | **KEY** | `X` | Sample reference |
| `SAMP_TYPE` | **KEY** | `PA` | Sample type |
| `SAMP_ID` | **KEY** | `ID` | Sample unique identifier |
| `SPEC_REF` | **KEY** | `X` | Specimen reference |
| `SPEC_DPTH` | **KEY** | `2DP` | Depth to top of test specimen |
| `SPEC_DESC` | OTHER | `X` | Specimen description |
| `SPEC_PREP` | OTHER | `X` | Details of specimen preparation including time between preparation and testing |
| `ESCG_TYPE` | OTHER | `PA` | Test type |
| `ESCG_CELL` | OTHER | `X` | Type of equipment used |
| `ESCG_COND` | OTHER | `PA` | Sample condition |
| `ESCG_SDIA` | OTHER | `2DP` | Test specimen diameter |
| `ESCG_HIGT` | OTHER | `2DP` | Test specimen height |
| `ESCG_MCI` | OTHER | `X` | Initial water/moisture content |
| `ESCG_MCF` | OTHER | `X` | Final water/moisture content |
| `ESCG_BDEN` | OTHER | `2DP` | Initial bulk density |
| `ESCG_BDEF` | OTHER | `2DP` | Final bulk density |
| `ESCG_DDEN` | OTHER | `2DP` | Initial dry density |
| `ESCG_PDEN` | OTHER | `XN` | Particle density with prefix # if value assumed |
| `ESCG_IVR` | OTHER | `3DP` | Initial voids ratio |
| `ESCG_SATR` | OTHER | `0DP` | Initial degree of saturation |
| `ESCG_LOAD` | OTHER | `X` | Type of loading ( strain ) |
| `ESCG_DRAG` | OTHER | `X` | Type of drainage |
| `ESCG_PPM` | OTHER | `X` | Pore pressure measurement location |
| `ESCG_SPRS` | OTHER | `2SF` | Swelling pressure, if measured |
| `ESCG_SATM` | OTHER | `X` | Method of saturation |
| `ESCG_SINC` | OTHER | `0DP` | Saturation increments |
| `ESCG_SDIF` | OTHER | `0DP` | Differential pressure during saturation |
| `ESCG_CELF` | OTHER | `0DP` | Cell or diaphragm pressure at end of saturation |
| `ESCG_BACF` | OTHER | `0DP` | Back pressure at end of saturation |
| `ESCG_BVAL` | OTHER | `2DP` | B value at end of saturation |
| `ESCG_SVOL` | OTHER | `1DP` | Volume of water taken in during saturation |
| `ESCG_REM` | OTHER | `X` | Remarks including commentary on effect of specimen disturbance on test result |
| `ESCG_METH` | OTHER | `X` | Test method |
| `ESCG_LAB` | OTHER | `X` | Name of testing laboratory/organization |
| `ESCG_CRED` | OTHER | `X` | Accrediting body and reference number (when appropriate) |
| `TEST_STAT` | OTHER | `X` | Test status |
| `FILE_FSET` | OTHER | `X` | Associated file reference (e.g. equipment calibrations) |
| `SPEC_BASE` | OTHER | `2DP` | Depth to base of specimen |
| `ESCG_DEV` | OTHER | `X` | Deviations from the test method |
| `ESCG_ISVR` | OTHER | `3DP` | Voids ratio at in situ vertical stress |
| `ESCG_ISVS` | OTHER | `0DP` | In situ vertical effective stress |
| `ESCG_ISST` | OTHER | `2DP` | Axial strain at in situ vertical effective stress |
| `ESCG_PCP` | OTHER | `0DP` | Preconsolidation stress (yield stress) |
| `ESCG_YSR` | OTHER | `1DP` | Yield stress ratio (based on Casagrande Method) |
| `ESCG_CC` | OTHER | `3DP` | Compression index over stress increment |
| `ESCG_CS` | OTHER | `3DP` | Swelling index over stress increment |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `SAMP_TOP`, `SAMP_REF`, `SAMP_TYPE`, `SAMP_ID`, `SPEC_REF`, `SPEC_DPTH`. Children (1): [[ESCT]]. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[SAMP]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[SAMP]] · [[ESCT]]
