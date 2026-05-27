//! Stage F2b-4: `read_db(path) -> PROJ`.
//!
//! Reads a `.ags5db` and returns a Python typed-graph PROJ tree. Walks
//! the file's own `_spec_groups` to learn what's present, instantiates
//! a compiled `#[pyclass]` for each standard group and a Rust-built
//! dynamic Python class for each passthrough group (via the
//! `laterite.dynamic.get_or_register` factory), and links the tree
//! through each row's `parent_id`.
//!
//! *Custom groups & passthrough* — for the full policy this
//! implements.

use std::collections::HashMap;
use std::path::Path;

use ags5db::conn::open_readonly;
use ags5db::db::{HeadingRow, headings_for};
use chrono::{DateTime, NaiveDate, NaiveTime};
use duckdb::Connection;
use duckdb::types::{TimeUnit, Value as DuckValue};
use pyo3::exceptions::{PyFileNotFoundError, PyRuntimeError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::ags5db_fns::map_cli_err;

#[pyfunction]
#[pyo3(signature = (path))]
pub fn ags5db_read_db(py: Python<'_>, path: &str) -> PyResult<Py<PyAny>> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(PyFileNotFoundError::new_err(path.to_string()));
    }
    let conn = open_readonly(p).map_err(map_cli_err)?;

    // Topo-sort: parents before children. The dictionary's recorded
    // parent edges (`_spec_groups.parent`) drive the order.
    let groups = read_groups_topo(&conn)?;

    // Resolve each group code to its Python class — compiled for the 92
    // standard ones, dynamically-built via laterite.dynamic for any
    // passthroughs the file declares.
    let class_map = build_class_map(py, &conn, &groups)?;

    // Build instances in topo order so each parent is already in `by_id`
    // by the time its children read their parent_id.
    let mut by_id: HashMap<String, Py<PyAny>> = HashMap::new();
    let mut root: Option<Py<PyAny>> = None;
    let mut root_count = 0usize;

    for g in &groups {
        let headings = headings_for(&conn, &g.code).map_err(map_cli_err)?;
        let class = class_map
            .get(&g.code)
            .expect("class_map populated above")
            .bind(py);

        let rows = read_rows_for_group(py, &conn, &g.code, &headings)?;

        for row in rows {
            let kwargs = PyDict::new(py);
            for (k, v) in row.field_values {
                kwargs.set_item(k, v)?;
            }
            let instance = class.call((), Some(&kwargs))?.unbind();

            // Attach to parent, or remember as root.
            if let Some(parent_id) = &row.parent_id {
                if let Some(parent) = by_id.get(parent_id) {
                    attach_child_to_parent(py, parent, &g.code, &instance)?;
                }
                // If parent_id points to nothing (corrupt file), skip
                // attachment silently — the instance still goes into
                // by_id so its own children can find it.
            } else if g.code == "PROJ" {
                root_count += 1;
                if root.is_none() {
                    root = Some(instance.clone_ref(py));
                }
            }

            by_id.insert(row.id, instance);
        }
    }

    let _ = root_count; // unused for now; future: warn if > 1
    root.ok_or_else(|| {
        PyRuntimeError::new_err("no PROJ row found in file (every .ags5db must have exactly one)")
    })
}

// --- topological order over `_spec_groups` -------------------------

struct GroupNode {
    code: String,
    parent: Option<String>,
}

