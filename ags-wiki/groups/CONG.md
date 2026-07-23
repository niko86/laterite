---
type: group
title: CONG — Consolidation Tests - General
status: drafted
tags: [group]
group_code: CONG
parent: SAMP
is_high_volume: false
varies_between_editions: false
key_headings: [LOCA_ID, SAMP_TOP, SAMP_REF, SAMP_TYPE, SAMP_ID, SPEC_REF, SPEC_DPTH]
required_headings: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=CONG]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, SAMP, CONS]
sources: []
---
# CONG — Consolidation Tests - General

## Purpose
> [!quote] The **CONG** group — Consolidation Tests - General. It is a **child of [[SAMP]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  SAMP ||--o{ CONG : has
  CONG ||--o{ CONS : has
  CONG {
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
- Children: [[CONS]]
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=CONG]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

32 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

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
| `CONG_TYPE` | OTHER | `PA` | Type of consolidation test |
| `CONG_COND` | OTHER | `PA` | Sample condition |
| `CONG_SDIA` | OTHER | `2DP` | Test specimen diameter |
| `CONG_HIGT` | OTHER | `2DP` | Test specimen height |
| `CONG_MCI` | OTHER | `X` | Initial water/moisture content |
| `CONG_MCF` | OTHER | `X` | Final water/moisture content |
| `CONG_BDEN` | OTHER | `2DP` | Initial bulk density |
| `CONG_DDEN` | OTHER | `2DP` | Initial dry density |
| `CONG_PDEN` | OTHER | `XN` | Particle density with prefix # if value assumed |
| `CONG_SATR` | OTHER | `0DP` | Initial degree of saturation |
| `CONG_SPRS` | OTHER | `2SF` | Swelling pressure |
| `CONG_SATH` | OTHER | `1DP` | Height change of specimen on saturation, or flooding as percentage of original height (BS1377 Settlement on saturation test) |
| `CONG_IVR` | OTHER | `3DP` | Initial voids ratio |
| `CONG_REM` | OTHER | `X` | Remarks including commentary on effect of specimen disturbance on test result |
| `CONG_METH` | OTHER | `X` | Test method |
| `CONG_LAB` | OTHER | `X` | Name of testing laboratory/organization |
| `CONG_CRED` | OTHER | `X` | Accrediting body and reference number (when appropriate) |
| `TEST_STAT` | OTHER | `X` | Test status |
| `FILE_FSET` | OTHER | `X` | Associated file reference (e.g. equipment calibrations) |
| `SPEC_BASE` | OTHER | `2DP` | Depth to base of specimen |
| `CONG_DEV` | OTHER | `X` | Deviations from the test method |
| `CONG_MCIS` | OTHER | `X` | Initial water/moisture content source |
| `CONG_CORR` | OTHER | `YN` | Results corrected for equipment deformation |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `SAMP_TOP`, `SAMP_REF`, `SAMP_TYPE`, `SAMP_ID`, `SPEC_REF`, `SPEC_DPTH`. Children (1): [[CONS]]. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[SAMP]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[SAMP]] · [[CONS]]
