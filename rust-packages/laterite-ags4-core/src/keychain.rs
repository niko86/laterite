//! Deterministic content-addressed keys — re-exported from the
//! `laterite-ags4-reference` leaf.
//!
//! The keychain (`row_ids`/`content_id`/`canonical_encode`/`group_row_ids` plus
//! the `key_heading_names` single-source of row identity) moved into
//! `laterite-ags4-reference` (the row-identity consolidation) so
//! `laterite-ags4-diff` — which depends on the reference leaf, not core — shares
//! the SAME KEY-heading derivation instead of re-deriving its own. This module
//! preserves the historical `laterite_ags4_core::keychain::…` path via a flat
//! re-export, so core's own `index` plus
//! `laterite-py`/`laterite-node`/`laterite-ags4-wasm`/`ags4-compliance` keep
//! compiling unchanged (mirrors the `registry.rs` re-export precedent).
pub use laterite_ags4_reference::keychain::*;