fn read_groups_topo(conn: &Connection) -> PyResult<Vec<GroupNode>> {
    let mut stmt = conn
        .prepare("SELECT code, parent FROM _spec_groups")
        .map_err(|e| PyRuntimeError::new_err(format!("_spec_groups: {e}")))?;
    let nodes: Vec<GroupNode> = stmt
        .query_map([], |r| {
            Ok(GroupNode {
                code: r.get::<_, String>(0)?,
                parent: r.get::<_, Option<String>>(1)?,
            })
        })
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    // Group by parent for a simple DFS-based topo.
    let mut by_parent: HashMap<Option<String>, Vec<String>> = HashMap::new();
    let mut all_codes: Vec<String> = Vec::with_capacity(nodes.len());
    let mut parent_of: HashMap<String, Option<String>> = HashMap::new();
    for n in nodes {
        by_parent
            .entry(n.parent.clone())
            .or_default()
            .push(n.code.clone());
        all_codes.push(n.code.clone());
        parent_of.insert(n.code.clone(), n.parent);
    }

    let mut out: Vec<GroupNode> = Vec::new();
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();

    fn visit(
        code: &str,
        parent_of: &HashMap<String, Option<String>>,
        by_parent: &HashMap<Option<String>, Vec<String>>,
        visited: &mut std::collections::HashSet<String>,
        out: &mut Vec<GroupNode>,
    ) {
        if visited.contains(code) {
            return;
        }
        // Walk parent chain first.
        if let Some(Some(parent)) = parent_of.get(code) {
            visit(parent, parent_of, by_parent, visited, out);
        }
        visited.insert(code.to_string());
        out.push(GroupNode {
            code: code.to_string(),
            parent: parent_of.get(code).cloned().flatten(),
        });
        // Then descend into children.
        if let Some(kids) = by_parent.get(&Some(code.to_string())) {
            for k in kids {
                visit(k, parent_of, by_parent, visited, out);
            }
        }
    }

    // Start from roots (parent = None — typically only PROJ).
    if let Some(roots) = by_parent.get(&None) {
        for r in roots {
            visit(r, &parent_of, &by_parent, &mut visited, &mut out);
        }
    }
    // Anything not reachable from a root (shouldn't normally happen)
    // still needs to be emitted so we don't silently drop groups.
    for c in &all_codes {
        if !visited.contains(c) {
            visit(c, &parent_of, &by_parent, &mut visited, &mut out);
        }
    }

    Ok(out)
}

// --- class resolution (compiled vs dynamic) ------------------------

fn build_class_map(
    py: Python<'_>,
    conn: &Connection,
    groups: &[GroupNode],
) -> PyResult<HashMap<String, Py<PyAny>>> {
    let native = py.import("laterite._laterite_native")?;
    let dynamic = py.import("laterite.dynamic")?;
    let factory = dynamic.getattr("get_or_register")?;

    let mut map: HashMap<String, Py<PyAny>> = HashMap::new();
    for g in groups {
        let cls = if let Ok(compiled) = native.getattr(g.code.as_str()) {
            compiled.unbind()
        } else {
            // Dynamic: hand the dictionary's headings to the factory.
            // We pass `[{name, type}, ...]` — the same shape
            // `_spec_headings` exposes (modulo the other columns the
            // factory doesn't need).
            let headings = headings_for(conn, &g.code).map_err(map_cli_err)?;
            let heading_dicts = PyList::empty(py);
            for h in &headings {
                let d = PyDict::new(py);
                d.set_item("name", &h.name)?;
                d.set_item("type", &h.ags_type)?;
                heading_dicts.append(d)?;
            }
            factory.call1((g.code.as_str(), heading_dicts))?.unbind()
        };
        map.insert(g.code.clone(), cls);
    }
    Ok(map)
}

// --- row extraction ------------------------------------------------

struct RowData {
    id: String,
    parent_id: Option<String>,
    field_values: Vec<(String, Py<PyAny>)>,
}

