//! Shared DuckDB helpers used by every read-side command.
//!
//! Three things live here so individual command modules stay terse:
//!   * resolving + validating `<db>` + `<group>` args (mirrors Python's
//!     `_resolve_db_and_group`), surfacing the right typed `CliError`;
//!   * a generic `value_to_json` converter for free-form SELECTs whose
//!     column types we don't know at compile time;
//!   * a `fetch_rows` helper that captures column names + records into the
//!     `output::Rows` shape rendered by `output::render_rows`.

use std::collections::HashMap;
use std::path::{MAIN_SEPARATOR_STR, Path, PathBuf};

/// Format a `&Path` as a display string with platform-native separators.
///
/// Python's `str(Path(...))` swaps `/` -> `\` on Windows via `WindowsPath`;
/// Rust's `Path::display()` preserves whatever the user typed. The parity
/// tests pass forward-slash paths into both binaries, and Python emits
/// backslashes on Windows — we replace `/` with `MAIN_SEPARATOR_STR` to
/// match. On POSIX `MAIN_SEPARATOR_STR == "/"` so the replace is a no-op.
pub fn display_native(p: &Path) -> String {
    p.display().to_string().replace('/', MAIN_SEPARATOR_STR)
}

use duckdb::types::{TimeUnit, ToSqlOutput, Value as DuckValue};
use duckdb::{Connection, ToSql};

// `ToSql` is the trait we implement on `JsonParam` below; the bare `ToSql`
// is referenced only there. Keeping the import explicit so `cargo build`'s
// "unused import" diagnostic stays loud if the impl gets removed.
use serde_json::{Map, Number, Value};

/// Newtype wrapping a `serde_json::Value` so it can be passed as a DuckDB
/// parameter. The duckdb crate provides `ToSql` impls for the concrete
/// primitive types but not for `serde_json::Value` (it would need to pick
/// one mapping among integer/float/string and there's no obvious right
/// choice). We make the choice explicit here: integers stay integers,
/// floats stay floats, strings stay strings, NULL maps to SQL NULL.
pub struct JsonParam<'a>(pub &'a Value);

impl<'a> ToSql for JsonParam<'a> {
    fn to_sql(&self) -> duckdb::Result<ToSqlOutput<'_>> {
        Ok(match self.0 {
            Value::Null => ToSqlOutput::Owned(DuckValue::Null),
            Value::Bool(b) => ToSqlOutput::Owned(DuckValue::Boolean(*b)),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    ToSqlOutput::Owned(DuckValue::BigInt(i))
                } else if let Some(f) = n.as_f64() {
                    ToSqlOutput::Owned(DuckValue::Double(f))
                } else {
                    ToSqlOutput::Owned(DuckValue::Text(n.to_string()))
                }
            }
            Value::String(s) => ToSqlOutput::Owned(DuckValue::Text(s.clone())),
            other => ToSqlOutput::Owned(DuckValue::Text(other.to_string())),
        })
    }
}

use crate::suggest::suggest;
use laterite_ags4_core::error::CliError;

/// Tabular query result: column order + one insertion-ordered JSON
/// object per row. This is its canonical home (a lib-level data type);
/// `output` re-exports it for the CLI renderers. Python's
/// order-preserving `json.dumps(dict)` (3.7+) depends on the per-record
/// key order, so `records` must stay ordered.
pub struct Rows {
    pub columns: Vec<String>,
    pub records: Vec<Map<String, Value>>,
}

/// Validate `<db>` exists and (optionally) `<group>` is present in this file's
/// `_spec_groups` table. Returns the uppercased group code on success.
///
/// On unknown group, builds the exit-4 `CliError` with fuzzy "did you mean…"
/// hints sourced from the file's actual `_spec_groups` (not the in-process
/// registry — Rust doesn't have one). For a typo on a group that isn't in
/// this file but exists in the AGS5 dictionary, callers can pass a broader
/// `extra_candidates` set to widen the suggestion pool.
pub fn resolve_db_and_group(db: &Path, group: &str) -> Result<(PathBuf, String), CliError> {
    if !db.exists() {
        return Err(CliError::FileNotFound(db.display().to_string()));
    }
    let code = group.to_uppercase();
    let conn = crate::conn::open_readonly(db)?;
    let codes = list_group_codes(&conn)?;
    if !codes.iter().any(|c| c == &code) {
        let hints = suggest(&code, &codes, 3, true);
        return Err(CliError::UnknownGroup { code, hints });
    }
    Ok((db.to_path_buf(), code))
}

/// Read every `code` from `_spec_groups`. Returns `CliError::PreVersion65`
/// if the table is missing — that means we're looking at a v6 file that
/// predates the self-describing tables and the read-side commands can't
/// operate on it without `ags5db-py migrate` first.
pub fn list_group_codes(conn: &Connection) -> Result<Vec<String>, CliError> {
    let mut stmt = conn
        .prepare("SELECT code FROM _spec_groups ORDER BY code")
        .map_err(|e| classify_catalog_error(e, "_spec_groups"))?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| CliError::Schema(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CliError::Schema(e.to_string()))?;
    Ok(rows)
}

