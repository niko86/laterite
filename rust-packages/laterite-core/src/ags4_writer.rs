//! AGS4 emitter — **moved** to the wasm-safe `laterite-ags4-emit` leaf so the
//! browser host can produce AGS4 without `laterite-core`'s wasm-hostile deps
//! (age/zstd/calamine/rust_xlsxwriter). This module is a thin re-export
//! kept so existing `laterite_core::ags4_writer::{EmitGroup, write_ags4}`
//! call sites (excel.rs, ags5db's convert.rs + lib.rs re-export) keep
//! resolving unchanged.
//!

pub use laterite_ags4_emit::{EmitGroup, write_ags4};
