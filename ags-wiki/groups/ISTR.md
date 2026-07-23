---
type: group
title: ISTR — In Situ Seismic Test - Signal Receiver
status: drafted
tags: [group]
group_code: ISTR
parent: ISTS
is_high_volume: false
varies_between_editions: true
key_headings: [LOCA_ID, ISTG_TESN, ISTS_SGLN, ISTR_DPTH, ISTR_REF]
required_headings: []
ags_editions: [4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=ISTR]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, ISTS]
sources: []
---
# ISTR — In Situ Seismic Test - Signal Receiver

## Purpose
> [!quote] The **ISTR** group — In Situ Seismic Test - Signal Receiver. It is a **child of [[ISTS]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  ISTS ||--o{ ISTR : has
  ISTR {
    KEY LOCA_ID
    KEY ISTG_TESN
    KEY ISTS_SGLN
    KEY ISTR_DPTH
    KEY ISTR_REF
  }
```

- Parent: [[ISTS]]
- Children: _none_
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=ISTR]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

10 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `ISTG_TESN` | **KEY** | `X` | Setup reference |
| `ISTS_SGLN` | **KEY** | `X` | Signal reference |
| `ISTR_DPTH` | **KEY** | `2DP` | Depth to receiver |
| `ISTR_REF` | **KEY** | `X` | Receiver reference |
| `ISTR_SSD` | OTHER | `2DP` | Source slant distance |
| `ISTR_QUAL` | OTHER | `X` | Quality of received signal |
| `ISTR_QUAM` | OTHER | `X` | Method of quality assessment |
| `ISTR_REM` | OTHER | `X` | Remarks |
| `FILE_FSET` | OTHER | `X` | Associated file reference |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `ISTG_TESN`, `ISTS_SGLN`, `ISTR_DPTH`, `ISTR_REF`. Children (0): _none_. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[ISTS]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[ISTS]]
