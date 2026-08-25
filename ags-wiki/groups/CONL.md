---
type: group
title: CONL — Consolidation Tests - Lab Data
status: drafted
tags: [group]
group_code: CONL
parent: SAMP
is_high_volume: true
varies_between_editions: false
key_headings: [LOCA_ID, SAMP_TOP, SAMP_REF, SAMP_TYPE, SAMP_ID, SPEC_REF, SPEC_DPTH, CONL_MNUM]
required_headings: []
ags_editions: []
repo_refs:
  absence_test: "repo:rust-packages/laterite-ags4-reference/src/union.rs::dropped_agsl_drafts_are_absent"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, SAMP]
sources: [ags-library-xlsx]
---
# CONL — Consolidation Tests - Lab Data

## Purpose
> [!quote] The **CONL** group — Consolidation Tests - Lab Data. It is a **child of [[SAMP]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

> [!warning] AGS-L draft group — not in the AGS4 4.x spec
> CONL is part of **AGS-L** (the AGS Library extension, expected publish
> 2026), hand-authored from the AGS-L draft library workbooks — see
> [[ags-library-xlsx]]. It is **not** part of the AGS4 4.x standard, so it is
> **absent from `ags_dictionary.json`**, the union that drives the generated
> group tier: there is no typed `laterite.groups` class for it and no validator
> dictionary membership. That absence is asserted, not incidental — see
> `repo:rust-packages/laterite-ags4-reference/src/union.rs::dropped_agsl_drafts_are_absent`.
> To carry the group in a file, declare it in the [[effective-dictionary]]
> (in-file `DICT`, [[rule-18-dict-group]]) or register it dynamically. See
> [[ags-4.2]].

## Position in the model

```mermaid
erDiagram
  SAMP ||--o{ CONL : has
  CONL {
    KEY LOCA_ID
    KEY SAMP_TOP
    KEY SAMP_REF
    KEY SAMP_TYPE
    KEY SAMP_ID
    KEY SPEC_REF
    KEY SPEC_DPTH
    KEY CONL_MNUM
  }
```

- Parent: [[SAMP]]
- Children: _none_
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Hand-authored from the AGS-L draft library workbooks (`AGSL4_2_*.xlsx` — not vendored; see [[ags-library-xlsx]]). **Not** rendered from `ags_dictionary.json`, which has no CONL entry. Suggested UNITs + worked examples live in the workbook, not duplicated here.

18 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `SAMP_TOP` | **KEY** | `2DP` | Depth to top of sample |
| `SAMP_REF` | **KEY** | `X` | Sample reference |
| `SAMP_TYPE` | **KEY** | `PA` | Sample type |
| `SAMP_ID` | **KEY** | `ID` | Sample unique identifier |
| `SPEC_REF` | **KEY** | `X` | Specimen reference |
| `SPEC_DPTH` | **KEY** | `2DP` | Depth to top of specimen |
| `CONL_MNUM` | **KEY** | `0DP` | Measurement number / record index |
| `CONL_TTIM` | OTHER | `1DP` | Elapsed time since start of test |
| `CONL_TTDT` | OTHER | `DT` | Test date time |
| `CONL_STIM` | OTHER | `1DP` | Elapsed time since start of stage |
| `CONL_STGN` | OTHER | `0DP` | Stage number |
| `CONL_STGD` | OTHER | `X` | Stage description |
| `CONL_SZT` | OTHER | `1DP` | Applied vertical stress |
| `CONL_HGHT` | OTHER | `3DP` | Specimen height |
| `CONL_EZET` | OTHER | `3DP` | Total vertical strain |
| `CONL_VR` | OTHER | `4DP` | Void ratio |
| `CONL_PWP` | OTHER | `1DP` | Pore pressure |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `SAMP_TOP`, `SAMP_REF`, `SAMP_TYPE`, `SAMP_ID`, `SPEC_REF`, `SPEC_DPTH`, `CONL_MNUM`. No child groups. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[SAMP]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
**Edition variation does not apply.** The AGS4 editions do not carry this group at all, so there is no per-edition delta to record and the AGS4 Change Log has nothing to say about it — the headings above track the AGS-L draft, which publishes on its own timetable (see [[ags-library-xlsx]] · [[ags-4.2]]). What the shipped rules do here is unchanged by that: they are frozen across editions ([[ags4-rules-frozen-dictionary-evolves]]), and a group outside the dictionary is reached through the [[effective-dictionary]], not through an edition.

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[SAMP]]
