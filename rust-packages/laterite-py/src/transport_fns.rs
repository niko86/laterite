//! PyO3 bindings for `laterite_ags4_core::transport` (zstd + age passphrase).
//!
//! Stage F2a-2c: exposes the lib API so `laterite.transport` can
//! drive pack/unpack/lock/unlock from Python without subprocessing
//! the binary.
//!
//! These operations are **content-agnostic** — zstd/age over raw bytes,
//! so they work on any file (`.ags`, anything). The fns were
//! renamed off a legacy prefix (#111 Facet B) that wrongly implied a
//! single file type, the same misnomer W2 fixed for the
//! Excel fns. Internal `_native.*` names (not public), so no alias —
//! `laterite.transport` calls these directly.

use std::path::Path;

use laterite_ags4_core::transport;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyModule};
use pyo3::wrap_pyfunction;

use crate::map_cli_err;

/// zstd-compress any file to `dest`. Returns `{bytes, ratio, elapsed_s}`.
#[pyfunction]
#[pyo3(signature = (src, dest, level = 9))]
// PyO3 boundary: owns the deserialized input
#[allow(clippy::needless_pass_by_value)]
fn transport_pack(
    py: Python<'_>,
    src: String,
    dest: String,
    level: i32,
) -> PyResult<Bound<'_, PyDict>> {
    let stats =
        transport::pack(Path::new(&src), Path::new(&dest), level).map_err(|e| map_cli_err(&e))?;
    let d = PyDict::new(py);
    d.set_item("bytes", stats.bytes)?;
    d.set_item("ratio", stats.ratio)?;
    d.set_item("elapsed_s", stats.elapsed_s)?;
    Ok(d)
}

/// zstd-decompress a `.zst` produced by `transport_pack` back to `dest`.
/// Returns `{bytes, elapsed_s}`.
#[pyfunction]
#[pyo3(signature = (src, dest))]
// PyO3 boundary: owns the deserialized input
#[allow(clippy::needless_pass_by_value)]
fn transport_unpack(py: Python<'_>, src: String, dest: String) -> PyResult<Bound<'_, PyDict>> {
    let stats =
        transport::unpack(Path::new(&src), Path::new(&dest)).map_err(|e| map_cli_err(&e))?;
    let d = PyDict::new(py);
    d.set_item("bytes", stats.bytes)?;
    d.set_item("elapsed_s", stats.elapsed_s)?;
    Ok(d)
}

/// zstd-compress + age-passphrase-encrypt any file to `dest`. Returns
/// `{bytes, ratio, elapsed_s}`. The age envelope is interoperable
/// with `pyrage`.
#[pyfunction]
#[pyo3(signature = (src, dest, password, level = 9, log_n = None))]
// PyO3 boundary: owns the deserialized input
#[allow(clippy::needless_pass_by_value)]
fn transport_lock(
    py: Python<'_>,
    src: String,
    dest: String,
    password: String,
    level: i32,
    log_n: Option<u8>,
) -> PyResult<Bound<'_, PyDict>> {
    let stats = transport::lock(
        Path::new(&src),
        Path::new(&dest),
        &password,
        level,
        log_n.unwrap_or(transport::SCRYPT_LOG_N),
    )
    .map_err(|e| map_cli_err(&e))?;
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
// PyO3 boundary: owns the deserialized input
#[allow(clippy::needless_pass_by_value)]
fn transport_unlock(
    py: Python<'_>,
    src: String,
    dest: String,
    password: String,
) -> PyResult<Bound<'_, PyDict>> {
    let stats = transport::unlock(Path::new(&src), Path::new(&dest), &password)
        .map_err(|e| map_cli_err(&e))?;
    let d = PyDict::new(py);
    d.set_item("bytes", stats.bytes)?;
    d.set_item("elapsed_s", stats.elapsed_s)?;
    Ok(d)
}

// --- in-memory (bytes) forms: no filesystem, for the "read → fix → package
//     straight to an upload, never a plaintext file on disk" flow. Same
//     envelopes as the path forms (a `*_bytes` blob interops with the file
//     API and vice versa — proven in laterite-transport's tests). ---

/// zstd-compress bytes → bytes.
#[pyfunction]
#[pyo3(signature = (data, level = 9))]
fn transport_pack_bytes<'py>(
    py: Python<'py>,
    data: &[u8],
    level: i32,
) -> PyResult<Bound<'py, PyBytes>> {
    let out = transport::pack_bytes(data, level).map_err(|e| map_cli_err(&e))?;
    Ok(PyBytes::new(py, &out))
}

/// zstd-decompress bytes → bytes.
#[pyfunction]
#[pyo3(signature = (data))]
fn transport_unpack_bytes<'py>(py: Python<'py>, data: &[u8]) -> PyResult<Bound<'py, PyBytes>> {
    let out = transport::unpack_bytes(data).map_err(|e| map_cli_err(&e))?;
    Ok(PyBytes::new(py, &out))
}

/// zstd-compress + age-passphrase-encrypt bytes → bytes.
#[pyfunction]
#[pyo3(signature = (data, password, level = 9, log_n = None))]
// PyO3 boundary: owns the deserialized input
#[allow(clippy::needless_pass_by_value)]
fn transport_lock_bytes<'py>(
    py: Python<'py>,
    data: &[u8],
    password: String,
    level: i32,
    log_n: Option<u8>,
) -> PyResult<Bound<'py, PyBytes>> {
    let out = transport::lock_bytes(
        data,
        &password,
        level,
        log_n.unwrap_or(transport::SCRYPT_LOG_N),
    )
    .map_err(|e| map_cli_err(&e))?;
    Ok(PyBytes::new(py, &out))
}

/// age-passphrase-decrypt + zstd-decompress bytes → bytes. Wrong passphrase /
/// non-passphrase envelopes surface as `RuntimeError`.
#[pyfunction]
#[pyo3(signature = (data, password))]
// PyO3 boundary: owns the deserialized input
#[allow(clippy::needless_pass_by_value)]
fn transport_unlock_bytes<'py>(
    py: Python<'py>,
    data: &[u8],
    password: String,
) -> PyResult<Bound<'py, PyBytes>> {
    let out = transport::unlock_bytes(data, &password).map_err(|e| map_cli_err(&e))?;
    Ok(PyBytes::new(py, &out))
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(transport_pack, m)?)?;
    m.add_function(wrap_pyfunction!(transport_unpack, m)?)?;
    m.add_function(wrap_pyfunction!(transport_lock, m)?)?;
    m.add_function(wrap_pyfunction!(transport_unlock, m)?)?;
    m.add_function(wrap_pyfunction!(transport_pack_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(transport_unpack_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(transport_lock_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(transport_unlock_bytes, m)?)?;
    Ok(())
}
