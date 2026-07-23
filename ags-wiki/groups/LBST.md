---
type: group
title: LBST — Testing Schedule Details
status: drafted
tags: [group]
group_code: LBST
parent: LBSG
is_high_volume: true
varies_between_editions: false
key_headings: [LOCA_ID, SAMP_TOP, SAMP_REF, SAMP_TYPE, SAMP_ID, LBSG_REF, LBST_TEST]
required_headings: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=LBST]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, LBSG]
sources: []
---
# LBST — Testing Schedule Details

## Purpose
> [!quote] The **LBST** group — Testing Schedule Details. It is a **child of [[LBSG]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  LBSG ||--o{ LBST : has
  LBST {
    KEY LOCA_ID
    KEY SAMP_TOP
    KEY SAMP_REF
    KEY SAMP_TYPE
    KEY SAMP_ID
    KEY LBSG_REF
    KEY LBST_TEST
  }
```

- Parent: [[LBSG]]
- Children: _none_
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=LBST]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

18 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `SAMP_TOP` | **KEY** | `2DP` | Depth to top of sample |
| `SAMP_REF` | **KEY** | `X` | Sample reference |
| `SAMP_TYPE` | **KEY** | `PA` | Sample type |
| `SAMP_ID` | **KEY** | `ID` | Sample unique identifier |
| `LBSG_REF` | **KEY** | `X` | Testing schedule reference |
| `LBST_TEST` | **KEY** | `X` | Test Name |
| `CHOC_REF` | OTHER | `X` | Chain of custody reference |
| `LBST_TTYP` | OTHER | `X` | Full test method or standard |
| `LBST_METH` | OTHER | `X` | Method and test parameters |
| `LBST_PREP` | OTHER | `X` | Preparation requirements |
| `LBST_DEPN` | OTHER | `X` | Dependent test options |
| `LBST_STAT` | OTHER | `PA` | Status of laboratory test |
| `LBST_REM` | OTHER | `X` | Remarks |
| `LBST_DUE` | OTHER | `DT` | Test results due date |
| `LBST_DETL` | OTHER | `X` | Details of testing carried out or reasons for no testing possible |
| `LBST_DONE` | OTHER | `DT` | Date test completed |
| `FILE_FSET` | OTHER | `X` | Associated file reference |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `SAMP_TOP`, `SAMP_REF`, `SAMP_TYPE`, `SAMP_ID`, `LBSG_REF`, `LBST_TEST`. Children (0): _none_. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[LBSG]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[LBSG]]
