//! Typed-graph IO functions exposed to the AGS5 wheel.
//!
//! Unlike the base `laterite-py` `typed_graph` (which `include!`s the
//! 92 `#[pyclass]` codegen from `build.rs`), this side only registers
//! the IO functions `ags5db_read_db` / `ags5db_write_db` /
//! `ags5db_attach_blobs`. The class types they consume are looked up
//! at runtime from `laterite._laterite_native` — no FFI sharing.

pub mod blobs;
pub mod read;
pub mod write;

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(read::ags5db_read_db, m)?)?;
    m.add_function(wrap_pyfunction!(write::ags5db_write_db, m)?)?;
    m.add_function(wrap_pyfunction!(blobs::ags5db_attach_blobs, m)?)?;
    Ok(())
}