fn read_rows_for_group(
    py: Python<'_>,
    conn: &Connection,
    code: &str,
    headings: &[HeadingRow],
) -> PyResult<Vec<RowData>> {
    // Read from `v_<code>` (the view) instead of `g_<code>` (the table):
    // the view lowercases every heading column to match the typed
    // class's Python attribute names, includes the parent_id chain
    // we need, and CAST'ing the UUID id/parent_id to VARCHAR sidesteps
    // duckdb-rs's lack of a native Uuid `Value` variant.
    let view = format!("v_{}", code.to_lowercase());
    let sql = format!(
        "SELECT id::VARCHAR AS id, parent_id::VARCHAR AS parent_id, \
         * EXCLUDE (id, parent_id) FROM {view}"
    );
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| PyRuntimeError::new_err(format!("{view}: {e}")))?;
    let mut rows = stmt
        .query([])
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;

    let col_names: Vec<String> = {
        let stmt_ref = rows
            .as_ref()
            .ok_or_else(|| PyRuntimeError::new_err("no statement after query"))?;
        (0..stmt_ref.column_count())
            .map(|i| {
                stmt_ref
                    .column_name(i)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|_| format!("col_{i}"))
            })
            .collect()
    };

    // `_spec_headings.name` is UPPERCASE; `g_<code>` columns are
    // lowercase. Build a lowercased-heading lookup so we know which
    // columns are headings (vs id / parent_id / _content_hash etc.)
    // and what their AGS type is for typed extraction.
    let headings_by_lower: HashMap<String, &HeadingRow> = headings
        .iter()
        .map(|h| (h.name.to_lowercase(), h))
        .collect();

    let mut out: Vec<RowData> = Vec::new();
    while let Some(row) = rows
        .next()
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?
    {
        let mut id: Option<String> = None;
        let mut parent_id: Option<String> = None;
        let mut field_values: Vec<(String, Py<PyAny>)> = Vec::new();

        for (i, col) in col_names.iter().enumerate() {
            let v: DuckValue = row
                .get(i)
                .map_err(|e| PyRuntimeError::new_err(format!("{col}: {e}")))?;
            match col.as_str() {
                "id" => {
                    if let DuckValue::Text(s) = &v {
                        id = Some(s.clone());
                    }
                }
                "parent_id" => {
                    if let DuckValue::Text(s) = &v {
                        parent_id = Some(s.clone());
                    }
                }
                "_content_hash" => {
                    // internal — not a class field
                }
                other if headings_by_lower.contains_key(other) => {
                    let py_val = duckvalue_to_py(py, &v)?;
                    field_values.push((other.to_string(), py_val));
                }
                _ => {
                    // Unknown column (e.g. an inherited KEY duplicated
                    // into a child group's table). Skip — the typed
                    // class won't accept it as a kwarg.
                }
            }
        }

        let row_id =
            id.ok_or_else(|| PyRuntimeError::new_err(format!("{view}: row without id")))?;
        out.push(RowData {
            id: row_id,
            parent_id,
            field_values,
        });
    }
    Ok(out)
}

// --- DuckDB value → Python object ----------------------------------

