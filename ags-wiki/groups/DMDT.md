---
type: group
title: DMDT — Flat Dilatometer Dissipation Test - Data
status: drafted
tags: [group]
group_code: DMDT
parent: DMDG
is_high_volume: false
varies_between_editions: true
key_headings: [LOCA_ID, DMTG_TESN, DMDG_DPTH, DMDT_TIME]
required_headings: []
ags_editions: [4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=DMDT]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, DMDG]
sources: []
---
# DMDT — Flat Dilatometer Dissipation Test - Data

## Purpose
> [!quote] The **DMDT** group — Flat Dilatometer Dissipation Test - Data. It is a **child of [[DMDG]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  DMDG ||--o{ DMDT : has
  DMDT {
    KEY LOCA_ID
    KEY DMTG_TESN
    KEY DMDG_DPTH
    KEY DMDT_TIME
  }
```

- Parent: [[DMDG]]
- Children: _none_
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=DMDT]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

7 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `DMTG_TESN` | **KEY** | `X` | Test reference |
| `DMDG_DPTH` | **KEY** | `2DP` | Depth of dissipation test |
| `DMDT_TIME` | **KEY** | `1DP` | Elapsed time since start of test |
| `DMDT_A` | OTHER | `2DP` | A-pressure test reading |
| `DMDT_REM` | OTHER | `X` | Note on individual record |
| `FILE_FSET` | OTHER | `X` | Associated File Reference |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `DMTG_TESN`, `DMDG_DPTH`, `DMDT_TIME`. Children (0): _none_. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[DMDG]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[DMDG]]
