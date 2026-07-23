---
type: group
title: HORN — Exploratory Hole Orientation and Inclination
status: drafted
tags: [group]
group_code: HORN
parent: LOCA
is_high_volume: false
varies_between_editions: false
key_headings: [LOCA_ID, HORN_TOP, HORN_BASE]
required_headings: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=HORN]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, LOCA]
sources: []
---
# HORN — Exploratory Hole Orientation and Inclination

## Purpose
> [!quote] The **HORN** group — Exploratory Hole Orientation and Inclination. It is a **child of [[LOCA]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  LOCA ||--o{ HORN : has
  HORN {
    KEY LOCA_ID
    KEY HORN_TOP
    KEY HORN_BASE
  }
```

- Parent: [[LOCA]]
- Children: _none_
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=HORN]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

7 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `HORN_TOP` | **KEY** | `2DP` | Depth to top of exploratory hole section |
| `HORN_BASE` | **KEY** | `2DP` | Depth to base of exploratory hole section |
| `HORN_ORNT` | OTHER | `0DP` | Orientation of exploratory hole section or traverse (degrees from north) |
| `HORN_INCL` | OTHER | `0DP` | Inclination of exploratory hole section or traverse (measured positively down from horizontal) |
| `HORN_REM` | OTHER | `X` | Remarks relating to orientation and inclination of hole section |
| `FILE_FSET` | OTHER | `X` | Associated file reference (e.g. contract data specification) |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `HORN_TOP`, `HORN_BASE`. Children (0): _none_. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[LOCA]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[LOCA]]
