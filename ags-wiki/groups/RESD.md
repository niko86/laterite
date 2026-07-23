---
type: group
title: RESD — Resonant Column Test - Data
status: drafted
tags: [group]
group_code: RESD
parent: RESG
is_high_volume: false
varies_between_editions: true
key_headings: [LOCA_ID, SAMP_TOP, SAMP_REF, SAMP_TYPE, SAMP_ID, SPEC_REF, SPEC_DPTH, RESD_TESN, RESD_MNUM]
required_headings: []
ags_editions: [4.1, 4.1.1, 4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=RESD]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, RESG, RESP]
sources: []
---
# RESD — Resonant Column Test - Data

## Purpose
> [!quote] The **RESD** group — Resonant Column Test - Data. It is a **child of [[RESG]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  RESG ||--o{ RESD : has
  RESD ||--o{ RESP : has
  RESD {
    KEY LOCA_ID
    KEY SAMP_TOP
    KEY SAMP_REF
    KEY SAMP_TYPE
    KEY SAMP_ID
    KEY SPEC_REF
    KEY SPEC_DPTH
    KEY RESD_TESN
    KEY RESD_MNUM
  }
```

- Parent: [[RESG]]
- Children: [[RESP]]
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=RESD]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

30 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `SAMP_TOP` | **KEY** | `2DP` | Depth to top of sample |
| `SAMP_REF` | **KEY** | `X` | Sample reference |
| `SAMP_TYPE` | **KEY** | `PA` | Sample type |
| `SAMP_ID` | **KEY** | `ID` | Sample unique identifier |
| `SPEC_REF` | **KEY** | `X` | Specimen reference |
| `SPEC_DPTH` | **KEY** | `2DP` | Depth to top of test specimen |
| `RESD_TESN` | **KEY** | `X` | Test / Stage Number |
| `RESD_MNUM` | **KEY** | `X` | Measurement Number |
| `RESD_CNDS` | OTHER | `PA` | Test Conditions |
| `RESD_SDIA` | OTHER | `2DP` | Specimen Diameter |
| `RESD_HIGH` | OTHER | `2DP` | Specimen Height |
| `RESD_CELL` | OTHER | `1DP` | Cell Pressure |
| `RESD_BP` | OTHER | `1DP` | Back Pressure |
| `RESD_AXL` | OTHER | `1DP` | Axial Stress |
| `RESD_BPWP` | OTHER | `1DP` | Base Pore Water Pressure |
| `RESD_MPWP` | OTHER | `1DP` | Mid-height Pore Water Pressure |
| `RESD_PPR` | OTHER | `2DP` | Pore Pressure Ratio |
| `RESD_PWPM` | OTHER | `1DP` | Maximum Excess Pore Water Pressure |
| `RESD_EAS` | OTHER | `3DP` | External Axial Strain |
| `RESD_VOL` | OTHER | `3DP` | Volumetric Strain |
| `RESD_DEV` | OTHER | `1DP` | Principal Stress Difference |
| `RESD_MEES` | OTHER | `1DP` | Mean Effective Stress |
| `RESD_MIPS` | OTHER | `1DP` | Minor Principal Stress (sigma 3) |
| `RESD_MAPS` | OTHER | `1DP` | Major Principal Stress (sigma 1) |
| `RESD_AVSS` | OTHER | `3DP` | Average Shear Strain |
| `RESD_SM` | OTHER | `2DP` | Shear Modulus |
| `RESD_DMP` | OTHER | `2DP` | Damping |
| `RESD_REM` | OTHER | `X` | Remarks |
| `FILE_FSET` | OTHER | `X` | Associated file reference |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `SAMP_TOP`, `SAMP_REF`, `SAMP_TYPE`, `SAMP_ID`, `SPEC_REF`, `SPEC_DPTH`, `RESD_TESN`, `RESD_MNUM`. Children (1): [[RESP]]. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[RESG]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[RESG]] · [[RESP]]
