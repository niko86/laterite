//! PyO3 wrappers over the `ags5db` conversion lib API (`convert.rs`).
//!
//! This is the fatter half of the laterite wheel: linking `ags5db`
//! pulls in bundled libduckdb (gate B.2 of
//! ~15-20 MB wheel accepted for v1). The validator half stays lean.
//! `ags5db` depends on `ags4-validator` (for `db-to-ags4 --validate`),
//! never the reverse, so the engine's pyo3-free guarantee still holds.
//!
//! Each fn does the data work in `ags5db::convert` and returns a stats
//! dict; failures map to `RuntimeError` carrying the `CliError`'s exit
//! code (the same code the binary would exit with). The nice Python
//! wrappers live in `laterite/ags5db.py`.

use std::path::Path;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde_json::{Map, Value};

use ags5db::convert;
use ags5db::db::Rows;
use ags5db::error::CliError;
use ags5db::query;

/// A `CliError` → `RuntimeError` carrying the binary's exit code, so a
/// Python caller can branch on it the way a shell would on `$?`.
pub(crate) fn map_cli_err(e: CliError) -> PyErr {
    let code = e.exit_code();
    PyRuntimeError::new_err(format!("ags5db error (exit {code}): {e}"))
}

/// AGS4 transfer file → `.ags5db`. Wraps [`convert::ags4_to_db`].
/// `append` merges into an existing DB (else it's overwritten);
/// `no_compact` skips the final CTAS rewrite; `attachments_dir`
/// resolves FILE references (defaults to the `.ags`'s parent).
#[pyfunction]
#[pyo3(signature = (ags4_path, db_path, append=false, no_compact=false, attachments_dir=None))]
fn ags5db_convert<'py>(
    py: Python<'py>,
    ags4_path: String,
    db_path: String,
    append: bool,
    no_compact: bool,
    attachments_dir: Option<String>,
) -> PyResult<Bound<'py, PyDict>> {
    let adir = attachments_dir.as_deref().map(Path::new);
    let stats = convert::ags4_to_db(
        Path::new(&ags4_path),
        Path::new(&db_path),
        append,
        no_compact,
        adir,
    )
    .map_err(map_cli_err)?;

    let d = PyDict::new(py);
    d.set_item("bytes", stats.bytes)?;
    d.set_item("mode", stats.mode)?;
    d.set_item("attachments", stats.attachments)?;
    d.set_item("attachment_bytes", stats.attachment_bytes)?;
    d.set_item("warnings", stats.warnings)?;
    Ok(d)
}

/// `.ags5db` → AGS4 transfer file. Wraps [`convert::db_to_ags4`] (the
/// data export only — attachment unspooling and `--validate` stay in
/// the bin's command layer). Bails on RL-typed headings (Rule 11).
#[pyfunction]
#[pyo3(signature = (db_path, ags4_path))]
fn ags5db_export<'py>(
    py: Python<'py>,
    db_path: String,
    ags4_path: String,
) -> PyResult<Bound<'py, PyDict>> {
    let stats =
        convert::db_to_ags4(Path::new(&db_path), Path::new(&ags4_path)).map_err(map_cli_err)?;

    let d = PyDict::new(py);
    d.set_item("groups_emitted", stats.groups_emitted)?;
    d.set_item("rows_emitted", stats.rows_emitted)?;
    d.set_item("warnings", stats.warnings)?;
    Ok(d)
}

// `.agsx` ↔ `.ags5db` conversion retired in Stage F2a (see
// Python-only inspection helper via `ags5_xml.ags4_to_agsx`.

// ---------------------------------------------------------------------
// read-side query API (over ags5db::query)
// ---------------------------------------------------------------------

/// Serialise `Rows` to an order-preserving `{columns, records}` JSON
/// string. The Python wrapper `json.loads`es it — robust for nested
/// values and avoids a manual serde_json::Value -> PyObject walk
/// (serde_json's `preserve_order` keeps record key order, which Python
/// dicts then preserve).
fn rows_json(rows: &Rows) -> String {
    let mut root = Map::new();
    root.insert("columns".into(), Value::from(rows.columns.clone()));
    root.insert(
        "records".into(),
        Value::Array(rows.records.iter().cloned().map(Value::Object).collect()),
    );
    serde_json::to_string(&Value::Object(root)).unwrap_or_default()
}

