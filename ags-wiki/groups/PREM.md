---
type: group
title: PREM — Project Specific Time Related Remarks
status: drafted
tags: [group]
group_code: PREM
parent: ""
is_high_volume: false
varies_between_editions: false
key_headings: [PREM_DTIM]
required_headings: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=PREM]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child]
sources: []
---
# PREM — Project Specific Time Related Remarks

## Purpose
> [!quote] The **PREM** group — Project Specific Time Related Remarks. It is a **root / non-hierarchy** group (file submission & description — Rules 13–18 territory). See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  PREM {
    KEY PREM_DTIM
  }
```

- Parent: _(root — no parent)_
- Children: _none_
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=PREM]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

6 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `PREM_DTIM` | **KEY** | `DT` | Date and time of remark or start of event |
| `PREM_COMP` | OTHER | `X` | Component or sub-activity |
| `PREM_REM` | OTHER | `X` | Time related remark |
| `PREM_DURN` | OTHER | `T` | Duration of event or activity |
| `PREM_ETIM` | OTHER | `DT` | Date and time of end of event |
| `FILE_FSET` | OTHER | `X` | Associated file reference (e.g. site journal records) |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `PREM_DTIM`. Children (0): _none_. Parent linkage is implicit/absent — Rule 10c is skipped for root groups (see [[non-hierarchy-ten-vs-parentless-list]]). See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]]
