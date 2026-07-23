---
type: edition
title: AGS 4.1
status: drafted
tags: [edition]
edition: 4.1
released: 2020-12-08
ags_status: deprecated
repo_refs:
  spec_pdf: "spec:AGS4-4.1-2020.pdf"
  resolution_logic: "repo:rust-packages/laterite-ags4-validator/src/lib.rs::resolve_dict_version"
related: [edition-resolution, O-30]
sources: [spec-4.1]
---
# AGS 4.1

## Overview
AGS **Edition 4.1 (new minor version)** — released 2020-12-08, status **deprecated**. Spec PDF: `spec:AGS4-4.1-*.pdf` (vault root). New minor version. Data Dictionary is the JSON authority's baseline (ags5_dictionary.json ags_edition=4.1). Rule prose frozen.

## Support in this repo
The validator resolves the dictionary edition from `TRAN_AGS` — see [[edition-resolution]] and [[O-30]] (bare `"4"`→4.0.4, AGS3 refusal). **Rule prose is identical to every other in-scope edition** ([[ags4-rules-frozen-dictionary-evolves]]); only the data model differs.

## Deltas vs adjacent editions
Dictionary lineage: New minor version. Data Dictionary is the JSON authority's baseline (ags5_dictionary.json ags_edition=4.1). Rule prose frozen. The numbered Rules are identical to 4.2 (frozen — [[ags4-rules-frozen-dictionary-evolves]]); deltas vs adjacent editions are **dictionary-only** (group/heading/type additions, deprecations, retypes). Per-group deltas are filled on the `groups/*` pages.

## Related
[[edition-resolution]] · [[O-30]] · [[start-here]]
