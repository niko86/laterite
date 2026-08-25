---
type: group
title: TREL — Triaxial Tests - Logged Data
status: drafted
tags: [group]
group_code: TREL
parent: TRET
is_high_volume: true
varies_between_editions: false
key_headings: [LOCA_ID, SAMP_TOP, SAMP_REF, SAMP_TYPE, SAMP_ID, SPEC_REF, SPEC_DPTH, TRET_TESN, TREL_MNUM]
required_headings: []
ags_editions: []
repo_refs:
  absence_test: "repo:rust-packages/laterite-ags4-reference/src/union.rs::dropped_agsl_drafts_are_absent"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, TRET]
sources: [ags-library-xlsx]
---
# TREL — Triaxial Tests - Logged Data

## Purpose
> [!quote] The **TREL** group — Triaxial Tests - Logged Data. It is a **child of [[TRET]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

> [!warning] AGS-L draft group — not in the AGS4 4.x spec
> TREL is part of **AGS-L** (the AGS Library extension, expected publish
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
  TRET ||--o{ TREL : has
  TREL {
    KEY LOCA_ID
    KEY SAMP_TOP
    KEY SAMP_REF
    KEY SAMP_TYPE
    KEY SAMP_ID
    KEY SPEC_REF
    KEY SPEC_DPTH
    KEY TRET_TESN
    KEY TREL_MNUM
  }
```

- Parent: [[TRET]]
- Children: _none_
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Hand-authored from the AGS-L draft library workbooks (`AGSL4_2_*.xlsx` — not vendored; see [[ags-library-xlsx]]). **Not** rendered from `ags_dictionary.json`, which has no TREL entry. Suggested UNITs + worked examples live in the workbook, not duplicated here.

33 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `SAMP_TOP` | **KEY** | `2DP` | Depth to top of sample |
| `SAMP_REF` | **KEY** | `X` | Sample reference |
| `SAMP_TYPE` | **KEY** | `PA` | Sample type |
| `SAMP_ID` | **KEY** | `ID` | Sample unique identifier |
| `SPEC_REF` | **KEY** | `X` | Specimen reference |
| `SPEC_DPTH` | **KEY** | `2DP` | Depth to top of specimen |
| `TRET_TESN` | **KEY** | `X` | Triaxial test number |
| `TREL_MNUM` | **KEY** | `0DP` | Measurement number / record index |
| `TREL_TTIM` | OTHER | `1DP` | Elapsed time since start of test |
| `TREL_TTDT` | OTHER | `DT` | Test date time |
| `TREL_STIM` | OTHER | `1DP` | Elapsed time since start of stage |
| `TREL_STGN` | OTHER | `0DP` | Stage number |
| `TREL_STGD` | OTHER | `X` | Stage description |
| `TREL_CELL` | OTHER | `1DP` | Cell pressure |
| `TREL_BACK` | OTHER | `1DP` | Back pressure |
| `TREL_PWP` | OTHER | `1DP` | Pore pressure (external instrumentation) |
| `TREL_PWPM` | OTHER | `1DP` | Pore pressure (mid-height) |
| `TREL_SZT` | OTHER | `1DP` | Vertical total stress |
| `TREL_SZE` | OTHER | `1DP` | Vertical effective stress |
| `TREL_SRT` | OTHER | `1DP` | Radial total stress |
| `TREL_SRE` | OTHER | `1DP` | Radial effective stress |
| `TREL_EZET` | OTHER | `3DP` | Total vertical strain (external) |
| `TREL_EZES` | OTHER | `3DP` | Stage vertical strain (external) |
| `TREL_EPET` | OTHER | `3DP` | Total volumetric strain (external) |
| `TREL_EPES` | OTHER | `3DP` | Stage volumetric strain (external) |
| `TREL_EZ1T` | OTHER | `3DP` | Total vertical strain (local LVDT 1) |
| `TREL_EZ1S` | OTHER | `3DP` | Stage vertical strain (local LVDT 1) |
| `TREL_EZ2T` | OTHER | `3DP` | Total vertical strain (local LVDT 2) |
| `TREL_EZ2S` | OTHER | `3DP` | Stage vertical strain (local LVDT 2) |
| `TREL_ER1T` | OTHER | `3DP` | Total radial strain (local LDT 1) |
| `TREL_ER1S` | OTHER | `3DP` | Stage radial strain (local LDT 1) |
| `TREL_CYCN` | OTHER | `0DP` | Cycle number |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `SAMP_TOP`, `SAMP_REF`, `SAMP_TYPE`, `SAMP_ID`, `SPEC_REF`, `SPEC_DPTH`, `TRET_TESN`, `TREL_MNUM`. No child groups. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[TRET]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
**Edition variation does not apply.** The AGS4 editions do not carry this group at all, so there is no per-edition delta to record and the AGS4 Change Log has nothing to say about it — the headings above track the AGS-L draft, which publishes on its own timetable (see [[ags-library-xlsx]] · [[ags-4.2]]). What the shipped rules do here is unchanged by that: they are frozen across editions ([[ags4-rules-frozen-dictionary-evolves]]), and a group outside the dictionary is reached through the [[effective-dictionary]], not through an edition.

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[TRET]]