fn duckvalue_to_py(py: Python<'_>, v: &DuckValue) -> PyResult<Py<PyAny>> {
    Ok(match v {
        DuckValue::Null => py.None(),
        DuckValue::Boolean(b) => b.into_pyobject(py)?.to_owned().unbind().into_any(),
        DuckValue::TinyInt(i) => (*i as i64).into_pyobject(py)?.unbind().into_any(),
        DuckValue::SmallInt(i) => (*i as i64).into_pyobject(py)?.unbind().into_any(),
        DuckValue::Int(i) => (*i as i64).into_pyobject(py)?.unbind().into_any(),
        DuckValue::BigInt(i) => i.into_pyobject(py)?.unbind().into_any(),
        DuckValue::HugeInt(i) => i.to_string().into_pyobject(py)?.unbind().into_any(),
        DuckValue::UTinyInt(i) => (*i as u64).into_pyobject(py)?.unbind().into_any(),
        DuckValue::USmallInt(i) => (*i as u64).into_pyobject(py)?.unbind().into_any(),
        DuckValue::UInt(i) => (*i as u64).into_pyobject(py)?.unbind().into_any(),
        DuckValue::UBigInt(i) => i.into_pyobject(py)?.unbind().into_any(),
        DuckValue::Float(f) => (*f as f64).into_pyobject(py)?.unbind().into_any(),
        DuckValue::Double(f) => f.into_pyobject(py)?.unbind().into_any(),
        DuckValue::Decimal(d) => {
            // Decimal → f64 (matches what `_spec_headings.canonical_type`
            // = "decimal" maps to elsewhere). Parse-fallback to string
            // if the decimal has more precision than f64 can hold.
            let s = d.to_string();
            match s.parse::<f64>() {
                Ok(f) => f.into_pyobject(py)?.unbind().into_any(),
                Err(_) => s.into_pyobject(py)?.unbind().into_any(),
            }
        }
        DuckValue::Text(s) => s.as_str().into_pyobject(py)?.unbind().into_any(),
        DuckValue::Enum(s) => s.as_str().into_pyobject(py)?.unbind().into_any(),
        DuckValue::Blob(b) => pyo3::types::PyBytes::new(py, b).unbind().into_any(),
        DuckValue::Date32(days) => {
            // DuckDB Date32 = days since Unix epoch (1970-01-01).
            let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).expect("epoch valid");
            let date = epoch
                .checked_add_signed(chrono::Duration::days(*days as i64))
                .ok_or_else(|| PyRuntimeError::new_err("date overflow"))?;
            date.into_pyobject(py)?.unbind().into_any()
        }
        DuckValue::Time64(unit, t) => {
            let nanos = unit_to_nanos(*unit, *t);
            let secs_total = nanos.div_euclid(1_000_000_000);
            let ns = (nanos.rem_euclid(1_000_000_000)) as u32;
            let h = (secs_total / 3600).rem_euclid(24) as u32;
            let m = ((secs_total / 60) % 60) as u32;
            let s = (secs_total % 60) as u32;
            let time = NaiveTime::from_hms_nano_opt(h, m, s, ns)
                .ok_or_else(|| PyRuntimeError::new_err("time overflow"))?;
            time.into_pyobject(py)?.unbind().into_any()
        }
        DuckValue::Timestamp(unit, ts) => {
            let nanos = unit_to_nanos(*unit, *ts);
            let secs = nanos.div_euclid(1_000_000_000);
            let ns_rem = nanos.rem_euclid(1_000_000_000) as u32;
            let dt = DateTime::from_timestamp(secs, ns_rem)
                .ok_or_else(|| PyRuntimeError::new_err("timestamp overflow"))?;
            dt.naive_utc().into_pyobject(py)?.unbind().into_any()
        }
        _ => py.None(),
    })
}

fn unit_to_nanos(unit: TimeUnit, value: i64) -> i64 {
    match unit {
        TimeUnit::Second => value.saturating_mul(1_000_000_000),
        TimeUnit::Millisecond => value.saturating_mul(1_000_000),
        TimeUnit::Microsecond => value.saturating_mul(1_000),
        TimeUnit::Nanosecond => value,
    }
}

// --- parent-child attachment --------------------------------------

fn attach_child_to_parent(
    py: Python<'_>,
    parent: &Py<PyAny>,
    child_code: &str,
    child: &Py<PyAny>,
) -> PyResult<()> {
    // Convention (matches build.rs codegen + ags5_models._modelgen):
    // each child group's list lives on the parent at field name
    // `{child_code.lower()}s`. For compiled-known children the field
    // exists as a Py<PyList> already; for passthrough children we
    // create a Python list on the parent's __dict__ via setattr
    // (enabled by #[pyclass(dict)] on every compiled class).
    let field_name = format!("{}s", child_code.to_lowercase());
    let parent_bound = parent.bind(py);
    if let Ok(existing) = parent_bound.getattr(field_name.as_str()) {
        existing.call_method1("append", (child.clone_ref(py),))?;
    } else {
        let list = PyList::empty(py);
        list.append(child.clone_ref(py))?;
        parent_bound.setattr(field_name.as_str(), list)?;
    }
    Ok(())
}
