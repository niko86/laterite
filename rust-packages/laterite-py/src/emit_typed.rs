//! `emit_ags4_from_arrow` — the native (PyO3) AGS4 producer.
//!
//! The host hands us per-group Arrow tables (the read boundary reversed):
//! a pandas/polars frame goes `con.register(...)` into the Python-owned
//! DuckDB engine, and DuckDB streams it back as an Arrow C-stream capsule
//! — **pyarrow-free for both backends**, because DuckDB's own scanner +
//! Arrow exporter do the work, not pyarrow. `pyo3_arrow::PyTable` imports
//! that capsule zero-copy. Each batch is transposed to typed
//! `serde_json::Value` rows by `laterite_ags4_emit::group_from_arrow` (the shared
//! Arrow→Value conversion the wasm host uses too) and fed to the orchestrator,
//! which formats (via `ags4_str` + dictionary UNIT/TYPE fill) and applies the
//! chosen validity mode (AutoFix / Report / Strict).

use arrow::array::{
    Array, BooleanArray, Float32Array, Float64Array, Int8Array, Int16Array, Int32Array, Int64Array,
    LargeStringArray, StringArray, UInt8Array, UInt16Array, UInt32Array, UInt64Array,
};
use arrow::datatypes::DataType;
use arrow::util::display::{ArrayFormatter, FormatOptions};
use laterite_ags4_emit::{DictVersion, EmitMode, EmitOpts, GroupInput, emit_ags4};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3_arrow::PyTable;

use crate::emit::GroupBlock;

fn parse_edition(s: Option<&str>) -> PyResult<DictVersion> {
    // Both the accepted SET and the rejection message come from the dictionary:
    // `from_edition` + `editions_joined` are generated from ags_dictionary.json.
    // This was a hand-written match with a hand-written message listing the editions
    // a second time — two copies of one set, in one function.
    //
    // `auto` resolves to FALLBACK (also generated: the union's `fallback_edition`),
    // which is V4_1_1 — the value this used to hard-code.
    match s.map(str::trim) {
        None | Some("") | Some("auto") => Ok(laterite_ags4_validator::dict::FALLBACK),
        Some(other) => DictVersion::from_edition(other).ok_or_else(|| {
            PyRuntimeError::new_err(format!(
                "unknown edition {other:?}; expected {}",
                laterite_ags4_validator::editions_joined("|")
            ))
        }),
    }
}

fn parse_mode(s: Option<&str>) -> PyResult<EmitMode> {
    match s.map(str::trim).map(str::to_ascii_lowercase).as_deref() {
        None | Some("") | Some("autofix") => Ok(EmitMode::AutoFix),
        Some("report") => Ok(EmitMode::Report),
        Some("strict") => Ok(EmitMode::Strict),
        Some(other) => Err(PyRuntimeError::new_err(format!(
            "unknown mode {other:?}; expected autofix|report|strict"
        ))),
    }
}

/// Build valid AGS4 from per-group Arrow tables.
///
/// `tables` is an ordered list of `(group_code, arrow_table)` — each
/// `arrow_table` is anything exposing the Arrow C-stream interface (a
/// DuckDB relation, a polars frame, …); its column names are the AGS
/// headings. Returns `(bytes, findings_json, applied, fixes_applied)` where
/// `findings_json` is the validator's `{rule: [finding, …]}` map and `applied`
/// is the safe-fix ledger (the same `{kind,label,rule,line,risk}` shape `fix()`
/// returns) AutoFix made — `fixes_applied` is its length (#294 F#7).
#[pyfunction]
#[pyo3(signature = (tables, edition=None, mode=None, units=None, types=None))]
pub fn emit_ags4_from_arrow<'py>(
    py: Python<'py>,
    tables: Vec<(String, PyTable)>,
    edition: Option<String>,
    mode: Option<String>,
    // Per-heading UNIT/TYPE overrides, keyed `{code → {heading → value}}` (#294
    // F#9). A group/heading absent from the map keeps the dictionary default.
    units: Option<std::collections::HashMap<String, std::collections::HashMap<String, String>>>,
    types: Option<std::collections::HashMap<String, std::collections::HashMap<String, String>>>,
) -> PyResult<(Py<PyBytes>, String, Bound<'py, pyo3::types::PyList>, usize)> {
    let opts = EmitOpts {
        mode: parse_mode(mode.as_deref())?,
        edition: parse_edition(edition.as_deref())?,
    };

    let mut groups: Vec<GroupInput> = Vec::with_capacity(tables.len());
    for (code, table) in tables {
        let (batches, schema) = table.into_inner();
        let u = units.as_ref().and_then(|m| m.get(&code));
        let t = types.as_ref().and_then(|m| m.get(&code));
        // The Arrow→Value transpose is shared with the wasm host in laterite-ags4-emit.
        groups.push(laterite_ags4_emit::group_from_arrow_with_meta(
            code,
            schema.as_ref(),
            &batches,
            u,
            t,
        ));
    }

    let res = emit_ags4(&groups, &opts).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let findings_json = serde_json::to_string(&res.findings).unwrap_or_else(|_| "{}".into());
    let bytes = PyBytes::new(py, &res.bytes).unbind();
    let applied = crate::fixes_to_pylist(py, &res.applied)?;
    Ok((bytes, findings_json, applied, res.fixes_applied))
}

