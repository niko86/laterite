//! AGS4 group registry — re-exported from the `laterite-ags4-reference` leaf.
//!
//! The union-projection code (the `GroupDescriptor`/`Heading` structs,
//! `union_groups`, the `Registry` singleton, and the parent-chain walks) moved
//! into `laterite-ags4-reference` in #475 (PR1) so consumers that only need the
//! dictionary — the read-only DuckDB extension, `laterite-ags4-diff` — can
//! depend on that leaf without pulling in the rest of core.
//!
//! This module preserves the historical `laterite_ags4_core::registry::…` path
//! via a flat re-export, so every existing `use` (core's own `keychain`, plus
//! `laterite-py`/`laterite-node`/`laterite-ags4-wasm`/`ags4-compliance`) keeps
//! compiling unchanged.
pub use laterite_ags4_reference::union::*;