/// `COUNT(*)` on a group's view, optional ANDed `where` predicates.
#[pyfunction]
#[pyo3(signature = (db_path, group, where_ = Vec::new()))]
fn ags5db_count(db_path: String, group: String, where_: Vec<String>) -> PyResult<i64> {
    query::count(Path::new(&db_path), &group, &where_).map_err(map_cli_err)
}

/// `SUM(field)` (cast to DOUBLE) on a numeric heading, optional `where`.
#[pyfunction]
#[pyo3(signature = (db_path, group, field, where_ = Vec::new()))]
fn ags5db_sum(db_path: String, group: String, field: String, where_: Vec<String>) -> PyResult<f64> {
    query::sum(Path::new(&db_path), &group, &field, &where_).map_err(map_cli_err)
}

/// Raw read-only SELECT. Returns a `{columns, records}` JSON string.
#[pyfunction]
#[pyo3(signature = (db_path, statement, limit = 1000, explain = false))]
fn ags5db_sql(db_path: String, statement: String, limit: usize, explain: bool) -> PyResult<String> {
    let rows = query::sql(Path::new(&db_path), &statement, limit, explain).map_err(map_cli_err)?;
    Ok(rows_json(&rows))
}

/// Browse a group's view (fields/where/limit/offset, optional null-col
/// drop). Returns a `{columns, records}` JSON string.
#[pyfunction]
#[pyo3(signature = (db_path, group, fields = None, where_ = Vec::new(), limit = 50, offset = 0, drop_null_cols = false))]
fn ags5db_peek(
    db_path: String,
    group: String,
    fields: Option<String>,
    where_: Vec<String>,
    limit: usize,
    offset: usize,
    drop_null_cols: bool,
) -> PyResult<String> {
    let rows = query::peek(
        Path::new(&db_path),
        &group,
        fields.as_deref(),
        &where_,
        limit,
        offset,
        drop_null_cols,
    )
    .map_err(map_cli_err)?;
    Ok(rows_json(&rows))
}

/// Spec-tables inspector: file-level meta + optional per-group block.
/// JSON-encoded. With `group=None`, just scalar meta + counts; with
/// `group=Some(code)`, also fills in the group block + its headings.
/// Stage F2a-2f.
#[pyfunction]
#[pyo3(signature = (db_path, group = None))]
fn ags5db_inspect(db_path: String, group: Option<String>) -> PyResult<String> {
    use ags5db::introspect;
    let report = introspect::inspect(Path::new(&db_path), group.as_deref()).map_err(map_cli_err)?;

    let mut root = Map::new();
    root.insert("format_version".into(), Value::from(report.format_version));
    root.insert(
        "library_version".into(),
        Value::from(report.library_version),
    );
    root.insert("written_at".into(), Value::from(report.written_at));
    root.insert("note".into(), Value::from(report.note));
    root.insert("n_groups".into(), Value::from(report.n_groups));
    root.insert("n_headings".into(), Value::from(report.n_headings));

    if let Some(g) = report.group {
        let mut gm = Map::new();
        gm.insert("code".into(), Value::from(g.code));
        gm.insert("contents".into(), Value::from(g.contents));
        gm.insert("parent".into(), Value::from(g.parent));
        gm.insert("is_high_volume".into(), Value::from(g.is_high_volume));
        // Outer None = field absent (pre-6.5.2); Some(None) = JSON null;
        // Some(Some(v)) = real value.
        if let Some(inner) = g.index_parent {
            gm.insert(
                "index_parent".into(),
                inner.map(Value::from).unwrap_or(Value::Null),
            );
        }
        root.insert("group".into(), Value::Object(gm));
    }

    if let Some(headings) = report.headings {
        let arr: Vec<Value> = headings
            .into_iter()
            .map(|h| {
                let mut m = Map::new();
                m.insert("name".into(), Value::from(h.name));
                m.insert("status".into(), Value::from(h.status));
                m.insert("ags_type".into(), Value::from(h.ags_type));
                m.insert("canonical_type".into(), Value::from(h.canonical_type));
                m.insert("unit".into(), Value::from(h.unit));
                m.insert("display_hint".into(), Value::from(h.display_hint));
                if let Some(inner) = h.indexed {
                    m.insert(
                        "indexed".into(),
                        inner.map(Value::from).unwrap_or(Value::Null),
                    );
                }
                Value::Object(m)
            })
            .collect();
        root.insert("headings".into(), Value::Array(arr));
    }

    Ok(serde_json::to_string(&Value::Object(root)).unwrap_or_default())
}

