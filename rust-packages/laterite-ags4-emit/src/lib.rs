//! `laterite-ags4-emit` — produce valid AGS4 plaintext from typed/string data.
//!
//! One shared, host-agnostic orchestrator ([`emit_ags4`]) over two thin
//! frontends (the native PyO3 binding and the browser wasm binding). The
//! byte-level [`write_ags4`] writer lives here too (moved out of
//! `laterite-ags4-core` so the wasm host can reach it without the wasm-hostile
//! `laterite-ags4-core` deps).
//!

#[cfg(feature = "arrow")]
mod arrow_in;
mod emit;
mod error;
mod writer;

#[cfg(feature = "arrow")]
pub use arrow_in::{
    cell_value, group_from_arrow, group_from_arrow_with_meta, group_from_arrow_with_meta_at_edition,
};
pub use emit::{
    EmitMode, EmitOpts, EmitResult, GroupInput, TranStamp, TranStampError, emit_ags4,
    emit_ags4_owned,
};
pub use error::EmitError;
pub use writer::{EmitGroup, write_ags4, write_ags4_matrix};

// Re-export the edition enum so callers configure emit without taking a
// direct `laterite-ags4-validator` dependency just for the type.
pub use laterite_ags4_validator::DictVersion;

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