/// Categorise a `_spec_*` table miss as a pre-6.5 file (exit 2) instead of
/// a generic schema error (exit 6). Looks for the catalog-not-found shape;
/// anything else falls through to `Schema`.
pub fn classify_catalog_error(err: duckdb::Error, table_name: &str) -> CliError {
    let msg = err.to_string();
    let lower = msg.to_lowercase();
    if lower.contains("does not exist") && lower.contains(&table_name.to_lowercase()) {
        CliError::PreVersion65
    } else {
        CliError::Schema(msg)
    }
}

/// Run a parameterised SELECT and return its `Rows` (column names + a
/// `serde_json::Map` per record so JSON output preserves key order).
///
/// Used by every command that does row-shaped output. The column-type
/// dispatch happens via `value_to_json` so callers don't have to know the
/// schema ahead of time — useful for `peek` (dynamic field list) and `sql`
/// (free-form SELECT).
pub fn fetch_rows(conn: &Connection, sql: &str, params: &[Value]) -> Result<Rows, CliError> {
    let mut stmt = conn
        .prepare(sql)
        .map_err(|e| CliError::Sql(e.to_string()))?;
    let wrapped: Vec<JsonParam> = params.iter().map(JsonParam).collect();
    let mut rows_iter = stmt
        .query(duckdb::params_from_iter(wrapped.iter()))
        .map_err(|e| CliError::Sql(e.to_string()))?;

    // duckdb-rs's `Statement::column_names()` panics if called before the
    // statement is executed (the schema is populated lazily by the engine).
    // The docs explicitly recommend going through `rows.as_ref().column_count()`
    // and `.column_name(i)` after `stmt.query([])?` instead — schema is
    // populated by then.
    let columns: Vec<String> = {
        let stmt_ref = rows_iter
            .as_ref()
            .ok_or_else(|| CliError::Sql("statement detached after query".into()))?;
        let n = stmt_ref.column_count();
        (0..n)
            .map(|i| {
                stmt_ref
                    .column_name(i)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|_| format!("col_{}", i))
            })
            .collect()
    };
    let n = columns.len();

    let mut records: Vec<Map<String, Value>> = Vec::new();
    while let Some(row) = rows_iter.next().map_err(|e| CliError::Sql(e.to_string()))? {
        let mut rec = Map::new();
        for (i, col) in columns.iter().enumerate().take(n) {
            let v: DuckValue = row
                .get(i)
                .map_err(|e| CliError::Sql(format!("col {col}: {e}")))?;
            rec.insert(col.clone(), value_to_json(v));
        }
        records.push(rec);
    }
    Ok(Rows { columns, records })
}

/// Convert a DuckDB `Value` to a JSON `Value`. Mirrors how Polars's
/// `write_ndjson` represents AGS column types: ints/floats as JSON numbers,
/// dates/times as ISO 8601 strings, blobs as hex-encoded strings.
pub fn value_to_json(v: DuckValue) -> Value {
    match v {
        DuckValue::Null => Value::Null,
        DuckValue::Boolean(b) => Value::Bool(b),
        DuckValue::TinyInt(i) => Value::from(i as i64),
        DuckValue::SmallInt(i) => Value::from(i as i64),
        DuckValue::Int(i) => Value::from(i as i64),
        DuckValue::BigInt(i) => Value::from(i),
        DuckValue::HugeInt(i) => Value::from(i.to_string()),
        DuckValue::UTinyInt(i) => Value::from(i as u64),
        DuckValue::USmallInt(i) => Value::from(i as u64),
        DuckValue::UInt(i) => Value::from(i as u64),
        DuckValue::UBigInt(i) => Value::from(i),
        DuckValue::Float(f) => json_float(f as f64),
        DuckValue::Double(f) => json_float(f),
        DuckValue::Decimal(d) => {
            // Decimal -> float for parity with Python's `_json_default`,
            // which coerces Decimal via float(d). For exact arithmetic
            // downstream consumers should use the SQL path with explicit
            // casts; the read-side CLI's job is human/agent legibility.
            let s = d.to_string();
            s.parse::<f64>().map(json_float).unwrap_or(Value::String(s))
        }
        DuckValue::Text(s) => Value::String(s),
        DuckValue::Blob(b) => Value::String(hex_encode(&b)),
        DuckValue::Enum(s) => Value::String(s),
        DuckValue::Date32(days) => Value::String(date_to_iso(days)),
        DuckValue::Time64(unit, t) => Value::String(time_to_iso(unit, t)),
        DuckValue::Timestamp(unit, ts) => Value::String(timestamp_to_iso(unit, ts)),
        DuckValue::Interval {
            months,
            days,
            nanos,
        } => Value::String(format!("P{}M{}DT{}N", months, days, nanos)),
        DuckValue::List(items) | DuckValue::Array(items) => {
            Value::Array(items.into_iter().map(value_to_json).collect())
        }
        DuckValue::Struct(map) => {
            let mut out = Map::new();
            for (k, v) in map.iter() {
                out.insert(k.clone(), value_to_json(v.clone()));
            }
            Value::Object(out)
        }
        DuckValue::Map(map) => {
            let mut out = Map::new();
            for (k, v) in map.iter() {
                out.insert(
                    value_to_json(k.clone()).to_string(),
                    value_to_json(v.clone()),
                );
            }
            Value::Object(out)
        }
        DuckValue::Union(inner) => value_to_json(*inner),
    }
}