/// File-level summary: file path + size_mb + format/library versions
/// + per-group row counts. JSON-encoded `InfoPayload` shape. Stage F2a-2e.
#[pyfunction]
#[pyo3(signature = (db_path))]
fn ags5db_info(db_path: String) -> PyResult<String> {
    use ags5db::introspect;
    let summary = introspect::info(Path::new(&db_path)).map_err(map_cli_err)?;
    let n_groups = summary.n_groups();
    let n_nonempty = summary.n_nonempty();
    let mut groups_arr: Vec<Value> = Vec::with_capacity(summary.groups.len());
    for g in &summary.groups {
        let mut m = Map::new();
        m.insert("code".into(), Value::from(g.code.clone()));
        m.insert("rows".into(), Value::from(g.rows));
        m.insert(
            "parent".into(),
            if g.parent.is_empty() {
                Value::Null
            } else {
                Value::from(g.parent.clone())
            },
        );
        groups_arr.push(Value::Object(m));
    }
    let mut root = Map::new();
    root.insert("file".into(), Value::from(summary.file));
    root.insert("size_mb".into(), Value::from(summary.size_mb));
    root.insert(
        "format_version".into(),
        summary
            .format_version
            .map(Value::from)
            .unwrap_or(Value::Null),
    );
    root.insert(
        "library_version".into(),
        summary
            .library_version
            .map(Value::from)
            .unwrap_or(Value::Null),
    );
    root.insert("n_groups".into(), Value::from(n_groups));
    root.insert("n_nonempty".into(), Value::from(n_nonempty));
    root.insert("groups".into(), Value::Array(groups_arr));
    Ok(serde_json::to_string(&Value::Object(root)).unwrap_or_default())
}

/// Every group in the file with row count + parent + contents. JSON
/// list of `{code, rows, parent, contents}` records. Stage F2a-2e.
#[pyfunction]
#[pyo3(signature = (db_path, nonempty = false))]
fn ags5db_groups(db_path: String, nonempty: bool) -> PyResult<String> {
    use ags5db::introspect;
    let groups = introspect::list_groups(Path::new(&db_path), nonempty).map_err(map_cli_err)?;
    let arr: Vec<Value> = groups
        .into_iter()
        .map(|g| {
            let mut m = Map::new();
            m.insert("code".into(), Value::from(g.code));
            m.insert("rows".into(), Value::from(g.rows));
            m.insert("parent".into(), Value::from(g.parent));
            m.insert("contents".into(), Value::from(g.contents));
            Value::Object(m)
        })
        .collect();
    Ok(serde_json::to_string(&Value::Array(arr)).unwrap_or_default())
}

/// Schema dump for one group. JSON list of `{name, status, ags_type,
/// canonical_type, unit, hint}` records. Stage F2a-2e.
#[pyfunction]
#[pyo3(signature = (db_path, group))]
fn ags5db_headings(db_path: String, group: String) -> PyResult<String> {
    use ags5db::introspect;
    let headings = introspect::headings(Path::new(&db_path), &group).map_err(map_cli_err)?;
    let arr: Vec<Value> = headings
        .into_iter()
        .map(|h| {
            let mut m = Map::new();
            m.insert("name".into(), Value::from(h.name));
            m.insert("status".into(), Value::from(h.status));
            m.insert("ags_type".into(), Value::from(h.ags_type));
            m.insert("canonical_type".into(), Value::from(h.canonical_type));
            m.insert("unit".into(), Value::from(h.unit));
            m.insert("hint".into(), Value::from(h.hint));
            Value::Object(m)
        })
        .collect();
    Ok(serde_json::to_string(&Value::Array(arr)).unwrap_or_default())
}

