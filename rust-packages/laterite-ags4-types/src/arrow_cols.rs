//! AGS4 group → typed Apache Arrow — the *single* typed emission shared by
//! every host that needs typed columns: the browser explorer (`laterite-ags4-wasm`
//! → IPC stream → duckdb-wasm) and the native base wheel (`laterite-py`
//! → zero-copy Arrow capsule → polars). Casting runs through this crate's
//! own `parse_value` / `parse_datetime`, so every host types a file
//! *identically* — parity by construction, not by re-implementation.
//!
//! Deliberately decoupled from any parser type. The caller supplies the
//! headings, the per-column AGS type codes, the row count, and a
//! `cell(col, row)` accessor — so `laterite-ags4-types` keeps being the tiny
//! wasm-safe leaf it is, gaining (behind the `arrow` feature) the `arrow`
//! dependency but NOT a dependency on the `laterite-ags4-validator` parser whose
//! `ParsedGroup` the callers happen to hold.

use std::sync::Arc;

use arrow::array::{
    ArrayRef, BooleanBuilder, Float64Builder, Int64Builder, StringArray, StringBuilder,
    Time64MicrosecondBuilder, TimestampMicrosecondBuilder,
};
use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
use arrow::error::ArrowError;
use arrow::record_batch::RecordBatch;
use chrono::Timelike;
use serde_json::Value;

use crate::{
    CanonicalType, canonical_type, is_time_of_day_unit, parse_ags_decimal, parse_ags_integer,
    parse_datetime, parse_datetime_for_unit, parse_time, parse_value,
};

/// Caller-computed synthetic columns folded into a batch in ONE place, rather
/// than bolted on per host afterward. `ids` prepends `_id` (col 0) / `_parent_id`
/// (col 1); `hashes` appends `_content_hash` LAST (after the headings — the
/// settled trailing position, so heading indices are invariant whether or not
/// the hash is present). The two are independent: a passthrough group has no
/// `_id` but can still carry a `_content_hash` (`ids: None, hashes: Some`) — the
/// 2×2 keyed×hashed case a single-purpose builder would silently drop.
#[derive(Default)]
pub struct SynthColumns<'a> {
    pub ids: Option<&'a [(String, Option<String>)]>,
    pub hashes: Option<&'a [String]>,
}

/// Build a group's typed Arrow `RecordBatch` with optional synthetic columns.
/// The one place `_id`/`_parent_id`/`_content_hash` are attached, so every host
/// gets identical column order and types by construction.
pub fn build_record_batch_synth<'a, F>(
    synth: &SynthColumns,
    headings: &[String],
    ags_types: &[String],
    n_rows: usize,
    cell: F,
) -> Result<RecordBatch, ArrowError>
where
    F: Fn(usize, usize) -> Option<&'a str>,
{
    build_record_batch_synth_with_units(synth, headings, ags_types, &[], n_rows, cell)
}

/// [`build_record_batch_synth`], typing each `DT` column against its heading's
/// declared UNIT (`units[col]`; a missing entry is no UNIT) — see
/// [`build_column_for_unit`]. A reader holding the file's UNIT row should call
/// this one: without the UNIT, a `DT` column declared `hh:mm` or `yyyy-mm`
/// cannot be typed, and every cell in it reads as null (#999).
pub fn build_record_batch_synth_with_units<'a, F>(
    synth: &SynthColumns,
    headings: &[String],
    ags_types: &[String],
    units: &[String],
    n_rows: usize,
    cell: F,
) -> Result<RecordBatch, ArrowError>
where
    F: Fn(usize, usize) -> Option<&'a str>,
{
    let cap = headings.len() + 3;
    let mut fields = Vec::with_capacity(cap);
    let mut columns: Vec<ArrayRef> = Vec::with_capacity(cap);

    if let Some(ids) = synth.ids {
        let mut id_b = StringBuilder::with_capacity(n_rows, n_rows * 36);
        let mut pid_b = StringBuilder::with_capacity(n_rows, n_rows * 36);
        for row in 0..n_rows {
            if let Some((id, parent)) = ids.get(row) {
                id_b.append_value(id);
                pid_b.append_option(parent.as_deref());
            } else {
                // Defensive: ids shorter than n_rows → null pair (never a panic).
                id_b.append_null();
                pid_b.append_null();
            }
        }
        fields.push(Field::new("_id", DataType::Utf8, true));
        columns.push(Arc::new(id_b.finish()) as ArrayRef);
        fields.push(Field::new("_parent_id", DataType::Utf8, true));
        columns.push(Arc::new(pid_b.finish()) as ArrayRef);
    }

    append_heading_columns(
        &mut fields,
        &mut columns,
        headings,
        ags_types,
        units,
        n_rows,
        cell,
    );

    if let Some(hashes) = synth.hashes {
        let mut h_b = StringBuilder::with_capacity(n_rows, n_rows * 36);
        for row in 0..n_rows {
            // Callers pass hashes.len() == n_rows; the fallback keeps the column
            // non-null (never a panic) for a defensively-short slice.
            h_b.append_value(hashes.get(row).map_or("", String::as_str));
        }
        fields.push(Field::new("_content_hash", DataType::Utf8, false));
        columns.push(Arc::new(h_b.finish()) as ArrayRef);
    }

    let schema = Arc::new(Schema::new(fields));
    RecordBatch::try_new(schema, columns)
}

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
    build_record_batch_synth(&SynthColumns::default(), headings, ags_types, n_rows, cell)
}

