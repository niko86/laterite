---
type: group
title: FGHI — Field Geohydraulic Testing - Instrumentation Details
status: drafted
tags: [group]
group_code: FGHI
parent: FGHG
is_high_volume: false
varies_between_editions: true
key_headings: [LOCA_ID, FGHG_TOP, FGHG_BASE, FGHG_TESN, FGHI_INST]
required_headings: []
ags_editions: [4.1, 4.1.1, 4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=FGHI]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, FGHG, FGHT]
sources: []
---
# FGHI — Field Geohydraulic Testing - Instrumentation Details

## Purpose
> [!quote] The **FGHI** group — Field Geohydraulic Testing - Instrumentation Details. It is a **child of [[FGHG]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  FGHG ||--o{ FGHI : has
  FGHI ||--o{ FGHT : has
  FGHI {
    KEY LOCA_ID
    KEY FGHG_TOP
    KEY FGHG_BASE
    KEY FGHG_TESN
    KEY FGHI_INST
  }
```

- Parent: [[FGHG]]
- Children: [[FGHT]]
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=FGHI]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

10 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `FGHG_TOP` | **KEY** | `2DP` | Depth to top of test zone |
| `FGHG_BASE` | **KEY** | `2DP` | Depth to base of test zone |
| `FGHG_TESN` | **KEY** | `X` | Test reference |
| `FGHI_INST` | **KEY** | `X` | Instrument reference / serial number |
| `FGHI_TYPE` | OTHER | `X` | Instrument measured parameters |
| `FGHI_DETL` | OTHER | `X` | Details of instrument |
| `FGHI_LOCT` | OTHER | `X` | Instrument position |
| `FGHI_REM` | OTHER | `X` | Test remarks |
| `FILE_FSET` | OTHER | `X` | Associated file reference (e.g. equipment calibrations) |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `FGHG_TOP`, `FGHG_BASE`, `FGHG_TESN`, `FGHI_INST`. Children (1): [[FGHT]]. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[FGHG]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[FGHG]] · [[FGHT]]
