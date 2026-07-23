---
type: group
title: PMTL — Pressuremeter Test Results - Individual Loops
status: drafted
tags: [group]
group_code: PMTL
parent: PMTG
is_high_volume: false
varies_between_editions: false
key_headings: [LOCA_ID, PMTG_DPTH, PMTG_TESN, PMTL_LNO, PMTD_SEQ]
required_headings: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=PMTL]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, PMTG]
sources: []
---
# PMTL — Pressuremeter Test Results - Individual Loops

## Purpose
> [!quote] The **PMTL** group — Pressuremeter Test Results - Individual Loops. It is a **child of [[PMTG]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  PMTG ||--o{ PMTL : has
  PMTL {
    KEY LOCA_ID
    KEY PMTG_DPTH
    KEY PMTG_TESN
    KEY PMTL_LNO
    KEY PMTD_SEQ
  }
```

- Parent: [[PMTG]]
- Children: _none_
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=PMTL]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

18 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `PMTG_DPTH` | **KEY** | `2DP` | Depth of test |
| `PMTG_TESN` | **KEY** | `X` | Test reference |
| `PMTL_LNO` | **KEY** | `0DP` | Unload/reload loop number |
| `PMTL_GAA` | OTHER | `3SF` | Unload/reload shear modulus, average |
| `PMTL_SINC` | OTHER | `2DP` | Mean strain |
| `PMTL_PINC` | OTHER | `0DP` | Mean pressure |
| `PMTL_STRA` | OTHER | `3DP` | Strain range or amplitude |
| `PMTL_PRSA` | OTHER | `0DP` | Pressure range or amplitude |
| `PMTL_NLSA` | OTHER | `3DP` | Shear stress coefficient (from Bolton and Whittle, 1999) |
| `PMTL_NLSB` | OTHER | `3DP` | Linearity exponent (from Bolton and Whittle, 1999) |
| `PMTL_REM` | OTHER | `X` | Remarks |
| `FILE_FSET` | OTHER | `X` | Associated file reference (e.g. test result sheets) |
| `PMTL_AXIS` | OTHER | `X` | Arm combination used for analysis |
| `PMTL_HP` | OTHER | `0DP` | Hold pressure |
| `PMTL_HT` | OTHER | `0DP` | Hold duration |
| `PMTL_CR` | OTHER | `4DP` | Creep rate |
| `PMTD_SEQ` | **KEY** | `0DP` | Sequence number |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `PMTG_DPTH`, `PMTG_TESN`, `PMTL_LNO`, `PMTD_SEQ`. Children (0): _none_. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[PMTG]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[PMTG]]
