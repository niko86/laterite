---
type: group
title: FILE — Associated Files
status: drafted
tags: [group]
group_code: FILE
parent: ""
is_high_volume: false
varies_between_editions: false
key_headings: [FILE_FSET, FILE_NAME]
required_headings: []
ags_editions: [4.0.3, 4.0.4, 4.1, 4.1.1, 4.2]
repo_refs:
  dictionary: "repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=FILE]"
related: [parent-child-graph, key-tuple-pseudo-keys, heading-status-vocabulary, rule-10c-parent-child]
sources: []
---
# FILE — Associated Files

## Purpose
> [!quote] The **FILE** group — Associated Files. It is a **root / non-hierarchy** group (file submission & description — Rules 13–18 territory). See [[parent-child-graph]].

## Position in the model

```mermaid
erDiagram
  FILE {
    KEY FILE_FSET
    KEY FILE_NAME
  }
```

- Parent: _(root — no parent)_
- Children: _none_
- See [[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]]

## Headings
> [!quote] Rendered from `repo:rust-packages/laterite-ags4-reference/data/ags_dictionary.json groups[code=FILE]` (the repo's model authority — AGS edition 4.2). Suggested UNITs + worked examples are in the cited spec PDF, not duplicated here.

8 heading(s) — `**KEY**` = pseudo-key tuple, `*REQ*` = REQUIRED (non-null, Rule 10b), `DEP` = deprecated, OTHER = scope-dependent.

| Heading | Status | Type | Description |
|---|---|---|---|
| `FILE_FSET` | **KEY** | `X` | File set reference |
| `FILE_NAME` | **KEY** | `X` | File name |
| `FILE_DESC` | OTHER | `X` | Description of content |
| `FILE_TYPE` | OTHER | `PA` | File type |
| `FILE_PROG` | OTHER | `X` | Parent program and version number |
| `FILE_DOCT` | OTHER | `PA` | Document type |
| `FILE_DATE` | OTHER | `DT` | File date |
| `FILE_REM` | OTHER | `X` | Comments on file |

Full cross-edition heading deltas: AGS Change Log (see [[ags4-rules-frozen-dictionary-evolves]]).

## Relational notes
KEY tuple: `FILE_FSET`, `FILE_NAME`. Children (0): _none_. Parent linkage is implicit/absent — Rule 10c is skipped for root groups (see [[non-hierarchy-ten-vs-parentless-list]]). See [[key-tuple-pseudo-keys]] · [[denormalised-child-rows]].

## Variations
No group-level change at 4.2 (present across the in-scope editions). Granular per-heading edition deltas live in the AGS online **Change Log** — the spec's own cited delta source (`spec:AGS4-4.2-2025.pdf` Foreword → ags.org.uk/.../change-log). Heading-level archaeology is deferred to a targeted Ingest if a rule/O-N interaction needs it (per [[ags4-rules-frozen-dictionary-evolves]]).

## Related
[[parent-child-graph]] · [[key-tuple-pseudo-keys]] · [[heading-status-vocabulary]] · [[rule-10c-parent-child]]
