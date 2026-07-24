//! Permanent guard that `build_column` stays byte-faithful to `parse_value`.
//!
//! `build_column` types Integer/Decimal/String by casting a whole `Utf8` column
//! in bulk through Arrow's kernels (T3), which is ~4× faster than the per-cell
//! `parse_value` it replaced. This test pins the two to the same output: side A
//! is the production `build_column`; side B (`reference_column`) is the exact
//! per-cell parse the arms used *before* T3. They must be Arrow-representation
//! identical — same `DataType`, null bitmap, and non-null values (`ArrayData`
//! logical equality, which is what the C-data interface and IPC carry to polars
//! / duckdb / arrow-js).
//!
//! Coverage: crafted edge cases at the exact seams where a generic cast could
//! diverge (truncation direction, the i64 range guard, inf/NaN, f64-precision
//! boundaries) plus every column of the forge fixture (self-skips if absent).
//!
//! Run: `cargo test -p laterite-types --features arrow --test typed_build_parity`
#![cfg(feature = "arrow")]

use std::path::PathBuf;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, BooleanBuilder, Float64Builder, Int64Builder, StringBuilder,
    TimestampMicrosecondBuilder,
};
use arrow::datatypes::{DataType, TimeUnit};
use arrow::util::display::{ArrayFormatter, FormatOptions};
use serde_json::Value;

use laterite_types::arrow_cols::build_column;
use laterite_types::{CanonicalType, canonical_type, parse_datetime, parse_value};

/// The pre-T3 per-cell build — the semantic reference `build_column` must match.
/// Deliberately a straight transcription of the old arms so a future change to
/// `build_column` is measured against `parse_value`, not against itself.
fn reference_column<'a, F>(n_rows: usize, ags_type: &str, cell: F) -> (ArrayRef, DataType)
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
        _ => {
            let mut b = StringBuilder::new();
            for row in 0..n_rows {
                match parse_value(cell(row), ags_type) {
                    Value::String(s) => b.append_value(s),
                    Value::Null => b.append_null(),
                    other => b.append_value(other.to_string()),
                }
            }
            (Arc::new(b.finish()) as ArrayRef, DataType::Utf8)
        }
    }
}

/// A cell rendered for comparison: `None` for a typed null, else its canonical
/// Arrow string form. `""`-valued strings never occur (empty → null), so `None`
/// unambiguously means null.
fn render(arr: &ArrayRef) -> Vec<Option<String>> {
    let opts = FormatOptions::default();
    let fmt = ArrayFormatter::try_new(arr.as_ref(), &opts).expect("formatter");
    (0..arr.len())
        .map(|i| (!arr.is_null(i)).then(|| fmt.value(i).to_string()))
        .collect()
}

/// Arrow-native equality — `DataType` + null bitmap + non-null values. This is
/// the object polars/duckdb/arrow-js import, so representation parity here is
/// end-to-end parity for every downstream consumer.
fn arrays_equal(a: &ArrayRef, b: &ArrayRef) -> bool {
    a.to_data() == b.to_data()
}

fn assert_column_parity(label: &str, ags_type: &str, raws: &[Option<&str>]) {
    let n = raws.len();
    let cell = |row: usize| raws[row];
    let (got, dt_got) = build_column(n, ags_type, cell);
    let (want, dt_want) = reference_column(n, ags_type, cell);

    assert_eq!(
        format!("{dt_got:?}"),
        format!("{dt_want:?}"),
        "{label}: datatype differs — build_column {dt_got:?} vs parse_value {dt_want:?}"
    );
    if arrays_equal(&got, &want) {
        return;
    }
    let rg = render(&got);
    let rw = render(&want);
    let show = |o: &Option<String>| o.clone().unwrap_or_else(|| "<null>".to_string());
    let mism: Vec<_> = (0..n).filter(|&r| rg[r] != rw[r]).collect();
    eprintln!(
        "\n=== {label} ({ags_type}) — {} divergent cell(s) ===",
        mism.len()
    );
    eprintln!("{:<28} {:>18} {:>18}", "raw", "parse_value", "build_column");
    for r in &mism {
        eprintln!(
            "{:<28} {:>18} {:>18}",
            raws[*r].unwrap_or("<None>"),
            show(&rw[*r]),
            show(&rg[*r])
        );
    }
    panic!(
        "{label}: build_column diverges from parse_value ({} cell(s))",
        mism.len()
    );
}

