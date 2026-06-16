//! PyO3 wrappers over `laterite_core::ags_types`.
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

use laterite_core::ags_types::{
    CanonicalType, canonical_type as rs_canonical_type, display_hint as rs_display_hint,
};

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
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
    rs_canonical_type(ags_type).map(|c| c.as_str())
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

    let ct = match rs_canonical_type(ags_type) {
        Some(c) => c,
        // Unknown AGS type — Python returns the trimmed string. Preserve.
        None => return Ok(s.into_pyobject(py)?.into_any()),
    };

    match ct {
        CanonicalType::String | CanonicalType::Enum => Ok(s.into_pyobject(py)?.into_any()),
        CanonicalType::Integer => match s.parse::<f64>() {
            // Python's `int(float(s))` tolerance for "5.0" notation.
            Ok(f) if f.is_finite() => Ok((f as i64).into_pyobject(py)?.into_any()),
            _ => Ok(PyNone::get(py).to_owned().into_any()),
        },
        CanonicalType::Decimal => match s.parse::<f64>() {
            Ok(f) if f.is_finite() => Ok(f.into_pyobject(py)?.into_any()),
            _ => Ok(PyNone::get(py).to_owned().into_any()),
        },
        CanonicalType::Datetime => Ok(parse_datetime(s)
            .map(|dt| dt.into_pyobject(py).map(|b| b.into_any()))
            .transpose()?
            .unwrap_or_else(|| PyNone::get(py).to_owned().into_any())),
        CanonicalType::Date => Ok(parse_date(s)
            .map(|d| d.into_pyobject(py).map(|b| b.into_any()))
            .transpose()?
            .unwrap_or_else(|| PyNone::get(py).to_owned().into_any())),
        CanonicalType::Time => Ok(parse_time(s)
            .map(|t| t.into_pyobject(py).map(|b| b.into_any()))
            .transpose()?
            .unwrap_or_else(|| PyNone::get(py).to_owned().into_any())),
        CanonicalType::Bool => Ok(parse_bool(s)
            .map(|b| b.into_pyobject(py).map(|x| x.to_owned().into_any()))
            .transpose()?
            .unwrap_or_else(|| PyNone::get(py).to_owned().into_any())),
    }
}

// --- format tables ---------------------------------------------------
//
// Mirror Python's `_DATETIME_FORMATS` / `_DATE_FORMATS` / `_TIME_FORMATS`
// exactly so cross-side parity is byte-faithful. (`ags_types.rs`'s own
// `DATETIME_FORMATS` is intended for JSON-Value output and shares the
// same order; we duplicate here to keep this module's intent — produce
// Python-typed values — separate from the JSON path.)

const DATETIME_FORMATS: &[&str] = &[
    "%Y-%m-%d %H:%M:%S",
    "%Y-%m-%d %H:%M",
    "%Y-%m-%dT%H:%M:%S",
    "%Y-%m-%d",
    "%Y/%m/%d",
    "%d/%m/%Y",
];
const DATE_FORMATS: &[&str] = &["%Y-%m-%d", "%Y/%m/%d", "%d/%m/%Y"];
const TIME_FORMATS: &[&str] = &["%H:%M:%S", "%H:%M"];

fn parse_datetime(s: &str) -> Option<NaiveDateTime> {
    for fmt in DATETIME_FORMATS {
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, fmt) {
            return Some(dt);
        }
        // Date-only DT fallback (matches the Stage C item 1 fix in
        // ags_types.rs): a date-only string under a `yyyy-mm-dd` format
        // is legally a DT value — promote to midnight.
        if let Ok(d) = NaiveDate::parse_from_str(s, fmt) {
            if let Some(dt) = d.and_hms_opt(0, 0, 0) {
                return Some(dt);
            }
        }
    }
    None
}

fn parse_date(s: &str) -> Option<NaiveDate> {
    for fmt in DATE_FORMATS {
        if let Ok(d) = NaiveDate::parse_from_str(s, fmt) {
            return Some(d);
        }
    }
    None
}

fn parse_time(s: &str) -> Option<NaiveTime> {
    for fmt in TIME_FORMATS {
        if let Ok(t) = NaiveTime::parse_from_str(s, fmt) {
            return Some(t);
        }
    }
    None
}

fn parse_bool(s: &str) -> Option<bool> {
    let u = s.to_uppercase();
    match u.as_str() {
        "Y" | "YES" | "TRUE" | "1" => Some(true),
        "N" | "NO" | "FALSE" | "0" => Some(false),
        _ => None,
    }
}

pub fn register(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(canonical_type, m)?)?;
    m.add_function(wrap_pyfunction!(display_hint, m)?)?;
    m.add_function(wrap_pyfunction!(parse_value, m)?)?;
    Ok(())
}
