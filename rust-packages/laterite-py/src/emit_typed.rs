//! `emit_ags4_from_arrow` — the native (PyO3) AGS4 producer.
//!
//! The host hands us per-group Arrow tables (the read boundary reversed):
//! a pandas/polars frame goes `con.register(...)` into the Python-owned
//! DuckDB engine, and DuckDB streams it back as an Arrow C-stream capsule
//! — **pyarrow-free for both backends**, because DuckDB's own scanner +
//! Arrow exporter do the work, not pyarrow. `pyo3_arrow::PyTable` imports
//! that capsule zero-copy. Each batch streams through
//! `laterite_ags4_emit::emit_ags4_from_arrow` (the shared Arrow door the wasm
//! and node hosts use too), which formats each cell straight off its array
//! (via `ags4_str` for typed non-strings + dictionary UNIT/TYPE fill — no
//! row-major input copy, #790) and applies the chosen validity mode
//! (`AutoFix` / Report / Strict).

use arrow::array::{
    Array, BooleanArray, Float32Array, Float64Array, Int8Array, Int16Array, Int32Array, Int64Array,
    LargeStringArray, StringArray, UInt8Array, UInt16Array, UInt32Array, UInt64Array,
};
use arrow::datatypes::DataType;
use arrow::util::display::{ArrayFormatter, FormatOptions};
use laterite_ags4_emit::{
    ArrowGroup, DictVersion, EmitMode, EmitOpts, emit_ags4_from_arrow as engine_emit_from_arrow,
    emit_ags4_from_arrow_unchecked as engine_emit_from_arrow_unchecked,
};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3_arrow::PyTable;

// Both parsers are `laterite_ags4_hostopts` (#923) — the one copy of the
// option normalisation every surface shares — narrowed to this boundary's error
// type. The edition set, the fallback and the rejection message are generated
// from ags_dictionary.json inside the shared module.
fn parse_edition(s: Option<&str>) -> PyResult<DictVersion> {
    laterite_ags4_hostopts::edition_or_fallback(s).map_err(|e| PyRuntimeError::new_err(e.message))
}

fn parse_mode(s: Option<&str>) -> PyResult<EmitMode> {
    laterite_ags4_hostopts::write_mode(s).map_err(|e| PyRuntimeError::new_err(e.message))
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

    let mut groups: Vec<ArrowGroup> = Vec::with_capacity(tables.len());
    for (code, table) in tables {
        let (batches, schema) = table.into_inner();
        // The Arrow door is shared with the wasm and node hosts in
        // laterite-ags4-emit; `opts.edition` drives the #695 DT-precision
        // rendering there (a typed temporal column renders at the precision
        // its heading's declared UNIT asks for, not Arrow's canonical form).
        // Streaming: each cell formats straight off its array — no transposed
        // input copy on top of the caller's own frames (#788/#789/#790 hold
        // the peak records this retired).
        let u = units.as_ref().and_then(|m| m.get(&code)).cloned();
        let t = types.as_ref().and_then(|m| m.get(&code)).cloned();
        groups.push(ArrowGroup {
            code,
            schema,
            batches,
            units: u,
            types: t,
        });
    }

    let res = engine_emit_from_arrow(groups, &opts)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    let findings_json = serde_json::to_string(&res.findings).unwrap_or_else(|_| "{}".into());
    let bytes = PyBytes::new(py, &res.bytes).unbind();
    let applied = crate::fixes_to_pylist(py, &res.applied)?;
    Ok((bytes, findings_json, applied, res.fixes_applied))
}

