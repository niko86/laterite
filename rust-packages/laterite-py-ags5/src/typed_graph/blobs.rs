//! Stage F2c-2: `laterite.ags5db.attach_blobs(db, blobs)`.
//!
//! Resolves each `BlobAttachment`'s target row UUID by querying the
//! file's `v_<code>` view on its declared KEY columns, then
//! bulk-inserts into the `blob` table. Mirrors the side-channel that
//! `ags5_db.blobs._insert_blobs` provided on the soon-to-retire
//! Python writer, but as a Rust-backed entry point so blob support
//! stays alive after F2c retires `ags5_db`.
//!
//! The resolver uses views (not table walks via dedup state) because
//! the file has already been written by the time `attach_blobs`
//! runs — there's no in-memory dedup context. `v_<code>` JOINs
//! inherited KEYs in from ancestors, so one parameterised SELECT
//! per blob recovers the target UUID without walking the ancestor
//! chain manually.

use std::collections::HashMap;
use std::path::Path;

use duckdb::Connection;
use duckdb::types::Value as DuckValue;
use laterite_ags5_db::registry::registry;
use pyo3::exceptions::{PyFileNotFoundError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

/// One blob attachment passed from Python. Matches the shape of
/// `laterite.ags5db.BlobAttachment` after `dataclasses.asdict` —
/// the laterite wrapper passes plain dicts so we extract by key.
#[derive(FromPyObject)]
pub struct BlobInput {
    #[pyo3(item)]
    target_code: String,
    #[pyo3(item)]
    target_keys: HashMap<String, Py<PyAny>>,
    #[pyo3(item)]
    kind: String,
    #[pyo3(item)]
    data: Py<PyBytes>,
    #[pyo3(item)]
    mime_type: Option<String>,
    #[pyo3(item)]
    filename: Option<String>,
}

#[pyfunction]
#[pyo3(signature = (db, blobs))]
pub fn ags5db_attach_blobs(py: Python<'_>, db: &str, blobs: Vec<BlobInput>) -> PyResult<usize> {
    let p = Path::new(db);
    if !p.exists() {
        return Err(PyFileNotFoundError::new_err(db.to_string()));
    }
    if blobs.is_empty() {
        return Ok(0);
    }

    // Open read-write — open_readonly is for queries; we need
    // INSERT on the blob table.
    let conn = Connection::open(p)
        .map_err(|e| PyRuntimeError::new_err(format!("open {}: {e}", p.display())))?;
    let reg = registry();

    let mut count = 0usize;
    for blob in &blobs {
        let code_upper = blob.target_code.to_uppercase();
        let g = reg.get(&code_upper).ok_or_else(|| {
            PyValueError::new_err(format!(
                "unknown target_code: {} (not in compiled registry)",
                blob.target_code,
            ))
        })?;

        // Build the resolver WHERE clause from whatever KEY values
        // the caller provided. `from_model` populates this from the
        // target group's own KEY set (which already includes
        // inherited KEYs in the view); callers constructing
        // BlobAttachment by hand need to do the same. We never go
        // further up than what the caller declared, which keeps the
        // PROJ-level KEY (PROJ_ID) optional — it's not a column on
        // most descendant views anyway.
        let key_names: Vec<String> = blob.target_keys.keys().cloned().collect();
        if key_names.is_empty() {
            return Err(PyValueError::new_err(format!(
                "BlobAttachment for {}: target_keys must contain at \
                 least one KEY heading",
                blob.target_code,
            )));
        }

        let view = format!("v_{}", g.code.to_lowercase());
        let where_clauses: Vec<String> = key_names
            .iter()
            .map(|n| format!("{} = ?", n.to_lowercase()))
            .collect();
        let sql = format!(
            "SELECT CAST(id AS VARCHAR) FROM {} WHERE {}",
            view,
            where_clauses.join(" AND "),
        );

        // Bind each KEY value as a DuckValue. The Python side carries
        // typed values (strings, ints, floats) — convert each.
        let mut bind_values: Vec<DuckValue> = Vec::with_capacity(key_names.len());
        for name in &key_names {
            let py_val = blob.target_keys.get(name).expect("just enumerated");
            bind_values.push(python_to_duck(py, py_val.bind(py), name)?);
        }

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| PyRuntimeError::new_err(format!("prepare: {e}")))?;
        let id_str: Option<String> = stmt
            .query_row(duckdb::params_from_iter(bind_values.iter()), |row| {
                row.get::<_, String>(0)
            })
            .ok();

        let Some(target_uuid) = id_str else {
            return Err(PyValueError::new_err(format!(
                "BlobAttachment: no {} row matches keys {:?}",
                blob.target_code, key_names,
            )));
        };

        // Insert: sha256, parent_table, parent_id, kind, mime_type,
        // filename, data. Compute sha256 over the bytes.
        let bytes = blob.data.bind(py).as_bytes();
        let sha256 = sha256_hex(bytes);
        let parent_table = format!("g_{}", g.code.to_lowercase());

        // Call nextval('seq_blob') explicitly to mint the id —
        // DuckDB's Rust driver doesn't auto-apply the column's
        // DEFAULT when prepared statements omit the column, even
        // with an explicit `DEFAULT` keyword inline. Python's driver
        // does. We materialise the sequence value first then bind it.
        let next_id: i64 = conn
            .query_row("SELECT nextval('seq_blob')", [], |r| r.get(0))
            .map_err(|e| PyRuntimeError::new_err(format!("nextval: {e}")))?;

        conn.execute(
            "INSERT INTO blob (id, parent_table, parent_id, kind, mime_type, \
             filename, sha256, data) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            duckdb::params![
                next_id,
                parent_table,
                target_uuid,
                blob.kind,
                blob.mime_type,
                blob.filename,
                sha256,
                bytes,
            ],
        )
        .map_err(|e| PyRuntimeError::new_err(format!("INSERT blob: {e}")))?;

        count += 1;
    }
    Ok(count)
}

/// Coerce a Python value into a DuckDB-typed value for parameter
/// binding. AGS KEY columns are typed (typically VARCHAR for ID/text,
/// DOUBLE for the few decimal-typed KEYs like SAMP_TOP), so we
/// dispatch on the Python runtime type.
fn python_to_duck(py: Python<'_>, val: &Bound<'_, PyAny>, name: &str) -> PyResult<DuckValue> {
    let _ = py;
    if val.is_none() {
        return Ok(DuckValue::Null);
    }
    if let Ok(s) = val.extract::<String>() {
        return Ok(DuckValue::Text(s));
    }
    if let Ok(i) = val.extract::<i64>() {
        return Ok(DuckValue::BigInt(i));
    }
    if let Ok(f) = val.extract::<f64>() {
        return Ok(DuckValue::Double(f));
    }
    if let Ok(b) = val.extract::<bool>() {
        return Ok(DuckValue::Boolean(b));
    }
    Err(PyValueError::new_err(format!(
        "BlobAttachment key {name}: unsupported Python type {:?}",
        val.get_type().name()?,
    )))
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(bytes);
    hex::encode(digest)
}