/// Reproduce python-ags4's `"" if v is None else str(v)` for `compat`'s
/// byte-verbatim write. python-ags4 frames are all-string (the reader
/// stores raw AGS4 strings), so the Utf8 + null paths are the exact,
/// parity-gated ones; numerics/bools (a non-default workflow) match
/// Python's `str()` best-effort.
fn compat_cell_string(array: &dyn Array, row: usize) -> String {
    if array.is_null(row) {
        return String::new(); // "" if v is None
    }
    macro_rules! istr {
        ($ty:ty) => {
            array
                .as_any()
                .downcast_ref::<$ty>()
                .unwrap()
                .value(row)
                .to_string()
        };
    }
    match array.data_type() {
        DataType::Utf8 => array
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap()
            .value(row)
            .to_string(),
        DataType::LargeUtf8 => array
            .as_any()
            .downcast_ref::<LargeStringArray>()
            .unwrap()
            .value(row)
            .to_string(),
        // Python `str(True)` == "True".
        DataType::Boolean => {
            if array
                .as_any()
                .downcast_ref::<BooleanArray>()
                .unwrap()
                .value(row)
            {
                "True".to_string()
            } else {
                "False".to_string()
            }
        }
        DataType::Int8 => istr!(Int8Array),
        DataType::Int16 => istr!(Int16Array),
        DataType::Int32 => istr!(Int32Array),
        DataType::Int64 => istr!(Int64Array),
        DataType::UInt8 => istr!(UInt8Array),
        DataType::UInt16 => istr!(UInt16Array),
        DataType::UInt32 => istr!(UInt32Array),
        DataType::UInt64 => istr!(UInt64Array),
        DataType::Float32 => py_float_str(
            array
                .as_any()
                .downcast_ref::<Float32Array>()
                .unwrap()
                .value(row) as f64,
        ),
        DataType::Float64 => py_float_str(
            array
                .as_any()
                .downcast_ref::<Float64Array>()
                .unwrap()
                .value(row),
        ),
        _ => match ArrayFormatter::try_new(array, &FormatOptions::default()) {
            Ok(fmt) => fmt.value(row).to_string(),
            Err(_) => String::new(),
        },
    }
}

/// Python prints integral floats with a trailing `.0` (`str(13.0) ==
/// "13.0"`), which Rust's `{}` drops; non-integral uses the shortest
/// round-trip, matching Python for the common cases.
fn py_float_str(f: f64) -> String {
    if f.is_finite() && f == f.trunc() && f.abs() < 1e16 {
        format!("{f:.1}")
    } else {
        format!("{f}")
    }
}

/// `compat`'s all-Rust AGS4 write. The frames are python-ags4-shaped:
/// the column **names** are the HEADING line (row 0), and every data row
/// carries its `UNIT`/`TYPE`/`DATA` tag in the `"HEADING"` column. We
/// stringify each cell (the loop that was `compat._matrix_from_df`) and
/// hand the verbatim string matrix to the existing byte-faithful emitter.
/// Columns arrive pre-selected/ordered by the Python side, so the schema
/// field order *is* the AGS heading order.
#[pyfunction]
pub fn emit_ags4_compat(tables: Vec<(String, PyTable)>) -> PyResult<String> {
    let mut blocks: Vec<GroupBlock> = Vec::with_capacity(tables.len());
    for (code, table) in tables {
        let (batches, schema) = table.into_inner();
        let header: Vec<String> = schema
            .fields()
            .iter()
            .map(|f| f.name().to_string())
            .collect();
        let mut matrix: Vec<Vec<String>> = Vec::new();
        matrix.push(header);
        for batch in &batches {
            for r in 0..batch.num_rows() {
                let mut cells = Vec::with_capacity(batch.num_columns());
                for c in 0..batch.num_columns() {
                    cells.push(compat_cell_string(batch.column(c).as_ref(), r));
                }
                matrix.push(cells);
            }
        }
        blocks.push(GroupBlock { code, matrix });
    }
    Ok(crate::emit::emit(&blocks))
}
