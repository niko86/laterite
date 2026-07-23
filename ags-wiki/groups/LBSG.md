---
type: group
title: LBSG — Testing Schedule
status: drafted
tags: [group]
group_code: LBSG
parent: ""
is_high_volume: false
varies_between_editions: false
key_headings: [LBSG_REF]
required_headings: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=LBSG]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child, LBST]
sources: []
---
# LBSG — Testing Schedule

## Purpose
> [!quote] The **LBSG** group — Testing Schedule. It is a **root / non-hierarchy** group (file submission & description — Rules 13–18 territory). See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  LBSG ||--o{ LBST : has
  LBSG {
    KEY LBSG_REF
  }
```

- Parent: _(root — no parent)_
- Children: [[LBST]]
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=LBSG]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

8 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `LBSG_REF` | **KEY** | `X` | Schedule reference |
| `LBSG_DATE` | OTHER | `DT` | Date of issue |
| `LBSG_FROM` | OTHER | `X` | Schedule prepared by |
| `LBSG_TO` | OTHER | `X` | Schedule issued to |
| `LBSG_DUE` | OTHER | `DT` | Date schedule to be completed and reported |
| `LBSG_REM` | OTHER | `X` | Comments on schedule |
| `LBSG_STAT` | OTHER | `X` | Status of schedule |
| `FILE_FSET` | OTHER | `X` | Associated file reference (e.g. schedule sheets) |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `LBSG_REF`. Children (1): [[LBST]]. Parent linkage is implicit/absent — Rule 10c is skipped for root groups (see [[non-hierarchy-ten-vs-parentless-list]]). See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]] · [[LBST]]
