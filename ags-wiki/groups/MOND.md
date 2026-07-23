---
type: group
title: MOND — Monitoring Readings
status: drafted
tags: [group]
group_code: MOND
parent: MONG
is_high_volume: true
varies_between_editions: false
key_headings: [LOCA_ID, MONG_ID, MONG_DIS, MOND_DTIM, MOND_TYPE, MOND_REF]
required_headings: [MOND_UNIT]
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=MOND]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, MONG]
sources: []
---
# MOND — Monitoring Readings

## Purpose
> [!quote] The **MOND** group — Monitoring Readings. It is a **child of [[MONG]]** in the PROJ-rooted hierarchy. See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  MONG ||--o{ MOND : has
  MOND {
    KEY LOCA_ID
    KEY MONG_ID
    KEY MONG_DIS
    KEY MOND_DTIM
    KEY MOND_TYPE
    KEY MOND_REF
  }
```

- Parent: [[MONG]]
- Children: _none_
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=MOND]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

17 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LOCA_ID` | **KEY** | `ID` | Location identifier |
| `MONG_ID` | **KEY** | `X` | Monitoring point reference |
| `MONG_DIS` | **KEY** | `2DP` | Initial distance of monitoring point from LOCA_ID datum |
| `MOND_DTIM` | **KEY** | `DT` | Date and time of reading |
| `MOND_TYPE` | **KEY** | `PA` | Reading type |
| `MOND_REF` | **KEY** | `X` | Reading reference |
| `MOND_INST` | OTHER | `X` | Instrument reference / serial number |
| `MOND_RDNG` | OTHER | `XN` | Reading |
| `MOND_UNIT` | *REQ* | `PU` | Units of reading |
| `MOND_METH` | OTHER | `X` | Measurement method |
| `MOND_LIM` | OTHER | `U` | Instrument/method reading/detection limit |
| `MOND_ULIM` | OTHER | `U` | Instrument/method upper reading/detection (when appropriate) |
| `MOND_NAME` | OTHER | `X` | Client preferred name of measurement |
| `MOND_CRED` | OTHER | `X` | Accrediting body and reference number (when appropriate) |
| `MOND_CONT` | OTHER | `X` | Organization taking reading |
| `MOND_REM` | OTHER | `X` | Comments on reading |
| `FILE_FSET` | OTHER | `X` | Associated file reference (e.g. monitoring field sheets, instrument logging file) |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LOCA_ID`, `MONG_ID`, `MONG_DIS`, `MOND_DTIM`, `MOND_TYPE`, `MOND_REF`. Children (0): _none_. As a child it **denormalises** its parent's KEY columns into every row; [[rule-10c-parent-child]] re-resolves that repeated tuple upward to [[MONG]]. See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[MONG]]
