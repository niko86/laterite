//! The Arrow → [`Cell`] cell transpose — the single shared
//! conversion for both emit hosts (behind the `arrow` feature):
//!
//! - **native** (`laterite-py`): a DuckDB relation's Arrow C-stream capsule
//!   → `PyTable` → `(batches, schema)` → here;
//! - **wasm** (`laterite-ags4-wasm`): an Arrow IPC stream → `StreamReader` →
//!   `(batches, schema)` → here.
//!
//! Living in `laterite-ags4-emit` keeps the type→[`Cell`] mapping in one place, so the
//! two hosts can't drift. Symmetric with the read path's shared *builder*
//! (`laterite-ags4-types::arrow_cols`, `Value`→Arrow). The cell stopped being a
//! `serde_json::Value` in #790 (`ags-wiki/design/dec-emit-cell-representation.md`).

use arrow::array::{
    Array, BooleanArray, Float32Array, Float64Array, Int8Array, Int16Array, Int32Array, Int64Array,
    LargeStringArray, StringArray, UInt8Array, UInt16Array, UInt32Array, UInt64Array,
};
use std::collections::HashMap;

use arrow::datatypes::{DataType, Schema};
use arrow::record_batch::RecordBatch;
use arrow::util::display::{ArrayFormatter, FormatOptions};
use laterite_ags4_types::{Cell, dt_to_unit_precision};
use laterite_ags4_validator::{DictVersion, Dictionary};

use crate::GroupInput;

