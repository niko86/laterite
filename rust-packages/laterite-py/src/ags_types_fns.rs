//! PyO3 wrappers over `laterite_ags4_core::ags_types`.
//!
//! `parse_value` / `canonical_type` / `display_hint` route through Rust
//! so the conversion engine and Python ingest paths share one
//! implementation. Stage C item 1 already proved Rust authoritative for
//! AGS type coercion (the date-only DT fix landed there). What's left is
//! the Python wrapper.
//!
//! `parse_value` returns native Python types (int / float / bool /
//! datetime / date / time / str / None) so callers see the same shape
//! the pure-Python implementation produced. The pyo3 `chrono` feature
//! gives us `IntoPyObject` for `NaiveDateTime` / `NaiveDate` /
//! `NaiveTime` — no manual `PyDateTime` boilerplate needed.
//!
//! The parsing itself is NOT re-implemented here: the format tables and the
//! typed parsers (`parse_datetime` / `parse_date` / `parse_time` /
//! `parse_bool`) live in the leaf (`laterite_ags4_core::ags_types`, i.e.
//! `laterite-ags4-types`) and back the leaf's own `parse_value` too, so there is
//! one parser and one set of format tables (#531). This wrapper only
//! dispatches on `canonical_type` and maps each parsed value to its Python
//! object — the same canonicalisation that feeds `_content_hash`, now with
//! a single source instead of a drift-prone second copy (see #503).

use laterite_ags4_core::ags_types::{
    CanonicalType, canonical_type as rs_canonical_type, display_hint as rs_display_hint,
    parse_ags_decimal, parse_ags_integer, parse_bool, parse_date, parse_datetime, parse_time,
};

use pyo3::prelude::*;
use pyo3::types::PyNone;

/// AGS spec type code → canonical category label
/// (`"string"`, `"integer"`, `"decimal"`, `"datetime"`, `"date"`,
/// `"time"`, `"bool"`, `"enum"`). Returns `None` for unknown codes —
/// matches Python's `_types.canonical_type` semantics, where the
/// Python wrapper re-raises as `ValueError` (the Rust side raises
/// nothing; unknown codes pass through to string in `parse_value`).
#[pyfunction]
fn canonical_type(ags_type: &str) -> Option<&'static str> {
    rs_canonical_type(ags_type).map(laterite_ags4_types::CanonicalType::as_str)
}

/// Presentation hint for numeric AGS types: `"2DP"` → `Some("%.2f")`,
/// `"3SF"` → `Some("%.3g")`, `"1SCI"` → `Some("%.1e")`. `None` for
/// non-numeric / unknown codes.
#[pyfunction]
fn display_hint(ags_type: &str) -> Option<String> {
    rs_display_hint(ags_type)
}

/// Parse an AGS4-shaped raw value into the canonical Python type.
/// Permissive: unparseable values come back as `None`.
///
/// Return shape (matching pure-Python `_types.parse_value`):
///   STRING / ENUM  → `str`
///   INTEGER        → `int`
///   DECIMAL        → `float`
///   DATETIME       → `datetime.datetime`
///   DATE           → `datetime.date`
///   TIME           → `datetime.time`
///   BOOL           → `bool`
///   unknown code   → `str` (the trimmed input)
///   unparseable    → `None`
///
/// `raw` may be ``None`` (explicit null), a Python ``str``, or any value
/// PyO3 can coerce to a string. Empty / whitespace-only strings → `None`.
#[pyfunction]
fn parse_value<'py>(
    py: Python<'py>,
    raw: Option<&str>,
    ags_type: &str,
) -> PyResult<Bound<'py, PyAny>> {
    let s = match raw {
        Some(r) => r.trim(),
        None => return Ok(PyNone::get(py).to_owned().into_any()),
    };
    if s.is_empty() {
        return Ok(PyNone::get(py).to_owned().into_any());
    }

    // Unknown AGS type — Python returns the trimmed string. Preserve.
    let Some(ct) = rs_canonical_type(ags_type) else {
        return Ok(s.into_pyobject(py)?.into_any());
    };

    match ct {
        CanonicalType::String | CanonicalType::Enum => Ok(s.into_pyobject(py)?.into_any()),
        // The same range-guarded parsers the leaf's `parse_value` uses, so the
        // typed-read Python object can't drift from the `_content_hash`
        // canonicalisation (#611 finishes the #531 dedup for Integer/Decimal).
        CanonicalType::Integer => Ok(parse_ags_integer(s)
            .map(|i| i.into_pyobject(py).map(pyo3::Bound::into_any))
            .transpose()?
            .unwrap_or_else(|| PyNone::get(py).to_owned().into_any())),
        CanonicalType::Decimal => Ok(parse_ags_decimal(s)
            .map(|f| f.into_pyobject(py).map(pyo3::Bound::into_any))
            .transpose()?
            .unwrap_or_else(|| PyNone::get(py).to_owned().into_any())),
        CanonicalType::Datetime => Ok(parse_datetime(s)
            .map(|dt| dt.into_pyobject(py).map(pyo3::Bound::into_any))
            .transpose()?
            .unwrap_or_else(|| PyNone::get(py).to_owned().into_any())),
        CanonicalType::Date => Ok(parse_date(s)
            .map(|d| d.into_pyobject(py).map(pyo3::Bound::into_any))
            .transpose()?
            .unwrap_or_else(|| PyNone::get(py).to_owned().into_any())),
        CanonicalType::Time => Ok(parse_time(s)
            .map(|t| t.into_pyobject(py).map(pyo3::Bound::into_any))
            .transpose()?
            .unwrap_or_else(|| PyNone::get(py).to_owned().into_any())),
        CanonicalType::Bool => Ok(parse_bool(s)
            .map(|b| b.into_pyobject(py).map(|x| x.to_owned().into_any()))
            .transpose()?
            .unwrap_or_else(|| PyNone::get(py).to_owned().into_any())),
    }
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(canonical_type, m)?)?;
    m.add_function(wrap_pyfunction!(display_hint, m)?)?;
    m.add_function(wrap_pyfunction!(parse_value, m)?)?;
    Ok(())
}
