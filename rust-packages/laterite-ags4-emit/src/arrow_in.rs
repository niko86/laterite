//! The Arrow → `serde_json::Value` cell transpose — the single shared
//! conversion for both emit hosts (behind the `arrow` feature):
//!
//! - **native** (`laterite-py`): a DuckDB relation's Arrow C-stream capsule
//!   → `PyTable` → `(batches, schema)` → here;
//! - **wasm** (`laterite-ags4-wasm`): an Arrow IPC stream → `StreamReader` →
//!   `(batches, schema)` → here.
//!
//! Living in `laterite-ags4-emit` keeps the type→`Value` mapping in one place, so the
//! two hosts can't drift. Symmetric with the read path's shared *builder*
//! (`laterite-ags4-types::arrow_cols`, `Value`→Arrow).

use arrow::array::{
    Array, BooleanArray, Float32Array, Float64Array, Int8Array, Int16Array, Int32Array, Int64Array,
    LargeStringArray, StringArray, UInt8Array, UInt16Array, UInt32Array, UInt64Array,
};
use std::collections::HashMap;

use arrow::datatypes::{DataType, Schema};
use arrow::record_batch::RecordBatch;
use arrow::util::display::{ArrayFormatter, FormatOptions};
use serde_json::Value;

use crate::GroupInput;

/// One cell of an Arrow column → the `serde_json::Value` the orchestrator
/// formats. Typed numerics/bools become JSON numbers/bools (so `ags4_str`
/// renders them canonically); strings stay strings (emitted verbatim, the
/// validity mode owns canonicalisation); temporal/decimal/other types fall
/// back to Arrow's own canonical display string (e.g. `2023-02-22T10:24:00`).
pub fn cell_value(array: &dyn Array, row: usize) -> Value {
    if array.is_null(row) {
        return Value::Null;
    }
    macro_rules! num {
        ($ty:ty) => {
            Value::from(i64::from(
                array.as_any().downcast_ref::<$ty>().unwrap().value(row),
            ))
        };
    }
    match array.data_type() {
        DataType::Utf8 => Value::String(
            array
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap()
                .value(row)
                .to_string(),
        ),
        DataType::LargeUtf8 => Value::String(
            array
                .as_any()
                .downcast_ref::<LargeStringArray>()
                .unwrap()
                .value(row)
                .to_string(),
        ),
        DataType::Boolean => Value::Bool(
            array
                .as_any()
                .downcast_ref::<BooleanArray>()
                .unwrap()
                .value(row),
        ),
        DataType::Int8 => num!(Int8Array),
        DataType::Int16 => num!(Int16Array),
        DataType::Int32 => num!(Int32Array),
        DataType::Int64 => num!(Int64Array),
        DataType::UInt8 => num!(UInt8Array),
        DataType::UInt16 => num!(UInt16Array),
        DataType::UInt32 => num!(UInt32Array),
        DataType::UInt64 => Value::from(
            array
                .as_any()
                .downcast_ref::<UInt64Array>()
                .unwrap()
                .value(row),
        ),
        DataType::Float32 => Value::from(f64::from(
            array
                .as_any()
                .downcast_ref::<Float32Array>()
                .unwrap()
                .value(row),
        )),
        DataType::Float64 => Value::from(
            array
                .as_any()
                .downcast_ref::<Float64Array>()
                .unwrap()
                .value(row),
        ),
        // Temporal / decimal / anything else: Arrow's display formatter gives
        // the canonical string (then `ags4_str`'s DT/passthrough handling +
        // the validity mode finish the job).
        _ => match ArrayFormatter::try_new(array, &FormatOptions::default()) {
            Ok(fmt) => Value::String(fmt.value(row).to_string()),
            Err(_) => Value::Null,
        },
    }
}

/// Build a [`GroupInput`] from Arrow record batches: headings are the schema
/// field names (the AGS headings); UNIT/TYPE are left to the dictionary; rows
/// are the transposed cells. `schema` is passed explicitly so a 0-batch group
/// still emits its (empty) section with the right headings.
#[must_use]
pub fn group_from_arrow(code: String, schema: &Schema, batches: &[RecordBatch]) -> GroupInput {
    // No override maps, so the hasher never matters — pin the default one to
    // give type inference something concrete for `S`.
    group_from_arrow_with_meta::<std::collections::hash_map::RandomState>(
        code, schema, batches, None, None,
    )
}

