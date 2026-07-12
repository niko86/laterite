//! AGS4 reference data — a wasm-safe leaf, single-sourced from
//! `ags_dictionary.json`.
//!
//! Extracted from `laterite-ags4-core` (#475) so consumers that only need the
//! dictionary — the read-only DuckDB extension, `laterite-ags4-diff` — can
//! depend on this leaf instead of pulling in the rest of core (or the whole
//! validator). PR1 carried the [`union`] registry projection. PR2 adds the
//! per-edition phf projection ([`dict`], moved from the validator's
//! `build.rs`) and the rules-catalogue data accessors ([`catalogue`]) — the
//! faithfulness gate that cross-checks the catalogue against the engine's own
//! emissions stays in the validator, since it needs the fix engine's
//! internals. A final commit relocates the bundled JSON files themselves into
//! this leaf's own `data/`, at which point the leaf owns them outright.

pub mod catalogue;
pub mod dict;
pub mod union;