/// Like [`build_record_batch`], but prepends the two content-addressed key
/// columns — `_id` (col 0) and `_parent_id` (col 1, NULL for a root group) —
/// from caller-computed ids, then the heading columns. The column order, the
/// `Utf8` type, and the root-NULL exactly match the DuckDB
/// extension's `read_ags` recipe, so a file's keys are byte-identical whether
/// they come from the extension or any host wheel. `ids[row]` is the
/// `(_id, _parent_id)` pair (see `laterite_ags4_core::keychain::group_row_ids`,
/// which wraps the one `keychain::row_ids` the extension also calls); a row past
/// the end of `ids` (defensive — callers pass `ids.len() == n_rows`) yields a
/// null id pair. This leaf stays **keychain-free**: the caller owns the id
/// computation, so `laterite-ags4-types` keeps its minimal wasm-safe dependency set.
pub fn build_record_batch_with_ids<'a, F>(
    ids: &[(String, Option<String>)],
    headings: &[String],
    ags_types: &[String],
    n_rows: usize,
    cell: F,
) -> Result<RecordBatch, ArrowError>
where
    F: Fn(usize, usize) -> Option<&'a str>,
{
    build_record_batch_synth(
        &SynthColumns {
            ids: Some(ids),
            hashes: None,
        },
        headings,
        ags_types,
        n_rows,
        cell,
    )
}

/// Build the python-ags4-shaped ("compat") all-`Utf8` `RecordBatch` for one
/// group. Unlike [`build_record_batch_synth`] (which *types* each column), this
/// is the drop-in shape `laterite.compat.AGS4_to_dataframe` hands back: a leading
/// `HEADING` tag column, then one raw-string column per heading, with rows laid
/// out as the `UNIT` row, the `TYPE` row, then the DATA rows. Every cell is a
/// string; a missing/ragged cell is `""` (matching python-ags4, which pads a
/// short DATA row to heading width). No canonical-type casting — compat keeps the
/// file's raw text, so the boxing/typing the typed path pays is skipped.
///
/// Field names are positional (`HEADING`, then `c0`, `c1`, …): python-ags4's
/// duplicate-heading renaming lives Python-side (it also owns the `rename=False`
/// raise), and positional names mean a duplicated heading can never collide in
/// the Arrow schema. `cell(col, row)` returns the raw DATA cell for data row
/// `row` and heading `col` (or `None` → `""`); the batch has `n_data_rows + 2`
/// rows (UNIT + TYPE + DATA).
pub fn build_record_batch_compat<'a, F>(
    headings: &[String],
    units: &[String],
    types: &[String],
    n_data_rows: usize,
    cell: F,
) -> Result<RecordBatch, ArrowError>
where
    F: Fn(usize, usize) -> Option<&'a str>,
{
    let n_rows = n_data_rows + 2;
    let mut fields = Vec::with_capacity(headings.len() + 1);
    let mut columns: Vec<ArrayRef> = Vec::with_capacity(headings.len() + 1);

    // Column 0: the row-tag column python-ags4 names "HEADING".
    let mut tag = StringBuilder::with_capacity(n_rows, n_rows * 4);
    tag.append_value("UNIT");
    tag.append_value("TYPE");
    for _ in 0..n_data_rows {
        tag.append_value("DATA");
    }
    fields.push(Field::new("HEADING", DataType::Utf8, false));
    columns.push(Arc::new(tag.finish()) as ArrayRef);

    // One raw-string column per heading: [unit, type, *data].
    for col in 0..headings.len() {
        let mut b = StringBuilder::with_capacity(n_rows, n_rows * 8);
        b.append_value(units.get(col).map_or("", String::as_str));
        b.append_value(types.get(col).map_or("", String::as_str));
        for row in 0..n_data_rows {
            b.append_value(cell(col, row).unwrap_or(""));
        }
        fields.push(Field::new(format!("c{col}"), DataType::Utf8, false));
        columns.push(Arc::new(b.finish()) as ArrayRef);
    }

    let schema = Arc::new(Schema::new(fields));
    RecordBatch::try_new(schema, columns)
}

