---
type: group
title: WGPG — Wireline Geophysics - General
status: drafted
tags: [group]
group_code: WGPG
parent: LOCA
is_high_volume: false
varies_between_editions: true
key_headings: [LOCA_ID, WGPG_ID, WGPG_TOOL]
required_headings: []
ags_editions: [4.1, 4.1.1, 4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=WGPG]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, LOCA, WGPT]
sources: []
---
# WGPG — Wireline Geophysics - General

## Purpose
> [!quote] The **WGPG** group — Wireline Geophysics - General. It is a **child of [[LOCA]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  LOCA ||--o{ WGPG : has
  WGPG ||--o{ WGPT : has
  WGPG {
    KEY LOCA_ID
    KEY WGPG_ID
    KEY WGPG_TOOL
  }
```

- Parent: [[LOCA]]
- Children: [[WGPT]]
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=WGPG]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

20 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `WGPG_ID` | **KEY** | `X` | Test reference |
| `WGPG_TOOL` | **KEY** | `PA` | Tool used |
| `WGPG_DATE` | OTHER | `DT` | Test date |
| `WGPG_STRT` | OTHER | `2DP` | Test start depth |
| `WGPG_STOP` | OTHER | `2DP` | Test stop depth |
| `WGPG_BHD` | OTHER | `2DP` | Depth of borehole |
| `WGPG_WAT` | OTHER | `XN` | Depth of water in borehole |
| `WGPG_DETL` | OTHER | `X` | Details of instrument |
| `WGPG_CDIA` | OTHER | `X` | Casing internal diameter as reported by drillers |
| `WGPG_REM` | OTHER | `X` | Remarks |
| `WGPG_ENV` | OTHER | `X` | Details of weather and environmental conditions during test |
| `WGPG_METH` | OTHER | `X` | Measurement method |
| `WGPG_CONT` | OTHER | `X` | Contractor who undertook testing |
| `WGPG_CRED` | OTHER | `X` | Accrediting body and reference number (Where appropriate) |
| `WGPG_STAT` | OTHER | `X` | Test status |
| `FILE_FSET` | OTHER | `X` | Associated file reference (e.g. equipment calibrations) |
| `WGPG_OPER` | OTHER | `X` | Name of test operator |
| `WGPG_LIM` | OTHER | `U` | Instrument/method reading/detection limit |
| `WGPG_ULIM` | OTHER | `U` | Instrument/method upper reading detection (when appropriate) |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `WGPG_ID`, `WGPG_TOOL`. Children (1): [[WGPT]]. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[LOCA]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[LOCA]] · [[WGPT]]
