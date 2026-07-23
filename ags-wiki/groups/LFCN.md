---
type: group
title: LFCN — Laboratory Fall Cone Test
status: drafted
tags: [group]
group_code: LFCN
parent: SAMP
is_high_volume: false
varies_between_editions: true
key_headings: [LOCA_ID, SAMP_TOP, SAMP_REF, SAMP_TYPE, SAMP_ID, SPEC_REF, SPEC_DPTH]
required_headings: []
ags_editions: [4.1, 4.1.1, 4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=LFCN]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, SAMP]
sources: []
---
# LFCN — Laboratory Fall Cone Test

## Purpose
> [!quote] The **LFCN** group — Laboratory Fall Cone Test. It is a **child of [[SAMP]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  SAMP ||--o{ LFCN : has
  LFCN {
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
- Children: _none_
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=LFCN]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

29 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

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
| `LFCN_DEV` | OTHER | `X` | Deviations from the procedure |
| `LFCN_CMAS` | OTHER | `0DP` | Mass of cone used |
| `LFCN_CANG` | OTHER | `0DP` | Angle of cone tip |
| `LFCN_PENA` | OTHER | `2DP` | Average cone penetration |
| `LFCN_PEN1` | OTHER | `2DP` | Individual penetration point 1 if values differ by more than 0.5mm from the average, for undisturbed tests. |
| `LFCN_PEN2` | OTHER | `2DP` | Individual penetration point 2 if values differ by more than 0.5mm from the average, for undisturbed tests. |
| `LFCN_PEN3` | OTHER | `2DP` | Individual penetration point 3 if values differ by more than 0.5mm from the average, for undisturbed tests. |
| `LFCN_PEN4` | OTHER | `2DP` | Individual penetration point 4 if values differ by more than 0.5mm from the average, for undisturbed tests. |
| `LFCN_CONF` | OTHER | `YN` | Non-conforming test (due to penetration range) |
| `LFCN_FCPK` | OTHER | `2SF` | Estimated undrained fall cone shear strength |
| `LFCN_FCRM` | OTHER | `2SF` | Estimated undrained fall cone shear strength, remoulded |
| `LFCN_WC` | OTHER | `X` | Water content of specimen |
| `LFCN_WCST` | OTHER | `X` | Water content determined on specimen trimmings or other if applicable. |
| `LFCN_REM` | OTHER | `X` | Test remarks |
| `LFCN_METH` | OTHER | `X` | Test method |
| `LFCN_LAB` | OTHER | `X` | Name of testing laboratory/organization |
| `LFCN_CRED` | OTHER | `X` | Accrediting body and reference number (when appropriate) |
| `TEST_STAT` | OTHER | `X` | Test status |
| `FILE_FSET` | OTHER | `X` | Associated file reference (e.g. equipment calibrations) |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `SAMP_TOP`, `SAMP_REF`, `SAMP_TYPE`, `SAMP_ID`, `SPEC_REF`, `SPEC_DPTH`. Children (0): _none_. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[SAMP]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[SAMP]]
