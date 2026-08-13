---
type: edition
title: AGS 4.2
status: drafted
tags: [edition]
edition: 4.2
released: 2025-12-31
ags_status: current
repo_refs:
  spec_pdf: "spec:AGS4-4.2-2025.pdf"
  resolution_logic: "repo:rust-packages/laterite-ags4-validator/src/lib.rs::resolve_dict_version"
related: [edition-resolution, O-30]
sources: [spec-4.2]
---
# AGS 4.2

## Overview
AGS **Edition 4.2 (current, Dec 2025)** — released 2025-12-31, status **current**. Spec PDF: `spec:AGS4-4.2-*.pdf` (not vendored here — see the `sources/spec-*` page). Current. Foreword: 'The AGS 4 Rules remain unchanged.' All deltas are Data Dictionary changes (below).

## Support in this repo
The validator resolves the dictionary edition from `TRAN_AGS` — see [[edition-resolution]] and [[O-30]] (bare `"4"`→4.0.4, AGS3 refusal). **Rule prose is identical to every other in-scope edition** ([[ags4-rules-frozen-dictionary-evolves]]); only the data model differs.

## Deltas vs adjacent editions
> [!note] 4.2 Foreword: *"The AGS 4 Rules remain unchanged."* Every 4.2 change is a **Data Dictionary** change:

- SCPx/SCTx (Static Cone Penetration & Dissipation) DEPRECATED → use CPDx/CPTx
- Pressuremeter: new Ménard groups PMMx; new PMTP (parameters) + PMTZ (zeros) extensions; PMTD data types revised
- Flat Dilatometer / Dissipation: new DMDx/DMTx
- In situ Seismic Testing: new ISTx
- Monitoring installations & instrument status: new MONS group
- Test operator added to all in situ groups
- In situ thermal conductivity: new ITCH group
- California Bearing Test Readings: new CBRP group
- ERES REMOVED from 4.2 → use ELRG
- IPRG/IPRT REMOVED → use FGHx
- Advanced testing CTRx/RESx DEPRECATED → see AGS-L (publish 2026)

**AGS-L draft groups** (the AGS Library extension, expected publish 2026 — **not** part of the AGS4 4.x standard, so absent from the `ags_dictionary.json` union that drives the generated group tier): [[CONL]] · [[TREL]] · [[TRIL]] are carried as hand-authored draft pages, anchored here (their parents are AGS4 groups whose children lists are dictionary-generated and so don't include them).

Full change log: AGS website `ags-4-2-0/change-log`. These deltas drive the `groups/*` `## Variations` (Phase A cont.) and the AGS5 model-stability analysis.

## Related
[[edition-resolution]] · [[O-30]] · [[start-here]] · [[CONL]] · [[TREL]] · [[TRIL]]
