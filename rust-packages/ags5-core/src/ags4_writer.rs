//! AGS4 emitter — **moved** to the wasm-safe `ags4-emit` leaf so the
//! browser host can produce AGS4 without `ags5-core`'s wasm-hostile deps
//! (age/zstd/calamine/rust_xlsxwriter). This module is a thin re-export
//! kept so existing `ags5_core::ags4_writer::{EmitGroup, write_ags4}`
//! call sites (excel.rs, ags5db's convert.rs + lib.rs re-export) keep
//! resolving unchanged.
//!

pub use ags4_emit::{EmitGroup, write_ags4};
