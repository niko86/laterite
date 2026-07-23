---
type: group
title: FGHT — Field Geohydraulic Testing - Data
status: drafted
tags: [group]
group_code: FGHT
parent: FGHI
is_high_volume: false
varies_between_editions: true
key_headings: [LOCA_ID, FGHG_TOP, FGHG_BASE, FGHG_TESN, FGHI_INST, FGHT_TIME, FGHT_TYPE]
required_headings: []
ags_editions: [4.1, 4.1.1, 4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=FGHT]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, FGHI]
sources: []
---
# FGHT — Field Geohydraulic Testing - Data

## Purpose
> [!quote] The **FGHT** group — Field Geohydraulic Testing - Data. It is a **child of [[FGHI]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  FGHI ||--o{ FGHT : has
  FGHT {
    KEY LOCA_ID
    KEY FGHG_TOP
    KEY FGHG_BASE
    KEY FGHG_TESN
    KEY FGHI_INST
    KEY FGHT_TIME
    KEY FGHT_TYPE
  }
```

- Parent: [[FGHI]]
- Children: _none_
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=FGHT]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

13 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `FGHG_TOP` | **KEY** | `2DP` | Depth to top of test zone |
| `FGHG_BASE` | **KEY** | `2DP` | Depth to base of test zone |
| `FGHG_TESN` | **KEY** | `X` | Test reference |
| `FGHI_INST` | **KEY** | `X` | Instrument reference / serial number |
| `FGHT_TIME` | **KEY** | `DT` | Test date / clock time of reading |
| `FGHT_TYPE` | **KEY** | `PA` | Test record type |
| `FGHS_STG` | OTHER | `0DP` | Stage number of multistage test |
| `FGHT_DURN` | OTHER | `T` | Elapsed time of reading during test or test stage |
| `FGHT_RDNG` | OTHER | `U` | Test record (reading) |
| `FGHT_UNIT` | OTHER | `PU` | Reading units |
| `FGHT_REM` | OTHER | `X` | Test record remark |
| `FILE_FSET` | OTHER | `X` | Associated file reference (e.g. equipment calibrations) |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `FGHG_TOP`, `FGHG_BASE`, `FGHG_TESN`, `FGHI_INST`, `FGHT_TIME`, `FGHT_TYPE`. Children (0): _none_. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[FGHI]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[FGHI]]