/// Build one typed column per heading and push its field + array. Shared by the
/// keyed and unkeyed batch builders so the casting is identical between them.
fn append_heading_columns<'a, F>(
    fields: &mut Vec<Field>,
    columns: &mut Vec<ArrayRef>,
    headings: &[String],
    ags_types: &[String],
    units: &[String],
    n_rows: usize,
    cell: F,
) where
    F: Fn(usize, usize) -> Option<&'a str>,
{
    for (col, heading) in headings.iter().enumerate() {
        let ags_type = ags_types.get(col).map_or("X", String::as_str);
        let unit = units.get(col).map_or("", String::as_str);
        let (array, dt) = build_column_for_unit(n_rows, ags_type, unit, |row| cell(col, row));
        fields.push(Field::new(heading, dt, true));
        columns.push(array);
    }
}

/// One `Utf8` column with `parse_value`'s String preprocessing — trim, then
/// empty → typed null — and nothing else, skipping the per-cell canonical-type
/// dispatch and the `serde_json::Value` boxing the old String arm paid.
fn build_utf8<'a, F>(n_rows: usize, cell: F) -> StringArray
where
    F: Fn(usize) -> Option<&'a str>,
{
    let mut b = StringBuilder::new();
    for row in 0..n_rows {
        match cell(row).map(str::trim).filter(|s| !s.is_empty()) {
            Some(s) => b.append_value(s),
            None => b.append_null(),
        }
    }
    b.finish()
}

