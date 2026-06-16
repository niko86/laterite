//! PyO3 wrappers for the Rust-backed Excel I/O lib functions.
//!
//! Exposes `ags4_to_excel` and `excel_to_ags4` to Python; thin Python
//! wrappers in `laterite.compat` (named `AGS4_to_excel` / `excel_to_AGS4`
//! to match python-ags4) route through these. These are pure AGS4↔XLSX
//! conversions (no DuckDB) — renamed off the legacy `ags5db_*` prefix
//! (W2): they have nothing to do with the `.ags5db` engine.

use std::path::PathBuf;

use laterite_core::excel::ExcelStats;
use pyo3::Bound;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyModule};
use pyo3::wrap_pyfunction;

use crate::map_cli_err;

fn stats_to_pydict<'py>(py: Python<'py>, stats: ExcelStats) -> PyResult<Bound<'py, PyDict>> {
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
    let stats = laterite_core::excel::ags4_to_excel(
        &PathBuf::from(input_file),
        &PathBuf::from(output_file),
        ordered_keys,
    )
    .map_err(map_cli_err)?;
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
    let stats = laterite_core::excel::excel_to_ags4(
        &PathBuf::from(input_file),
        &PathBuf::from(output_file),
        format_numeric_columns,
    )
    .map_err(map_cli_err)?;
    stats_to_pydict(py, stats)
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(ags4_to_excel, m)?)?;
    m.add_function(wrap_pyfunction!(excel_to_ags4, m)?)?;
    Ok(())
}
