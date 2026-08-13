---
type: edition
title: AGS 4.0.3
status: drafted
tags: [edition]
edition: 4.0.3
released: 2011-10-01
ags_status: deprecated
repo_refs:
  spec_pdf: "spec:AGS4-4.0.3-Addendum-3.pdf"
  resolution_logic: "repo:rust-packages/laterite-ags4-validator/src/lib.rs::resolve_dict_version"
related: [edition-resolution, O-30]
sources: [spec-4.0.3]
---
# AGS 4.0.3

## Overview
AGS **4th Edition Addendum 3** — released 2011-10-01, status **deprecated**. Spec PDF: `spec:AGS4-4.0.3-*.pdf` (not vendored here — see the `sources/spec-*` page). Addendum to 4.0.2. Earliest edition in scope. Rule prose as in 4.2 §4.1.1 (frozen).

## Support in this repo
The validator resolves the dictionary edition from `TRAN_AGS` — see [[edition-resolution]] and [[O-30]] (bare `"4"`→4.0.4, AGS3 refusal). **Rule prose is identical to every other in-scope edition** ([[ags4-rules-frozen-dictionary-evolves]]); only the data model differs.

## Deltas vs adjacent editions
Dictionary lineage: Addendum to 4.0.2. Earliest edition in scope. Rule prose as in 4.2 §4.1.1 (frozen). The numbered Rules are identical to 4.2 (frozen — [[ags4-rules-frozen-dictionary-evolves]]); deltas vs adjacent editions are **dictionary-only** (group/heading/type additions, deprecations, retypes). Per-group deltas are filled on the `groups/*` pages.

## Related
[[edition-resolution]] · [[O-30]] · [[start-here]]