/// The unchecked door (#858): [`emit_ags4_from_arrow`]'s marshalling handed
/// to the engine's judge-free entry — bytes out, nothing validated, pinned
/// byte-identical to the `report` build. No mode / synthesis / TRAN
/// parameters on purpose: there is no verdict for a mode to act on, and
/// synthesis + TRAN stamping are conveniences whose gaps only a judge would
/// have reported.
#[pyfunction]
#[pyo3(signature = (tables, edition=None, units=None, types=None))]
// PyO3 boundary: owns the deserialized input
#[allow(clippy::needless_pass_by_value)]
pub fn emit_ags4_from_arrow_unchecked(
    py: Python<'_>,
    tables: Vec<(String, PyTable)>,
    edition: Option<String>,
    units: Option<std::collections::HashMap<String, std::collections::HashMap<String, String>>>,
    types: Option<std::collections::HashMap<String, std::collections::HashMap<String, String>>>,
) -> PyResult<Py<PyBytes>> {
    let edition = parse_edition(edition.as_deref())?;
    let mut groups: Vec<ArrowGroup> = Vec::with_capacity(tables.len());
    for (code, table) in tables {
        let (batches, schema) = table.into_inner();
        let u = units.as_ref().and_then(|m| m.get(&code)).cloned();
        let t = types.as_ref().and_then(|m| m.get(&code)).cloned();
        groups.push(ArrowGroup {
            code,
            schema,
            batches,
            units: u,
            types: t,
        });
    }
    let bytes = engine_emit_from_arrow_unchecked(groups, edition)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    Ok(PyBytes::new(py, &bytes).unbind())
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
pub fn emit_ags4_compat(py: Python<'_>, tables: Vec<(String, PyTable)>) -> PyResult<Py<PyBytes>> {
    // The shared, GUARDED verbatim writer (was `laterite-py`'s own private emitter, which
    // lacked the embedded-CR/LF guard and could split a DATA row across two lines, #423).
    // `trailing_blank_line = true` keeps `compat` byte-faithful to python-ags4's
    // `dataframe_to_AGS4`; the guard is the only behaviour change — a cell containing a
    // newline is REFUSED, not silently torn into an illegal file.
    //
    // Streamed group-at-a-time (#805): one group's string matrix is live at a
    // time beside the output, not every group's at once — and the return is
    // `bytes`, so the Python side writes it straight to disk instead of
    // encoding a `str` it never wanted (the UTF-8 bytes ARE the file).
    let mut stream = laterite_ags4_emit::MatrixStream::new(Vec::new(), true);
    for (code, table) in tables {
        let matrix = compat_group_matrix(table);
        stream
            .group(&code, &matrix)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
    }
    let out = stream
        .finish()
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    Ok(PyBytes::new(py, &out).unbind())
}

/// `compat`'s all-Rust AGS4 write, straight to `path` (#805): groups stream
/// through [`laterite_ags4_emit::MatrixStream`] into a buffered temp file in
/// the destination's directory, renamed over `path` only on success — so the
/// refusal contract is preserved (a rejected cell, e.g. an embedded newline,
/// leaves whatever was at `path` untouched) while the process never holds
/// more than one group's matrix beside the file buffer.
#[pyfunction]
pub fn emit_ags4_compat_to_path(tables: Vec<(String, PyTable)>, path: &str) -> PyResult<()> {
    use std::io::Write as _;

    let dest = std::path::Path::new(path);
    let dir = dest.parent().filter(|p| !p.as_os_str().is_empty());
    let mut tmp = tempfile::Builder::new()
        .prefix(".laterite-compat-")
        .suffix(".tmp")
        .tempfile_in(dir.unwrap_or_else(|| std::path::Path::new(".")))
        .map_err(|e| PyRuntimeError::new_err(format!("compat write: temp file: {e}")))?;
    {
        let mut stream =
            laterite_ags4_emit::MatrixStream::new(std::io::BufWriter::new(tmp.as_file_mut()), true);
        for (code, table) in tables {
            let matrix = compat_group_matrix(table);
            stream
                .group(&code, &matrix)
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
        }
        let mut buf = stream
            .finish()
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        buf.flush()
            .map_err(|e| PyRuntimeError::new_err(format!("compat write: flush: {e}")))?;
    }
    // Success is the only path that touches `dest`; every error above drops
    // `tmp`, which unlinks it.
    tmp.persist(dest)
        .map_err(|e| PyRuntimeError::new_err(format!("compat write: rename into place: {e}")))?;
    Ok(())
}

/// One python-ags4-shaped frame → its verbatim string matrix: the column
/// names as the HEADING row, then every data row's cells stringified (the
/// loop that was `compat._matrix_from_df`).
fn compat_group_matrix(table: PyTable) -> Vec<Vec<String>> {
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
    matrix
}
