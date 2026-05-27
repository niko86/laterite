//! AGS4 type system — port of `ags5_models._types`.
//!
//! Three responsibilities live here:
//!
//!   1. `CanonicalType` — the small target type set that maps AGS codes to
//!      cross-system storage types (DuckDB / JSON value shapes).
//!   2. `canonical_type(code)` — AGS spec type code → canonical type.
//!   3. `parse_value(raw, code)` — permissive AGS4-string → typed JSON
//!      value, used by `migrate` and `ags4-to-db` to turn raw `data` JSON
//!      payload strings into the typed-column values v6.5 stores.
//!
//! Mirrors the Python module's semantics exactly: unparseable values
//! return `Value::Null`, unknown AGS codes fall through to string storage.

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde_json::{Number, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonicalType {
    String,
    Integer,
    Decimal,
    Datetime,
    Date,
    Time,
    Bool,
    Enum,
}

impl CanonicalType {
    /// Lower-case label that matches Python's `CanonicalType` StrEnum
    /// values (`"string"`, `"integer"`, …). Used in `_spec_headings`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Integer => "integer",
            Self::Decimal => "decimal",
            Self::Datetime => "datetime",
            Self::Date => "date",
            Self::Time => "time",
            Self::Bool => "bool",
            Self::Enum => "enum",
        }
    }

    /// DuckDB SQL storage type for the canonical category. Mirrors
    /// `_ddl._sql_type` exactly: decimal -> DOUBLE (Phase 6.5.1), integer
    /// -> BIGINT, datetime -> TIMESTAMP, etc.
    pub fn sql_type(self) -> &'static str {
        match self {
            Self::String | Self::Enum => "VARCHAR",
            Self::Integer => "BIGINT",
            Self::Decimal => "DOUBLE",
            Self::Datetime => "TIMESTAMP",
            Self::Date => "DATE",
            Self::Time => "TIME",
            Self::Bool => "BOOLEAN",
        }
    }
}

const STRING_AGS_TYPES: &[&str] = &["ID", "X", "PA", "PT", "PU", "T", "U", "DMS", "MC", "XN"];
const INTEGER_AGS_TYPES: &[&str] = &["0DP"];
const DATETIME_AGS_TYPES: &[&str] = &["DT"];
const BOOL_AGS_TYPES: &[&str] = &["YN"];

/// AGS spec type code → canonical category. Returns `None` on unknown
/// codes (Python's version raises `ValueError`; in Rust the caller picks
/// the fallback — `parse_value` treats unknown codes as String storage,
/// the DDL builder maps them to VARCHAR).
pub fn canonical_type(ags_type: &str) -> Option<CanonicalType> {
    let t = ags_type.trim().to_uppercase();
    if STRING_AGS_TYPES.contains(&t.as_str()) {
        return Some(CanonicalType::String);
    }
    if INTEGER_AGS_TYPES.contains(&t.as_str()) {
        return Some(CanonicalType::Integer);
    }
    if DATETIME_AGS_TYPES.contains(&t.as_str()) {
        return Some(CanonicalType::Datetime);
    }
    if BOOL_AGS_TYPES.contains(&t.as_str()) {
        return Some(CanonicalType::Bool);
    }
    if t == "RL" {
        return Some(CanonicalType::Decimal);
    }
    // nDP / nSF / nSCI numeric forms — split on the trailing letters,
    // validate the prefix is a positive integer.
    for (suffix, _) in [("DP", 2), ("SF", 2), ("SCI", 3)] {
        if t.ends_with(suffix) {
            let prefix = &t[..t.len() - suffix.len()];
            if !prefix.is_empty() && prefix.chars().all(|c| c.is_ascii_digit()) {
                return Some(CanonicalType::Decimal);
            }
        }
    }
    None
}

/// AGS spec → DuckDB SQL type. Falls back to VARCHAR on unknown codes
/// so passthrough rows from AGS4 ingest of an unfamiliar dictionary
/// still land somewhere queryable.
pub fn sql_type(ags_type: &str) -> &'static str {
    canonical_type(ags_type)
        .map(|c| c.sql_type())
        .unwrap_or("VARCHAR")
}

