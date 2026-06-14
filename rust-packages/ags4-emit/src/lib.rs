//! `ags4-emit` — produce valid AGS4 plaintext from typed/string data.
//!
//! One shared, host-agnostic orchestrator ([`emit_ags4`]) over two thin
//! frontends (the native PyO3 binding and the browser wasm binding). The
//! byte-level [`write_ags4`] writer lives here too (moved out of
//! `ags5-core` so the wasm host can reach it without the wasm-hostile
//! `ags5-core` deps).
//!

#[cfg(feature = "arrow")]
mod arrow_in;
mod emit;
mod error;
mod writer;

#[cfg(feature = "arrow")]
pub use arrow_in::{cell_value, group_from_arrow};
pub use emit::{EmitMode, EmitOpts, EmitResult, GroupInput, emit_ags4};
pub use error::EmitError;
pub use writer::{EmitGroup, write_ags4};

// Re-export the edition enum so callers configure emit without taking a
// direct `ags4-validator` dependency just for the type.
pub use ags4_validator::DictVersion;
