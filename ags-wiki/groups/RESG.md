---
type: group
title: RESG — Resonant Column Test - General
status: drafted
tags: [group]
group_code: RESG
parent: SAMP
is_high_volume: false
varies_between_editions: true
key_headings: [LOCA_ID, SAMP_TOP, SAMP_REF, SAMP_TYPE, SAMP_ID, SPEC_REF, SPEC_DPTH]
required_headings: []
ags_editions: [4.1, 4.1.1, 4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=RESG]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, SAMP, RESC, RESD, RESS]
sources: []
---
# RESG — Resonant Column Test - General

## Purpose
> [!quote] The **RESG** group — Resonant Column Test - General. It is a **child of [[SAMP]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  SAMP ||--o{ RESG : has
  RESG ||--o{ RESC : has
  RESG ||--o{ RESD : has
  RESG ||--o{ RESS : has
  RESG {
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
- Children: [[RESC]] [[RESD]] [[RESS]]
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=RESG]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

34 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

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
| `SPEC_BASE` | OTHER | `2DP` | Depth to base of specimen |
| `RESG_COND` | OTHER | `PA` | Sample condition |
| `RESG_CONS` | OTHER | `X` | Specific condition statements |
| `RESG_DRAG` | OTHER | `X` | Type of Drainage |
| `RESG_ORNT` | OTHER | `PA` | Orientation of Specimen |
| `RESG_SDIA` | OTHER | `2DP` | Initial specimen diameter |
| `RESG_HIGT` | OTHER | `2DP` | Initial specimen Height |
| `RESG_MCI` | OTHER | `X` | Initial Water/moisture Content |
| `RESG_MCF` | OTHER | `X` | Final Water/moisture Content |
| `RESG_BDEN` | OTHER | `2DP` | Initial Bulk Density |
| `RESG_DDEN` | OTHER | `2DP` | Initial Dry Density |
| `RESG_MIDD` | OTHER | `2DP` | Minimum dry density for sand |
| `RESG_MADD` | OTHER | `2DP` | Maximum dry density for sand |
| `RESG_IRDI` | OTHER | `1DP` | Initial relative density index |
| `RESG_IVR` | OTHER | `3DP` | Initial void ratio |
| `RESG_ISAT` | OTHER | `0DP` | Initial degree of saturation |
| `RESG_PDEN` | OTHER | `XN` | Particle density with prefix # if value assumed |
| `RESG_DAMP` | OTHER | `X` | Damping measurement method |
| `RESG_DEV` | OTHER | `X` | Deviation from the specified procedure |
| `RESG_REM` | OTHER | `X` | Remarks |
| `RESG_METH` | OTHER | `X` | Test method |
| `RESG_LAB` | OTHER | `X` | Name of testing laboratory/organization |
| `RESG_CRED` | OTHER | `X` | Accrediting body and reference number ({when appropriate) |
| `TEST_STAT` | OTHER | `X` | Test Status |
| `FILE_FSET` | OTHER | `X` | Associated file reference |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `SAMP_TOP`, `SAMP_REF`, `SAMP_TYPE`, `SAMP_ID`, `SPEC_REF`, `SPEC_DPTH`. Children (3): [[RESC]] [[RESD]] [[RESS]]. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[SAMP]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[SAMP]] · [[RESC]] · [[RESD]] · [[RESS]]