/// Like [`group_from_arrow`] but with per-heading UNIT/TYPE **overrides**: a
/// `{heading → value}` map (#294 F#9). The map is aligned to the group's
/// headings into the per-heading `Option<Vec<String>>` [`GroupInput`] wants — a
/// heading named in the map takes that value, any other heading gets a blank
/// entry ("fill from the dictionary"). `None` leaves the whole tier to the
/// dictionary (identical to `group_from_arrow`). Order-independent: it keys off
/// the heading name, not the column position.
#[must_use]
pub fn group_from_arrow_with_meta<S: std::hash::BuildHasher>(
    code: String,
    schema: &Schema,
    batches: &[RecordBatch],
    units: Option<&HashMap<String, String, S>>,
    types: Option<&HashMap<String, String, S>>,
) -> GroupInput {
    let headings: Vec<String> = schema.fields().iter().map(|f| f.name().clone()).collect();
    let mut rows: Vec<Vec<Value>> = Vec::new();
    for batch in batches {
        let ncols = batch.num_columns();
        for r in 0..batch.num_rows() {
            let mut row = Vec::with_capacity(ncols);
            for c in 0..ncols {
                row.push(cell_value(batch.column(c).as_ref(), r));
            }
            rows.push(row);
        }
    }
    // Align a {heading → value} override map to the heading order; a heading not
    // in the map gets "" (the emit orchestrator reads that as "fill from dict").
    let align = |m: Option<&HashMap<String, String, S>>| -> Option<Vec<String>> {
        m.map(|map| {
            headings
                .iter()
                .map(|h| map.get(h).cloned().unwrap_or_default())
                .collect()
        })
    };
    let units = align(units);
    let types = align(types);
    GroupInput {
        code,
        headings,
        units,
        types,
        rows,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{
        ArrayRef, Date32Array, Decimal128Array, TimestampMillisecondArray, UnionArray,
    };
    use arrow::datatypes::{Field, UnionFields};
    use std::sync::Arc;

    fn one(a: &dyn Array) -> Value {
        cell_value(a, 0)
    }

    /// Every arm the `match` names, pinned to the `Value` variant the emit
    /// orchestrator expects. `ags4_str` renders numbers and bools differently
    /// from strings, so an arm returning the wrong *variant* changes the output
    /// bytes even when the text looks right.
    #[test]
    fn typed_arms_map_to_their_json_variant() {
        assert_eq!(one(&StringArray::from(vec!["BH01"])), Value::from("BH01"));
        assert_eq!(
            one(&LargeStringArray::from(vec!["BH01"])),
            Value::from("BH01")
        );
        assert_eq!(one(&BooleanArray::from(vec![true])), Value::Bool(true));
        assert_eq!(one(&Int8Array::from(vec![-8i8])), Value::from(-8));
        assert_eq!(one(&Int16Array::from(vec![-16i16])), Value::from(-16));
        assert_eq!(one(&Int32Array::from(vec![-32i32])), Value::from(-32));
        assert_eq!(one(&Int64Array::from(vec![-64i64])), Value::from(-64));
        assert_eq!(one(&UInt8Array::from(vec![8u8])), Value::from(8));
        assert_eq!(one(&UInt16Array::from(vec![16u16])), Value::from(16));
        assert_eq!(one(&UInt32Array::from(vec![32u32])), Value::from(32));
        assert_eq!(one(&UInt64Array::from(vec![64u64])), Value::from(64));
        assert_eq!(one(&Float64Array::from(vec![1.5f64])), Value::from(1.5));
        // f32 widens to f64 — pinned because a naive `Value::from(f32)` would
        // serialise 1.5f32 as 1.5000000596046448.
        assert_eq!(one(&Float32Array::from(vec![1.5f32])), Value::from(1.5));
    }

    /// A null cell is `Value::Null` for EVERY type, checked before the match —
    /// so the `unwrap()`s on the downcasts are only ever reached for a valid row.
    #[test]
    fn nulls_short_circuit_before_the_downcast() {
        assert_eq!(one(&StringArray::from(vec![None::<&str>])), Value::Null);
        assert_eq!(one(&Int64Array::from(vec![None::<i64>])), Value::Null);
        assert_eq!(one(&BooleanArray::from(vec![None::<bool>])), Value::Null);
        assert_eq!(one(&Date32Array::from(vec![None::<i32>])), Value::Null);
    }

    /// The fallback arm. **Every `DT` column on the emit path lands here**, so
    /// these exact strings are load-bearing: they are what `ags4_str` receives
    /// and what ends up in the file.
    #[test]
    fn temporal_and_decimal_fall_back_to_arrows_canonical_string() {
        assert_eq!(
            one(&Date32Array::from(vec![19738])),
            Value::from("2024-01-16")
        );
        assert_eq!(
            one(&TimestampMillisecondArray::from(vec![1_677_064_000_000i64])),
            Value::from("2023-02-22T11:06:40")
        );
        let dec = Decimal128Array::from(vec![123_456i128])
            .with_precision_and_scale(10, 2)
            .expect("valid precision/scale");
        assert_eq!(one(&dec), Value::from("1234.56"));
    }

    /// The `Err(_) => Value::Null` arm is **defensive, not live**. Probed across
    /// arrow 59's exotic types — union, run-end, dictionary, struct, map,
    /// interval, duration, binary, fixed-size-binary, time64 — `try_new` does
    /// not fail for any of them. This test pins the one most likely to regress
    /// if arrow narrows its formatter, so the arm cannot start silently nulling
    /// real cells without a test going red first.
    #[test]
    fn the_formatter_fallback_is_not_silently_swallowing_a_live_type() {
        let fields: UnionFields = [(0i8, Arc::new(Field::new("a", DataType::Int32, false)))]
            .into_iter()
            .collect();
        let u = UnionArray::try_new(
            fields,
            vec![0i8].into(),
            None,
            vec![Arc::new(Int32Array::from(vec![7])) as ArrayRef],
        )
        .expect("valid union");
        assert_ne!(
            cell_value(&u, 0),
            Value::Null,
            "a formattable type reached the Err arm — the fallback is now eating real data"
        );
    }

    fn schema2() -> Schema {
        Schema::new(vec![
            Field::new("LOCA_ID", DataType::Utf8, true),
            Field::new("LOCA_GL", DataType::Float64, true),
        ])
    }

    fn batch(ids: Vec<&str>, gls: Vec<f64>) -> RecordBatch {
        RecordBatch::try_new(
            Arc::new(schema2()),
            vec![
                Arc::new(StringArray::from(ids)) as ArrayRef,
                Arc::new(Float64Array::from(gls)) as ArrayRef,
            ],
        )
        .expect("batch")
    }

    #[test]
    fn headings_come_from_the_schema_and_rows_are_transposed() {
        let g = group_from_arrow(
            "LOCA".into(),
            &schema2(),
            &[batch(vec!["A", "B"], vec![1.0, 2.0])],
        );
        assert_eq!(g.code, "LOCA");
        assert_eq!(g.headings, ["LOCA_ID", "LOCA_GL"]);
        // Column-major in, row-major out.
        assert_eq!(
            g.rows,
            vec![
                vec![Value::from("A"), Value::from(1.0)],
                vec![Value::from("B"), Value::from(2.0)],
            ]
        );
        assert!(
            g.units.is_none() && g.types.is_none(),
            "left to the dictionary"
        );
    }

    /// Batches concatenate in order — a multi-batch stream is one group, not one
    /// group per batch.
    #[test]
    fn multiple_batches_concatenate() {
        let g = group_from_arrow(
            "LOCA".into(),
            &schema2(),
            &[batch(vec!["A"], vec![1.0]), batch(vec!["B"], vec![2.0])],
        );
        assert_eq!(g.rows.len(), 2);
        assert_eq!(g.rows[1][0], Value::from("B"));
    }

    /// The reason `schema` is a separate argument: a group whose stream carried
    /// no batches still has to emit its section with the right headings.
    #[test]
    fn zero_batches_still_yields_the_headings() {
        let g = group_from_arrow("LOCA".into(), &schema2(), &[]);
        assert_eq!(g.headings, ["LOCA_ID", "LOCA_GL"]);
        assert!(g.rows.is_empty());
    }

    /// Overrides align by heading NAME, not column position, and a heading the
    /// map does not mention gets "" — which the orchestrator reads as "fill from
    /// the dictionary", not as an empty UNIT.
    #[test]
    fn meta_overrides_align_by_name_and_leave_unnamed_headings_blank() {
        let units: HashMap<String, String> = [("LOCA_GL".to_string(), "m".to_string())]
            .into_iter()
            .collect();
        let g = group_from_arrow_with_meta("LOCA".into(), &schema2(), &[], Some(&units), None);
        assert_eq!(g.units.expect("units"), ["", "m"]);
        assert!(
            g.types.is_none(),
            "None leaves the whole tier to the dictionary"
        );
    }
}
