//! PyO3 wrappers for the Rust-backed Excel I/O lib functions.
//!
//! Exposes `ags4_to_excel` and `excel_to_ags4` to Python; thin Python
//! wrappers in `laterite.compat` (named `AGS4_to_excel` / `excel_to_AGS4`
//! to match python-ags4) route through these. These are pure AGS4↔XLSX
//! conversions (no DuckDB) — renamed off a legacy prefix (W2): they
//! are generic conversions with no ties to any DuckDB-backed engine.

use std::path::PathBuf;

use laterite_ags4_excel::ExcelStats;
use pyo3::Bound;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList, PyModule};
use pyo3::wrap_pyfunction;

use crate::map_cli_err;

fn stats_to_pydict(py: Python<'_>, stats: ExcelStats) -> PyResult<Bound<'_, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("sheets_written", stats.sheets_written)?;
    dict.set_item("rows_written", stats.rows_written)?;
    let warns = PyList::empty(py);
    for w in stats.warnings {
        warns.append(w)?;
    }
    dict.set_item("warnings", warns)?;
    Ok(dict)
}

#[pyfunction]
#[pyo3(signature = (input_file, output_file, ordered_keys=None))]
fn ags4_to_excel<'py>(
    py: Python<'py>,
    input_file: &str,
    output_file: &str,
    ordered_keys: Option<Vec<String>>,
) -> PyResult<Bound<'py, PyDict>> {
    let stats = laterite_ags4_excel::ags4_to_excel(
        &PathBuf::from(input_file),
        &PathBuf::from(output_file),
        ordered_keys,
    )
    .map_err(|e| map_cli_err(&e))?;
    stats_to_pydict(py, stats)
}

#[pyfunction]
#[pyo3(signature = (input_file, output_file, format_numeric_columns=true))]
fn excel_to_ags4<'py>(
    py: Python<'py>,
    input_file: &str,
    output_file: &str,
    format_numeric_columns: bool,
) -> PyResult<Bound<'py, PyDict>> {
    let stats = laterite_ags4_excel::excel_to_ags4(
        &PathBuf::from(input_file),
        &PathBuf::from(output_file),
        format_numeric_columns,
    )
    .map_err(|e| map_cli_err(&e))?;
    stats_to_pydict(py, stats)
}

/// AGS4 bytes → XLSX bytes, in memory (no filesystem). The bytes twin of
/// `ags4_to_excel` — the same FS-free core the wasm/browser surface uses. Returns
/// `(xlsx_bytes, stats)` so a caller who wants the workbook without a temp file
/// (e.g. streaming it to an upload) gets both.
#[pyfunction]
#[pyo3(signature = (data, ordered_keys=None, recover_duplicate_headings=false, truncate_excess_fields=false))]
fn ags4_bytes_to_xlsx<'py>(
    py: Python<'py>,
    data: &[u8],
    ordered_keys: Option<Vec<String>>,
    recover_duplicate_headings: bool,
    truncate_excess_fields: bool,
) -> PyResult<(Bound<'py, PyBytes>, Bound<'py, PyDict>)> {
    let (xlsx, stats) = laterite_ags4_excel::ags4_bytes_to_xlsx_with(
        data,
        ordered_keys,
        read_opts(recover_duplicate_headings, truncate_excess_fields),
    )
    .map_err(|e| map_cli_err(&e))?;
    Ok((PyBytes::new(py, &xlsx), stats_to_pydict(py, stats)?))
}

/// XLSX bytes → AGS4 bytes, in memory (no filesystem). The bytes twin of
/// `excel_to_ags4`. Returns `(ags_bytes, stats)`.
#[pyfunction]
#[pyo3(signature = (data, format_numeric_columns=true))]
fn xlsx_bytes_to_ags4<'py>(
    py: Python<'py>,
    data: &[u8],
    format_numeric_columns: bool,
) -> PyResult<(Bound<'py, PyBytes>, Bound<'py, PyDict>)> {
    let (ags, stats) = laterite_ags4_excel::xlsx_bytes_to_ags4(data, format_numeric_columns)
        .map_err(|e| map_cli_err(&e))?;
    Ok((PyBytes::new(py, &ags), stats_to_pydict(py, stats)?))
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(ags4_to_excel, m)?)?;
    m.add_function(wrap_pyfunction!(excel_to_ags4, m)?)?;
    m.add_function(wrap_pyfunction!(ags4_bytes_to_xlsx, m)?)?;
    m.add_function(wrap_pyfunction!(xlsx_bytes_to_ags4, m)?)?;
    Ok(())
}

/// Map the surface-level booleans onto core's read policy. Both leniencies are
/// off by default on every read surface — a file the reader cannot represent
/// faithfully is refused; a caller opts in.
fn read_opts(
    recover_duplicate_headings: bool,
    truncate_excess_fields: bool,
) -> laterite_ags4_core::ags4_codec::ReadOptions {
    use laterite_ags4_core::ags4_codec::{DuplicateHeadings, ExcessFields, ReadOptions};
    ReadOptions {
        duplicate_headings: if recover_duplicate_headings {
            DuplicateHeadings::Recover
        } else {
            DuplicateHeadings::Error
        },
        excess_fields: if truncate_excess_fields {
            ExcessFields::Truncate
        } else {
            ExcessFields::Error
        },
    }
}