/// One cell of an Arrow column → the [`Cell`] the orchestrator formats.
/// Typed numerics/bools become `Int`/`Float`/`Bool` (so `ags4_str` renders
/// them canonically); strings stay `Text` (emitted verbatim, the validity
/// mode owns canonicalisation); temporal/decimal/other types fall back to
/// Arrow's own canonical display string (e.g. `2023-02-22T10:24:00`).
///
/// A temporal cell lands in that last arm as a **string**, which the
/// orchestrator then emits verbatim — `ags4_str`'s DT handling is unreachable
/// from here, whatever its own doc comment implies. That is why a DT column
/// needs its declared precision applied at THIS layer, where the value is
/// still known to have come from a typed column rather than from the caller's
/// keyboard: see [`group_from_arrow_with_meta_at_edition`] (#695).
pub fn cell_value(array: &dyn Array, row: usize) -> Cell {
    if array.is_null(row) {
        return Cell::Null;
    }
    macro_rules! num {
        ($ty:ty) => {
            Cell::Int(i64::from(
                array.as_any().downcast_ref::<$ty>().unwrap().value(row),
            ))
        };
    }
    match array.data_type() {
        DataType::Utf8 => Cell::Text(
            array
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap()
                .value(row)
                .to_string(),
        ),
        DataType::LargeUtf8 => Cell::Text(
            array
                .as_any()
                .downcast_ref::<LargeStringArray>()
                .unwrap()
                .value(row)
                .to_string(),
        ),
        DataType::Boolean => Cell::Bool(
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
        // Above i64: fall to f64, as `Cell`'s deserialiser does for the same
        // shape — no AGS heading holds a 19-digit count.
        DataType::UInt64 => {
            let v = array
                .as_any()
                .downcast_ref::<UInt64Array>()
                .unwrap()
                .value(row);
            #[allow(clippy::cast_precision_loss)]
            i64::try_from(v).map_or(Cell::from(v as f64), Cell::Int)
        }
        // Both float arms go through `Cell::from`, which nulls a non-finite —
        // the behaviour `Value::from(f64)` gave these cells before #790 (a
        // NaN measurement emits as blank, not as the text `NaN`).
        DataType::Float32 => Cell::from(f64::from(
            array
                .as_any()
                .downcast_ref::<Float32Array>()
                .unwrap()
                .value(row),
        )),
        DataType::Float64 => Cell::from(
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
            Ok(fmt) => Cell::Text(fmt.value(row).to_string()),
            Err(_) => Cell::Null,
        },
    }
}

/// Is this column a typed temporal one — a value the caller handed us as an
/// instant, whose string form is OUR choice rather than theirs?
///
/// That distinction is the whole basis for rendering it to the heading's
/// declared precision: reformatting a caller's own string would take
/// canonicalisation away from the validity mode, but a temporal column has no
/// caller-authored string to preserve.
fn is_temporal(dt: &DataType) -> bool {
    matches!(
        dt,
        DataType::Date32
            | DataType::Date64
            | DataType::Timestamp(_, _)
            | DataType::Time32(_)
            | DataType::Time64(_)
    )
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
    group_from_arrow_with_meta_at_edition(code, schema, batches, units, types, None)
}

/// Like [`group_from_arrow_with_meta`], but rendering each **typed temporal**
/// column at the precision its heading's declared UNIT asks for.
///
/// `edition: None` is exactly [`group_from_arrow_with_meta`] — Arrow's own
/// display string, whatever the heading declares. Existing callers keep that
/// behaviour; this entry point is additive.
///
/// # Why this layer
///
/// An AGS4 `DT` column declares its precision in its UNIT, and Rule 8 judges
/// the cell against it. A typed instant carries no such precision: a date-only
/// cell read back from disk is a midnight timestamp, and Arrow renders that
/// `2021-08-09T00:00:00`, which fails the `yyyy-mm-dd` unit its own heading
/// declares. The orchestrator cannot fix that downstream — it emits strings
/// verbatim so the validity *mode* owns canonicalisation — so the precision
/// has to be applied here, at the point where the value is still known to be
/// a typed instant rather than a string the caller wrote (#695).
///
/// The rendering is refused whenever it would lose information (a real time
/// under a date-only unit), so a genuine mismatch between the data and its
/// heading still reaches the caller as a Rule 8 finding instead of being
/// silently trimmed away.
///
/// # Scope
///
/// Only headings in the **standard** dictionary carry a declared UNIT here.
/// A heading defined by the file's own `DICT` group is not visible at this
/// layer, so its temporal columns keep Arrow's rendering — the pre-#695
/// behaviour — rather than being guessed at.
#[must_use]
pub fn group_from_arrow_with_meta_at_edition<S: std::hash::BuildHasher>(
    code: String,
    schema: &Schema,
    batches: &[RecordBatch],
    units: Option<&HashMap<String, String, S>>,
    types: Option<&HashMap<String, String, S>>,
    edition: Option<DictVersion>,
) -> GroupInput {
    let headings: Vec<String> = schema.fields().iter().map(|f| f.name().clone()).collect();

    // Per column: the declared UNIT to render against, or None to leave the
    // cell as Arrow rendered it. Resolved once, not per row.
    let dict = edition.map(Dictionary::bundled);
    let dt_units: Vec<Option<String>> = schema
        .fields()
        .iter()
        .map(|f| {
            let d = dict.as_ref()?;
            if !is_temporal(f.data_type()) {
                return None;
            }
            let unit = d.heading(&code, f.name()).map(|h| h.unit.to_string())?;
            (!unit.trim().is_empty()).then_some(unit)
        })
        .collect();

    let mut rows: Vec<Vec<Cell>> = Vec::new();
    for batch in batches {
        let ncols = batch.num_columns();
        for r in 0..batch.num_rows() {
            let mut row = Vec::with_capacity(ncols);
            for c in 0..ncols {
                let v = cell_value(batch.column(c).as_ref(), r);
                row.push(match (&v, dt_units.get(c).and_then(Option::as_deref)) {
                    (Cell::Text(s), Some(unit)) => {
                        dt_to_unit_precision(s, unit).map_or(v, Cell::Text)
                    }
                    _ => v,
                });
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

    /// Build a one-row group from a single named column and read the cell back.
    fn cell_for(code: &str, heading: &str, col: ArrayRef, edition: Option<DictVersion>) -> Cell {
        let schema = Schema::new(vec![Field::new(heading, col.data_type().clone(), true)]);
        let batch = RecordBatch::try_new(Arc::new(schema.clone()), vec![col]).expect("valid batch");
        let g = group_from_arrow_with_meta_at_edition::<std::collections::hash_map::RandomState>(
            code.to_string(),
            &schema,
            std::slice::from_ref(&batch),
            None,
            None,
            edition,
        );
        g.rows[0][0].clone()
    }

    /// 2021-08-09T00:00:00 UTC — a date-only cell as `read()` hands it back.
    const MIDNIGHT_MS: i64 = 1_628_467_200_000;
    /// 2021-08-09T14:30:00 UTC — a genuine time of day.
    const AFTERNOON_MS: i64 = 1_628_519_400_000;

    /// #695. A typed temporal column is rendered at the precision its heading's
    /// declared UNIT asks for — because Arrow's canonical form fails the very
    /// Rule 8 the heading declares, and the orchestrator downstream emits
    /// strings verbatim, so this is the last layer that can act.
    #[test]
    fn a_temporal_column_is_rendered_at_its_headings_declared_precision() {
        // TRAN_DATE declares `yyyy-mm-dd` — one of 40 DT headings that do.
        assert_eq!(
            cell_for(
                "TRAN",
                "TRAN_DATE",
                Arc::new(TimestampMillisecondArray::from(vec![MIDNIGHT_MS])),
                Some(DictVersion::V4_2),
            ),
            Cell::from("2021-08-09"),
            "midnight under a date-only unit should render date-only"
        );
        // MOND_DTIM declares `yyyy-mm-ddThh:mm:ss`. The SAME instant must keep
        // its time here: truncating would break a file that is clean today.
        assert_eq!(
            cell_for(
                "MOND",
                "MOND_DTIM",
                Arc::new(TimestampMillisecondArray::from(vec![MIDNIGHT_MS])),
                Some(DictVersion::V4_2),
            ),
            Cell::from("2021-08-09T00:00:00"),
            "the declared precision, not the value's zeros, decides"
        );
        // A real time under a date-only unit is a genuine mismatch between the
        // data and its heading: left alone, so Rule 8 reports it rather than
        // this layer silently discarding the afternoon.
        assert_eq!(
            cell_for(
                "TRAN",
                "TRAN_DATE",
                Arc::new(TimestampMillisecondArray::from(vec![AFTERNOON_MS])),
                Some(DictVersion::V4_2),
            ),
            Cell::from("2021-08-09T14:30:00"),
            "a lossy render must be refused, not applied"
        );
    }

    /// The behaviour is opt-in at the call site: `edition: None` is exactly the
    /// pre-#695 rendering, so an existing caller of `group_from_arrow` sees no
    /// change. Same for a heading the standard dictionary does not carry — a
    /// DICT-defined one — whose declared UNIT is not visible at this layer.
    #[test]
    fn without_an_edition_or_a_dictionary_heading_arrow_s_own_rendering_stands() {
        assert_eq!(
            cell_for(
                "TRAN",
                "TRAN_DATE",
                Arc::new(TimestampMillisecondArray::from(vec![MIDNIGHT_MS])),
                None,
            ),
            Cell::from("2021-08-09T00:00:00"),
            "no edition: unchanged from before #695"
        );
        assert_eq!(
            cell_for(
                "ZZZZ",
                "ZZZZ_WHEN",
                Arc::new(TimestampMillisecondArray::from(vec![MIDNIGHT_MS])),
                Some(DictVersion::V4_2),
            ),
            Cell::from("2021-08-09T00:00:00"),
            "a heading outside the standard dictionary declares no precision here"
        );
    }

    /// A STRING column is the caller's own text, even under a DT heading, so it
    /// is passed through untouched — the "strings verbatim, the validity mode
    /// owns canonicalisation" contract this fix was designed not to spend.
    #[test]
    fn a_string_column_under_a_dt_heading_is_never_reformatted() {
        assert_eq!(
            cell_for(
                "TRAN",
                "TRAN_DATE",
                Arc::new(StringArray::from(vec!["2021-08-09T00:00:00"])),
                Some(DictVersion::V4_2),
            ),
            Cell::from("2021-08-09T00:00:00"),
            "a caller-written string stays the caller's"
        );
    }

    fn one(a: &dyn Array) -> Cell {
        cell_value(a, 0)
    }

    /// Every arm the `match` names, pinned to the [`Cell`] variant the emit
    /// orchestrator expects. `ags4_str` renders numbers and bools differently
    /// from strings, so an arm returning the wrong *variant* changes the output
    /// bytes even when the text looks right.
    #[test]
    fn typed_arms_map_to_their_cell_variant() {
        assert_eq!(one(&StringArray::from(vec!["BH01"])), Cell::from("BH01"));
        assert_eq!(
            one(&LargeStringArray::from(vec!["BH01"])),
            Cell::from("BH01")
        );
        assert_eq!(one(&BooleanArray::from(vec![true])), Cell::Bool(true));
        assert_eq!(one(&Int8Array::from(vec![-8i8])), Cell::from(-8));
        assert_eq!(one(&Int16Array::from(vec![-16i16])), Cell::from(-16));
        assert_eq!(one(&Int32Array::from(vec![-32i32])), Cell::from(-32));
        assert_eq!(one(&Int64Array::from(vec![-64i64])), Cell::from(-64));
        assert_eq!(one(&UInt8Array::from(vec![8u8])), Cell::from(8));
        assert_eq!(one(&UInt16Array::from(vec![16u16])), Cell::from(16));
        assert_eq!(one(&UInt32Array::from(vec![32u32])), Cell::from(32));
        assert_eq!(one(&UInt64Array::from(vec![64u64])), Cell::from(64));
        assert_eq!(one(&Float64Array::from(vec![1.5f64])), Cell::from(1.5));
        // f32 widens to f64 — pinned because a naive `Cell::from(f32)` would
        // serialise 1.5f32 as 1.5000000596046448.
        assert_eq!(one(&Float32Array::from(vec![1.5f32])), Cell::from(1.5));
    }

    /// A null cell is `Cell::Null` for EVERY type, checked before the match —
    /// so the `unwrap()`s on the downcasts are only ever reached for a valid row.
    #[test]
    fn nulls_short_circuit_before_the_downcast() {
        assert_eq!(one(&StringArray::from(vec![None::<&str>])), Cell::Null);
        assert_eq!(one(&Int64Array::from(vec![None::<i64>])), Cell::Null);
        assert_eq!(one(&BooleanArray::from(vec![None::<bool>])), Cell::Null);
        assert_eq!(one(&Date32Array::from(vec![None::<i32>])), Cell::Null);
    }

    /// The fallback arm. **Every `DT` column on the emit path lands here**, so
    /// these exact strings are load-bearing: they are what `ags4_str` receives
    /// and what ends up in the file.
    #[test]
    fn temporal_and_decimal_fall_back_to_arrows_canonical_string() {
        assert_eq!(
            one(&Date32Array::from(vec![19738])),
            Cell::from("2024-01-16")
        );
        assert_eq!(
            one(&TimestampMillisecondArray::from(vec![1_677_064_000_000i64])),
            Cell::from("2023-02-22T11:06:40")
        );
        let dec = Decimal128Array::from(vec![123_456i128])
            .with_precision_and_scale(10, 2)
            .expect("valid precision/scale");
        assert_eq!(one(&dec), Cell::from("1234.56"));
    }

    /// The `Err(_) => Cell::Null` arm is **defensive, not live**. Probed across
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
            Cell::Null,
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
                vec![Cell::from("A"), Cell::from(1.0)],
                vec![Cell::from("B"), Cell::from(2.0)],
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
        assert_eq!(g.rows[1][0], Cell::from("B"));
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
