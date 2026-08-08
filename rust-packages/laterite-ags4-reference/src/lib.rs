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
// Runtime-owned custom-dictionary overlay (#568): the sparse delta a `--dict`
// override contributes over a bundled base. `dict::Dictionary::Layered` borrows it.
pub mod overlay;
// The runtime `.ags` DICT-group reader `overlay::parse_dict` dispatches to for the
// `.ags` custom-dict format (the JSON format reuses `union`'s serde). Crate-private:
// its one entry point is `pub(crate)`, reached only through `overlay`.
mod dict_read;
// Content-addressed row keys (`_id`/`_parent_id`) + the single `key_heading_names`
// definition of row identity. Relocated from core (row-identity consolidation)
// so `laterite-ags4-diff` — which depends on this leaf, not core — shares the
// same KEY-heading derivation instead of re-deriving its own.
pub mod keychain;
pub mod union;

// The README's example is a doctest, not a second copy of one. `cfg(doctest)`
// means this module exists only while rustdoc collects doctests: it is absent
// from a normal build and from the rendered docs.rs page, so the crate's own
// `//!` docs are untouched and nothing is duplicated. The README is the single
// source, and `cargo test --workspace` already compiles it.
//
// The example is written out in full — no rustdoc `# ` hidden lines. A README is
// also read as plain Markdown on crates.io, where `# let x = …` renders as an
// <h1>. Visible boilerplate is the price of a page that is checked AND readable.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme_doctests {}
