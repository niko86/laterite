//! PyO3 bindings for `ags5_core::transport` (zstd + age passphrase).
//!
//! Stage F2a-2c: exposes the lib API so `laterite.transport` can
//! drive pack/unpack/lock/unlock from Python without subprocessing
//! the binary.

use std::path::Path;

use ags5_core::transport;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyModule};
use pyo3::wrap_pyfunction;

use crate::map_cli_err;

/// zstd-compress a `.ags5db` to `dest`. Returns `{bytes, ratio,
/// elapsed_s}`.
#[pyfunction]
#[pyo3(signature = (src, dest, level = 9))]
fn ags5db_pack<'py>(
    py: Python<'py>,
    src: String,
    dest: String,
    level: i32,
) -> PyResult<Bound<'py, PyDict>> {
    let stats = transport::pack(Path::new(&src), Path::new(&dest), level).map_err(map_cli_err)?;
    let d = PyDict::new(py);
    d.set_item("bytes", stats.bytes)?;
    d.set_item("ratio", stats.ratio)?;
    d.set_item("elapsed_s", stats.elapsed_s)?;
    Ok(d)
}

/// zstd-decompress a `.ags5db.zst` back to `dest`. Returns
/// `{bytes, elapsed_s}`.
#[pyfunction]
#[pyo3(signature = (src, dest))]
fn ags5db_unpack<'py>(py: Python<'py>, src: String, dest: String) -> PyResult<Bound<'py, PyDict>> {
    let stats = transport::unpack(Path::new(&src), Path::new(&dest)).map_err(map_cli_err)?;
    let d = PyDict::new(py);
    d.set_item("bytes", stats.bytes)?;
    d.set_item("elapsed_s", stats.elapsed_s)?;
    Ok(d)
}

/// zstd-compress + age-passphrase-encrypt `src` to `dest`. Returns
/// `{bytes, ratio, elapsed_s}`. The age envelope is interoperable
/// with `pyrage`.
#[pyfunction]
#[pyo3(signature = (src, dest, password, level = 9))]
fn ags5db_lock<'py>(
    py: Python<'py>,
    src: String,
    dest: String,
    password: String,
    level: i32,
) -> PyResult<Bound<'py, PyDict>> {
    let stats = transport::lock(Path::new(&src), Path::new(&dest), &password, level)
        .map_err(map_cli_err)?;
    let d = PyDict::new(py);
    d.set_item("bytes", stats.bytes)?;
    d.set_item("ratio", stats.ratio)?;
    d.set_item("elapsed_s", stats.elapsed_s)?;
    Ok(d)
}

/// age-passphrase-decrypt + zstd-decompress `src` to `dest`. Returns
/// `{bytes, elapsed_s}`. Wrong passphrase / non-passphrase envelopes
/// surface as `RuntimeError`.
#[pyfunction]
#[pyo3(signature = (src, dest, password))]
fn ags5db_unlock<'py>(
    py: Python<'py>,
    src: String,
    dest: String,
    password: String,
) -> PyResult<Bound<'py, PyDict>> {
    let stats =
        transport::unlock(Path::new(&src), Path::new(&dest), &password).map_err(map_cli_err)?;
    let d = PyDict::new(py);
    d.set_item("bytes", stats.bytes)?;
    d.set_item("elapsed_s", stats.elapsed_s)?;
    Ok(d)
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(ags5db_pack, m)?)?;
    m.add_function(wrap_pyfunction!(ags5db_unpack, m)?)?;
    m.add_function(wrap_pyfunction!(ags5db_lock, m)?)?;
    m.add_function(wrap_pyfunction!(ags5db_unlock, m)?)?;
    Ok(())
}
