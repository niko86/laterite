---
type: group
title: IRDX — In Situ Redox Tests
status: drafted
tags: [group]
group_code: IRDX
parent: LOCA
is_high_volume: false
varies_between_editions: false
key_headings: [LOCA_ID, IRDX_DPTH, IRDX_TESN]
required_headings: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=IRDX]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, LOCA]
sources: []
---
# IRDX — In Situ Redox Tests

## Purpose
> [!quote] The **IRDX** group — In Situ Redox Tests. It is a **child of [[LOCA]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  LOCA ||--o{ IRDX : has
  IRDX {
    KEY LOCA_ID
    KEY IRDX_DPTH
    KEY IRDX_TESN
  }
```

- Parent: [[LOCA]]
- Children: _none_
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=IRDX]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

16 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `IRDX_DPTH` | **KEY** | `2DP` | Depth of redox test |
| `IRDX_TESN` | **KEY** | `X` | Test reference |
| `IRDX_DATE` | OTHER | `DT` | Test date |
| `IRDX_PH` | OTHER | `1DP` | pH |
| `IRDX_MPOT` | OTHER | `0DP` | Mean value of the potential of the two platinum probes |
| `IRDX_IRDX` | OTHER | `0DP` | Redox potential |
| `IRDX_REM` | OTHER | `X` | Details of redox test and probe type |
| `IRDX_ENV` | OTHER | `X` | Details of weather and environmental conditions during test |
| `IRDX_METH` | OTHER | `X` | Test method |
| `IRDX_CONT` | OTHER | `X` | Name of testing organization |
| `IRDX_CRED` | OTHER | `X` | Accrediting body and reference number (when appropriate) |
| `TEST_STAT` | OTHER | `X` | Test status |
| `GEOL_STAT` | OTHER | `X` | Stratum reference shown on trial pit or traverse sketch |
| `FILE_FSET` | OTHER | `X` | Associated file reference (e.g. test result sheets) |
| `IRDX_OPER` | OTHER | `X` | Name of test operator |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `IRDX_DPTH`, `IRDX_TESN`. Children (0): _none_. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[LOCA]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[LOCA]]
