---
type: group
title: DCPT — Dynamic Cone Penetrometer Tests - Data
status: drafted
tags: [group]
group_code: DCPT
parent: DCPG
is_high_volume: false
varies_between_editions: false
key_headings: [LOCA_ID, DCPG_DATE, DCPG_TESN, DCPG_DPTH, DCPT_CBLO]
required_headings: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=DCPT]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, DCPG]
sources: []
---
# DCPT — Dynamic Cone Penetrometer Tests - Data

## Purpose
> [!quote] The **DCPT** group — Dynamic Cone Penetrometer Tests - Data. It is a **child of [[DCPG]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  DCPG ||--o{ DCPT : has
  DCPT {
    KEY LOCA_ID
    KEY DCPG_DATE
    KEY DCPG_TESN
    KEY DCPG_DPTH
    KEY DCPT_CBLO
  }
```

- Parent: [[DCPG]]
- Children: _none_
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=DCPT]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

8 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `DCPG_DATE` | **KEY** | `DT` | Test date |
| `DCPG_TESN` | **KEY** | `X` | Test reference |
| `DCPG_DPTH` | **KEY** | `2DP` | Depth from surface to start of test |
| `DCPT_CBLO` | **KEY** | `0DP` | Cumulative blows |
| `DCPT_PEN` | OTHER | `0DP` | Penetration at DCPT_CBLO |
| `DCPT_DEL` | OTHER | `T` | Delay before increment started |
| `DCPT_REM` | OTHER | `X` | Test reading remarks |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `DCPG_DATE`, `DCPG_TESN`, `DCPG_DPTH`, `DCPT_CBLO`. Children (0): _none_. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[DCPG]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[DCPG]]
