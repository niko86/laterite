//! Typed-graph engine — Rust-side `#[pyclass]` per AGS group.
//!
//! Stage F2b-1: scalar-only classes generated at Rust build time by
//! `build.rs` from `ags_dictionary.json`. Each of the 174 AGS4
//! groups becomes one `#[pyclass]` with `Option<T>` scalar fields and
//! a kwargs `#[new]` constructor. Children (`Py<PyList>`), `walk()`,
//! passthrough, `read/write_db`, and `.pyi` stubs land in F2b-2 onward.
//!
//! The generated file lives at `$OUT_DIR/typed_groups.rs`; the
//! `groups` submodule below pulls it in via `include!`. The flat
//! `register()` fn walks every generated class and adds it to the
//! given Python module.
//!
//! Why include! over a proc-macro: see

// AGS group codes are inherently 4-letter acronyms (PROJ, LOCA, ...);
// the codegen emits `format!("CODE(...)")` for `__repr__` and similar
// idioms clippy flags as `useless_format`. Allow at the module level
// so the generated body inside `include!` doesn't trigger them. Also
// blankets pedantic here (chore/clippy-pedantic): generated code can't
// be hand-fixed since fixes vanish on the next `build.rs` regen.
#[allow(clippy::upper_case_acronyms, clippy::useless_format, clippy::pedantic)]
pub mod groups {
    // The generated file is the body of this module: it defines one
    // `#[pyclass]` struct per AGS group plus a `register()` fn that
    // adds them all to a given PyModule.
    include!(concat!(env!("OUT_DIR"), "/typed_groups.rs"));
}

// S3b (release/v0.1.0-prep): `read`/`write`/`blobs` moved to the
// separate `laterite-py-ags5` cdylib, decoupled to the dormant `ags5/`
// holding folder (#177). The base wheel keeps the 174 typed-graph
// classes — IO functions on that side resolve them at runtime via
// `py.import("laterite._laterite_native")`.

use pyo3::prelude::*;

/// Register every typed-graph class on `m`. Called from
/// `_laterite_native`'s top-level `#[pymodule]` init.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    groups::register(m)?;
    Ok(())
}
