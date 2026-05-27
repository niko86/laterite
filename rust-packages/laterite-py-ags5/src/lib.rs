//! PyO3 module `laterite_ags5._laterite_ags5_native`.
//!
//! The experimental AGS5 (`.ags5db`) surface — DuckDB-backed read /
//! write / query / convert / diff / attach_blobs. Ships as the
//! separate `laterite-ags5` PyPI wheel pulled in by the
//! `laterite[ags5]` extra. The base `laterite._laterite_native` wheel
//! has none of this (no DuckDB bundled).
//!
//! Typed-graph classes (`PROJ`, `LOCA`, …, all 92 standard groups)
//! live in the base wheel. `read_db` / `write_db` here look them up
//! at runtime via `py.import("laterite._laterite_native").getattr(...)`
//! so no FFI type sharing is needed — each cdylib has its own
//! independent PyO3 init.

use pyo3::prelude::*;

mod ags5db_fns;
mod typed_graph;

#[pymodule]
fn _laterite_ags5_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    ags5db_fns::register(m)?;
    typed_graph::register(m)?;
    Ok(())
}
