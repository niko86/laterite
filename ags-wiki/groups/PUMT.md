---
type: group
title: PUMT — Pumping Tests - Data
status: drafted
tags: [group]
group_code: PUMT
parent: PUMG
is_high_volume: false
varies_between_editions: false
key_headings: [LOCA_ID, PUMG_TEST, PUMT_DTIM]
required_headings: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=PUMT]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, PUMG]
sources: []
---
# PUMT — Pumping Tests - Data

## Purpose
> [!quote] The **PUMT** group — Pumping Tests - Data. It is a **child of [[PUMG]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  PUMG ||--o{ PUMT : has
  PUMT {
    KEY LOCA_ID
    KEY PUMG_TEST
    KEY PUMT_DTIM
  }
```

- Parent: [[PUMG]]
- Children: _none_
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=PUMT]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

7 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `PUMG_TEST` | **KEY** | `X` | Test reference |
| `PUMT_DTIM` | **KEY** | `DT` | Date and time of reading |
| `PUMT_DPTH` | OTHER | `2DP` | Depth to water below ground |
| `PUMT_QUAT` | OTHER | `1DP` | Pumping rate from hole |
| `PUMT_REM` | OTHER | `X` | Remarks |
| `FILE_FSET` | OTHER | `X` | Associated file reference (e.g. test result sheets) |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `PUMG_TEST`, `PUMT_DTIM`. Children (0): _none_. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[PUMG]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[PUMG]]