#[test]
fn integer_0dp_matches_parse_value() {
    let raws = [
        Some("5"),
        Some("5.0"),
        Some("5.7"),    // truncate toward zero → 5
        Some("-5.7"),   // → -5
        Some("12.999"), // → 12
        Some("0"),
        Some("-0.0"),
        Some("1E-30"),                // → 0
        Some(" 42 "),                 // trimmed
        Some("1e30"),                 // > 2^63 → null
        Some("99999999999999999999"), // 1e20, > 2^63 → null
        Some("1e400"),                // inf → null
        Some("-1e400"),               // -inf → null
        Some("9007199254740993"),     // 2^53+1: f64-lossy in BOTH (parse_ags_integer uses f64)
        Some("9223372036854775807"),  // i64::MAX: f64 rounds to 2^63 → range guard nulls
        Some("abc"),
        Some(""),
        Some("   "),
        None,
    ];
    assert_column_parity("integer", "0DP", &raws);
}

#[test]
fn decimal_matches_parse_value() {
    let raws = [
        Some("12.34"),
        Some("1.23"),
        Some("-5.0"),
        Some("0"),
        Some("-0.0"),
        Some("1E-30"),
        Some("1e30"),
        Some(" 3.14 "),
        Some("inf"), // non-finite → null (Arrow admits it; the finite pass nulls it)
        Some("-inf"),
        Some("Infinity"),
        Some("nan"),
        Some("NaN"),
        Some("1e400"), // overflow → inf → null
        Some("abc"),
        Some(""),
        Some("   "),
        None,
    ];
    assert_column_parity("decimal-2dp", "2DP", &raws);
    // A significant-figures decimal code exercises the same arm via a different
    // canonical_type entry point.
    assert_column_parity("decimal-3sf", "3SF", &raws);
}

#[test]
fn string_bool_datetime_match_parse_value() {
    assert_column_parity(
        "string",
        "ID",
        &[Some("BH1"), Some(" BH2 "), Some(""), Some("  "), None],
    );
    assert_column_parity(
        "bool",
        "YN",
        &[
            Some("Y"),
            Some("n"),
            Some("YES"),
            Some("maybe"),
            Some(""),
            None,
        ],
    );
    assert_column_parity(
        "datetime",
        "DT",
        &[
            Some("2024-01-15T09:30:00"),
            Some("2020-08-18"),
            Some("garbage"),
            Some(""),
            None,
        ],
    );
}

fn fixture() -> Option<PathBuf> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../output/bench-fixtures/large.ags");
    p.exists().then_some(p)
}

#[test]
fn fixture_every_column_matches_parse_value() {
    let Some(path) = fixture() else {
        eprintln!("SKIP fixture parity: large.ags absent (run tools/gen-bench-fixtures.sh)");
        return;
    };
    let bytes = std::fs::read(&path).expect("fixture readable");
    let pf = laterite_ags4_parse::parse_bytes(&bytes, encoding_rs::UTF_8).expect("parses");

    let mut total_cells = 0usize;
    let mut total_cols = 0usize;
    let mut diverging: Vec<(String, String, usize)> = Vec::new();

    for code in &pf.group_order {
        let Some(grp) = pf.groups.get(code) else {
            continue;
        };
        let n = grp.rows.len();
        for (col, heading) in grp.headings.iter().enumerate() {
            let ags_type = grp.types.get(col).map_or("X", String::as_str);
            let cell = |row: usize| grp.cell(col, row);
            let (got, dt_got) = build_column(n, ags_type, cell);
            let (want, dt_want) = reference_column(n, ags_type, cell);
            total_cells += n;
            total_cols += 1;
            if dt_got != dt_want || !arrays_equal(&got, &want) {
                let rg = render(&got);
                let rw = render(&want);
                diverging.push((
                    code.clone(),
                    heading.clone(),
                    (0..n).filter(|&i| rg[i] != rw[i]).count(),
                ));
            }
        }
    }

    eprintln!(
        "\ntyped-build parity: {total_cols} columns / {total_cells} cells across {} groups",
        pf.group_order.len()
    );
    if diverging.is_empty() {
        eprintln!("  → 0 diverging columns (build_column == parse_value on real data)");
    } else {
        eprintln!("  → {} diverging columns:", diverging.len());
        for (g, h, m) in diverging.iter().take(30) {
            eprintln!("    {g}.{h}: {m} cell(s)");
        }
    }
    assert!(
        diverging.is_empty(),
        "{} columns diverge from parse_value",
        diverging.len()
    );
}
