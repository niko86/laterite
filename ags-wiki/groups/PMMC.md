---
type: group
title: PMMC — Menard Pressuremeter Test Results - Unload/Reload Cycles
status: drafted
tags: [group]
group_code: PMMC
parent: PMMG
is_high_volume: false
varies_between_editions: true
key_headings: [LOCA_ID, PMMG_DPTH, PMMG_TESN, PMMC_CYNO]
required_headings: []
ags_editions: [4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=PMMC]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, PMMG]
sources: []
---
# PMMC — Menard Pressuremeter Test Results - Unload/Reload Cycles

## Purpose
> [!quote] The **PMMC** group — Menard Pressuremeter Test Results - Unload/Reload Cycles. It is a **child of [[PMMG]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  PMMG ||--o{ PMMC : has
  PMMC {
    KEY LOCA_ID
    KEY PMMG_DPTH
    KEY PMMG_TESN
    KEY PMMC_CYNO
  }
```

- Parent: [[PMMG]]
- Children: _none_
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=PMMC]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

9 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `PMMG_DPTH` | **KEY** | `2DP` | Depth of test |
| `PMMG_TESN` | **KEY** | `X` | Test reference |
| `PMMC_CYNO` | **KEY** | `X` | Cycle number |
| `PMMC_P1CY` | OTHER | `3DP` | Corrected pressure at origin of cyclic pressure range |
| `PMMC_P2CY` | OTHER | `3DP` | Corrected pressure at end of cyclic pressure range |
| `PMMC_EMCY` | OTHER | `1DP` | Cyclic Menard modulus |
| `PMMC_REM` | OTHER | `X` | Remarks |
| `FILE_FSET` | OTHER | `X` | Associated file reference (e.g. test result sheets) |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `PMMG_DPTH`, `PMMG_TESN`, `PMMC_CYNO`. Children (0): _none_. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[PMMG]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[PMMG]]
