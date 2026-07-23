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
ags_editions: [4.1]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=CONL]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, SAMP]
sources: []
---
# CONL — Consolidation Tests - Lab Data

## Purpose
> [!quote] The **CONL** group — Consolidation Tests - Lab Data. It is a **child of [[SAMP]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

> [!warning] AGS-L draft group — not in the AGS4 4.x spec
> CONL is part of **AGS-L** (the AGS Library extension, expected publish 2026),
> scaffolded from `repo:reports/AGSL4_2_*.xlsx`. It is **retained** in the
> dictionary (never deleted) but is **not** a standard AGS4 4.x group. The
> AGS-L correction (PR #45) flags its dictionary `contents`
> `(AGS-L draft, publish 2026)`. See [[ags-4.2]].

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
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=CONL]` (the repo's model authority — AGS edition 4.1). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

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
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[SAMP]]
