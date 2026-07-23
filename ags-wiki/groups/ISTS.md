---
type: group
title: ISTS — In Situ Seismic Test - Signal
status: drafted
tags: [group]
group_code: ISTS
parent: ISTG
is_high_volume: false
varies_between_editions: true
key_headings: [LOCA_ID, ISTG_TESN, ISTS_SGLN]
required_headings: []
ags_editions: [4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=ISTS]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, ISTG, ISTR]
sources: []
---
# ISTS — In Situ Seismic Test - Signal

## Purpose
> [!quote] The **ISTS** group — In Situ Seismic Test - Signal. It is a **child of [[ISTG]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  ISTG ||--o{ ISTS : has
  ISTS ||--o{ ISTR : has
  ISTS {
    KEY LOCA_ID
    KEY ISTG_TESN
    KEY ISTS_SGLN
  }
```

- Parent: [[ISTG]]
- Children: [[ISTR]]
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=ISTS]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

10 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `ISTG_TESN` | **KEY** | `X` | Setup reference |
| `ISTS_SGLN` | **KEY** | `X` | Signal reference |
| `ISTS_TYPE` | OTHER | `PA` | Source type |
| `ISTS_DTIM` | OTHER | `DT` | Date and time of signal |
| `ISTS_RATE` | OTHER | `3DP` | Raw sampling rate |
| `ISTS_PTRT` | OTHER | `1DP` | Pre-trigger recording time |
| `ISTS_TTLY` | OTHER | `3DP` | Trigger time latency (positive for late recording) |
| `ISTS_REM` | OTHER | `X` | Remarks |
| `FILE_FSET` | OTHER | `X` | Associated file reference |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `ISTG_TESN`, `ISTS_SGLN`. Children (1): [[ISTR]]. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[ISTG]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[ISTG]] · [[ISTR]]
