//! `ags5-core` — DuckDB-free pure-string AGS modules.
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
//! external consumers can still write `use ags5db::registry::…` and it
//! resolves transparently. Internal `ags5db` source uses
//! `ags5_core::…` directly.

pub mod ags4_codec;
pub mod ags4_writer;
pub mod ags_types;
pub mod ddl;
pub mod error;
pub mod excel;
pub mod registry;
pub mod transport;
