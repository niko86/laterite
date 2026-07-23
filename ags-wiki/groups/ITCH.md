---
type: group
title: ITCH — In Situ Thermal Conductivity
status: drafted
tags: [group]
group_code: ITCH
parent: LOCA
is_high_volume: false
varies_between_editions: true
key_headings: [LOCA_ID, ITCH_DPTH, ITCH_TESN]
required_headings: []
ags_editions: [4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=ITCH]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, LOCA]
sources: []
---
# ITCH — In Situ Thermal Conductivity

## Purpose
> [!quote] The **ITCH** group — In Situ Thermal Conductivity. It is a **child of [[LOCA]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  LOCA ||--o{ ITCH : has
  ITCH {
    KEY LOCA_ID
    KEY ITCH_DPTH
    KEY ITCH_TESN
  }
```

- Parent: [[LOCA]]
- Children: _none_
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=ITCH]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

16 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `ITCH_DPTH` | **KEY** | `2DP` | Depth of thermal conductivity test |
| `ITCH_TESN` | **KEY** | `X` | Test reference |
| `ITCH_DATE` | OTHER | `DT` | Test date |
| `ITCH_TCON` | OTHER | `2DP` | Thermal Conductivity |
| `ITCH_TRES` | OTHER | `2DP` | Thermal Resistivity |
| `ITCH_TEMP` | OTHER | `0DP` | Ambient temperature at which test is performed |
| `ITCH_REM` | OTHER | `X` | Details of probe used and method description |
| `ITCH_ENV` | OTHER | `X` | Details of weather and environmental conditions during test |
| `ITCH_METH` | OTHER | `X` | Test method |
| `ITCH_OPER` | OTHER | `X` | Name of test operator |
| `ITCH_CONT` | OTHER | `X` | Name of testing organisation |
| `ITCH_CRED` | OTHER | `X` | Accrediting body and reference number (when appropriate) |
| `TEST_STAT` | OTHER | `X` | Test status |
| `GEOL_STAT` | OTHER | `X` | Stratum reference shown on trial pit or traverse sketch |
| `FILE_FSET` | OTHER | `X` | Associated file reference (e.g. test result sheets) |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `ITCH_DPTH`, `ITCH_TESN`. Children (0): _none_. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[LOCA]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[LOCA]]
