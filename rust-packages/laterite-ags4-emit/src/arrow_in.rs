//! The Arrow door: record batches → formatted AGS4 cells, streaming — no
//! row-major intermediate (behind the `arrow` feature).
//!
//! - **native** (`laterite-py`): a DuckDB relation's Arrow C-stream capsule
//!   → `PyTable` → `(batches, schema)` → [`ArrowGroup`] → here;
//! - **wasm** (`laterite-ags4-wasm`) / **node** (`laterite-node`): an Arrow
//!   IPC stream → `StreamReader` → `(batches, schema)` → [`ArrowGroup`] → here.
//!
//! Living in `laterite-ags4-emit` keeps the type→cell mapping in one place, so
//! the hosts can't drift. Symmetric with the read path's shared *builder*
//! (`laterite-ags4-types::arrow_cols`, `Value`→Arrow).
//!
//! This door used to materialise a full `Vec<Vec<Cell>>` transpose of the
//! input and hand it to the cell-rows door — pure overhead for a caller whose
//! batches are already resident, and (as a `serde_json::Value`) the single
//! largest slice of `build_ags4`'s peak. Now each cell formats straight off
//! its array into the final string, and the two doors meet at the formatted
//! [`OwnedGroup`] instead of at the input — the JOIN a differential test
//! holds (#790; `ags-wiki/design/dec-emit-cell-representation.md`).

use arrow::array::{
    Array, BooleanArray, Float32Array, Float64Array, Int8Array, Int16Array, Int32Array, Int64Array,
    LargeStringArray, StringArray, UInt8Array, UInt16Array, UInt32Array, UInt64Array,
};
use std::collections::HashMap;

use arrow::datatypes::{DataType, SchemaRef};
use arrow::record_batch::RecordBatch;
use arrow::util::display::{ArrayFormatter, FormatOptions};
use laterite_ags4_types::{Cell, ags4_str, dt_to_unit_precision};
use laterite_ags4_validator::{DictVersion, Dictionary};

