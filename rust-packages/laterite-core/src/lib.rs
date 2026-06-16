//! `laterite-core` — DuckDB-free pure-string AGS modules.
//!
//! Extracted from `ags5db` in S3a of `release/v0.1.0-prep`. This crate
//! holds the modules that don't need DuckDB:
//!
//! - [`registry`] — the 92-group AGS5 dictionary (loaded at build time
//!   from `data/ags5_dictionary.json`) + group-tree descriptors.
//! - [`ags_types`] — canonical type system (ID, X, 1DP, 2DP, PA, …)
//!   + value parsing/formatting.
//! - [`ddl`] — pure-string DDL emitter (no DuckDB connection needed).
//! - [`ags4_codec`] — AGS4 reader (CRLF lines, double-quoted CSV).
//! - [`ags4_writer`] — AGS4 spec-correct emitter (CRLF, every field
//!   quoted, `"` → `""`, blank line between groups).
//! - [`excel`] — AGS4 ↔ XLSX (calamine reader + rust_xlsxwriter writer).
//! - [`transport`] — zstd + age envelope for the `pack`/`unpack`/
//!   `lock`/`unlock` CLI commands.
//! - [`error`] — `CliError` shared across both crates.
//!
//! `ags5db` re-exports every module from here for source compat, so
//! external consumers can still write `use laterite_ags5_db::registry::…` and it
//! resolves transparently. Internal `ags5db` source uses
//! `laterite_core::…` directly.

pub mod ags4_codec;
pub mod ags4_writer;
// The AGS type system now lives in the wasm-safe leaf crate `laterite-types`
// (so `laterite-ags4-wasm` can share the exact casting logic). Re-exported here
// as `laterite_core::ags_types` so every existing consumer — ddl.rs,
// ags5db's {convert,query,spec_tables}, the `laterite_ags5_db::ags_types` 2nd-hop
// re-export, laterite-py — keeps working unchanged.
pub use laterite_types as ags_types;
pub mod error;
pub mod excel;
pub mod registry;
pub mod transport;
