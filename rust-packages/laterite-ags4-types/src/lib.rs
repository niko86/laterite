//! AGS4 type system — ported from the original Python typed-value parser.
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
// laterite-ags4-types stays a tiny wasm-safe leaf for consumers that only need the
// type system (e.g. laterite-ags4-core's downstream consumers). Enabled by the two hosts that emit
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
    /// Lower-case label that matches Python's `CanonicalType` `StrEnum`
    /// values (`"string"`, `"integer"`, …). Used in `_spec_headings`.
    #[must_use]
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
    #[must_use]
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

// `RL` (Record Link) belongs here, NOT with the numerics: an RL cell is a
// DELIMITED REFERENCE — `GROUP|KEY1|KEY2`, split on `TRAN_DLIM` (AGS Rule 11) —
// not a number. It was special-cased to `Decimal` (→ `sql_type` DOUBLE), which
// silently DESTROYED every record link on read: `parse_value("SAMP|BH01|1.00", "RL")`
// returned Null, so the column came back as an all-null f64. Two tests pinned the
// wrong answer, which is how it survived (laterite#503).
const STRING_AGS_TYPES: &[&str] = &[
    "ID", "X", "PA", "PT", "PU", "T", "U", "DMS", "MC", "XN", "RL",
];
const INTEGER_AGS_TYPES: &[&str] = &["0DP"];
const DATETIME_AGS_TYPES: &[&str] = &["DT"];
const BOOL_AGS_TYPES: &[&str] = &["YN"];

