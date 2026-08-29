//! `emit_ags4_from_arrow` — the native (PyO3) AGS4 producer.
//!
//! The host hands us per-group Arrow tables (the read boundary reversed):
//! a pandas/polars frame goes `con.register(...)` into the Python-owned
//! DuckDB engine, and DuckDB streams it back as an Arrow C-stream capsule
//! — **pyarrow-free for both backends**, because DuckDB's own scanner +
//! Arrow exporter do the work, not pyarrow. `pyo3_arrow::PyTable` imports
//! that capsule zero-copy. Each batch is transposed to typed
//! `Cell` rows by `laterite_ags4_emit::group_from_arrow` (the shared
//! Arrow→Cell conversion the wasm host uses too) and fed to the orchestrator,
//! which formats (via `ags4_str` for typed non-strings + dictionary UNIT/TYPE
//! fill) and applies the
//! chosen validity mode (`AutoFix` / Report / Strict).

use arrow::array::{
    Array, BooleanArray, Float32Array, Float64Array, Int8Array, Int16Array, Int32Array, Int64Array,
    LargeStringArray, StringArray, UInt8Array, UInt16Array, UInt32Array, UInt64Array,
};
use arrow::datatypes::DataType;
use arrow::util::display::{ArrayFormatter, FormatOptions};
use laterite_ags4_emit::{DictVersion, EmitMode, EmitOpts, GroupInput, emit_ags4_owned};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3_arrow::PyTable;

fn parse_edition(s: Option<&str>) -> PyResult<DictVersion> {
    // Both the accepted SET and the rejection message come from the dictionary:
    // `from_edition` + `editions_joined` are generated from ags_dictionary.json.
    // This was a hand-written match with a hand-written message listing the editions
    // a second time — two copies of one set, in one function.
    //
    // `auto` resolves to FALLBACK (also generated: the union's `fallback_edition`),
    // which is V4_1_1 — the value this used to hard-code.
    match s.map(str::trim) {
        None | Some("" | "auto") => Ok(laterite_ags4_validator::dict::FALLBACK),
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
        None | Some("" | "autofix") => Ok(EmitMode::AutoFix),
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
/// returns) `AutoFix` made — `fixes_applied` is its length (#294 F#7).
#[pyfunction]
#[allow(clippy::too_many_arguments)]
#[pyo3(signature = (tables, edition=None, mode=None, units=None, types=None, synthesise_metadata=false, tran_issue=None, tran_date=None, tran_producer=None, tran_recipient=None, tran_status=None, tran_description=None, tran_remarks=None))]
// PyO3 boundary: owns the deserialized input
#[allow(clippy::needless_pass_by_value)]
pub fn emit_ags4_from_arrow(
    py: Python<'_>,
    tables: Vec<(String, PyTable)>,
    edition: Option<String>,
    mode: Option<String>,
    // Per-heading UNIT/TYPE overrides, keyed `{code → {heading → value}}` (#294
    // F#9). A group/heading absent from the map keeps the dictionary default.
    units: Option<std::collections::HashMap<String, std::collections::HashMap<String, String>>>,
    types: Option<std::collections::HashMap<String, std::collections::HashMap<String, String>>>,
    // Off unless asked: minting UNIT/TYPE/TRAN/ABBR the caller never wrote is
    // opt-in across every surface (2026-07-24). See EmitOpts::synthesise_metadata.
    synthesise_metadata: bool,
    // The transmission this file represents. `None` (all five absent) means no
    // TRAN is minted and Rule 14 reports the gap — the engine cannot know who
    // sent what to whom, and a placeholder that SATISFIES Rule 14 is worse than
    // an honest absence. Same five arguments `merge` takes on this surface.
    tran_issue: Option<String>,
    tran_date: Option<String>,
    tran_producer: Option<String>,
    tran_recipient: Option<String>,
    tran_status: Option<String>,
    tran_description: Option<String>,
    tran_remarks: Option<String>,
) -> PyResult<(Py<PyBytes>, String, Bound<'_, pyo3::types::PyList>, usize)> {
    let opts = EmitOpts {
        mode: parse_mode(mode.as_deref())?,
        edition: parse_edition(edition.as_deref())?,
        tran: laterite_ags4_emit::TranStamp::from_parts(
            tran_issue,
            tran_date,
            tran_producer,
            tran_recipient,
            tran_status,
            tran_description,
            tran_remarks,
        )
        .map_err(|e| PyValueError::new_err(e.to_string()))?,
        synthesise_metadata,
    };

    let mut groups: Vec<GroupInput> = Vec::with_capacity(tables.len());
    for (code, table) in tables {
        let (batches, schema) = table.into_inner();
        let u = units.as_ref().and_then(|m| m.get(&code));
        let t = types.as_ref().and_then(|m| m.get(&code));
        // The Arrow→Cell transpose is shared with the wasm host in laterite-ags4-emit.
        // The edition goes in so a typed temporal column is rendered at the
        // precision its heading's declared UNIT asks for, instead of Arrow's
        // canonical form — otherwise a date-only DT cell read from disk
        // re-emits as midnight ISO and fails the Rule 8 its own heading
        // declares (#695).
        groups.push(laterite_ags4_emit::group_from_arrow_with_meta_at_edition(
            code,
            schema.as_ref(),
            &batches,
            u,
            t,
            Some(opts.edition),
        ));
    }

    // Consuming entry: `groups` holds a transposed cell copy on top of the
    // caller's own frames — the borrowed entry kept all of it live across the
    // write and the validating re-parse (#788/#789 hold the records; #790
    // shrank the cells themselves from `serde_json::Value` to `Cell`).
    let res = emit_ags4_owned(groups, &opts).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
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
        DataType::Float32 => py_float_str(f64::from(
            array
                .as_any()
                .downcast_ref::<Float32Array>()
                .unwrap()
                .value(row),
        )),
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
    // Exact equality is the point here, not a false positive: this tests
    // whether `f` IS a whole number (Python's `str(13.0) == "13.0"`), so
    // comparing to its own `trunc()` needs bit-exact equality — an epsilon
    // check would misclassify genuinely non-integral floats near a boundary.
    #[allow(clippy::float_cmp)]
    let is_integral = f.is_finite() && f == f.trunc() && f.abs() < 1e16;
    if is_integral {
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
    let mut blocks: Vec<(String, Vec<Vec<String>>)> = Vec::with_capacity(tables.len());
    for (code, table) in tables {
        let (batches, schema) = table.into_inner();
        let header: Vec<String> = schema.fields().iter().map(|f| f.name().clone()).collect();
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
        blocks.push((code, matrix));
    }
    // The shared, GUARDED verbatim writer (was `laterite-py`'s own private emitter, which
    // lacked the embedded-CR/LF guard and could split a DATA row across two lines, #423).
    // `trailing_blank_line = true` keeps `compat` byte-faithful to python-ags4's
    // `dataframe_to_AGS4`; the guard is the only behaviour change — a cell containing a
    // newline is now REFUSED, not silently torn into an illegal file.
    let mut out = Vec::new();
    laterite_ags4_emit::write_ags4_matrix(&mut out, &blocks, true)
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
    String::from_utf8(out).map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
}
