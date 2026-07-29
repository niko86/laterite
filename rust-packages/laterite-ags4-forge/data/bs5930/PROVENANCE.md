# Vendored BS 5930 soil-description data

These three JSON files are vendored verbatim from a **private first-party
repository** — the owner's own field-data app — specifically its
`bs5930-soil-description` skill:

- **source:** a private first-party repository (name withheld; this repo is a
  public mirror). The owner can resolve it from the commit below.
- **path:** `.agents/skills/bs5930-soil-description/data/`
- **commit:** `3f93315b6e905e9f8563dfdddca9b52aac89065a`
- **retrieved:** 2026-06-18

| File | What it holds |
|---|---|
| `terms.json` | term vocabularies — strength/consistency, relative density, colour (lightness/chroma/hue), discontinuities, bedding, angularity, shape, grading, mineral constituents, carbonate, tertiary amounts, plasticity, peat, secondary-organic |
| `proportion-rules.json` | machine-usable secondary-constituent bands per principal class (Tables 16/17), the mutual exclusions, the cumulative-≤100% rule, footnotes, worked examples |
| `particle-sizes.json` | particle-size fractions with mm boundaries + coarse/medium/fine sub-bands (Table 7) |

**Provenance note (from the source):** *factual data (term vocabularies,
numeric thresholds, table structure, decision logic) extracted for the
app's own description builder — NOT a reproduction of BS 5930:2015+A1:2020.*
Rock content (Clause 36) is reference-only in the source and not used here.

The forge's `synth::bs5930` module parses `terms.json` + `particle-sizes.json`
for the open vocabularies (so they stay synced to this vendored source) and
encodes the proportion bands + constraint engine in Rust (citing
`proportion-rules.json` Tables 16/17). If these files are refreshed from a
newer skill revision, re-run the forge tests — the constraint asserts pin
the band semantics, not the exact term lists.
