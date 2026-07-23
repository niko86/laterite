---
type: group
title: PIPE — Monitoring Installation Pipe Work
status: drafted
tags: [group]
group_code: PIPE
parent: LOCA
is_high_volume: false
varies_between_editions: false
key_headings: [LOCA_ID, PIPE_REF, PIPE_TOP, PIPE_BASE]
required_headings: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=PIPE]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, LOCA]
sources: []
---
# PIPE — Monitoring Installation Pipe Work

## Purpose
> [!quote] The **PIPE** group — Monitoring Installation Pipe Work. It is a **child of [[LOCA]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  LOCA ||--o{ PIPE : has
  PIPE {
    KEY LOCA_ID
    KEY PIPE_REF
    KEY PIPE_TOP
    KEY PIPE_BASE
  }
```

- Parent: [[LOCA]]
- Children: _none_
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=PIPE]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

9 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `PIPE_REF` | **KEY** | `X` | Pipe reference |
| `PIPE_TOP` | **KEY** | `2DP` | Top of construction zone |
| `PIPE_BASE` | **KEY** | `2DP` | Base of construction zone |
| `PIPE_DIAM` | OTHER | `0DP` | Diameter of pipe |
| `PIPE_TYPE` | OTHER | `PA` | Type of pipe |
| `PIPE_CONS` | OTHER | `X` | Details of pipe construction |
| `PIPE_REM` | OTHER | `X` | Remarks |
| `FILE_FSET` | OTHER | `X` | Associated file reference (e.g. drilling journals) |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `PIPE_REF`, `PIPE_TOP`, `PIPE_BASE`. Children (0): _none_. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[LOCA]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[LOCA]]
