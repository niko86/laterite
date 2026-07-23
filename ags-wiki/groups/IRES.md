---
type: group
title: IRES — In Situ Resistivity Tests
status: drafted
tags: [group]
group_code: IRES
parent: LOCA
is_high_volume: false
varies_between_editions: false
key_headings: [LOCA_ID, IRES_DPTH, IRES_TESN]
required_headings: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=IRES]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, LOCA]
sources: []
---
# IRES — In Situ Resistivity Tests

## Purpose
> [!quote] The **IRES** group — In Situ Resistivity Tests. It is a **child of [[LOCA]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  LOCA ||--o{ IRES : has
  IRES {
    KEY LOCA_ID
    KEY IRES_DPTH
    KEY IRES_TESN
  }
```

- Parent: [[LOCA]]
- Children: _none_
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=IRES]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

18 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `IRES_DPTH` | **KEY** | `2DP` | Depth to which in situ resistivity test relates |
| `IRES_TESN` | **KEY** | `X` | Test reference |
| `IRES_BASE` | OTHER | `2DP` | Base depth to which in-situ resistivity test relates |
| `IRES_TYPE` | OTHER | `PA` | Type of resistivity test |
| `IRES_DATE` | OTHER | `DT` | Test date |
| `IRES_IRES` | OTHER | `2SF` | Mean value of the apparent resistivity |
| `IRES_RES1` | OTHER | `2SF` | First value of apparent resistivity when more than 15% different to mean |
| `IRES_RES2` | OTHER | `2SF` | Second value of apparent resistivity when more than 15% different to mean |
| `IRES_REM` | OTHER | `X` | Details of test e.g. electrode spacing and configuration |
| `IRES_ENV` | OTHER | `X` | Details of weather and environmental conditions during test |
| `IRES_METH` | OTHER | `X` | Test method |
| `IRES_CONT` | OTHER | `X` | Name of testing organization |
| `IRES_CRED` | OTHER | `X` | Accrediting body and reference number (when appropriate) |
| `TEST_STAT` | OTHER | `X` | Test status |
| `GEOL_STAT` | OTHER | `X` | Stratum reference shown on trial pit or traverse sketch |
| `FILE_FSET` | OTHER | `X` | Associated file reference (e.g. test result sheets) |
| `IRES_OPER` | OTHER | `X` | Name of test operator |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `IRES_DPTH`, `IRES_TESN`. Children (0): _none_. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[LOCA]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[LOCA]]
