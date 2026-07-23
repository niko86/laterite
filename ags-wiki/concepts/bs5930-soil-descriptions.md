---
type: concept
title: BS 5930 soil descriptions
status: drafted
tags: [concept, geotech, synthesis]
ags_editions: []
repo_refs:
  generator: "repo:rust-packages/ags4-forge/src/synth/bs5930.rs"
  data: "repo:rust-packages/ags4-forge/data/bs5930/"
related: [ags4-forge]
sources: []
---
# BS 5930 soil descriptions

## Definition

The forge's `GEOL_DESC` engine: a **constraint-valid** generator of
**BS 5930:2015+A1:2020 Section 6** field soil descriptions, so the
synthetic AGS4 files the forge produces are realistic at the *geotechnical*
layer (proper strata descriptions), not merely structurally valid. Lives at
`repo:rust-packages/ags4-forge/src/synth/bs5930.rs`; previewed with
`ags4-forge describe --count N --seed S`.

## How it works

- **Vendored vocabularies.** The open term lists (strength/consistency,
  relative density, colour lightness·chroma·hue, particle angularity, size
  sub-bands) are parsed at runtime (`include_str!` + serde) from the
  vendored skill data at `repo:rust-packages/ags4-forge/data/bs5930/`
  (`terms.json`, `particle-sizes.json`) — copied verbatim from the owner's
  `solmek-field-app` `bs5930-soil-description` skill (see that dir's
  `PROVENANCE.md` for the source ref + commit). They stay synced to that
  source.
- **The constraint engine (Rust).** The secondary-constituent proportion
  bands (Tables 16/17), the cumulative-≤100% rule, the silty/clayey mutual
  exclusion, the coarse-then-fine word order and the colour order are
  encoded in Rust, citing the vendored `proportion-rules.json`. The crux:
  percentages are **drawn first** within a shared budget that respects the
  cumulative rule *by construction*, then mapped to their band qualifier —
  so the generator can never emit the impossible "very sandy very gravelly
  CLAY" (65%+65% > 100%). Picking a high sand fraction forces gravel down a
  band, exactly as the standard requires.
- **Variety is combinatorial.** principal × strength/density × (lightness ×
  chroma × hue) × secondaries × bands → millions of distinct,
  standard-compliant descriptions from a seed (same seed → identical text).
  Realism guards: no chroma before an achromatic hue (white/grey/black/
  cream), no redundant same-family chroma ("greenish green"), no
  contradictory lightness ("light black").

The constraint asserts (cumulative ≤100, mutual exclusion, qualifier↔band,
no-double-very, word order, colour rules, determinism) are pinned by unit
tests, so a vocab refresh can't silently break the rules.

## Scope

v1 covers natural inorganic **coarse** (SAND/GRAVEL) and **fine**
(SILT/CLAY) soils — the two lanes that share the standard word order. Very
coarse (BOULDERS/COBBLES), peat/organic and anthropogenic ground use
different apportionment/word orders and are staged for later (as the source
skill itself stages them).

## In the synthetic files

The engine is wired into the `loca-samp` borehole scaffold as a **`GEOL`**
stratum group (a child of `LOCA`; KEY `LOCA_ID`+`GEOL_TOP`, REQUIRED
`GEOL_BASE`+`GEOL_DESC`). Each borehole gets 2–5 **contiguous** strata
(tops climb from `0.00`, each `GEOL_TOP` == the previous `GEOL_BASE`), so
the KEY is unique, the parent LOCA always exists, and the depths read like
a real log — `GEOL` stays clean-by-construction (the `varied_baseline`
RustResult::Clean guard now covers it). The leading term obeys the lane:
*consistency* (Firm/Stiff…) for SILT/CLAY, *relative density* (Medium
dense…) for SAND/GRAVEL.

## Related

[[ags4-forge]]
