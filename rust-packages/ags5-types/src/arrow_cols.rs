//! AGS4 group → typed Apache Arrow — the *single* typed emission shared by
//! every host that needs typed columns: the browser explorer (`ags4-wasm`
//! → IPC stream → duckdb-wasm) and the native base wheel (`laterite-py`
//! → zero-copy Arrow capsule → polars). Casting runs through this crate's
//! own `parse_value` / `parse_datetime`, so every host types a file
//! *identically* — parity by construction, not by re-implementation.
//!
//! Deliberately decoupled from any parser type. The caller supplies the
//! headings, the per-column AGS type codes, the row count, and a
//! `cell(col, row)` accessor — so `ags5-types` keeps being the tiny
//! wasm-safe leaf it is, gaining (behind the `arrow` feature) the `arrow`
//! dependency but NOT a dependency on the `ags4-validator` parser whose
//! `ParsedGroup` the callers happen to hold.

use std::sync::Arc;

use arrow::array::{
    ArrayRef, BooleanBuilder, Float64Builder, Int64Builder, StringBuilder,
    TimestampMicrosecondBuilder,
};
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatch;
use serde_json::Value;

use crate::{CanonicalType, canonical_type, parse_datetime, parse_value};

/// Build one group's typed Arrow `RecordBatch`: one column per heading,
/// each cast through the canonical-type machinery. `cell(col, row)`
/// returns the raw string for a cell, or `None` for a short/ragged row
/// (→ null, never a panic). A missing TYPE entry falls back to `"X"`
/// (text), matching the native passthrough default.
pub fn build_record_batch<'a, F>(
    headings: &[String],
    ags_types: &[String],
    n_rows: usize,
    cell: F,
) -> Result<RecordBatch, ArrowError>
where
    F: Fn(usize, usize) -> Option<&'a str>,
{
    let ncols = headings.len();
    let mut fields = Vec::with_capacity(ncols);
    let mut columns: Vec<ArrayRef> = Vec::with_capacity(ncols);
    for (col, heading) in headings.iter().enumerate() {
        let ags_type = ags_types.get(col).map(String::as_str).unwrap_or("X");
        let (array, dt) = build_column(n_rows, ags_type, |row| cell(col, row));
        fields.push(Field::new(heading, dt, true));
        columns.push(array);
    }
    let schema = Arc::new(Schema::new(fields));
    RecordBatch::try_new(schema, columns)
}

/// Build one typed Arrow column of `n_rows` cells, reading each via
/// `cell(row)`. Public so a host wanting per-column control (e.g. the
/// native sparse-override pass) reuses the exact same casting.
pub fn build_column<'a, F>(n_rows: usize, ags_type: &str, cell: F) -> (ArrayRef, DataType)
where
    F: Fn(usize) -> Option<&'a str>,
{
    match canonical_type(ags_type) {
        Some(CanonicalType::Integer) => {
            let mut b = Int64Builder::with_capacity(n_rows);
            for row in 0..n_rows {
                match parse_value(cell(row), ags_type) {
                    Value::Number(num) => b.append_option(num.as_i64()),
                    _ => b.append_null(),
                }
            }
            (Arc::new(b.finish()) as ArrayRef, DataType::Int64)
        }
        Some(CanonicalType::Decimal) => {
            let mut b = Float64Builder::with_capacity(n_rows);
            for row in 0..n_rows {
                match parse_value(cell(row), ags_type) {
                    Value::Number(num) => b.append_option(num.as_f64()),
                    _ => b.append_null(),
                }
            }
            (Arc::new(b.finish()) as ArrayRef, DataType::Float64)
        }
        Some(CanonicalType::Bool) => {
            let mut b = BooleanBuilder::with_capacity(n_rows);
            for row in 0..n_rows {
                match parse_value(cell(row), ags_type) {
                    Value::Bool(v) => b.append_value(v),
                    _ => b.append_null(),
                }
            }
            (Arc::new(b.finish()) as ArrayRef, DataType::Boolean)
        }
        Some(CanonicalType::Datetime) => {
            // tz-naive microseconds — matches DuckDB TIMESTAMP and the native
            // .ags5db. parse_datetime (not parse_value, which re-formats back
            // to a string) gives the typed instant; an empty / unparseable
            // cell → null, the same null-ness as parse_value's Datetime arm.
            let mut b = TimestampMicrosecondBuilder::with_capacity(n_rows);
            for row in 0..n_rows {
                let micros = cell(row)
                    .filter(|s| !s.trim().is_empty())
                    .and_then(parse_datetime)
                    .map(|dt| dt.and_utc().timestamp_micros());
                b.append_option(micros);
            }
            (
                Arc::new(b.finish()) as ArrayRef,
                DataType::Timestamp(TimeUnit::Microsecond, None),
            )
        }
        // String / Enum / unknown(None). Date / Time canonical types never
        // arise from real AGS4 codes (only DT → Datetime), so they fall here
        // → Utf8, defensively.
        _ => {
            let mut b = StringBuilder::new();
            for row in 0..n_rows {
                match parse_value(cell(row), ags_type) {
                    Value::String(s) => b.append_value(s),
                    Value::Null => b.append_null(),
                    // String/Enum/unknown always yield String|Null; other
                    // variants can't occur, but keep the match total.
                    other => b.append_value(other.to_string()),
                }
            }
            (Arc::new(b.finish()) as ArrayRef, DataType::Utf8)
        }
    }
}