fn json_float(f: f64) -> Value {
    if f.is_finite() {
        Number::from_f64(f)
            .map(Value::Number)
            .unwrap_or(Value::Null)
    } else {
        // NaN/±inf are not representable in JSON; emit null (matching
        // Polars's write_ndjson, which serialises non-finite floats as null).
        Value::Null
    }
}

fn hex_encode(b: &[u8]) -> String {
    let mut s = String::with_capacity(b.len() * 2);
    for byte in b {
        s.push_str(&format!("{:02x}", byte));
    }
    s
}

fn date_to_iso(days_since_epoch: i32) -> String {
    use chrono::{Duration, NaiveDate};
    let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).expect("epoch valid");
    let d = epoch + Duration::days(days_since_epoch as i64);
    d.format("%Y-%m-%d").to_string()
}

fn time_to_iso(unit: TimeUnit, value: i64) -> String {
    use chrono::{NaiveTime, Timelike};
    let nanos = unit_to_nanos(unit, value);
    let secs = (nanos / 1_000_000_000) as u32 % 86_400;
    let ns_rem = (nanos % 1_000_000_000) as u32;
    let t = NaiveTime::from_num_seconds_from_midnight_opt(secs, ns_rem)
        .unwrap_or_else(|| NaiveTime::from_hms_opt(0, 0, 0).unwrap());
    if t.nanosecond() == 0 {
        t.format("%H:%M:%S").to_string()
    } else {
        t.format("%H:%M:%S%.f").to_string()
    }
}

fn timestamp_to_iso(unit: TimeUnit, value: i64) -> String {
    use chrono::DateTime;
    let nanos = unit_to_nanos(unit, value);
    let secs = nanos.div_euclid(1_000_000_000);
    let ns_rem = nanos.rem_euclid(1_000_000_000) as u32;
    match DateTime::from_timestamp(secs, ns_rem) {
        Some(dt) => dt.naive_utc().format("%Y-%m-%dT%H:%M:%S%.f").to_string(),
        None => format!("epoch_ns:{}", nanos),
    }
}

fn unit_to_nanos(unit: TimeUnit, value: i64) -> i64 {
    match unit {
        TimeUnit::Second => value.saturating_mul(1_000_000_000),
        TimeUnit::Millisecond => value.saturating_mul(1_000_000),
        TimeUnit::Microsecond => value.saturating_mul(1_000),
        TimeUnit::Nanosecond => value,
    }
}

/// Read `_spec_headings` for a group, optionally restricted to a column
/// subset. Returns one record per heading; an empty `cols` filter means all.
pub fn headings_for(conn: &Connection, group_code: &str) -> Result<Vec<HeadingRow>, CliError> {
    let mut stmt = conn
        .prepare(
            "SELECT name, status, ags_type, canonical_type, unit, display_hint
               FROM _spec_headings WHERE group_code = ? ORDER BY name",
        )
        .map_err(|e| classify_catalog_error(e, "_spec_headings"))?;
    let rows = stmt
        .query_map([group_code], |row| {
            Ok(HeadingRow {
                name: row.get(0)?,
                status: row.get(1)?,
                ags_type: row.get(2)?,
                canonical_type: row.get(3)?,
                unit: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                hint: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
            })
        })
        .map_err(|e| CliError::Schema(e.to_string()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| CliError::Schema(e.to_string()))?;
    Ok(rows)
}

#[derive(Debug, Clone)]
pub struct HeadingRow {
    pub name: String,
    pub status: String,
    pub ags_type: String,
    pub canonical_type: String,
    pub unit: String,
    pub hint: String,
}

impl HeadingRow {
    /// Column header second line for `peek` table mode: `"<canonical_type>, <unit>"`.
    pub fn type_label(&self) -> String {
        if self.unit.is_empty() {
            self.canonical_type.clone()
        } else {
            format!("{}, {}", self.canonical_type, self.unit)
        }
    }
}

/// Build a `{py_name: "canonical_type[, unit]"}` map for a group's headings.
/// `peek` uses this to attach canonical-type labels under each column header
/// in TABLE mode — saves a round-trip through `headings` to learn what a
/// column is.
pub fn type_labels_for(
    conn: &Connection,
    group_code: &str,
    columns: &[String],
) -> Result<HashMap<String, String>, CliError> {
    let all = headings_for(conn, group_code)?;
    // _spec_headings.name is UPPERCASE (e.g. "LOCA_ID"); the view exposes
    // them lowercased ("loca_id"). Build a lookup by lowercase to match.
    let by_lower: HashMap<String, &HeadingRow> =
        all.iter().map(|h| (h.name.to_lowercase(), h)).collect();
    let mut out = HashMap::new();
    for col in columns {
        if let Some(h) = by_lower.get(&col.to_lowercase()) {
            out.insert(col.clone(), h.type_label());
        }
    }
    Ok(out)
}