/// Build one typed Arrow column of `n_rows` cells, reading each via
/// `cell(row)`. Public so a host wanting per-column control (e.g. the
/// native sparse-override pass) reuses the exact same casting.
///
/// Each arm parses the borrowed cell straight into its typed Arrow builder,
/// bypassing `parse_value` — which re-resolved `canonical_type` per cell
/// (`trim().to_uppercase()` + table lookups) and boxed every value through a
/// `serde_json::Value`. Dropping that per-cell overhead is ~5× faster than the
/// old path and byte-parity with it (proven cell-for-cell in
/// `tests/typed_build_parity.rs`). Deliberately NOT via `arrow::compute::cast`:
/// the cast is a touch slower here (it builds an intermediate `Utf8` column,
/// then a second pass to the numeric type) AND it links the arrow-cast kernels,
/// which bloats the wasm bundle ~3.5 MB — the direct parse needs neither.
pub fn build_column<'a, F>(n_rows: usize, ags_type: &str, cell: F) -> (ArrayRef, DataType)
where
    F: Fn(usize) -> Option<&'a str>,
{
    match canonical_type(ags_type) {
        Some(CanonicalType::Integer) => {
            // `parse_ags_integer` is `int(float(s))` (tolerates `"5.0"`/`"5.7"`,
            // truncates toward zero) with the laterite-dev#611 i64 range guard nulling
            // out-of-range values. A None/empty cell → null. Same result the old
            // `parse_value` Integer arm produced, without the `Value` round-trip.
            let mut b = Int64Builder::with_capacity(n_rows);
            for row in 0..n_rows {
                b.append_option(
                    cell(row)
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .and_then(parse_ags_integer),
                );
            }
            (Arc::new(b.finish()) as ArrayRef, DataType::Int64)
        }
        Some(CanonicalType::Decimal) => {
            // `parse_ags_decimal` is `s.parse::<f64>()` keeping only finite values
            // (inf/NaN → null), so — unlike Arrow's string→float kernel — no
            // separate finite reconciliation is needed. None/empty → null.
            let mut b = Float64Builder::with_capacity(n_rows);
            for row in 0..n_rows {
                b.append_option(
                    cell(row)
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .and_then(parse_ags_decimal),
                );
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
            // DuckDB extension. parse_datetime (not parse_value, which re-formats back
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
        // → Utf8, defensively. `parse_value`'s String arm was just trim +
        // empty→null, which `build_utf8` does directly (no dispatch).
        _ => (
            Arc::new(build_utf8(n_rows, cell)) as ArrayRef,
            DataType::Utf8,
        ),
    }
}

/// [`build_column`], but a `DT` column is typed by the precision its UNIT
/// declares rather than always as a datetime:
///
/// * a time-of-day UNIT (`hh:mm`, `hh:mm:ss`) holds a time with no date, so
///   the column is Arrow `Time64(Microsecond)` — a fixed "datum" date would
///   put a day nobody recorded into the data (#999);
/// * a year or month UNIT (`yyyy`, `yyyy-mm`) reads each value as the start
///   of the period it names (`2026-03` → 2026-03-01 00:00), as a date-only
///   value already reads as midnight;
/// * anything else is exactly [`build_column`].
///
/// A value that does not fit its UNIT is null, as an unparseable value always
/// has been; Rule 8 is what reports it.
pub fn build_column_for_unit<'a, F>(
    n_rows: usize,
    ags_type: &str,
    unit: &str,
    cell: F,
) -> (ArrayRef, DataType)
where
    F: Fn(usize) -> Option<&'a str>,
{
    if canonical_type(ags_type) != Some(CanonicalType::Datetime) {
        return build_column(n_rows, ags_type, cell);
    }
    if is_time_of_day_unit(unit) {
        let mut b = Time64MicrosecondBuilder::with_capacity(n_rows);
        for row in 0..n_rows {
            let micros = cell(row).and_then(parse_time).map(|t| {
                i64::from(t.num_seconds_from_midnight()) * 1_000_000
                    + i64::from(t.nanosecond() / 1_000)
            });
            b.append_option(micros);
        }
        return (
            Arc::new(b.finish()) as ArrayRef,
            DataType::Time64(TimeUnit::Microsecond),
        );
    }
    let mut b = TimestampMicrosecondBuilder::with_capacity(n_rows);
    for row in 0..n_rows {
        let micros = cell(row)
            .filter(|s| !s.trim().is_empty())
            .and_then(|s| parse_datetime_for_unit(s, unit))
            .map(|dt| dt.and_utc().timestamp_micros());
        b.append_option(micros);
    }
    (
        Arc::new(b.finish()) as ArrayRef,
        DataType::Timestamp(TimeUnit::Microsecond, None),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Value-correctness for every arm of `build_column`. The existing tests
    /// check the SCHEMA (field names, column set); none checked the decoded
    /// CONTENTS, so a caster returning the right Arrow *type* with the wrong
    /// *value* would pass silently. Pins one cell per family arm — Integer,
    /// Decimal, Bool, Datetime, String — to its decoded value.
    #[test]
    fn build_column_decodes_each_family_to_the_right_value() {
        use arrow::array::{
            Array, BooleanArray, Float64Array, Int64Array, StringArray, TimestampMicrosecondArray,
        };
        let headings: Vec<String> = ["N", "F", "B", "D", "S"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        let types: Vec<String> = ["0DP", "2DP", "YN", "DT", "ID"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        // Column-major cell values for a single row.
        let vals = ["42", "12.34", "Y", "2024-01-15T09:30:00", "BH1"];
        let batch =
            build_record_batch(&headings, &types, 1, |col, _row| vals.get(col).copied()).unwrap();

        let col = |i: usize| batch.column(i).as_ref();
        assert_eq!(
            col(0)
                .as_any()
                .downcast_ref::<Int64Array>()
                .unwrap()
                .value(0),
            42
        );
        let f = col(1)
            .as_any()
            .downcast_ref::<Float64Array>()
            .unwrap()
            .value(0);
        assert!((f - 12.34).abs() < 1e-9, "2DP decoded to {f}");
        assert!(
            col(2)
                .as_any()
                .downcast_ref::<BooleanArray>()
                .unwrap()
                .value(0)
        );
        // DT parses to a non-null timestamp; the exact micros are parse_value's
        // contract (tested there) — here we pin that the Datetime arm produced a
        // valid, non-null value rather than silently nulling.
        assert!(
            col(3)
                .as_any()
                .downcast_ref::<TimestampMicrosecondArray>()
                .unwrap()
                .is_valid(0)
        );
        assert_eq!(
            col(4)
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap()
                .value(0),
            "BH1"
        );
    }

    /// A `None` (missing/ragged) cell must decode to a typed NULL in every arm,
    /// not a default value — the branch a real sparse delivery takes constantly,
    /// and the one the `null-half` bench rung exercises for cost.
    #[test]
    fn a_dt_column_is_typed_by_the_precision_its_unit_declares() {
        use arrow::array::{Array, Time64MicrosecondArray, TimestampMicrosecondArray};
        // #999: a time-of-day UNIT → Time64 (no invented date); a mismatched
        // value is null, as an unparseable value always was.
        let cells = ["09:15", "09:15:30", "2026-03", ""];
        let (arr, dt) = build_column_for_unit(cells.len(), "DT", "hh:mm", |r| Some(cells[r]));
        assert_eq!(dt, DataType::Time64(TimeUnit::Microsecond));
        let t = arr
            .as_any()
            .downcast_ref::<Time64MicrosecondArray>()
            .unwrap();
        assert_eq!(t.value(0), (9 * 3600 + 15 * 60) * 1_000_000);
        assert_eq!(t.value(1), (9 * 3600 + 15 * 60 + 30) * 1_000_000);
        assert!(t.is_null(2) && t.is_null(3));

        // A period UNIT → the start of the period, still a timestamp.
        let micros = |y, m| {
            chrono::NaiveDate::from_ymd_opt(y, m, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc()
                .timestamp_micros()
        };
        for (unit, cell, want) in [
            ("yyyy-mm", "2026-03", micros(2026, 3)),
            ("yyyy", "2026", micros(2026, 1)),
        ] {
            let (arr, dt) = build_column_for_unit(1, "DT", unit, |_| Some(cell));
            assert_eq!(dt, DataType::Timestamp(TimeUnit::Microsecond, None));
            let ts = arr
                .as_any()
                .downcast_ref::<TimestampMicrosecondArray>()
                .unwrap();
            assert_eq!(ts.value(0), want, "{cell:?} under {unit:?}");
        }

        // Any other UNIT, and any non-DT type, is exactly build_column.
        let (_, dt) = build_column_for_unit(1, "DT", "yyyy-mm-dd", |_| Some("2026-03-02"));
        assert_eq!(dt, build_column(1, "DT", |_| Some("2026-03-02")).1);
        let (_, dt) = build_column_for_unit(1, "X", "hh:mm", |_| Some("09:15"));
        assert_eq!(dt, DataType::Utf8);
    }

    #[test]
    fn the_unit_less_batch_builder_is_the_unit_aware_one_with_no_units() {
        let headings = vec!["T".to_string()];
        let types = vec!["DT".to_string()];
        let cell = |_c: usize, _r: usize| Some("2026-03-02T09:15");
        let plain = build_record_batch(&headings, &types, 1, cell).unwrap();
        let with = build_record_batch_synth_with_units(
            &SynthColumns::default(),
            &headings,
            &types,
            &["yyyy-mm-ddThh:mm".to_string()],
            1,
            cell,
        )
        .unwrap();
        assert_eq!(plain, with);
    }

    #[test]
    fn build_column_maps_a_missing_cell_to_a_typed_null() {
        use arrow::array::{
            Array, BooleanArray, Float64Array, Int64Array, StringArray, TimestampMicrosecondArray,
        };
        let headings: Vec<String> = ["N", "F", "B", "D", "S"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        let types: Vec<String> = ["0DP", "2DP", "YN", "DT", "ID"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        // Every cell missing.
        let batch = build_record_batch(&headings, &types, 1, |_c, _r| None).unwrap();
        let col = |i: usize| batch.column(i).as_ref();
        assert!(
            col(0)
                .as_any()
                .downcast_ref::<Int64Array>()
                .unwrap()
                .is_null(0)
        );
        assert!(
            col(1)
                .as_any()
                .downcast_ref::<Float64Array>()
                .unwrap()
                .is_null(0)
        );
        assert!(
            col(2)
                .as_any()
                .downcast_ref::<BooleanArray>()
                .unwrap()
                .is_null(0)
        );
        assert!(
            col(3)
                .as_any()
                .downcast_ref::<TimestampMicrosecondArray>()
                .unwrap()
                .is_null(0)
        );
        // The String arm maps an absent cell to null (not "").
        assert!(
            col(4)
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap()
                .is_null(0)
        );
    }

    /// The 2×2 the single-purpose builders would have silently dropped:
    /// keyed×hashed are independent knobs, so all four combinations must
    /// produce the expected column set — in particular `ids: None, hashes:
    /// Some` (a passthrough group carrying a content hash but no `_id`).
    #[test]
    fn synth_columns_cover_the_2x2() {
        let headings = vec!["LOCA_ID".to_string(), "LOCA_GL".to_string()];
        let ags_types = vec!["ID".to_string(), "2DP".to_string()];
        let n_rows = 2;
        let cell = |col: usize, row: usize| -> Option<&'static str> {
            match (col, row) {
                (0, 0) => Some("BH1"),
                (0, 1) => Some("BH2"),
                (1, 0) => Some("1.23"),
                (1, 1) => Some("4.56"),
                _ => None,
            }
        };
        let ids = vec![
            ("id0".to_string(), None),
            ("id1".to_string(), Some("p1".to_string())),
        ];
        let hashes = vec!["h0".to_string(), "h1".to_string()];

        let field_names = |batch: &RecordBatch| -> Vec<String> {
            batch
                .schema()
                .fields()
                .iter()
                .map(|f| f.name().clone())
                .collect()
        };

        // No synth columns.
        let batch = build_record_batch_synth(
            &SynthColumns::default(),
            &headings,
            &ags_types,
            n_rows,
            cell,
        )
        .unwrap();
        assert_eq!(field_names(&batch), vec!["LOCA_ID", "LOCA_GL"]);

        // ids only.
        let batch = build_record_batch_synth(
            &SynthColumns {
                ids: Some(&ids),
                hashes: None,
            },
            &headings,
            &ags_types,
            n_rows,
            cell,
        )
        .unwrap();
        assert_eq!(
            field_names(&batch),
            vec!["_id", "_parent_id", "LOCA_ID", "LOCA_GL"]
        );
        assert!(batch.schema().field(0).is_nullable());

        // hashes only — the passthrough-with-hash case: no `_id`, but a
        // `_content_hash` still appears.
        let batch = build_record_batch_synth(
            &SynthColumns {
                ids: None,
                hashes: Some(&hashes),
            },
            &headings,
            &ags_types,
            n_rows,
            cell,
        )
        .unwrap();
        assert_eq!(
            field_names(&batch),
            vec!["LOCA_ID", "LOCA_GL", "_content_hash"]
        );
        assert!(!batch.schema().field(2).is_nullable());

        // both.
        let batch = build_record_batch_synth(
            &SynthColumns {
                ids: Some(&ids),
                hashes: Some(&hashes),
            },
            &headings,
            &ags_types,
            n_rows,
            cell,
        )
        .unwrap();
        assert_eq!(
            field_names(&batch),
            vec!["_id", "_parent_id", "LOCA_ID", "LOCA_GL", "_content_hash"]
        );
    }

    /// The compat builder lays out python-ags4's frame shape: a leading `HEADING`
    /// tag column (`UNIT`/`TYPE`/`DATA`), one raw-string column per heading with
    /// positional names, `unit`/`type`/`*data` rows, and `""` for a short/ragged
    /// cell (never a null).
    #[test]
    fn compat_batch_shape_and_ragged() {
        let headings = vec!["LOCA_ID".to_string(), "LOCA_GL".to_string()];
        let units = vec![String::new(), "m".to_string()];
        let types = vec!["ID".to_string(), "2DP".to_string()];
        // row 0 full; row 1 ragged (only one value) → col 1 becomes "".
        let data = [vec!["BH1", "1.23"], vec!["BH2"]];
        let cell = |col: usize, row: usize| -> Option<&str> {
            data.get(row).and_then(|r| r.get(col)).copied()
        };
        let batch = build_record_batch_compat(&headings, &units, &types, data.len(), cell).unwrap();

        let names: Vec<String> = batch
            .schema()
            .fields()
            .iter()
            .map(|f| f.name().clone())
            .collect();
        assert_eq!(names, vec!["HEADING", "c0", "c1"]);
        assert_eq!(batch.num_rows(), 4); // UNIT + TYPE + 2 DATA

        let col = |i: usize| -> Vec<String> {
            batch
                .column(i)
                .as_any()
                .downcast_ref::<arrow::array::StringArray>()
                .unwrap()
                .iter()
                .map(|o| o.unwrap_or("").to_string())
                .collect()
        };
        assert_eq!(col(0), vec!["UNIT", "TYPE", "DATA", "DATA"]);
        assert_eq!(col(1), vec!["", "ID", "BH1", "BH2"]);
        // ragged: row 1 had no second value → "".
        assert_eq!(col(2), vec!["m", "2DP", "1.23", ""]);
    }
}