/// Format a typed value back to the AGS4 string form the codec would
/// have read from the source file. Inverse of `parse_value` (lossy on
/// width — we only carry the AGS-spec precision hint, not the original
/// trailing-zero count beyond what the type implies). Used by
/// `ags4-to-db --append` to reconstruct shared-key lookup tuples from
/// on-disk rows so on-disk + new rows match string-wise.
///
/// Examples:
///   `ags4_str(Value::from(100.5), "2DP")` -> `"100.50"`
///   `ags4_str(Value::from(5_i64), "0DP")` -> `"5"`
///   `ags4_str(Value::Null,        _   )` -> `""`
pub fn ags4_str(value: &Value, ags_type: &str) -> String {
    if value.is_null() {
        return String::new();
    }
    let t = ags_type.trim().to_uppercase();
    // DT (date+time) values come from two sources:
    //   * `parse_value` (--append shared-key lookup) → `yyyy-mm-dd HH:MM:SS`
    //   * `value_to_json` of a DuckDB TIMESTAMP → `yyyy-mm-ddTHH:MM:SS.fff`
    // Normalize the second form: strip the fractional tail when it's all
    // zeros, and drop a `T00:00:00` time portion entirely so date-only
    // AGS4 inputs (`2023-02-22`) round-trip back to date-only form. This
    // matches the Rule 8 expectation for DATE-formatted DT columns.
    // YN values arrive from DuckDB as bool, but AGS4 spec wants the
    // letters `Y` / `N` (Rule 8 type check). `parse_value` does the
    // forward mapping; we do the reverse here.
    if t == "YN" {
        if let Value::Bool(b) = value {
            return (if *b { "Y" } else { "N" }).to_string();
        }
    }
    if t == "DT" {
        if let Value::String(s) = value {
            let trimmed = if let Some(idx) = s.find('.') {
                let (head, tail) = s.split_at(idx);
                if tail.trim_start_matches('.').chars().all(|c| c == '0') {
                    head
                } else {
                    s.as_str()
                }
            } else {
                s.as_str()
            };
            if let Some(date_part) = trimmed.strip_suffix("T00:00:00") {
                return date_part.to_string();
            }
            // Otherwise keep the ISO 8601 `T` separator. The AGS4.1
            // spec UNIT for DT columns is typically `yyyy-mm-ddThh:mm:ss`
            // so this matches the validator's expected form.
            return trimmed.to_string();
        }
    }
    if t == "0DP" {
        return value
            .as_i64()
            .map(|i| i.to_string())
            .or_else(|| value.as_f64().map(|f| (f as i64).to_string()))
            .unwrap_or_default();
    }
    if t.ends_with("DP") {
        let n = t[..t.len() - 2].parse::<usize>().unwrap_or(0);
        if let Some(f) = value.as_f64() {
            return format!("{:.*}", n, f);
        }
    }
    if t.ends_with("SF") {
        let n = t[..t.len() - 2].parse::<usize>().unwrap_or(0);
        if let Some(f) = value.as_f64() {
            // n significant figures in fixed-point — python-ags4's
            // validator rejects scientific notation under nSF
            // ("Value 1.0e2 not of data type 2SF. Expected: 100"). The
            // canonical form: round to n sig figs, emit as a plain
            // decimal — trailing zeros for small magnitudes show the
            // precision (0.002 -> "0.00200" under 3SF); large
            // magnitudes get integer-rounded (1234 -> "1230" under 3SF).
            if f == 0.0 {
                return format!("{:.*}", n.saturating_sub(1), 0.0);
            }
            let exp = f.abs().log10().floor() as i32;
            let dp = (n as i32) - exp - 1;
            if dp >= 0 {
                return format!("{:.*}", dp as usize, f);
            }
            // dp < 0: round to nearest 10^|dp|, emit as integer
            // (no decimal point — `{:.0}` does that).
            let scale = 10f64.powi(-dp);
            let rounded = (f / scale).round() * scale;
            return format!("{:.0}", rounded);
        }
    }
    if t.ends_with("SCI") {
        let n = t[..t.len() - 3].parse::<usize>().unwrap_or(0);
        if let Some(f) = value.as_f64() {
            return format!("{:.*e}", n, f);
        }
    }
    match value {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// Format a DT-shaped value string to match the precision declared by
/// the column's UNIT row. AGS4 Rule 8 checks that DT values match their
/// declared format both ways:
///
///   * Too-long: a value with seconds (`2023-02-22T10:24:00`) under a
///     no-seconds unit (`yyyy-mm-ddThh:mm`) is rejected. We truncate.
///     **Lossy** when the trimmed suffix is non-zero
///     (`2023-02-22T10:24:37` → `2023-02-22T10:24`).
///   * Too-short: a date-only value (`2022-09-21`) under a unit
///     demanding time (`yyyy-mm-ddThh:mm`) is also rejected. We pad
///     with a `T` separator + zero-time portion so the result still
///     parses (`2022-09-21` → `2022-09-21T00:00`). Lossless — the
///     padded time is structural, not data.
///
/// Returns the input unchanged if the unit isn't a recognised DT format.
pub fn truncate_dt_to_unit(value: &str, unit: &str) -> String {
    let u = unit.trim();
    // Quick guards: a non-DT-shaped unit (e.g. "m", "%", "") leaves the
    // value alone.
    if !u.starts_with("yyyy") {
        return value.to_string();
    }
    let target_len = match () {
        _ if u == "yyyy-mm-dd" => 10,
        // 16 chars: "yyyy-mm-ddThh:mm" or "yyyy-mm-dd hh:mm"
        _ if u.ends_with(":mm") && !u.ends_with(":mm:ss") => 16,
        // 19 chars: "yyyy-mm-ddThh:mm:ss"
        _ if u.ends_with(":ss") => 19,
        _ => return value.to_string(),
    };
    if value.len() == target_len {
        return value.to_string();
    }
    if value.len() > target_len {
        return value[..target_len].to_string();
    }
    // value.len() < target_len: pad with the "T<zero-time>" suffix the
    // unit asks for. We only handle date-only → datetime promotion;
    // partial/odd-length inputs (e.g. "2023-02") aren't legal AGS4 DT
    // values to start with, so leave them as-is.
    if value.len() != 10 {
        return value.to_string();
    }
    let pad = &"T00:00:00"[..target_len - 10];
    format!("{}{}", value, pad)
}

/// Presentation hint for a numeric AGS type: `'2DP'` → `Some("%.2f")`,
/// `'3SF'` → `Some("%.3g")`, `'1SCI'` → `Some("%.1e")`. Mirrors Python's
/// `display_hint`. String/datetime/bool types return `None`.
pub fn display_hint(ags_type: &str) -> Option<String> {
    let t = ags_type.trim().to_uppercase();
    for (suffix, fmt_letter) in [("DP", 'f'), ("SF", 'g'), ("SCI", 'e')] {
        if t.ends_with(suffix) {
            let prefix = &t[..t.len() - suffix.len()];
            if !prefix.is_empty() && prefix.chars().all(|c| c.is_ascii_digit()) {
                return Some(format!("%.{}{}", prefix, fmt_letter));
            }
        }
    }
    None
}

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
const BOOL_TRUE: &[&str] = &["Y", "YES", "TRUE", "1"];
const BOOL_FALSE: &[&str] = &["N", "NO", "FALSE", "0"];

/// Parse an AGS4-shaped raw value into a typed JSON value suitable for
/// DuckDB parameter binding. Permissive: unparseable values yield
/// `Value::Null`. Unknown AGS codes pass through as string.
///
/// Two input shapes are accepted:
///   * `Some(s)` — the raw AGS4 string from a v6 file's `data` JSON;
///   * `None`    — explicit null / missing column.
pub fn parse_value(raw: Option<&str>, ags_type: &str) -> Value {
    let s = match raw {
        Some(s) => s.trim(),
        None => return Value::Null,
    };
    if s.is_empty() {
        return Value::Null;
    }
    let ct = match canonical_type(ags_type) {
        Some(c) => c,
        None => return Value::String(s.to_string()),
    };
    match ct {
        CanonicalType::String | CanonicalType::Enum => Value::String(s.to_string()),
        CanonicalType::Integer => match s.parse::<f64>() {
            // Tolerate "5.0" notation for integers (Python `int(float(s))`).
            Ok(f) if f.is_finite() => Value::from(f as i64),
            _ => Value::Null,
        },
        CanonicalType::Decimal => match s.parse::<f64>() {
            Ok(f) if f.is_finite() => Number::from_f64(f)
                .map(Value::Number)
                .unwrap_or(Value::Null),
            _ => Value::Null,
        },
        CanonicalType::Datetime => parse_with_formats(s, DATETIME_FORMATS, |fmt| {
            // Full datetime first. Fall back to a date-only parse
            // (promoted to midnight) — a `DT` cell legally carries just
            // `2020-08-18` under a `yyyy-mm-dd` UNIT, and
            // `NaiveDateTime::parse_from_str` can't build a datetime from
            // a time-less string, so without this the value silently
            // dropped to NULL (data loss; the date-only formats listed in
            // DATETIME_FORMATS were dead). On export `ags4_str` +
            // `truncate_dt_to_unit` render it back to date-only form.
            NaiveDateTime::parse_from_str(s, fmt)
                .ok()
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .or_else(|| {
                    NaiveDate::parse_from_str(s, fmt)
                        .ok()
                        .map(|d| d.format("%Y-%m-%d 00:00:00").to_string())
                })
        })
        .map(Value::String)
        .unwrap_or(Value::Null),
        CanonicalType::Date => parse_with_formats(s, DATE_FORMATS, |fmt| {
            NaiveDate::parse_from_str(s, fmt)
                .ok()
                .map(|d| d.format("%Y-%m-%d").to_string())
        })
        .map(Value::String)
        .unwrap_or(Value::Null),
        CanonicalType::Time => parse_with_formats(s, TIME_FORMATS, |fmt| {
            NaiveTime::parse_from_str(s, fmt)
                .ok()
                .map(|t| t.format("%H:%M:%S").to_string())
        })
        .map(Value::String)
        .unwrap_or(Value::Null),
        CanonicalType::Bool => {
            let u = s.to_uppercase();
            if BOOL_TRUE.contains(&u.as_str()) {
                Value::Bool(true)
            } else if BOOL_FALSE.contains(&u.as_str()) {
                Value::Bool(false)
            } else {
                Value::Null
            }
        }
    }
}

fn parse_with_formats<F>(_s: &str, formats: &[&str], mut try_one: F) -> Option<String>
where
    F: FnMut(&str) -> Option<String>,
{
    for fmt in formats {
        if let Some(out) = try_one(fmt) {
            return Some(out);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ags_string_types_resolve() {
        assert_eq!(canonical_type("ID"), Some(CanonicalType::String));
        assert_eq!(canonical_type("X"), Some(CanonicalType::String));
        assert_eq!(canonical_type("PA"), Some(CanonicalType::String));
    }

    #[test]
    fn ndp_resolves_to_decimal() {
        assert_eq!(canonical_type("2DP"), Some(CanonicalType::Decimal));
        assert_eq!(canonical_type("10DP"), Some(CanonicalType::Decimal));
        assert_eq!(canonical_type("3SF"), Some(CanonicalType::Decimal));
        assert_eq!(canonical_type("1SCI"), Some(CanonicalType::Decimal));
    }

    #[test]
    fn unknown_code_is_none() {
        assert_eq!(canonical_type("BANANA"), None);
    }

    #[test]
    fn display_hint_round_trips() {
        assert_eq!(display_hint("2DP"), Some("%.2f".to_string()));
        assert_eq!(display_hint("3SF"), Some("%.3g".to_string()));
        assert_eq!(display_hint("X"), None);
    }

    #[test]
    fn parse_decimal_via_float_works() {
        assert_eq!(parse_value(Some("0.00200"), "2DP"), Value::from(0.002));
        assert_eq!(parse_value(Some("5"), "0DP"), Value::from(5));
        assert_eq!(parse_value(Some("5.0"), "0DP"), Value::from(5));
        assert_eq!(parse_value(Some(""), "X"), Value::Null);
        assert_eq!(parse_value(None, "X"), Value::Null);
    }

    #[test]
    fn parse_datetime_normalises() {
        assert_eq!(
            parse_value(Some("2024-01-02T03:04:05"), "DT"),
            Value::String("2024-01-02 03:04:05".to_string()),
        );
    }

    #[test]
    fn parse_dt_date_only_promotes_to_midnight() {
        // Regression: a date-only DT cell (legal under a `yyyy-mm-dd`
        // UNIT) used to drop to NULL because NaiveDateTime can't parse a
        // time-less string. It must now store as midnight and survive.
        assert_eq!(
            parse_value(Some("2020-08-18"), "DT"),
            Value::String("2020-08-18 00:00:00".to_string()),
        );
        assert_eq!(
            parse_value(Some("2020/08/18"), "DT"),
            Value::String("2020-08-18 00:00:00".to_string()),
        );
    }

    #[test]
    fn parse_bool_y_n_works() {
        assert_eq!(parse_value(Some("Y"), "YN"), Value::Bool(true));
        assert_eq!(parse_value(Some("N"), "YN"), Value::Bool(false));
        assert_eq!(parse_value(Some("maybe"), "YN"), Value::Null);
    }

    #[test]
    fn truncate_dt_strips_to_date_only_unit() {
        assert_eq!(
            truncate_dt_to_unit("2023-02-22T10:24:00", "yyyy-mm-dd"),
            "2023-02-22",
        );
        // Date-only value under a date-only unit: no-op.
        assert_eq!(
            truncate_dt_to_unit("2023-02-22", "yyyy-mm-dd"),
            "2023-02-22"
        );
    }

    #[test]
    fn truncate_dt_strips_seconds_for_minute_precision_unit() {
        assert_eq!(
            truncate_dt_to_unit("2023-02-22T10:24:00", "yyyy-mm-ddThh:mm"),
            "2023-02-22T10:24",
        );
        // Lossy case: source had non-zero seconds; we drop them to
        // match the declared unit precision.
        assert_eq!(
            truncate_dt_to_unit("2023-02-22T10:24:37", "yyyy-mm-ddThh:mm"),
            "2023-02-22T10:24",
        );
    }

    #[test]
    fn truncate_dt_strips_fractional_seconds_for_second_precision_unit() {
        assert_eq!(
            truncate_dt_to_unit("2023-02-22T10:24:00.000", "yyyy-mm-ddThh:mm:ss"),
            "2023-02-22T10:24:00",
        );
        // No-op when value already matches.
        assert_eq!(
            truncate_dt_to_unit("2023-02-22T10:24:00", "yyyy-mm-ddThh:mm:ss"),
            "2023-02-22T10:24:00",
        );
    }

    #[test]
    fn nsf_emits_fixed_point_for_small_values() {
        // Match python-ags4 validator's expected form: 3SF of 0.002 is
        // "0.00200" (fixed-point, three sig figs visible), not "2.00e-3".
        assert_eq!(ags4_str(&Value::from(0.002), "3SF"), "0.00200");
        assert_eq!(ags4_str(&Value::from(0.006), "3SF"), "0.00600");
        assert_eq!(ags4_str(&Value::from(0.020), "3SF"), "0.0200");
        assert_eq!(ags4_str(&Value::from(1.23), "3SF"), "1.23");
    }

    #[test]
    fn nsf_rounds_large_values_to_integer_form() {
        // python-ags4's validator wants plain decimal under nSF: 100
        // under 2SF stays "100", 1234 under 3SF rounds to "1230" — not
        // "1.0e2" / "1.23e3" scientific forms.
        assert_eq!(ags4_str(&Value::from(100.0), "2SF"), "100");
        assert_eq!(ags4_str(&Value::from(1234.0), "3SF"), "1230");
        assert_eq!(ags4_str(&Value::from(10.0), "1SF"), "10");
    }

    #[test]
    fn truncate_dt_pads_date_only_to_minute_precision() {
        // Date-only value under a minute-precision unit: pad with
        // T00:00 so the result satisfies the declared format. The
        // padded time is structural (matches the unit), not fake data.
        assert_eq!(
            truncate_dt_to_unit("2022-09-21", "yyyy-mm-ddThh:mm"),
            "2022-09-21T00:00",
        );
        assert_eq!(
            truncate_dt_to_unit("2022-09-21", "yyyy-mm-ddThh:mm:ss"),
            "2022-09-21T00:00:00",
        );
    }

    #[test]
    fn truncate_dt_passthrough_on_non_dt_units() {
        assert_eq!(truncate_dt_to_unit("100.50", "m"), "100.50");
        assert_eq!(truncate_dt_to_unit("anything", ""), "anything");
        assert_eq!(truncate_dt_to_unit("anything", "%"), "anything");
    }
}