use crate::emit::{EmitStream, OwnedGroup, resolved_meta_parts};
use crate::{EmitError, EmitOpts, EmitResult};

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
/// keyboard: see [`owned_group_from_arrow`] (#695).
///
/// `pub(crate)` since #790: the public transpose entry points are retired
/// (see the reliquary), but the type→cell MAPPING stays this one function —
/// [`owned_group_from_arrow`] streams through it cell by cell, so the doors
/// still share one conversion without sharing a materialised transpose.
pub(crate) fn cell_value(array: &dyn Array, row: usize) -> Cell {
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

/// One group's Arrow data, ready for [`emit_ags4_from_arrow`]: the code, the
/// schema (field names are the AGS headings — passed even when `batches` is
/// empty, so a 0-batch group still emits its section), the batches, and
/// optional per-heading UNIT/TYPE overrides as `{heading → value}` maps
/// (#294 F#9; a heading absent from the map is filled from the dictionary;
/// order-independent — keyed by name, not column position).
pub struct ArrowGroup {
    pub code: String,
    pub schema: SchemaRef,
    pub batches: Vec<RecordBatch>,
    pub units: Option<HashMap<String, String>>,
    pub types: Option<HashMap<String, String>>,
}

/// Build valid AGS4 bytes straight from Arrow record batches per `opts` —
/// the columnar door beside [`crate::emit_ags4`]'s cell-rows door.
///
/// Streaming on purpose: each cell formats directly off its array into the
/// final string, so the only per-group allocation is the formatted output
/// itself — there is no row-major input copy to peak alongside it, which for
/// the profiled build workload was the single largest slice of live-at-peak
/// bytes (#790; the records live on the issue). Groups convert one at a
/// time, so one group's batches drop before the next converts.
///
/// # DT precision (#695)
///
/// A typed temporal column is rendered at the precision its heading's
/// declared UNIT asks for — `opts.edition` names the dictionary consulted. A
/// typed instant carries no precision of its own: a date-only cell read back
/// from disk is a midnight timestamp, and Arrow renders that
/// `2021-08-09T00:00:00`, which fails the `yyyy-mm-dd` unit its own heading
/// declares. The orchestrator cannot fix that downstream — it emits strings
/// verbatim so the validity *mode* owns canonicalisation — so the precision
/// applies here, where the value is still known to be a typed instant rather
/// than a string the caller wrote. The rendering is refused whenever it
/// would lose information (a real time under a date-only unit), so a genuine
/// mismatch still reaches the caller as a Rule 8 finding rather than being
/// silently trimmed. Only headings in the **standard** dictionary carry a
/// declared UNIT here; a DICT-defined heading's temporal columns keep
/// Arrow's rendering rather than being guessed at.
pub fn emit_ags4_from_arrow(
    groups: Vec<ArrowGroup>,
    opts: &EmitOpts,
) -> Result<EmitResult, EmitError> {
    let dict = Dictionary::bundled(opts.edition);
    let mut stream = EmitStream::new(opts, &dict);
    for g in groups {
        // Consumed per iteration: a group's batches (our refs to them) drop
        // here, not at return — and the formatted `OwnedGroup` drops inside
        // `push` once its section is written, so no whole-file slab of
        // formatted cells ever exists (dec-emit-streamed-verdict).
        stream.push(owned_group_from_arrow(g, &dict))?;
    }
    stream.finish()
}

/// [`emit_ags4_from_arrow`] with NO validity verdict — the Arrow half of the
/// unchecked pair beside [`crate::emit_ags4_unchecked`] (#858).
///
/// The caller is choosing to ship unchecked bytes: nothing here confirms the
/// output satisfies any AGS4 rule, and nothing downstream will. Everything
/// up to the verdict is the judged door's — the same streaming conversion,
/// the same #695 DT-precision rendering (it lives in the door, not the
/// judge), the same fills and section order; a test pins the bytes equal to
/// the judged `Report` build's.
pub fn emit_ags4_from_arrow_unchecked(
    groups: Vec<ArrowGroup>,
    edition: DictVersion,
) -> Result<Vec<u8>, EmitError> {
    let opts = crate::emit::unchecked_opts(edition);
    let dict = Dictionary::bundled(edition);
    let mut stream = EmitStream::new(&opts, &dict);
    for g in groups {
        stream.push(owned_group_from_arrow(g, &dict))?;
    }
    stream.finish_unchecked()
}

/// One [`ArrowGroup`] → the formatted [`OwnedGroup`] — the Arrow door's half
/// of the two-door join (#790).
///
/// Cell semantics are `cell_value` + [`crate::emit::format_cell`]'s, fused:
/// a string-producing arm (Utf8, and the display-formatter fallback) goes out
/// verbatim — with the #695 DT-precision rendering applied to a temporal
/// column's fallback string — and a typed scalar formats through `ags4_str`
/// against the heading's resolved TYPE. The transient [`Cell`] for a typed
/// scalar lives on the stack; the only heap allocation per cell is the final
/// string itself.
fn owned_group_from_arrow(g: ArrowGroup, dict: &Dictionary) -> OwnedGroup {
    let headings: Vec<String> = g.schema.fields().iter().map(|f| f.name().clone()).collect();

    // Align a {heading → value} override map to the heading order; a heading
    // not in the map gets "" ("fill from the dictionary").
    let align = |m: Option<&HashMap<String, String>>| -> Option<Vec<String>> {
        m.map(|map| {
            headings
                .iter()
                .map(|h| map.get(h).cloned().unwrap_or_default())
                .collect()
        })
    };
    let unit_overrides = align(g.units.as_ref());
    let type_overrides = align(g.types.as_ref());
    let (units, types) = resolved_meta_parts(
        &g.code,
        &headings,
        unit_overrides.as_ref(),
        type_overrides.as_ref(),
        dict,
    );

    // Per column: the declared UNIT to render a temporal cell against, or
    // None to leave Arrow's rendering. Resolved once, not per row.
    let dt_units: Vec<Option<String>> = g
        .schema
        .fields()
        .iter()
        .map(|f| {
            if !is_temporal(f.data_type()) {
                return None;
            }
            let unit = dict
                .heading(&g.code, f.name())
                .map(|h| h.unit.to_string())?;
            (!unit.trim().is_empty()).then_some(unit)
        })
        .collect();

    let nrows: usize = g.batches.iter().map(RecordBatch::num_rows).sum();
    let mut rows: Vec<Vec<String>> = Vec::with_capacity(nrows);
    for batch in &g.batches {
        let ncols = batch.num_columns();
        for r in 0..batch.num_rows() {
            let mut row = Vec::with_capacity(ncols);
            for c in 0..ncols {
                let ags_type = types.get(c).map_or("X", String::as_str);
                let cell = cell_value(batch.column(c).as_ref(), r);
                row.push(match (cell, dt_units.get(c).and_then(Option::as_deref)) {
                    // A temporal fallback string, rendered at its declared
                    // precision when that is lossless (#695) — then verbatim,
                    // like every string.
                    (Cell::Text(s), Some(unit)) => dt_to_unit_precision(&s, unit).unwrap_or(s),
                    (Cell::Text(s), None) => s,
                    // Typed scalars: the canonical wire form. The Cell here
                    // is stack-only (no Text), so nothing was allocated to
                    // be thrown away.
                    (cell, _) => ags4_str(&cell, ags_type),
                });
            }
            rows.push(row);
        }
    }
    OwnedGroup {
        code: g.code,
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
    use arrow::datatypes::{Field, Schema, UnionFields};
    use laterite_ags4_validator::DictVersion;
    use std::sync::Arc;

    /// One [`ArrowGroup`] from loose parts — the shape every host builds.
    fn arrow_group(code: &str, schema: Schema, batches: Vec<RecordBatch>) -> ArrowGroup {
        ArrowGroup {
            code: code.to_string(),
            schema: Arc::new(schema),
            batches,
            units: None,
            types: None,
        }
    }

    /// Build a one-row group from a single named column, run it through the
    /// door's group conversion, and read the FORMATTED cell back. The old
    /// transpose returned a `Cell` here; the door returns wire strings, which
    /// is the honest layer to pin — it is what the file will carry.
    #[allow(clippy::needless_pass_by_value)] // test helper: owned reads clearer at call sites
    fn cell_for(code: &str, heading: &str, col: ArrayRef, edition: DictVersion) -> String {
        let schema = Schema::new(vec![Field::new(heading, col.data_type().clone(), true)]);
        let batch =
            RecordBatch::try_new(Arc::new(schema.clone()), vec![col.clone()]).expect("valid batch");
        let g = owned_group_from_arrow(
            arrow_group(code, schema, vec![batch]),
            &Dictionary::bundled(edition),
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
                DictVersion::V4_2,
            ),
            "2021-08-09",
            "midnight under a date-only unit should render date-only"
        );
        // MOND_DTIM declares `yyyy-mm-ddThh:mm:ss`. The SAME instant must keep
        // its time here: truncating would break a file that is clean today.
        assert_eq!(
            cell_for(
                "MOND",
                "MOND_DTIM",
                Arc::new(TimestampMillisecondArray::from(vec![MIDNIGHT_MS])),
                DictVersion::V4_2,
            ),
            "2021-08-09T00:00:00",
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
                DictVersion::V4_2,
            ),
            "2021-08-09T14:30:00",
            "a lossy render must be refused, not applied"
        );
    }

    /// A heading the standard dictionary does not carry — a DICT-defined one —
    /// declares no UNIT visible at this layer, so its temporal columns keep
    /// Arrow's own rendering rather than being guessed at. (The old transpose
    /// also took `edition: None` for the pre-#695 rendering; the door retired
    /// that option — every host was already passing its edition.)
    #[test]
    fn a_heading_outside_the_dictionary_keeps_arrows_own_rendering() {
        assert_eq!(
            cell_for(
                "ZZZZ",
                "ZZZZ_WHEN",
                Arc::new(TimestampMillisecondArray::from(vec![MIDNIGHT_MS])),
                DictVersion::V4_2,
            ),
            "2021-08-09T00:00:00",
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
                DictVersion::V4_2,
            ),
            "2021-08-09T00:00:00",
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

    /// The FALLBACK edition, spelled once for these tests: the door always
    /// resolves against a dictionary now, so the old "units/types left None"
    /// assertions become assertions about WHAT the dictionary filled.
    fn dict() -> Dictionary<'static> {
        Dictionary::bundled(DictVersion::V4_1_1)
    }

    #[test]
    fn headings_come_from_the_schema_and_rows_format_straight_to_wire_strings() {
        let g = owned_group_from_arrow(
            arrow_group(
                "LOCA",
                schema2(),
                vec![batch(vec!["A", "B"], vec![1.0, 2.0])],
            ),
            &dict(),
        );
        assert_eq!(g.code, "LOCA");
        assert_eq!(g.headings, ["LOCA_ID", "LOCA_GL"]);
        // Column-major in, row-major out — already formatted: LOCA_GL's
        // dictionary TYPE is 2DP, so the float is at wire precision here,
        // where the transpose used to carry `Cell::Float(1.0)` for the
        // orchestrator to format later. Same bytes, one fewer copy.
        assert_eq!(
            g.rows,
            vec![
                vec!["A".to_string(), "1.00".to_string()],
                vec!["B".to_string(), "2.00".to_string()]
            ]
        );
        assert_eq!(g.types, ["ID", "2DP"], "filled from the dictionary");
        assert_eq!(g.units, ["", "m"], "filled from the dictionary");
    }

    /// Batches concatenate in order — a multi-batch stream is one group, not one
    /// group per batch.
    #[test]
    fn multiple_batches_concatenate() {
        let g = owned_group_from_arrow(
            arrow_group(
                "LOCA",
                schema2(),
                vec![batch(vec!["A"], vec![1.0]), batch(vec!["B"], vec![2.0])],
            ),
            &dict(),
        );
        assert_eq!(g.rows.len(), 2);
        assert_eq!(g.rows[1][0], "B");
    }

    /// The reason `schema` rides in [`ArrowGroup`] beside the batches: a group
    /// whose stream carried no batches still has to emit its section with the
    /// right headings.
    #[test]
    fn zero_batches_still_yields_the_headings() {
        let g = owned_group_from_arrow(arrow_group("LOCA", schema2(), vec![]), &dict());
        assert_eq!(g.headings, ["LOCA_ID", "LOCA_GL"]);
        assert!(g.rows.is_empty());
    }

    /// Overrides align by heading NAME, not column position; a heading the map
    /// does not mention gets the dictionary's value — the same hybrid
    /// resolution as the cell-rows door, via the same `resolved_meta_parts`.
    #[test]
    fn meta_overrides_align_by_name_and_the_dictionary_fills_the_rest() {
        let units: HashMap<String, String> = [("LOCA_GL".to_string(), "ft".to_string())]
            .into_iter()
            .collect();
        let mut g = arrow_group("LOCA", schema2(), vec![]);
        g.units = Some(units);
        let g = owned_group_from_arrow(g, &dict());
        assert_eq!(
            g.units,
            ["", "ft"],
            "the named heading takes the override; LOCA_ID has no dict UNIT"
        );
        assert_eq!(
            g.types,
            ["ID", "2DP"],
            "None leaves the tier to the dictionary"
        );
    }

    /// THE JOIN TEST (#790): the same logical data through the cell-rows door
    /// ([`crate::emit_ags4`]) and the Arrow door must produce byte-identical
    /// output. `OwnedGroup` is where the two doors meet, and everything after
    /// it is one code path — so this equality is exactly the drift surface,
    /// and the whole reason the streaming rewrite was allowed to split the
    /// input handling in two.
    ///
    /// Report mode on purpose: it emits unmodified, so the bytes compared are
    /// the doors' own output, not the fixer's.
    #[test]
    fn the_two_doors_emit_identical_bytes_for_the_same_data() {
        let opts = EmitOpts {
            mode: crate::EmitMode::Report,
            ..EmitOpts::default()
        };

        let via_cells = crate::emit_ags4(
            &[crate::GroupInput {
                code: "LOCA".into(),
                headings: vec!["LOCA_ID".into(), "LOCA_GL".into()],
                units: None,
                types: None,
                rows: vec![
                    vec![Cell::from("BH01"), Cell::from(12.5)],
                    vec![Cell::from("BH02"), Cell::Null],
                ],
            }],
            &opts,
        )
        .expect("cell-rows door emits");

        let via_arrow = emit_ags4_from_arrow(
            vec![arrow_group(
                "LOCA",
                schema2(),
                vec![
                    RecordBatch::try_new(
                        Arc::new(schema2()),
                        vec![
                            Arc::new(StringArray::from(vec!["BH01", "BH02"])) as ArrayRef,
                            Arc::new(Float64Array::from(vec![Some(12.5), None])) as ArrayRef,
                        ],
                    )
                    .expect("batch"),
                ],
            )],
            &opts,
        )
        .expect("arrow door emits");

        assert_eq!(
            String::from_utf8_lossy(&via_cells.bytes),
            String::from_utf8_lossy(&via_arrow.bytes),
            "the doors drifted — they may only differ before the OwnedGroup join"
        );
    }

    /// #858: the Arrow door's unchecked variant returns exactly the judged
    /// `Report` build's bytes — including the #695 DT-precision rendering,
    /// which lives in the door conversion, upstream of the judge. The judged
    /// build must OBJECT to the fixture (a data-only TRAN, no PROJ or
    /// catalogs), or the identity proves nothing.
    #[test]
    fn unchecked_arrow_bytes_equal_the_judged_report_bytes() {
        let mk = || {
            let schema = Schema::new(vec![Field::new(
                "TRAN_DATE",
                arrow::datatypes::DataType::Timestamp(
                    arrow::datatypes::TimeUnit::Millisecond,
                    None,
                ),
                true,
            )]);
            let batch = RecordBatch::try_new(
                Arc::new(schema.clone()),
                vec![Arc::new(TimestampMillisecondArray::from(vec![MIDNIGHT_MS])) as ArrayRef],
            )
            .expect("batch");
            vec![arrow_group("TRAN", schema, vec![batch])]
        };
        let opts = EmitOpts {
            mode: crate::EmitMode::Report,
            edition: DictVersion::V4_2,
            ..EmitOpts::default()
        };

        let judged = emit_ags4_from_arrow(mk(), &opts).expect("judged door emits");
        assert!(
            judged.findings.values().flatten().count() > 0,
            "the fixture must draw findings, or the identity proves nothing"
        );
        let bytes =
            emit_ags4_from_arrow_unchecked(mk(), DictVersion::V4_2).expect("unchecked door emits");
        assert_eq!(
            bytes, judged.bytes,
            "unchecked bytes must be the judged Report bytes, DT rendering included"
        );
    }

    /// #858: the Arrow door's zero-group refusal matches its judged twin,
    /// worded identically — same guarantee `emit.rs` pins for the cell-rows
    /// pair; both finishers share `assemble`, where the refusal lives.
    #[test]
    fn unchecked_arrow_zero_group_build_fails_like_the_judged_door() {
        let opts = EmitOpts {
            mode: crate::EmitMode::Report,
            ..EmitOpts::default()
        };
        let Err(judged) = emit_ags4_from_arrow(vec![], &opts) else {
            panic!("a zero-group judged build must refuse");
        };
        let Err(unchecked) = emit_ags4_from_arrow_unchecked(vec![], opts.edition) else {
            panic!("a zero-group unchecked build must refuse");
        };
        assert_eq!(unchecked.to_string(), judged.to_string());
    }

    /// The one INTENDED divergence, pinned so the join test above can never be
    /// "fixed" into hiding it: a typed temporal column renders at its
    /// heading's declared UNIT precision (#695), while the same instant
    /// arriving as a caller string emits verbatim — a string is the caller's
    /// own text, and reformatting it would take canonicalisation away from
    /// the validity mode. Weakening either side of this pair is the failure
    /// mode the ADR warns about; both behaviours are load-bearing.
    #[test]
    fn the_doors_deliberately_diverge_on_typed_temporals() {
        let opts = EmitOpts {
            mode: crate::EmitMode::Report,
            edition: DictVersion::V4_2,
            ..EmitOpts::default()
        };

        // TRAN_DATE declares `yyyy-mm-dd`; MIDNIGHT_MS is that date at 00:00.
        let schema = Schema::new(vec![Field::new(
            "TRAN_DATE",
            arrow::datatypes::DataType::Timestamp(arrow::datatypes::TimeUnit::Millisecond, None),
            true,
        )]);
        let batch = RecordBatch::try_new(
            Arc::new(schema.clone()),
            vec![Arc::new(TimestampMillisecondArray::from(vec![MIDNIGHT_MS])) as ArrayRef],
        )
        .expect("batch");
        let via_arrow = emit_ags4_from_arrow(vec![arrow_group("TRAN", schema, vec![batch])], &opts)
            .expect("arrow door emits");

        let via_cells = crate::emit_ags4(
            &[crate::GroupInput {
                code: "TRAN".into(),
                headings: vec!["TRAN_DATE".into()],
                units: None,
                types: None,
                rows: vec![vec![Cell::from("2021-08-09T00:00:00")]],
            }],
            &opts,
        )
        .expect("cell-rows door emits");

        let arrow_text = String::from_utf8_lossy(&via_arrow.bytes).into_owned();
        let cells_text = String::from_utf8_lossy(&via_cells.bytes).into_owned();
        assert!(
            arrow_text.contains("\"2021-08-09\""),
            "typed instant renders at the declared date-only precision: {arrow_text}"
        );
        assert!(
            cells_text.contains("\"2021-08-09T00:00:00\""),
            "a caller's string emits verbatim: {cells_text}"
        );
    }
}
