//! `laterite-ags4-core` — DuckDB-free pure-string AGS modules.
//!
//! Extracted from the original DuckDB-backed engine crate in S3a of
//! `release/v0.1.0-prep`. This crate holds the modules that don't need DuckDB:
//!
//! - [`registry`] — re-exports the AGS4 multi-edition dictionary union
//!   (`laterite-ags4-reference::union`, moved out in laterite-dev#475) + group-tree
//!   descriptors.
//! - [`ags_types`] — canonical type system (ID, X, 1DP, 2DP, PA, …)
//!   + value parsing/formatting.
//! - [`ags4_codec`] — AGS4 reader (CRLF lines, double-quoted CSV).
//! - [`transport`] — zstd + age envelope for the `pack`/`unpack`/
//!   `lock`/`unlock` CLI commands.
//! - [`error`] — `CliError` shared across both crates.
//!
//! That original crate re-exports every module from here for source
//! compat, so its external consumers can still use the pre-split
//! import path and it resolves transparently. Its own internal source
//! uses `laterite_ags4_core::…` directly.

pub mod ags4_codec;
// The AGS type system now lives in the wasm-safe leaf crate `laterite-ags4-types`
// (so `laterite-ags4-wasm` can share the exact casting logic). Re-exported here
// as `laterite_ags4_core::ags_types` so every existing consumer — the
// original DuckDB-backed crate's {convert,query,spec_tables} modules and its
// own 2nd-hop `ags_types` re-export, laterite-py — keeps working unchanged.
pub use laterite_ags4_types as ags_types;
// The Rule 18 effective dictionary (standard ∪ the file's own DICT group),
// homed in `laterite-ags4-reference` beside the dictionary it unions with and
// re-exported here — the same move as `registry` — plus the adapter for this
// crate's own read codec, so a read-only consumer that takes only this crate
// can bind a file-declared group's columns (#777).
pub mod effective_dict;
pub mod error;
pub mod index;
pub mod keychain;
// The `lat read` output formats. Lives beside the read codec that produces the
// rows: every surface (the `lat` binary, laterite-py, laterite-node) renders
// `read --json` / `--csv` through these, instead of each hand-writing its own
// CSV quoter and reaching for its own JSON library (laterite-dev#530).
pub mod read_render;
pub mod registry;
// `transport` (pack/unpack/lock/unlock) is behind the default-on `transport`
// feature — its `age` dep pulls getrandom, which doesn't build on wasm32. A
// wasm-safe consumer takes `default-features = false` for the pure keychain /
// registry / codec. (#303 Phase 5)
#[cfg(feature = "transport")]
pub mod transport;

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