/// Diff two `.ags5db` files. Returns a JSON-encoded
/// `{changed_groups: [...], groups_only_in_a: [...], groups_only_in_b: [...]}`
/// payload. `samples` caps the per-group sample KEY tuples shown for
/// each change category. Stage F2a-2d.
#[pyfunction]
#[pyo3(signature = (a_path, b_path, samples = 3))]
fn ags5db_diff(a_path: String, b_path: String, samples: usize) -> PyResult<String> {
    use ags5db::diff;
    let result =
        diff::diff_dbs(Path::new(&a_path), Path::new(&b_path), samples).map_err(map_cli_err)?;

    let mut changed = Vec::with_capacity(result.changed_groups.len());
    for gd in &result.changed_groups {
        let mut m = Map::new();
        m.insert("code".into(), Value::from(gd.code.clone()));
        m.insert("added".into(), Value::from(gd.added));
        m.insert("removed".into(), Value::from(gd.removed));
        m.insert("modified".into(), Value::from(gd.modified));
        m.insert("unchanged".into(), Value::from(gd.unchanged));
        m.insert(
            "sample_added".into(),
            Value::Array(
                gd.sample_added
                    .iter()
                    .map(|t| Value::Array(t.clone()))
                    .collect(),
            ),
        );
        m.insert(
            "sample_removed".into(),
            Value::Array(
                gd.sample_removed
                    .iter()
                    .map(|t| Value::Array(t.clone()))
                    .collect(),
            ),
        );
        m.insert(
            "sample_modified".into(),
            Value::Array(
                gd.sample_modified
                    .iter()
                    .map(|t| Value::Array(t.clone()))
                    .collect(),
            ),
        );
        changed.push(Value::Object(m));
    }

    let mut root = Map::new();
    root.insert("changed_groups".into(), Value::Array(changed));
    root.insert(
        "groups_only_in_a".into(),
        Value::Array(
            result
                .groups_only_in_a
                .into_iter()
                .map(Value::from)
                .collect(),
        ),
    );
    root.insert(
        "groups_only_in_b".into(),
        Value::Array(
            result
                .groups_only_in_b
                .into_iter()
                .map(Value::from)
                .collect(),
        ),
    );
    Ok(serde_json::to_string(&Value::Object(root)).unwrap_or_default())
}

/// Validate a `.ags5db` file. Returns a JSON-encoded list of findings:
/// `[{severity, code, where, message}, ...]`. Stage F2a-2.
#[pyfunction]
#[pyo3(signature = (db_path, check_abbr = true, check_dt = true))]
fn ags5db_validate(db_path: String, check_abbr: bool, check_dt: bool) -> PyResult<String> {
    let findings =
        query::validate_db(Path::new(&db_path), check_abbr, check_dt).map_err(map_cli_err)?;
    let arr: Vec<Value> = findings
        .into_iter()
        .map(|f| {
            let mut m = Map::new();
            m.insert("severity".into(), Value::from(f.severity));
            m.insert("code".into(), Value::from(f.code));
            m.insert("where".into(), Value::from(f.where_));
            m.insert("message".into(), Value::from(f.message));
            Value::Object(m)
        })
        .collect();
    Ok(serde_json::to_string(&Value::Array(arr)).unwrap_or_default())
}

/// List blob rows with optional filters. Returns a `{columns, records}`
/// JSON string (data BLOB excluded — metadata only). Stage F2a-2.
#[pyfunction]
#[pyo3(signature = (db_path, parent_code = None, kind = None))]
fn ags5db_list_blobs(
    db_path: String,
    parent_code: Option<String>,
    kind: Option<String>,
) -> PyResult<String> {
    let rows = query::list_blobs(Path::new(&db_path), parent_code.as_deref(), kind.as_deref())
        .map_err(map_cli_err)?;
    Ok(rows_json(&rows))
}

/// Register the ags5db conversion + query fns on `_laterite_native`.
pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(ags5db_convert, m)?)?;
    m.add_function(wrap_pyfunction!(ags5db_export, m)?)?;
    m.add_function(wrap_pyfunction!(ags5db_count, m)?)?;
    m.add_function(wrap_pyfunction!(ags5db_sum, m)?)?;
    m.add_function(wrap_pyfunction!(ags5db_sql, m)?)?;
    m.add_function(wrap_pyfunction!(ags5db_peek, m)?)?;
    m.add_function(wrap_pyfunction!(ags5db_list_blobs, m)?)?;
    m.add_function(wrap_pyfunction!(ags5db_validate, m)?)?;
    m.add_function(wrap_pyfunction!(ags5db_diff, m)?)?;
    m.add_function(wrap_pyfunction!(ags5db_info, m)?)?;
    m.add_function(wrap_pyfunction!(ags5db_groups, m)?)?;
    m.add_function(wrap_pyfunction!(ags5db_headings, m)?)?;
    m.add_function(wrap_pyfunction!(ags5db_inspect, m)?)?;
    Ok(())
}