/// AGS spec type code → canonical category. Returns `None` on unknown
/// codes (Python's version raises `ValueError`; in Rust the caller picks
/// the fallback — `parse_value` treats unknown codes as String storage,
/// the DDL builder maps them to VARCHAR).
#[must_use]
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
#[must_use]
pub fn sql_type(ags_type: &str) -> &'static str {
    canonical_type(ags_type).map_or("VARCHAR", CanonicalType::sql_type)
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
#[must_use]
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
        if let Some(i) = value.as_i64() {
            return i.to_string();
        }
        // A float value under a 0DP type is unusual — `parse_value` now nulls an
        // out-of-range 0DP cell (#611), so this only sees a float from an odd
        // re-typed import. Render it only if it fits i64 (see `f64_fits_i64`);
        // otherwise blank, rather than fabricate a saturated integer.
        if let Some(f) = value.as_f64().filter(|f| f64_fits_i64(*f)) {
            #[allow(clippy::cast_possible_truncation)] // range-checked by the filter
            return (f as i64).to_string();
        }
        return String::new();
    }
    if t.ends_with("DP") {
        let n = t[..t.len() - 2].parse::<usize>().unwrap_or(0);
        if let Some(f) = value.as_f64() {
            return format_ndp(f, n);
        }
    }
    if t.ends_with("SF") {
        let n = t[..t.len() - 2].parse::<usize>().unwrap_or(0);
        if let Some(f) = value.as_f64() {
            return format_nsf(f, n);
        }
    }
    if t.ends_with("SCI") {
        let n = t[..t.len() - 3].parse::<usize>().unwrap_or(0);
        if let Some(f) = value.as_f64() {
            return format_nsci(f, n);
        }
    }
    match value {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

// --- AGS4 field quoting (the write-side line primitive) ---------------
//
// The inverse of the tokenizer's inner-value unescape. Kept here beside
// `ags4_str` (its value→wire-form sibling) as the SINGLE authority for AGS4
// field quoting: `laterite-ags4-emit`'s byte-faithful `write_row` streams
// through it, and the browser tokenizer's tiny wasm reuses it via
// `quote_field`, so the browser's old TS copy (`quoteAgsField` in agsline.ts)
// is retired against one Rust source (#533, part of the #527 convergence arc).

/// Write `value` as one AGS4-quoted field to `out`: wrap it in double quotes,
/// doubling any embedded `"` (`""`) — AGS4's Rule-1 field escaping.
///
/// Streaming (generic over `W`) so the writer pays NO per-cell allocation on
/// its hot path — the no-quote branch writes the value's bytes straight
/// through. It carries NO Rule-6 CR/LF check: that stays a row-level guarantee
/// in the emitter (a field primitive can't reject what it can't see), so a
/// value containing a raw CR/LF is quoted verbatim here.
pub fn write_quoted_field<W: std::io::Write>(out: &mut W, value: &str) -> std::io::Result<()> {
    out.write_all(b"\"")?;
    // Escape embedded `"` → `""` (allocating only when there's one to double).
    if value.contains('"') {
        out.write_all(value.replace('"', "\"\"").as_bytes())?;
    } else {
        out.write_all(value.as_bytes())?;
    }
    out.write_all(b"\"")
}

/// [`write_quoted_field`] into an owned `String` — the AGS4-quoted form of
/// `value`. This IS the browser field-quoter (`quoteAgsField`), exposed through
/// the tiny wasm; native writers should prefer the streaming form to avoid the
/// per-cell allocation this necessarily makes.
///
/// Examples:
///   `quote_field("LOCA")`  -> `"\"LOCA\""`
///   `quote_field("a\"b")`  -> `"\"a\"\"b\""`
#[must_use]
pub fn quote_field(value: &str) -> String {
    let mut buf: Vec<u8> = Vec::with_capacity(value.len() + 2);
    // Writing to a `Vec<u8>` is infallible, and we only wrap valid UTF-8 in
    // ASCII quotes, so the bytes are always valid UTF-8.
    write_quoted_field(&mut buf, value).expect("Vec<u8> write is infallible");
    String::from_utf8(buf).expect("quote_field: ASCII quotes around UTF-8 stays UTF-8")
}

// --- the numeric expected-forms ---------------------------------------
//
// The three AGS4 numeric renderings, as `(value, n)` functions so the two
// callers that need them share ONE implementation: `ags4_str` above (which
// resolves `n` out of the type code) and laterite-ags4-validator's Rule 8 /
// fixes engine (which already has `n` parsed and re-exports these). The
// validator used to carry a hand-port of all three, kept honest only by a
// "ported from ags_types::ags4_str" comment — it agreed, but nothing checked
// it, so a validator could have judged a value by a different formatter than
// the one that WRITES it (#528). laterite-ags4-excel keeps its own, deliberately
// divergent formatter (uppercase `E`, bare `"0"` for SF-of-zero) — that
// divergence is by design and pinned in xcheck-allow.json.

/// Numeric TYPE counts (the `n` in "3DP" / "3SF" / "3SCI") are read straight
/// from a file's TYPE spec with no upper bound, and each `format_*` below feeds
/// `n` into a format width. f64 carries only ~17 significant decimal digits, so
/// a count past this generous ceiling is meaningless — clamping to it stops a
/// crafted spec like "9999999999DP" from asking for a ~10-billion-char string
/// (an OOM/DoS), or wrapping the i32 cast inside `format_nsf`. Real AGS4 numeric
/// counts are single-digit, so no legitimate value is affected. Hardens the
/// #610 Class B divergence (O-49); python-ags4 shares the same unbounded read.
const MAX_NUMERIC_COUNT: usize = 30;

/// nDP expected form — fixed-point with exactly `n` fractional digits.
/// NB `ags4_str` routes `0DP` to a truncating integer path BEFORE reaching
/// here, so `format_ndp(f, 0)` (which rounds) is the validator's grammar
/// form, not `ags4_str`'s `0DP` rendering. Both are deliberate.
#[must_use]
pub fn format_ndp(f: f64, n: usize) -> String {
    let n = n.min(MAX_NUMERIC_COUNT);
    format!("{f:.n$}")
}

/// nSF expected form — `n` significant figures in FIXED-POINT. python-ags4's
/// validator rejects scientific notation under nSF ("Value 1.0e2 not of data
/// type 2SF. Expected: 100"), so the canonical form rounds to `n` sig figs and
/// emits a plain decimal: trailing zeros for small magnitudes show the
/// precision (`0.002` → `"0.00200"` @3SF); large magnitudes get
/// integer-rounded (`1234` → `"1230"` @3SF).
#[must_use]
pub fn format_nsf(f: f64, n: usize) -> String {
    // Clamp first (see MAX_NUMERIC_COUNT): the bounded count fits i32, so the
    // dp arithmetic below can't wrap on a crafted "9999999999SF".
    let n = i32::try_from(n.min(MAX_NUMERIC_COUNT)).unwrap_or(i32::MAX);
    if f == 0.0 {
        return format!("{:.*}", (n - 1).max(0) as usize, 0.0);
    }
    // log10 of any finite f64 is bounded to roughly ±308 (f64's exponent
    // range), always fits i32 regardless of `f`'s magnitude.
    #[allow(clippy::cast_possible_truncation)]
    let exp = f.abs().log10().floor() as i32;
    let dp = n - exp - 1;
    if dp >= 0 {
        return format!("{:.*}", dp as usize, f);
    }
    // dp < 0: round to nearest 10^|dp|, emit as integer (no decimal
    // point — `{:.0}` does that).
    let scale = 10f64.powi(-dp);
    let rounded = (f / scale).round() * scale;
    format!("{rounded:.0}")
}

/// nSCI expected form — scientific notation, one digit before the point and
/// `n` after. Rust's `{:e}` already emits a single leading mantissa digit and
/// a sign-less-or-minus exponent with no leading zero (`1.5e2`, `3.1e-5`) —
/// exactly the AGS form, and LOWERCASE `e` (laterite-ags4-excel's uppercase `E` is
/// the registered by-design divergence).
#[must_use]
pub fn format_nsci(f: f64, n: usize) -> String {
    let n = n.min(MAX_NUMERIC_COUNT);
    format!("{f:.n$e}")
}

/// Presentation hint for a numeric AGS type: `'2DP'` → `Some("%.2f")`,
/// `'3SF'` → `Some("%.3g")`, `'1SCI'` → `Some("%.1e")`. Mirrors Python's
/// `display_hint`. String/datetime/bool types return `None`.
#[must_use]
pub fn display_hint(ags_type: &str) -> Option<String> {
    let t = ags_type.trim().to_uppercase();
    for (suffix, fmt_letter) in [("DP", 'f'), ("SF", 'g'), ("SCI", 'e')] {
        if t.ends_with(suffix) {
            let prefix = &t[..t.len() - suffix.len()];
            if !prefix.is_empty() && prefix.chars().all(|c| c.is_ascii_digit()) {
                return Some(format!("%.{prefix}{fmt_letter}"));
            }
        }
    }
    None
}

/// The **decimal-places family**: `Some(n)` iff the code is exactly `<digits>DP`
/// (`"0DP"` → `0`, `"2DP"` → `2`). `None` for everything else.
///
/// Deliberately narrower than [`canonical_type`], which lumps `nDP`, `nSF` and
/// `nSCI` together as [`CanonicalType::Decimal`]. They are *not* interchangeable:
/// decimal places are largely a presentation convention, whereas significant
/// figures and scientific notation are explicit claims about **measurement
/// precision**. A caller that may rewrite values (merge's `promote`) must key off
/// the code family, never the canonical class — hence this.
#[must_use]
pub fn decimal_places(ags_type: &str) -> Option<usize> {
    let t = ags_type.trim().to_uppercase();
    let prefix = t.strip_suffix("DP")?;
    if prefix.is_empty() || !prefix.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    prefix.parse().ok()
}

/// Widen a raw AGS numeric cell to **exactly** `n` decimal places by appending
/// zeros — the *lossless* half of a precision change (`"10.00"`, 5 → `"10.00000"`;
/// `"10"`, 2 → `"10.00"`; `"-3"`, 1 → `"-3.0"`). Idempotent when the value already
/// carries `n` places.
///
/// Returns `None` — meaning "keep the producer's bytes verbatim" — whenever the
/// pad cannot be done without changing the number:
///
/// - **More** than `n` decimal places. Shortening would *round* (`"10.005"` → 2dp
///   → `"10.00"`), and rounding is data loss. Padding only ever adds trailing
///   zeros; it never demotes.
/// - Blank, or anything outside the `-?\d+(\.\d*)?` grammar — there is no number
///   to pad, so the raw text is left alone for the validator to judge.
///
/// **String-only, by design: no `f64` is ever constructed.** A float round-trip
/// would silently perturb values beyond 2^53 (`"12345678901234567.00"` comes back
/// as `…568`), and a *widen* that alters a digit is a contradiction in terms.
/// Contrast the validator's `format_ndp`, which *is* an f64 round-and-render —
/// correct for a Rule 8 **fix** (where rounding is the intent), wrong here.
#[must_use]
pub fn pad_decimals(raw: &str, n: usize) -> Option<String> {
    let s = raw.trim();
    let body = s.strip_prefix('-').unwrap_or(s);
    let (int_part, frac) = body.split_once('.').unwrap_or((body, ""));
    if int_part.is_empty()
        || !int_part.chars().all(|c| c.is_ascii_digit())
        || !frac.chars().all(|c| c.is_ascii_digit())
        || frac.len() > n
    {
        return None;
    }
    let sign = if s.starts_with('-') { "-" } else { "" };
    if n == 0 {
        return Some(format!("{sign}{int_part}"));
    }
    let zeros = "0".repeat(n - frac.len());
    Some(format!("{sign}{int_part}.{frac}{zeros}"))
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
    let Some(ct) = canonical_type(ags_type) else {
        return Value::String(s.to_string());
    };
    match ct {
        CanonicalType::String | CanonicalType::Enum => Value::String(s.to_string()),
        // One integer parser (range-guarded, #611) for the leaf and laterite-py.
        CanonicalType::Integer => parse_ags_integer(s).map_or(Value::Null, Value::from),
        CanonicalType::Decimal => parse_ags_decimal(s)
            .and_then(Number::from_f64)
            .map_or(Value::Null, Value::Number),
        // Datetime / Date / Time / Bool route through the shared typed
        // parsers below, so there is ONE set of format tables and one parse
        // per category (the PyO3 wrapper in laterite-py calls the same four).
        // `parse_datetime` owns the date-only-promoted-to-midnight rule (a
        // `DT` cell legally carries just `2020-08-18` under a `yyyy-mm-dd`
        // UNIT); on export the midnight value renders back to date-only form.
        CanonicalType::Datetime => parse_datetime(s).map_or(Value::Null, |dt| {
            Value::String(dt.format("%Y-%m-%d %H:%M:%S").to_string())
        }),
        CanonicalType::Date => parse_date(s).map_or(Value::Null, |d| {
            Value::String(d.format("%Y-%m-%d").to_string())
        }),
        CanonicalType::Time => parse_time(s).map_or(Value::Null, |t| {
            Value::String(t.format("%H:%M:%S").to_string())
        }),
        CanonicalType::Bool => parse_bool(s).map_or(Value::Null, Value::Bool),
    }
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
#[must_use]
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

/// Parse an AGS4 DATE cell into a `NaiveDate`, trying the same
/// `DATE_FORMATS` `parse_value` uses. `None` when no format matches.
/// The typed twin of `parse_value`'s Date arm — shared with laterite-py's
/// PyO3 wrapper so there is one date parser, not two.
#[must_use]
pub fn parse_date(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    for fmt in DATE_FORMATS {
        if let Ok(d) = NaiveDate::parse_from_str(s, fmt) {
            return Some(d);
        }
    }
    None
}

/// Parse an AGS4 TIME cell into a `NaiveTime`, trying the same
/// `TIME_FORMATS` `parse_value` uses. `None` when no format matches.
/// Shared with laterite-py's PyO3 wrapper.
#[must_use]
pub fn parse_time(s: &str) -> Option<NaiveTime> {
    let s = s.trim();
    for fmt in TIME_FORMATS {
        if let Ok(t) = NaiveTime::parse_from_str(s, fmt) {
            return Some(t);
        }
    }
    None
}

/// Parse an AGS4 boolean (`YN`) cell: the `BOOL_TRUE` / `BOOL_FALSE`
/// token sets, case-folded. `None` for anything else. Shared with
/// laterite-py's PyO3 wrapper so both agree on the token set.
#[must_use]
pub fn parse_bool(s: &str) -> Option<bool> {
    let u = s.trim().to_uppercase();
    if BOOL_TRUE.contains(&u.as_str()) {
        Some(true)
    } else if BOOL_FALSE.contains(&u.as_str()) {
        Some(false)
    } else {
        None
    }
}

/// The f64 magnitude that bounds the i64 range. `i64::MAX` (2^63 − 1) is not
/// f64-representable, so the usable upper bound is the exclusive 2^63; `i64::MIN`
/// (−2^63) is exact, so the lower bound is inclusive `-I64_F64_BOUND`.
const I64_F64_BOUND: f64 = 9_223_372_036_854_775_808.0; // 2^63

/// True when a finite `f` maps onto i64 with only its fraction truncated (i.e.
/// `f as i64` is faithful, not saturating). Non-finite `f` (NaN/±∞) fails both
/// comparisons and returns `false`.
fn f64_fits_i64(f: f64) -> bool {
    (-I64_F64_BOUND..I64_F64_BOUND).contains(&f)
}

/// Parse an AGS4 Integer (`0DP`) cell to an `i64`, or `None` when the text is
/// not a finite number OR falls outside i64's range. The single source for the
/// leaf's own `parse_value`/`ags4_str` and laterite-py's PyO3 wrapper (#611
/// finishes the #531 dedup — that PR single-sourced the date/time/bool parsers
/// but left this Integer arm copied three ways).
///
/// The range guard is the #611 hardening: an out-of-range value returns `None`
/// (→ a Null typed value / `None` in Python) instead of the silently
/// *fabricated* `i64::MAX` a saturating `as` cast produced. Real geotech
/// integers never approach i64 (~9.2e18) — the largest integer column in
/// practice is a cyclic-triaxial cycle count (~1e4) — so the guard only ever
/// fires on a 19-digit value, which in a whole-number column is an Excel/export
/// error we want surfaced (the validator already flags it via Rule 8), not
/// coerced into a wrong number.
///
/// TO CHANGE THIS: if a data model ever needs integers past i64 and must
/// PRESERVE them (matching python-ags4's arbitrary-precision `int(float(s))`
/// → the deferred "full precision" option), you would store them at arbitrary
/// precision — `serde_json`'s `arbitrary_precision` feature or a bigint variant
/// in `Value` — thread that through `_content_hash` and the PyO3/typed-read
/// surfaces, and relax this guard. This guard is the minimal, low-risk option;
/// widening the store is the larger one.
#[must_use]
pub fn parse_ags_integer(s: &str) -> Option<i64> {
    match s.parse::<f64>() {
        // Tolerate "5.0" notation (Python `int(float(s))`); the cast truncates
        // the fraction toward zero, and `f64_fits_i64` has range-checked it.
        #[allow(clippy::cast_possible_truncation)]
        Ok(f) if f64_fits_i64(f) => Some(f as i64),
        _ => None,
    }
}

/// Parse an AGS4 Decimal-typed cell to an `f64`, or `None` when not a finite
/// number. The single source for the leaf's `parse_value` Decimal arm and
/// laterite-py's wrapper (the #531/#611 dedup, Decimal half).
#[must_use]
pub fn parse_ags_decimal(s: &str) -> Option<f64> {
    match s.parse::<f64>() {
        Ok(f) if f.is_finite() => Some(f),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal_places_matches_only_the_ndp_family() {
        assert_eq!(decimal_places("0DP"), Some(0));
        assert_eq!(decimal_places("2DP"), Some(2));
        assert_eq!(decimal_places("12DP"), Some(12));
        assert_eq!(decimal_places(" 3dp "), Some(3)); // trimmed + case-folded
        // nSF / nSCI are Decimal to `canonical_type` but are NOT decimal places:
        // they claim measurement precision, so they must never be zero-padded.
        assert_eq!(decimal_places("3SF"), None);
        assert_eq!(decimal_places("2SCI"), None);
        assert_eq!(decimal_places("X"), None);
        assert_eq!(decimal_places("DP"), None); // empty prefix
        assert_eq!(decimal_places("XDP"), None); // non-digit prefix
    }

    #[test]
    fn pad_decimals_appends_zeros_and_is_idempotent() {
        assert_eq!(pad_decimals("10.00", 5).as_deref(), Some("10.00000"));
        assert_eq!(pad_decimals("10", 2).as_deref(), Some("10.00")); // 0DP → 2DP
        assert_eq!(pad_decimals("-3", 1).as_deref(), Some("-3.0"));
        assert_eq!(pad_decimals("-0.5", 3).as_deref(), Some("-0.500"));
        assert_eq!(pad_decimals("10.00", 2).as_deref(), Some("10.00")); // already n
        assert_eq!(pad_decimals("7", 0).as_deref(), Some("7"));
    }

    #[test]
    fn pad_decimals_refuses_anything_it_cannot_do_losslessly() {
        // Would have to ROUND — that is data loss, so refuse and keep the bytes.
        assert_eq!(pad_decimals("10.005", 2), None);
        assert_eq!(pad_decimals("10.5", 0), None);
        // Nothing to pad.
        assert_eq!(pad_decimals("", 2), None);
        assert_eq!(pad_decimals("   ", 2), None);
        assert_eq!(pad_decimals("N/A", 2), None);
        assert_eq!(pad_decimals("1.2e3", 2), None); // exponent is not the nDP grammar
        assert_eq!(pad_decimals("+10", 2), None); // AGS nDP has no leading '+'
        assert_eq!(pad_decimals("1,000.0", 2), None); // thousands separator
    }

    /// The reason `pad_decimals` is string-only. An f64 round-trip — which is what
    /// the validator's `format_ndp` does, correctly, for a *fix* — cannot represent
    /// this value, so a "widen" through a float would silently change a digit.
    /// Padding must never alter the number it is padding.
    #[test]
    fn pad_decimals_never_perturbs_a_value_an_f64_would_mangle() {
        let raw = "12345678901234567.00"; // 17 significant digits, > 2^53
        let padded = pad_decimals(raw, 5).expect("pads");
        assert_eq!(padded, "12345678901234567.00000");
        // The digits the producer wrote survive verbatim...
        assert!(padded.starts_with("12345678901234567."));
        // ...whereas the float route loses the last one.
        let via_f64 = format!("{:.5}", raw.parse::<f64>().unwrap());
        assert_ne!(via_f64, padded);
        assert_eq!(via_f64, "12345678901234568.00000");
    }

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

    #[test]
    fn nsf_count_is_clamped_so_a_crafted_type_cannot_dos() {
        // The SF count is read straight from a file's TYPE spec ("3SF") with no
        // upper bound (#610 Class B, O-49). python-ags4's `_format_SF` reads it
        // the same way at arbitrary precision, so a crafted "9999999999SF" makes
        // it request a ~10-billion-place format width and OOM. We clamp to
        // MAX_NUMERIC_COUNT first, so an absurd count collapses to a bounded
        // string instead of wrapping the i32 cast or blowing up the width.
        let ceiling = format_nsf(1.5, MAX_NUMERIC_COUNT);
        assert_eq!(format_nsf(1.5, 9_999_999_999), ceiling);
        assert_eq!(format_nsf(1.5, usize::MAX), ceiling); // saturating clamp path
        assert!(ceiling.len() < 40, "clamped output stays bounded");
        // The zero-value branch uses the count for its width too — also clamped.
        assert!(format_nsf(0.0, usize::MAX).len() < 40);
        // Legit SF counts (≤ the ceiling) render exactly as before.
        assert_eq!(format_nsf(1.23, 3), "1.23");
        assert_eq!(format_nsf(1234.0, 3), "1230");
        // The nDP / nSCI siblings feed `n` straight into the width — clamp too.
        assert!(format_ndp(1.5, usize::MAX).len() < 40);
        assert!(format_nsci(1.5, usize::MAX).len() < 40);
        assert_eq!(format_ndp(1.5, 3), "1.500"); // legit count untouched
        assert_eq!(format_nsci(1500.0, 2), "1.50e3");
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
        // RL is a delimited RECORD LINK (`GROUP|KEY1|KEY2`, AGS Rule 11), so it
        // stores as text. It was DOUBLE — which nulled every link on read (#503).
        assert_eq!(sql_type("RL"), "VARCHAR");
        // Unknown / passthrough code.
        assert_eq!(sql_type("BANANA"), "VARCHAR");
    }

    #[test]
    fn canonical_type_rl_is_a_text_record_link_not_a_number() {
        // #503: RL was Decimal. A record link is `SAMP|BH01|1.00` — parsing it as a
        // float yields Null, so the column read back as an all-null f64 and the
        // link was destroyed. This assertion previously pinned the bug.
        assert_eq!(canonical_type("RL"), Some(CanonicalType::String));
        assert_eq!(canonical_type("DT"), Some(CanonicalType::Datetime));
        assert_eq!(canonical_type("YN"), Some(CanonicalType::Bool));
        assert_eq!(canonical_type("0DP"), Some(CanonicalType::Integer));
    }

    /// **The `_content_hash` canonicalisation contract, pinned literally.**
    ///
    /// `parse_value`'s output is exactly what `keychain::content_hash` hashes
    /// (through `serde_json::Value::to_string`), yet the two crates are bound
    /// only by this behaviour — nothing at compile time couples them. So a
    /// change to any literal below silently re-computes every `_content_hash`
    /// already in the wild: the #503 RL episode did precisely this (RL went
    /// Decimal→String, the hashed form of every record-link cell changed, and
    /// no test failed). If you move one of these deliberately you MUST bump
    /// `keychain::CONTENT_HASH_DOMAIN` in the same change, so an old hash and a
    /// new one can never be conflated. This table is the tripwire that says so.
    #[test]
    fn parse_value_canonical_form_is_pinned_for_the_content_hash_contract() {
        let pv = |ty: &str, raw: &str| parse_value(Some(raw), ty).to_string();

        // String family — quoted, verbatim. RL is a record LINK (text), never a
        // number: `SAMP|BH01|1.00` stays a string (#503).
        assert_eq!(pv("X", "silty CLAY"), "\"silty CLAY\"");
        assert_eq!(pv("ID", "BH01"), "\"BH01\"");
        assert_eq!(pv("PA", "CU"), "\"CU\"");
        assert_eq!(pv("RL", "SAMP|BH01|1.00"), "\"SAMP|BH01|1.00\"");

        // Integer — an i64; "5.0" is tolerated as 5 (Python `int(float(s))`).
        assert_eq!(pv("0DP", "5"), "5");
        assert_eq!(pv("0DP", "5.0"), "5");

        // Decimal — an f64. Trailing-zero precision collapses, so "10.00",
        // "10.0" and "10" hash alike (a re-emit is not a value change).
        assert_eq!(pv("2DP", "10.00"), "10.0");
        assert_eq!(pv("2DP", "10.0"), "10.0");
        assert_eq!(pv("3SF", "0.00"), "0.0");

        // Datetime — normalised to "%Y-%m-%d %H:%M:%S"; a date-only cell is
        // promoted to midnight.
        assert_eq!(pv("DT", "2020-08-18 09:30:00"), "\"2020-08-18 09:30:00\"");
        assert_eq!(pv("DT", "2020-08-18"), "\"2020-08-18 00:00:00\"");

        // Bool.
        assert_eq!(pv("YN", "Y"), "true");
        assert_eq!(pv("YN", "N"), "false");

        // Blank ≡ absent → Null → the pair is DROPPED from the hash entirely.
        assert_eq!(pv("X", ""), "null");
        assert_eq!(pv("2DP", "   "), "null");

        // Unknown / passthrough code → falls through to String.
        assert_eq!(pv("BANANA", "whatever"), "\"whatever\"");
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
    // 3.14159 is a deliberate test input chosen to round to "3.142" at
    // 3dp — not an attempt to approximate PI.
    #[allow(clippy::approx_constant)]
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

    // --- quote_field / write_quoted_field -------------------------

    #[test]
    fn quote_field_wraps_and_doubles_embedded_quotes() {
        assert_eq!(quote_field("LOCA"), "\"LOCA\"");
        assert_eq!(quote_field(""), "\"\"");
        // Embedded `"` is doubled (AGS4 Rule-1 escaping).
        assert_eq!(quote_field(r#"he said "hi""#), r#""he said ""hi""""#);
        // A comma is data inside the quotes, not a delimiter.
        assert_eq!(quote_field("a,b"), "\"a,b\"");
    }

    #[test]
    fn quote_field_is_the_streaming_form_collected() {
        // The owned wrapper must be byte-identical to streaming into a buffer —
        // they are one authority, so a divergence here is a real bug.
        for v in ["", "x", "a\"b", "a\"\"b", "with,comma", "  padded  ", "🦀"] {
            let mut buf: Vec<u8> = Vec::new();
            write_quoted_field(&mut buf, v).unwrap();
            assert_eq!(
                String::from_utf8(buf).unwrap(),
                quote_field(v),
                "value {v:?}"
            );
        }
    }

    #[test]
    fn write_quoted_field_carries_no_cr_lf_check() {
        // The field primitive quotes a raw CR/LF verbatim — Rule 6 rejection is
        // the emitter's ROW-level job, not this primitive's.
        assert_eq!(quote_field("a\r\nb"), "\"a\r\nb\"");
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
    fn parse_ags_integer_guards_the_i64_range() {
        // In-range: truncate toward zero (Python `int(float(s))`); "5.0" tolerated.
        assert_eq!(parse_ags_integer("42"), Some(42));
        assert_eq!(parse_ags_integer("-42"), Some(-42));
        assert_eq!(parse_ags_integer("5.0"), Some(5));
        assert_eq!(parse_ags_integer("5.7"), Some(5));
        assert_eq!(parse_ags_integer("1E-30"), Some(0)); // tiny -> 0, as Python
        // The largest f64-representable i64 still parses; 2^63 and up do not.
        assert_eq!(
            parse_ags_integer("9223372036854774784"),
            Some(9_223_372_036_854_774_784)
        );
        // Out-of-range -> None, NOT a fabricated i64::MAX (the #611 hardening).
        assert_eq!(parse_ags_integer("1E30"), None);
        assert_eq!(parse_ags_integer("99999999999999999999"), None);
        assert_eq!(parse_ags_integer("-1E30"), None);
        assert_eq!(parse_ags_integer("9223372036854775808"), None); // 2^63
        // Non-numeric / non-finite -> None.
        assert_eq!(parse_ags_integer("abc"), None);
        assert_eq!(parse_ags_integer("inf"), None);
        assert_eq!(parse_ags_integer("NaN"), None);
        assert_eq!(parse_ags_integer(""), None);
    }

    #[test]
    fn parse_value_0dp_overflow_is_null_not_fabricated() {
        // The observable #611 change: a giant 0DP value no longer canonicalises
        // to a fabricated i64::MAX — it becomes Null. In-range is unchanged, so
        // _content_hash for real data is untouched.
        assert_eq!(parse_value(Some("42"), "0DP"), Value::from(42));
        assert_eq!(parse_value(Some("5.0"), "0DP"), Value::from(5));
        assert_eq!(parse_value(Some("1E30"), "0DP"), Value::Null);
    }

    #[test]
    fn parse_ags_decimal_takes_finite_floats_only() {
        assert_eq!(parse_ags_decimal("2.5"), Some(2.5));
        // Decimals hold big magnitudes fine — the i64 guard is Integer-only.
        assert_eq!(parse_ags_decimal("1E30"), Some(1e30));
        assert_eq!(parse_ags_decimal("inf"), None);
        assert_eq!(parse_ags_decimal("NaN"), None);
        assert_eq!(parse_ags_decimal("x"), None);
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

// The README's example is a doctest, not a second copy of one. `cfg(doctest)`
// means this module exists only while rustdoc collects doctests: it is absent
// from a normal build and from the rendered docs.rs page, so the crate's own
// `//!` docs are untouched and nothing is duplicated. The README is the single
// source, and `cargo test --workspace` already compiles it.
//
// The example is written out in full — no rustdoc `# ` hidden lines. A README is
// also read as plain Markdown on crates.io, where `# let x = …` renders as an
// <h1>. Visible boilerplate is the price of a page that is checked AND readable.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme_doctests {}
