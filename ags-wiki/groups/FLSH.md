---
type: group
title: FLSH — Drilling Flush Details
status: drafted
tags: [group]
group_code: FLSH
parent: LOCA
is_high_volume: false
varies_between_editions: false
key_headings: [LOCA_ID, FLSH_TOP, FLSH_BASE]
required_headings: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=FLSH]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, LOCA]
sources: []
---
# FLSH — Drilling Flush Details

## Purpose
> [!quote] The **FLSH** group — Drilling Flush Details. It is a **child of [[LOCA]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  LOCA ||--o{ FLSH : has
  FLSH {
    KEY LOCA_ID
    KEY FLSH_TOP
    KEY FLSH_BASE
  }
```

- Parent: [[LOCA]]
- Children: _none_
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=FLSH]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

9 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `FLSH_TOP` | **KEY** | `2DP` | Depth to top of flush zone |
| `FLSH_BASE` | **KEY** | `2DP` | Depth to bottom of flush zone |
| `FLSH_TYPE` | OTHER | `PA` | Type of flush |
| `FLSH_RETN` | OTHER | `0DP` | Flush return minimum (as percentage) |
| `FLSH_RETX` | OTHER | `0DP` | Flush return maximum (as percentage) |
| `FLSH_COL` | OTHER | `X` | Colour of flush return |
| `FLSH_REM` | OTHER | `X` | Remarks |
| `FILE_FSET` | OTHER | `X` | Associated file reference (e.g. drilling journal, mud logging or test records) |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `FLSH_TOP`, `FLSH_BASE`. Children (0): _none_. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[LOCA]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[LOCA]]
