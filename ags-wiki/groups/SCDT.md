---
type: group
title: SCDT — Static Cone Dissipation Tests - Data
status: drafted
tags: [group]
group_code: SCDT
parent: SCDG
is_high_volume: true
varies_between_editions: false
key_headings: [LOCA_ID, SCPG_TESN, SCDG_DPTH, SCDT_SECS]
required_headings: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=SCDT]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, SCDG]
sources: []
---
# SCDT — Static Cone Dissipation Tests - Data

## Purpose
> [!quote] The **SCDT** group — Static Cone Dissipation Tests - Data. It is a **child of [[SCDG]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  SCDG ||--o{ SCDT : has
  SCDT {
    KEY LOCA_ID
    KEY SCPG_TESN
    KEY SCDG_DPTH
    KEY SCDT_SECS
  }
```

- Parent: [[SCDG]]
- Children: _none_
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=SCDT]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

10 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `SCPG_TESN` | **KEY** | `X` | Test reference or push number |
| `SCDG_DPTH` | **KEY** | `2DP` | Depth of dissipation test |
| `SCDT_SECS` | **KEY** | `1DP` | Seconds elapsed since start of test |
| `SCDT_RES` | OTHER | `3DP` | Cone resistance |
| `SCDT_PWP1` | OTHER | `4DP` | Face porewater pressure (u1) |
| `SCDT_PWP2` | OTHER | `4DP` | Shoulder porewater pressure (u2) |
| `SCDT_PWP3` | OTHER | `4DP` | Top of sleeve porewater pressure (u3) |
| `SCDT_REM` | OTHER | `X` | Comments |
| `FILE_FSET` | OTHER | `X` | Associated file reference (e.g. test result sheets) |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `SCPG_TESN`, `SCDG_DPTH`, `SCDT_SECS`. Children (0): _none_. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[SCDG]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[SCDG]]
