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
//! (`laterite-types::arrow_cols`, `Value`→Arrow).

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
            Value::from(array.as_any().downcast_ref::<$ty>().unwrap().value(row) as i64)
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
        DataType::Float32 => Value::from(
            array
                .as_any()
                .downcast_ref::<Float32Array>()
                .unwrap()
                .value(row) as f64,
        ),
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
pub fn group_from_arrow(code: String, schema: &Schema, batches: &[RecordBatch]) -> GroupInput {
    group_from_arrow_with_meta(code, schema, batches, None, None)
}

/// Like [`group_from_arrow`] but with per-heading UNIT/TYPE **overrides**: a
/// `{heading → value}` map (#294 F#9). The map is aligned to the group's
/// headings into the per-heading `Option<Vec<String>>` [`GroupInput`] wants — a
/// heading named in the map takes that value, any other heading gets a blank
/// entry ("fill from the dictionary"). `None` leaves the whole tier to the
/// dictionary (identical to `group_from_arrow`). Order-independent: it keys off
/// the heading name, not the column position.
pub fn group_from_arrow_with_meta(
    code: String,
    schema: &Schema,
    batches: &[RecordBatch],
    units: Option<&HashMap<String, String>>,
    types: Option<&HashMap<String, String>>,
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
    let align = |m: Option<&HashMap<String, String>>| -> Option<Vec<String>> {
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
