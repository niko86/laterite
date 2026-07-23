---
type: group
title: DMTZ — Flat Dilatometer Test - Zeros
status: drafted
tags: [group]
group_code: DMTZ
parent: DMTG
is_high_volume: false
varies_between_editions: true
key_headings: [LOCA_ID, DMTG_TESN, DMTZ_DATE]
required_headings: []
ags_editions: [4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=DMTZ]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, DMTG]
sources: []
---
# DMTZ — Flat Dilatometer Test - Zeros

## Purpose
> [!quote] The **DMTZ** group — Flat Dilatometer Test - Zeros. It is a **child of [[DMTG]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  DMTG ||--o{ DMTZ : has
  DMTZ {
    KEY LOCA_ID
    KEY DMTG_TESN
    KEY DMTZ_DATE
  }
```

- Parent: [[DMTG]]
- Children: _none_
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=DMTZ]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

8 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `DMTG_TESN` | **KEY** | `X` | Test reference |
| `DMTZ_DATE` | **KEY** | `DT` | Test date and time of zero readings |
| `DMTZ_TYPE` | OTHER | `PA` | When were the zeros performed |
| `DMTZ_BCVA` | OTHER | `2DP` | Blade zero value, delta A |
| `DMTZ_BCVB` | OTHER | `2DP` | Blade zero value, delta B |
| `DMTZ_REM` | OTHER | `X` | Remarks on the zero values |
| `FILE_FSET` | OTHER | `X` | Associated file reference |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `DMTG_TESN`, `DMTZ_DATE`. Children (0): _none_. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[DMTG]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[DMTG]]
