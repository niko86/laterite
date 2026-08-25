---
type: concept
title: BS 5930 soil descriptions
status: drafted
tags: [concept, geotech, synthesis]
ags_editions: []
repo_refs:
  generator: "repo:rust-packages/laterite-ags4-forge/src/synth/bs5930.rs"
  data: "repo:rust-packages/laterite-ags4-forge/data/bs5930/"
related: [laterite-ags4-forge]
sources: []
---
# BS 5930 soil descriptions

## Definition

The forge's `GEOL_DESC` engine: a **constraint-valid** generator of
**BS 5930:2015+A1:2020 Section 6** field soil descriptions, so the
synthetic AGS4 files the forge produces are realistic at the *geotechnical*
layer (proper strata descriptions), not merely structurally valid. Lives at
`repo:rust-packages/laterite-ags4-forge/src/synth/bs5930.rs`; previewed with
`laterite-ags4-forge describe --count N --seed S`.

## How it works

- **Vendored vocabularies.** The open term lists (strength/consistency,
  relative density, colour lightness·chroma·hue, particle angularity, size
  sub-bands) are parsed at runtime (`include_str!` + serde) from the
  vendored skill data at `repo:rust-packages/laterite-ags4-forge/data/bs5930/`
  (`terms.json`, `particle-sizes.json`) — copied verbatim from a private
  first-party repository's `bs5930-soil-description` skill (see that dir's
  `PROVENANCE.md` for the pinned commit; the repository is deliberately
  unnamed here, as this repo is a public mirror). They stay synced to that
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

The **default** draw is natural inorganic **coarse** (SAND/GRAVEL) and
**fine** (SILT/CLAY) — the two lanes that share the standard word order.

**Organic ground is a third lane, opt-in behind `describe --organic`** (#697).
It covers organic coarse/fine soils (Table 20's *slightly/very organic*
qualifiers, carried as an ordinary secondary so the standard word order still
holds) and **PEAT** as a principal in its own right, led by Table 21
condition rather than consistency or relative density, and typed by clause
33.4.6 (fibrous / pseudo-fibrous / amorphous). Peat draws from its own hue
set: the general colour palette is inorganic-plausible and reads wrong on
peat. The **von Post humification scale is deliberately absent** — the source
standard does not use it for field description, and inventing a scale reading
would be a fabricated measurement rather than a generated description.

The lane is opt-in because the draw is a shared random sequence: adding a
third branch shifts every subsequent draw, so every seed would produce a
different file. See [[dec-forge-audience-boundary]] for who that breaks and
what flipping the default would oblige.

Very coarse (BOULDERS/COBBLES) and anthropogenic ground use different
apportionment/word orders and remain staged for later (as the source skill
itself stages them).

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

[[laterite-ags4-forge]]
