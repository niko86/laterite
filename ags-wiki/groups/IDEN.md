---
type: group
title: IDEN — In Situ Density Tests
status: drafted
tags: [group]
group_code: IDEN
parent: LOCA
is_high_volume: false
varies_between_editions: false
key_headings: [LOCA_ID, IDEN_DPTH, IDEN_TESN]
required_headings: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=IDEN]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, LOCA]
sources: []
---
# IDEN — In Situ Density Tests

## Purpose
> [!quote] The **IDEN** group — In Situ Density Tests. It is a **child of [[LOCA]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  LOCA ||--o{ IDEN : has
  IDEN {
    KEY LOCA_ID
    KEY IDEN_DPTH
    KEY IDEN_TESN
  }
```

- Parent: [[LOCA]]
- Children: _none_
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=IDEN]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

18 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `IDEN_DPTH` | **KEY** | `2DP` | Depth of in situ density test |
| `IDEN_TESN` | **KEY** | `X` | Test reference |
| `IDEN_DATE` | OTHER | `DT` | Test date |
| `IDEN_TYPE` | OTHER | `PA` | Type of density test performed |
| `IDEN_IDEN` | OTHER | `2DP` | In situ bulk density (after any calibration / corrections applied, i.e. reported value) |
| `IDEN_MC` | OTHER | `X` | Water/moisture content relating to in situ test (after any calibration / corrections applied, i.e. reported value) |
| `IDEN_STAB` | OTHER | `2SF` | Amount of stabiliser added |
| `IDEN_STYP` | OTHER | `X` | Type of stabiliser added |
| `IDEN_REM` | OTHER | `X` | Remarks |
| `IDEN_ENV` | OTHER | `X` | Details of weather and environmental conditions during test |
| `IDEN_METH` | OTHER | `X` | Test method |
| `IDEN_CONT` | OTHER | `X` | Name of testing organization |
| `IDEN_CRED` | OTHER | `X` | Accrediting body and reference number (when appropriate) |
| `TEST_STAT` | OTHER | `X` | Test status |
| `GEOL_STAT` | OTHER | `X` | Stratum reference shown on trial pit or traverse sketch |
| `FILE_FSET` | OTHER | `X` | Associated file reference (e.g. test result sheets) |
| `IDEN_OPER` | OTHER | `X` | Name of test operator |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `IDEN_DPTH`, `IDEN_TESN`. Children (0): _none_. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[LOCA]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[LOCA]]
