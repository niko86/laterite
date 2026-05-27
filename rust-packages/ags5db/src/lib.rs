//! `ags5db` library — the CLI-dep-free logic of the AGS5 `.ags5db`
//! toolkit: AGS4↔.ags5db conversion, DuckDB DDL/query helpers, the
//! registry, and the AGS4 codec.
//!
//! The `ags5db` binary (`src/main.rs`) is a thin CLI shim over this
//! lib (clap arg parsing + `output` rendering live only in the bin).
//! `laterite-py` depends on this lib to expose conversion to Python —
//! The lib pulls NO clap / comfy-table / indicatif. `.agsx` retired in
//! Stage F2a — `.agsx` is now a Python-only inspection helper, not a
//! pipeline format.

// S3a (release/v0.1.0-prep): the DuckDB-free modules moved out to
// `ags5-core`. Re-exported here for source compat — external code
// can still write `use ags5db::registry::…` and resolve transparently.
pub use ags5_core::{ags_types, ags4_codec, ags4_writer, ddl, error, excel, registry, transport};

// DuckDB-bound modules — stay in ags5db.
pub mod attachments;
pub mod conn;
pub mod convert;
pub mod db;
pub mod diff;
pub mod introspect;
pub mod predicate;
pub mod query;
pub mod recipes;
pub mod spec_tables;
pub mod suggest;
pub mod uuid7;
pub mod writer;
