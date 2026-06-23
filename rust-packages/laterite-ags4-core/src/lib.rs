//! `laterite-ags4-core` — DuckDB-free pure-string AGS modules.
//!
//! Extracted from `ags5db` in S3a of `release/v0.1.0-prep`. This crate
//! holds the modules that don't need DuckDB:
//!
//! - [`registry`] — the AGS4 multi-edition dictionary union (loaded at
//!   build time from `data/ags_dictionary.json`) + group-tree descriptors.
//! - [`ags_types`] — canonical type system (ID, X, 1DP, 2DP, PA, …)
//!   + value parsing/formatting.
//! - [`ags4_codec`] — AGS4 reader (CRLF lines, double-quoted CSV).
//! - [`transport`] — zstd + age envelope for the `pack`/`unpack`/
//!   `lock`/`unlock` CLI commands.
//! - [`error`] — `CliError` shared across both crates.
//!
//! `ags5db` re-exports every module from here for source compat, so
//! external consumers can still write `use laterite_ags5_db::registry::…` and it
//! resolves transparently. Internal `ags5db` source uses
//! `laterite_ags4_core::…` directly.

pub mod ags4_codec;
// The AGS type system now lives in the wasm-safe leaf crate `laterite-types`
// (so `laterite-ags4-wasm` can share the exact casting logic). Re-exported here
// as `laterite_ags4_core::ags_types` so every existing consumer —
// ags5db's {convert,query,spec_tables}, the `laterite_ags5_db::ags_types` 2nd-hop
// re-export, laterite-py — keeps working unchanged.
pub use laterite_types as ags_types;
pub mod error;
pub mod index;
pub mod keychain;
pub mod registry;
pub mod transport;
