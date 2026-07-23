---
type: group
title: PMTZ — Pressuremeter Test Results - Zeros
status: drafted
tags: [group]
group_code: PMTZ
parent: PMTG
is_high_volume: false
varies_between_editions: true
key_headings: [LOCA_ID, PMTG_DPTH, PMTG_TESN, PMTZ_PARM]
required_headings: []
ags_editions: [4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=PMTZ]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, PMTG]
sources: []
---
# PMTZ — Pressuremeter Test Results - Zeros

## Purpose
> [!quote] The **PMTZ** group — Pressuremeter Test Results - Zeros. It is a **child of [[PMTG]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  PMTG ||--o{ PMTZ : has
  PMTZ {
    KEY LOCA_ID
    KEY PMTG_DPTH
    KEY PMTG_TESN
    KEY PMTZ_PARM
  }
```

- Parent: [[PMTG]]
- Children: _none_
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=PMTZ]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

13 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `PMTG_DPTH` | **KEY** | `2DP` | Depth of test |
| `PMTG_TESN` | **KEY** | `X` | Test reference |
| `PMTZ_PARM` | **KEY** | `PA` | Measured Parameter |
| `PMTZ_MRS` | OTHER | `X` | Measuring ranges of the sensors, min to max unit |
| `PMTZ_ZC` | OTHER | `U` | Zero from calibration |
| `PMTZ_ZB` | OTHER | `U` | Zero before at surface |
| `PMTZ_ZH` | OTHER | `U` | Zero in hole before test at test depth |
| `PMTZ_ZA` | OTHER | `U` | Zero after at surface |
| `PMTZ_ZD` | OTHER | `U` | Zero drift |
| `PMTZ_EGUT` | OTHER | `PU` | Unit for PMTZ_ZD |
| `PMTZ_REM` | OTHER | `X` | Remarks |
| `FILE_FSET` | OTHER | `X` | Associated file reference (e.g. equipment calibrations) |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `PMTG_DPTH`, `PMTG_TESN`, `PMTZ_PARM`. Children (0): _none_. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[PMTG]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[PMTG]]
