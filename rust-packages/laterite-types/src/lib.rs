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

// Typed Arrow column/record-batch building. Behind the `arrow` feature so
// laterite-types stays a tiny wasm-safe leaf for consumers that only need the
// type system (laterite-ags4-core → ags5db). Enabled by the two hosts that emit
// Arrow: laterite-ags4-wasm (→ IPC stream) and laterite-py (→ zero-copy capsule).
#[cfg(feature = "arrow")]
pub mod arrow_cols;
// Frame a typed group as a single-batch Arrow IPC stream — the shared
// composition (build_record_batch + StreamWriter) laterite-node and
// laterite-ags4-wasm both need. Parser-agnostic (closure-fed), so the leaf
// gains no parser dependency.
#[cfg(feature = "arrow")]
pub mod ipc;

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
            // DATETIME_FORMATS were dead). On export the value is rendered
            // back to date-only form.
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

/// Parse an AGS4 DATETIME cell into a `NaiveDateTime`, trying the same
/// `DATETIME_FORMATS` `parse_value` uses (a full datetime first, else a
/// date-only value promoted to midnight — a `DT` cell legally carries
/// just `2020-08-18` under a `yyyy-mm-dd` UNIT).
///
/// `parse_value` formats a DATETIME back to a `Value::String`, which is
/// right for the JSON path but can't fill an Arrow `Timestamp` column.
/// Callers that need the typed value — the browser explorer building
/// typed Arrow (epoch-µs timestamps) — use this instead, so they cast
/// *identically* to how `parse_value` decides what is a valid datetime.
/// Returns `None` when no format matches (the caller appends a null).
pub fn parse_datetime(s: &str) -> Option<NaiveDateTime> {
    let s = s.trim();
    for fmt in DATETIME_FORMATS {
        if let Ok(dt) = NaiveDateTime::parse_from_str(s, fmt) {
            return Some(dt);
        }
        if let Ok(d) = NaiveDate::parse_from_str(s, fmt) {
            return d.and_hms_opt(0, 0, 0);
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

    // --- CanonicalType label / SQL-type mapping ---------------------

    #[test]
    fn canonical_type_as_str_covers_every_variant() {
        // The labels feed `_spec_headings`; they must match Python's
        // StrEnum values exactly.
        assert_eq!(CanonicalType::String.as_str(), "string");
        assert_eq!(CanonicalType::Integer.as_str(), "integer");
        assert_eq!(CanonicalType::Decimal.as_str(), "decimal");
        assert_eq!(CanonicalType::Datetime.as_str(), "datetime");
        assert_eq!(CanonicalType::Date.as_str(), "date");
        assert_eq!(CanonicalType::Time.as_str(), "time");
        assert_eq!(CanonicalType::Bool.as_str(), "bool");
        assert_eq!(CanonicalType::Enum.as_str(), "enum");
    }

    #[test]
    fn canonical_type_sql_type_covers_every_variant() {
        assert_eq!(CanonicalType::String.sql_type(), "VARCHAR");
        assert_eq!(CanonicalType::Enum.sql_type(), "VARCHAR");
        assert_eq!(CanonicalType::Integer.sql_type(), "BIGINT");
        assert_eq!(CanonicalType::Decimal.sql_type(), "DOUBLE");
        assert_eq!(CanonicalType::Datetime.sql_type(), "TIMESTAMP");
        assert_eq!(CanonicalType::Date.sql_type(), "DATE");
        assert_eq!(CanonicalType::Time.sql_type(), "TIME");
        assert_eq!(CanonicalType::Bool.sql_type(), "BOOLEAN");
    }

    #[test]
    fn sql_type_fn_falls_back_to_varchar_on_unknown() {
        assert_eq!(sql_type("ID"), "VARCHAR");
        assert_eq!(sql_type("0DP"), "BIGINT");
        assert_eq!(sql_type("2DP"), "DOUBLE");
        assert_eq!(sql_type("DT"), "TIMESTAMP");
        assert_eq!(sql_type("YN"), "BOOLEAN");
        // RL maps to Decimal -> DOUBLE.
        assert_eq!(sql_type("RL"), "DOUBLE");
        // Unknown / passthrough code.
        assert_eq!(sql_type("BANANA"), "VARCHAR");
    }

    #[test]
    fn canonical_type_rl_is_decimal() {
        assert_eq!(canonical_type("RL"), Some(CanonicalType::Decimal));
        assert_eq!(canonical_type("DT"), Some(CanonicalType::Datetime));
        assert_eq!(canonical_type("YN"), Some(CanonicalType::Bool));
        assert_eq!(canonical_type("0DP"), Some(CanonicalType::Integer));
    }

    #[test]
    fn canonical_type_rejects_malformed_numeric_prefix() {
        // Trailing-letter forms with a non-digit / empty prefix are NOT
        // numeric AGS codes — they fall through to None.
        assert_eq!(canonical_type("DP"), None); // empty prefix
        assert_eq!(canonical_type("XDP"), None); // non-digit prefix
        assert_eq!(canonical_type("SF"), None);
        assert_eq!(canonical_type("SCI"), None);
    }

    // --- ags4_str: the reverse formatter --------------------------

    #[test]
    fn ags4_str_null_is_empty() {
        assert_eq!(ags4_str(&Value::Null, "2DP"), "");
        assert_eq!(ags4_str(&Value::Null, "X"), "");
        assert_eq!(ags4_str(&Value::Null, "DT"), "");
    }

    #[test]
    fn ags4_str_yn_bool_renders_letters() {
        assert_eq!(ags4_str(&Value::Bool(true), "YN"), "Y");
        assert_eq!(ags4_str(&Value::Bool(false), "YN"), "N");
        // A non-bool value under YN (shouldn't happen, but the branch
        // falls through to the generic tail).
        assert_eq!(ags4_str(&Value::String("Y".into()), "YN"), "Y");
    }

    #[test]
    fn ags4_str_dt_strips_zero_time_to_date_only() {
        // ISO form with a midnight time portion collapses to date-only.
        assert_eq!(
            ags4_str(&Value::String("2023-02-22T00:00:00".into()), "DT"),
            "2023-02-22",
        );
        // Zero fractional seconds get trimmed first, then the zero-time
        // collapse applies.
        assert_eq!(
            ags4_str(&Value::String("2023-02-22T00:00:00.000".into()), "DT"),
            "2023-02-22",
        );
    }

    #[test]
    fn ags4_str_dt_keeps_iso_separator_for_real_times() {
        // A non-midnight time keeps the full ISO form.
        assert_eq!(
            ags4_str(&Value::String("2023-02-22T10:24:37".into()), "DT"),
            "2023-02-22T10:24:37",
        );
        // Non-zero fractional seconds are preserved verbatim (the all-zero
        // strip guard does not fire).
        assert_eq!(
            ags4_str(&Value::String("2023-02-22T10:24:37.500".into()), "DT"),
            "2023-02-22T10:24:37.500",
        );
    }

    #[test]
    fn ags4_str_dt_non_string_falls_through() {
        // A DT-typed numeric value isn't a string, so the DT branch is
        // skipped and the generic tail stringifies it.
        assert_eq!(ags4_str(&Value::from(5_i64), "DT"), "5");
    }

    #[test]
    fn ags4_str_0dp_handles_int_and_float() {
        assert_eq!(ags4_str(&Value::from(5_i64), "0DP"), "5");
        // A float-valued 0DP cell truncates toward zero.
        assert_eq!(ags4_str(&Value::from(5.9_f64), "0DP"), "5");
        // A non-numeric value under 0DP yields the empty default.
        assert_eq!(ags4_str(&Value::String("x".into()), "0DP"), "");
    }

    #[test]
    fn ags4_str_ndp_formats_to_precision() {
        assert_eq!(ags4_str(&Value::from(100.5_f64), "2DP"), "100.50");
        assert_eq!(ags4_str(&Value::from(3.14159_f64), "3DP"), "3.142");
    }

    #[test]
    fn ags4_str_nsci_emits_scientific() {
        // nSCI uses Rust's lowercase `e` scientific format with n
        // fractional digits.
        assert_eq!(ags4_str(&Value::from(12345.0_f64), "2SCI"), "1.23e4");
        assert_eq!(ags4_str(&Value::from(0.0012_f64), "1SCI"), "1.2e-3");
    }

    #[test]
    fn ags4_str_string_passthrough_for_text_types() {
        assert_eq!(ags4_str(&Value::String("LOCA1".into()), "ID"), "LOCA1");
        // A non-string, non-numeric-typed value stringifies via the
        // generic arm.
        assert_eq!(ags4_str(&Value::from(7_i64), "X"), "7");
    }

    #[test]
    fn ags4_str_nsf_zero_renders_fixed_point() {
        // Zero under nSF takes the dedicated `f == 0.0` branch:
        // n-1 fractional digits.
        assert_eq!(ags4_str(&Value::from(0.0_f64), "3SF"), "0.00");
        assert_eq!(ags4_str(&Value::from(0.0_f64), "1SF"), "0");
    }

    // --- display_hint ---------------------------------------------

    #[test]
    fn display_hint_covers_all_numeric_families() {
        assert_eq!(display_hint("1SCI"), Some("%.1e".to_string()));
        assert_eq!(display_hint("12DP"), Some("%.12f".to_string()));
        // Non-numeric / malformed prefixes return None.
        assert_eq!(display_hint("DP"), None);
        assert_eq!(display_hint("XSF"), None);
        assert_eq!(display_hint("DT"), None);
    }

    // --- parse_value remaining branches ---------------------------

    #[test]
    fn parse_value_unknown_code_is_string() {
        // An unrecognised AGS code stores the raw (trimmed) string.
        assert_eq!(
            parse_value(Some("  hello  "), "ZZZ"),
            Value::String("hello".into()),
        );
    }

    #[test]
    fn parse_value_integer_rejects_non_numeric() {
        assert_eq!(parse_value(Some("abc"), "0DP"), Value::Null);
        // Infinity is not finite -> Null.
        assert_eq!(parse_value(Some("inf"), "0DP"), Value::Null);
    }

    #[test]
    fn parse_value_decimal_rejects_non_finite() {
        assert_eq!(parse_value(Some("not-a-number"), "2DP"), Value::Null);
        assert_eq!(parse_value(Some("NaN"), "2DP"), Value::Null);
        assert_eq!(parse_value(Some("inf"), "2DP"), Value::Null);
    }

    #[test]
    fn parse_value_datetime_unparseable_is_null() {
        assert_eq!(parse_value(Some("garbage"), "DT"), Value::Null);
    }

    #[test]
    fn parse_value_bool_full_token_set() {
        for t in ["Y", "YES", "TRUE", "1", "yes", "true"] {
            assert_eq!(parse_value(Some(t), "YN"), Value::Bool(true), "{t}");
        }
        for f in ["N", "NO", "FALSE", "0", "no", "false"] {
            assert_eq!(parse_value(Some(f), "YN"), Value::Bool(false), "{f}");
        }
    }

    #[test]
    fn parse_value_datetime_alternate_formats() {
        // dd/mm/yyyy and yyyy/mm/dd are in DATETIME_FORMATS and normalise
        // to the canonical `yyyy-mm-dd HH:MM:SS`.
        assert_eq!(
            parse_value(Some("18/08/2020"), "DT"),
            Value::String("2020-08-18 00:00:00".into()),
        );
        // datetime without seconds.
        assert_eq!(
            parse_value(Some("2020-08-18 13:05"), "DT"),
            Value::String("2020-08-18 13:05:00".into()),
        );
    }

    // --- parse_datetime (typed Arrow path) ------------------------

    #[test]
    fn parse_datetime_full_and_date_only() {
        let dt = parse_datetime("2024-01-02T03:04:05").unwrap();
        assert_eq!(
            dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2024-01-02 03:04:05"
        );
        // Date-only promotes to midnight.
        let midnight = parse_datetime("2020-08-18").unwrap();
        assert_eq!(midnight.format("%H:%M:%S").to_string(), "00:00:00");
        // Alternate date format.
        assert!(parse_datetime("18/08/2020").is_some());
    }

    #[test]
    fn parse_datetime_rejects_garbage() {
        assert_eq!(parse_datetime("not a date"), None);
        assert_eq!(parse_datetime(""), None);
    }
}

/// Property-based tests for the permissive caster + type resolver.
///
/// Why properties here: `parse_value` / `canonical_type` are the *single*
/// AGS-typing surface for both the DuckDB engine and the wasm explorer
/// (crate header), so an arbitrary-input panic would crash either side on
/// a hostile file. These check the universal contracts — totality
/// (never-panic), determinism, normalisation, and the parse↔format
/// inverse — across input domains the hand-written examples can't cover.
#[cfg(test)]
mod proptest_suite {
    use super::*;
    use proptest::prelude::*;

    /// Every AGS spec TYPE code `canonical_type` recognises, plus the
    /// nDP/nSF/nSCI numeric families. Drives the "valid-code" properties.
    fn ags_type_code() -> impl Strategy<Value = String> {
        let fixed = prop::sample::select(vec![
            "ID", "X", "PA", "PT", "PU", "T", "U", "DMS", "MC", "XN", "0DP", "DT", "YN", "RL",
        ])
        .prop_map(String::from);
        // nDP / nSF / nSCI with a small positive prefix.
        let numeric = (1usize..=12, prop::sample::select(vec!["DP", "SF", "SCI"]))
            .prop_map(|(n, suf)| format!("{n}{suf}"));
        prop_oneof![fixed, numeric]
    }

    proptest! {
        /// Totality: `parse_value` returns a `Value` for ANY
        /// `(Option<&str>, &str)` — arbitrary raw text against arbitrary
        /// type codes (recognised AGS codes *and* junk). The harness fails
        /// the case if the call panics; reaching the assert means it didn't.
        #[test]
        fn parse_value_never_panics(
            raw in prop::option::of(".*"),
            ty in ".*",
        ) {
            let _ = parse_value(raw.as_deref(), &ty);
            prop_assert!(true);
        }

        /// `parse_value` against the real AGS code set also never panics —
        /// exercises every typed branch (datetime/date/time/bool/numeric)
        /// with adversarial value strings.
        #[test]
        fn parse_value_typed_branches_never_panic(
            raw in ".*",
            ty in ags_type_code(),
        ) {
            let _ = parse_value(Some(&raw), &ty);
            prop_assert!(true);
        }

        /// `canonical_type` is total (never panics) on arbitrary text.
        #[test]
        fn canonical_type_never_panics(ty in ".*") {
            let _ = canonical_type(&ty);
            prop_assert!(true);
        }

        /// `canonical_type` is deterministic — the same input always maps
        /// to the same output (pure fn, no hidden state).
        #[test]
        fn canonical_type_deterministic(ty in ".*") {
            prop_assert_eq!(canonical_type(&ty), canonical_type(&ty));
        }

        /// `canonical_type` normalises by trim + uppercase (per its body):
        /// surrounding ASCII whitespace and letter-case are irrelevant.
        #[test]
        fn canonical_type_trims_and_uppercases(
            ty in ags_type_code(),
            lead in r"[ \t\r\n]{0,4}",
            trail in r"[ \t\r\n]{0,4}",
        ) {
            let base = canonical_type(&ty);
            // Whitespace padding doesn't change the verdict.
            let padded = format!("{lead}{ty}{trail}");
            prop_assert_eq!(canonical_type(&padded), base);
            // Lower-casing the code doesn't either.
            prop_assert_eq!(canonical_type(&ty.to_lowercase()), base);
        }

        /// nDP round-trip: a value parsed under an nDP type and formatted
        /// back via `ags4_str` preserves the NUMERIC value (byte form may
        /// re-canonicalise trailing zeros). Generate a value already at the
        /// declared precision so no rounding loss is expected.
        #[test]
        fn ndp_parse_format_preserves_numeric_value(
            n in 0usize..=6,
            int_part in 0i64..1_000_000,
            neg in any::<bool>(),
        ) {
            let ty = format!("{n}DP");
            // Build a canonical nDP string: integer part + n fractional
            // digits (here all zeros — exact, no rounding ambiguity).
            let frac = "0".repeat(n);
            let body = if n == 0 {
                int_part.to_string()
            } else {
                format!("{int_part}.{frac}")
            };
            let s = if neg && int_part != 0 { format!("-{body}") } else { body };

            let parsed = parse_value(Some(&s), &ty);
            let formatted = ags4_str(&parsed, &ty);
            // The formatted form re-parses to the SAME number.
            let reparsed = parse_value(Some(&formatted), &ty);
            prop_assert_eq!(&reparsed, &parsed, "s={:?} fmt={:?}", s, formatted);
        }

        /// DT idempotence proxy: `parse_value(Some(s), "DT")` is itself
        /// idempotent on its own output string — parsing the normalised
        /// `yyyy-mm-dd HH:MM:SS` form again yields the same Value (the
        /// caster is a stable projection, not a one-shot transform).
        #[test]
        fn parse_dt_is_a_stable_projection(
            y in 1900i32..2200,
            mo in 1u32..=12,
            d in 1u32..=28,
            h in 0u32..=23,
            mi in 0u32..=59,
            se in 0u32..=59,
        ) {
            let s = format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{se:02}");
            let first = parse_value(Some(&s), "DT");
            // Re-feed the normalised string form.
            if let Value::String(norm) = &first {
                let second = parse_value(Some(norm), "DT");
                prop_assert_eq!(&second, &first, "norm={:?}", norm);
            } else {
                prop_assert!(false, "DT of a valid datetime should be a String, got {first:?}");
            }
        }
    }
}
