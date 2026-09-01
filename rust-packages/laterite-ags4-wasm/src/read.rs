//! `read()` — typed Arrow IPC (and typed JSON) for the DuckDB-wasm explorer.
//!
//! AGS4 isn't a format DuckDB reads natively. We parse it in Rust, build ONE
//! correctly-typed Arrow RecordBatch per group, and hand JS the IPC bytes;
//! DuckDB-wasm's `insertArrowFromIPCStream` ingests it as the final typed table
//! — no per-cell JS objects, no staging table, no TRY_CAST.
//!
//! Typing uses the SAME `laterite_ags4_types::{canonical_type, parse_value,
//! parse_datetime}` the native DuckDB conversion uses, off the file's own TYPE
//! row (`convert.rs` does the same), so the explorer casts a file IDENTICALLY to
//! the native conversion — parity by construction.
use crate::resolve::resolve_encoding;
use laterite_ags4_parse::{ParsedFile, parse_bytes};
use laterite_ags4_types::sql_type;
use laterite_ags4_validator::ValidatorError;
use serde::Serialize;
use wasm_bindgen::prelude::*;

/// A parsed AGS4 file held in wasm memory as the lightweight string
/// `ParsedFile`. Each group's typed rows are built lazily, one group per call,
/// and dropped once returned — so peak residency is one group, not the whole
/// file typed at once.
#[cfg_attr(
    feature = "arrow",
    doc = "\nTwo doors onto those rows: `arrow_ipc(code)` frames one group as an Arrow\n\
           IPC `RecordBatch` for duckdb-wasm, and `rows_json(code)` returns the\n\
           same values as JSON. Without the `arrow` feature only the second\n\
           exists (#330)."
)]
#[wasm_bindgen]
pub struct ParsedDataset {
    parsed: ParsedFile,
}

ts_section! {
    TS_GROUP_META,
    TS_GROUP_META_SECTION,
    r#"
/** Per-group schema: four PARALLEL arrays, one entry per heading, so
 *  `headings[i]` / `units[i]` / `types[i]` / `sql_types[i]` describe the same
 *  column. `meta()` returns `null` for a code the file does not contain. */
export interface GroupMeta {
  headings: string[];
  units: string[];
  /** AGS TYPE codes from the file's TYPE row (`"2DP"`, `"DT"`, `"ID"`, …). */
  types: string[];
  /** The DuckDB column type each heading lands as (`"DOUBLE"`, `"BIGINT"`,
   *  `"TIMESTAMP"`, `"VARCHAR"`, …) — what the table will report. */
  sql_types: string[];
}
"#
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "GroupMeta | null")]
    pub type GroupMetaJs;
}

/// Per-group schema for the UI: parallel arrays (one entry per heading).
#[derive(Serialize)]
pub(crate) struct GroupMeta {
    pub(crate) headings: Vec<String>,
    pub(crate) units: Vec<String>,
    /// AGS TYPE codes from the file's TYPE row (e.g. "2DP", "DT", "ID").
    pub(crate) types: Vec<String>,
    /// The DuckDB column type each heading lands as ("DOUBLE", "BIGINT",
    /// "TIMESTAMP", "VARCHAR", …) — what the table will report.
    pub(crate) sql_types: Vec<String>,
}

#[wasm_bindgen]
impl ParsedDataset {
    /// Group codes in file order (the order to load tables in).
    pub fn group_codes(&self) -> Vec<String> {
        self.parsed.group_order.clone()
    }

    /// `{headings, units, types, sql_types}` for one group, or `null` if
    /// the code isn't present.
    pub fn meta(&self, code: &str) -> GroupMetaJs {
        let Some(meta) = self.meta_core(code) else {
            return JsValue::NULL.unchecked_into();
        };
        let serializer = serde_wasm_bindgen::Serializer::json_compatible();
        meta.serialize(&serializer)
            .unwrap_or(JsValue::NULL)
            .unchecked_into()
    }

    /// One group's rows as an Arrow IPC **stream** (Uint8Array), columns
    /// already correctly typed. Built lazily here and dropped on return.
    ///
    /// `keys` (default `false`) prepends the two content-addressed key columns
    /// `_id`/`_parent_id` — the SAME UUIDv8s the wheel / Node / DuckDB
    /// extension produce (via the one shared keychain). Pass `true` when feeding
    /// duckdb-wasm so cross-group joins (`s._parent_id = l._id`) resolve; leave
    /// it off (the default) for a plain typed frame. A custom/passthrough group
    /// carries no keys, so `keys` is a no-op for it. (#303)
    ///
    /// `content_hash` (default `false`) appends a trailing `_content_hash`
    /// value fingerprint (SHA-256 over the typed, blank-normalised heading
    /// values) — the SAME hash Node/Python produce via the one shared
    /// keychain. Unlike `keys` this needs no registry entry, so a
    /// custom/passthrough group still gets a usable `_content_hash` even
    /// without an `_id`. (#448)
    ///
    /// Behind the `arrow` feature (#330). The Arrow machinery exists to feed
    /// duckdb-wasm; a caller without it wants [`Self::rows_json`], which is
    /// always present.
    #[cfg(feature = "arrow")]
    pub fn arrow_ipc(
        &self,
        code: &str,
        keys: Option<bool>,
        content_hash: Option<bool>,
    ) -> Result<Vec<u8>, JsError> {
        self.arrow_ipc_core(code, keys.unwrap_or(false), content_hash.unwrap_or(false))
            .map_err(|m| JsError::new(&m))
    }

    /// One group's rows as a JSON array-of-arrays.
    #[cfg_attr(
        feature = "arrow",
        doc = "\nThe **non-Arrow** door onto the same data `arrow_ipc` frames. \
               Without the `arrow` feature it is the only one (#330)."
    )]
    ///
    /// Values are born typed, through the SAME
    /// `laterite_ags4_types::parse_value` the native surfaces and the Arrow cast
    /// use, off the file's own TYPE row: a `2DP` heading arrives as a JSON
    /// number, a `DT` as a `"yyyy-mm-dd hh:mm:ss"` string, a blank or
    /// unparseable cell as `null`. So this is not "the strings, unparsed" — it
    /// is the same casting decision, serialised differently.
    ///
    /// A **string**, not a JS array. Two reasons, and the second is the real
    /// one: `JSON.parse` on a single string beats building one boxed `JsValue`
    /// per cell across the boundary — the cost the columnar build door exists
    /// to avoid on the way back — and `list_rules` already returns its JSON
    /// that way, so this is the crate's established shape rather than a new
    /// third convention.
    //
    // Both named precedents are ungated on purpose, and this note is `//` not
    // `///` for the same reason: doc comments here are COPIED into the
    // published `.d.ts`, so citing `build_ags4_ipc` or `MergeResult`
    // would point a reader at exports their build does not have — and a note
    // explaining that is housekeeping a consumer should never be shown.
    ///
    /// Rows are positional against [`Self::meta`]'s `headings`, and padded to
    /// its length with `null` when a DATA row is short — so the two zip.
    pub fn rows_json(&self, code: &str) -> Result<String, JsError> {
        self.rows_json_core(code).map_err(|m| JsError::new(&m))
    }
}

/// The host-testable half of [`ParsedDataset`]. Same methods, plain Rust types —
/// the `#[wasm_bindgen]` block above is now only defaults and error marshalling.
impl ParsedDataset {
    /// The core of [`ParsedDataset::meta`]: `None` for a code the file does not
    /// contain, which the caller renders as JS `null`.
    ///
    /// The parallel-array contract lives here — `headings[i]` / `units[i]` /
    /// `types[i]` / `sql_types[i]` describe the same column, and a file whose
    /// UNIT or TYPE row is SHORTER than its HEADING row (common, and legal
    /// enough to reach the explorer) must still produce four arrays of equal
    /// length. That padding — `""` for a missing unit, `"X"` for a missing type
    /// — is the reason this is not a one-liner, and it was unreachable from a
    /// test while it sat behind a `GroupMetaJs` return.
    fn meta_core(&self, code: &str) -> Option<GroupMeta> {
        let group = self.parsed.groups.get(code)?;
        let n = group.headings.len();
        let types: Vec<String> = (0..n)
            .map(|i| {
                group
                    .types
                    .get(i)
                    .cloned()
                    .unwrap_or_else(|| "X".to_string())
            })
            .collect();
        Some(GroupMeta {
            headings: group.headings.clone(),
            units: (0..n)
                .map(|i| group.units.get(i).cloned().unwrap_or_default())
                .collect(),
            sql_types: types.iter().map(|t| sql_type(t).to_string()).collect(),
            types,
        })
    }

    /// The core of [`ParsedDataset::rows_json`].
    ///
    /// The padding is the part worth testing, and it is the same decision
    /// [`Self::meta_core`] makes one row up: a missing TYPE reads as `"X"`
    /// (free text — the safe assumption, since nothing in the file said the
    /// column was numeric), and a short DATA row pads with `null` rather than
    /// coming back narrow. Both keep this positional against `meta`'s parallel
    /// arrays, which is the contract a consumer zips the two by.
    fn rows_json_core(&self, code: &str) -> Result<String, String> {
        let group = self
            .parsed
            .groups
            .get(code)
            .ok_or_else(|| format!("group {code:?} not in dataset"))?;
        let cols = group.headings.len();
        let rows: Vec<serde_json::Value> = (0..group.rows.len())
            .map(|row| {
                serde_json::Value::Array(
                    (0..cols)
                        .map(|col| {
                            let ags_type = group.types.get(col).map_or("X", String::as_str);
                            // `cell` yields None past the end of a short row,
                            // and parse_value maps None -> Null: the padding
                            // falls out of the shared cast rather than being a
                            // second rule written here.
                            laterite_ags4_types::parse_value(group.cell(col, row), ags_type)
                        })
                        .collect(),
                )
            })
            .collect();
        serde_json::to_string(&rows).map_err(|e| format!("rows json for {code}: {e}"))
    }

    /// The core of [`ParsedDataset::arrow_ipc`], with the two flags already
    /// defaulted.
    #[cfg(feature = "arrow")]
    fn arrow_ipc_core(
        &self,
        code: &str,
        keys: bool,
        content_hash: bool,
    ) -> Result<Vec<u8>, String> {
        let group = self
            .parsed
            .groups
            .get(code)
            .ok_or_else(|| format!("group {code:?} not in dataset"))?;

        // Typed columns + IPC framing both come from laterite-ags4-types now
        // (`ipc::build_group_ipc_synth` = the shared `arrow_cols` cast + StreamWriter,
        // `_id`/`_parent_id` col 0/1, `_content_hash` trailing) — the SAME
        // composition the napi host frames, so the browser, Node and Python type
        // a file byte-identically by construction. Framed here only for
        // duckdb-wasm.
        let reg = laterite_ags4_core::registry::registry();
        let ids = if keys && reg.get(code).is_some() {
            Some(laterite_ags4_core::keychain::group_row_ids(
                reg,
                code,
                &group.headings,
                group.rows.len(),
                |col, row| group.cell(col, row),
            ))
        } else if keys {
            // Rule 18 (#815): a file-declared group mints from its declared
            // KEY tuple + parent; declared keyless (or undeclared) stays
            // unkeyed. The DICT walk is paid only on this rare branch.
            let fd = laterite_ags4_core::effective_dict::FileDict::from_parsed(&self.parsed);
            let v = laterite_ags4_core::keychain::group_row_ids_effective(
                reg,
                &fd,
                code,
                &group.headings,
                group.rows.len(),
                |col, row| group.cell(col, row),
            );
            (!v.is_empty()).then_some(v)
        } else {
            None
        };
        let hashes = if content_hash {
            Some(laterite_ags4_core::keychain::group_content_hashes(
                code,
                &group.headings,
                &group.units,
                &group.types,
                group.rows.len(),
                |col, row| group.cell(col, row),
            ))
        } else {
            None
        };
        let buf = laterite_ags4_types::ipc::build_group_ipc_synth(
            &laterite_ags4_types::arrow_cols::SynthColumns {
                ids: ids.as_deref(),
                hashes: hashes.as_deref(),
            },
            &group.headings,
            &group.types,
            group.rows.len(),
            |col, row| group.cell(col, row),
        )
        .map_err(|e| format!("arrow ipc for {code}: {e}"))?;
        Ok(buf)
    }
}

/// Parse AGS4 bytes into a typed dataset for the explorer. Validation is
/// a separate concern (`validate`); this is permissive — it builds typed
/// columns for whatever parsed, so the explorer works even on a file with
/// findings. Only an unparseable-as-AGS4 input returns `Err`.
#[wasm_bindgen]
pub fn read(data: &[u8], encoding_label: Option<String>) -> Result<ParsedDataset, JsError> {
    console_error_panic_hook::set_once();
    read_core(data, encoding_label.as_deref()).map_err(|m| JsError::new(&m))
}

/// The host-testable core of [`read`]. The `ParseError` → `ValidatorError`
/// bridge is the point: an unparseable file must report the SAME text here as
/// it does from `validate`, and only this conversion makes that true.
fn read_core(data: &[u8], encoding_label: Option<&str>) -> Result<ParsedDataset, String> {
    let encoding = resolve_encoding(encoding_label)?;
    let parsed = parse_bytes(data, encoding).map_err(|e| ValidatorError::from(e).to_string())?;
    Ok(ParsedDataset { parsed })
}

#[cfg(test)]
mod tests {
    //! Parity-by-construction guard for `read()`'s typed-Arrow path.
    //!
    //! The shared column builder is the whole casting surface (the
    //! `#[wasm_bindgen]` wrappers only marshal it), and it casts through the
    //! SAME `laterite_ags4_types` entry points the native DuckDB conversion
    //! uses — so these assert the browser's types, and the native surface's, in
    //! one place.
    use super::*;
    use crate::build::{BuildOptions, build_ags4_core};
    use crate::testdata::{CLEAN, LOCA_A, err};

    // `Array` provides `is_null`/`len`; ArrayRef/DataType/TimeUnit assert the
    // shape of what the shared laterite-ags4-types builder hands back.
    //
    // The gates go on the individual items rather than the `mod`: the JSON row
    // door and `read_core` itself are ungated, so gating the whole block would
    // take them out of a slim test run along with Arrow.
    #[cfg(feature = "arrow")]
    use arrow::array::{
        Array, ArrayRef, BooleanArray, Float64Array, Int64Array, StringArray,
        TimestampMicrosecondArray,
    };
    #[cfg(feature = "arrow")]
    use arrow::datatypes::{DataType, TimeUnit};
    #[cfg(feature = "arrow")]
    use chrono::NaiveDate;

    // Exercises every canonical category: ID/X -> Utf8, 2DP -> Float64,
    // DT -> Timestamp (full datetime, date-only -> midnight, empty ->
    // null), 0DP -> Int64, YN -> Bool. BH03's blank coords/dates check
    // the null path; SAMP is a child group with a YN column.
    #[cfg(feature = "arrow")]
    const FIXTURE: &[u8] = b"\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\",\"LOCA_STAR\",\"LOCA_ENDD\",\"LOCA_REM\"\r\n\
\"UNIT\",\"\",\"m\",\"yyyy-mm-dd\",\"yyyy-mm-dd\",\"\"\r\n\
\"TYPE\",\"ID\",\"2DP\",\"DT\",\"DT\",\"X\"\r\n\
\"DATA\",\"BH01\",\"523145.67\",\"2020-08-18 09:30:00\",\"2020-08-19\",\"first\"\r\n\
\"DATA\",\"BH02\",\"523200.00\",\"2020-08-20\",\"\",\"second\"\r\n\
\"DATA\",\"BH03\",\"\",\"\",\"\",\"third\"\r\n\
\r\n\
\"GROUP\",\"GEOL\"\r\n\
\"HEADING\",\"LOCA_ID\",\"GEOL_STAT\"\r\n\
\"UNIT\",\"\",\"\"\r\n\
\"TYPE\",\"ID\",\"0DP\"\r\n\
\"DATA\",\"BH01\",\"1\"\r\n\
\"DATA\",\"BH01\",\"2\"\r\n\
\r\n\
\"GROUP\",\"SAMP\"\r\n\
\"HEADING\",\"LOCA_ID\",\"SAMP_DEPTH_OK\"\r\n\
\"UNIT\",\"\",\"\"\r\n\
\"TYPE\",\"ID\",\"YN\"\r\n\
\"DATA\",\"BH01\",\"Y\"\r\n\
\"DATA\",\"BH01\",\"N\"\r\n";

    #[cfg(feature = "arrow")]
    fn parsed() -> ParsedFile {
        parse_bytes(FIXTURE, encoding_rs::UTF_8).expect("fixture parses")
    }

    /// Build the typed column for `group`'s heading `name`, returning the
    /// array + its `DataType`. Routes through the shared laterite-ags4-types builder
    /// (the production path), feeding it this column's cells.
    #[cfg(feature = "arrow")]
    fn column(file: &ParsedFile, group: &str, name: &str) -> (ArrayRef, DataType) {
        let g = &file.groups[group];
        let col = g.headings.iter().position(|h| h == name).expect("heading");
        let ags_type = &g.types[col];
        laterite_ags4_types::arrow_cols::build_column(g.rows.len(), ags_type, |row| {
            g.cell(col, row)
        })
    }

    #[cfg(feature = "arrow")]
    fn micros(y: i32, m: u32, d: u32, hh: u32, mm: u32, ss: u32) -> i64 {
        NaiveDate::from_ymd_opt(y, m, d)
            .unwrap()
            .and_hms_opt(hh, mm, ss)
            .unwrap()
            .and_utc()
            .timestamp_micros()
    }

    #[cfg(feature = "arrow")]
    #[test]
    fn id_and_x_are_utf8() {
        let file = parsed();
        for name in ["LOCA_ID", "LOCA_REM"] {
            let (arr, dt) = column(&file, "LOCA", name);
            assert_eq!(dt, DataType::Utf8, "{name}");
            assert!(arr.as_any().is::<StringArray>());
        }
        let (rem, _) = column(&file, "LOCA", "LOCA_REM");
        let rem = rem.as_any().downcast_ref::<StringArray>().unwrap();
        assert_eq!(rem.value(0), "first");
        assert_eq!(rem.value(2), "third");
    }

    #[cfg(feature = "arrow")]
    #[test]
    fn two_dp_is_float64_with_nulls() {
        let file = parsed();
        let (arr, dt) = column(&file, "LOCA", "LOCA_NATE");
        assert_eq!(dt, DataType::Float64);
        let a = arr.as_any().downcast_ref::<Float64Array>().unwrap();
        assert_eq!(a.value(0), 523145.67);
        assert_eq!(a.value(1), 523200.00);
        assert!(a.is_null(2), "blank 2DP cell -> null");
    }

    #[cfg(feature = "arrow")]
    #[test]
    fn dt_is_timestamp_full_dateonly_and_null() {
        let file = parsed();
        let (star, dt) = column(&file, "LOCA", "LOCA_STAR");
        assert_eq!(dt, DataType::Timestamp(TimeUnit::Microsecond, None));
        let star = star
            .as_any()
            .downcast_ref::<TimestampMicrosecondArray>()
            .unwrap();
        // full datetime kept; date-only promoted to midnight; blank null.
        assert_eq!(star.value(0), micros(2020, 8, 18, 9, 30, 0));
        assert_eq!(star.value(1), micros(2020, 8, 20, 0, 0, 0));
        assert!(star.is_null(2));

        let (end, _) = column(&file, "LOCA", "LOCA_ENDD");
        let end = end
            .as_any()
            .downcast_ref::<TimestampMicrosecondArray>()
            .unwrap();
        assert_eq!(end.value(0), micros(2020, 8, 19, 0, 0, 0));
        assert!(end.is_null(1), "blank DT cell -> null");
    }

    #[cfg(feature = "arrow")]
    #[test]
    fn zero_dp_is_int64() {
        let file = parsed();
        let (arr, dt) = column(&file, "GEOL", "GEOL_STAT");
        assert_eq!(dt, DataType::Int64);
        let a = arr.as_any().downcast_ref::<Int64Array>().unwrap();
        assert_eq!(a.value(0), 1);
        assert_eq!(a.value(1), 2);
    }

    #[cfg(feature = "arrow")]
    #[test]
    fn yn_is_bool() {
        let file = parsed();
        let (arr, dt) = column(&file, "SAMP", "SAMP_DEPTH_OK");
        assert_eq!(dt, DataType::Boolean);
        let a = arr.as_any().downcast_ref::<BooleanArray>().unwrap();
        assert!(a.value(0));
        assert!(!a.value(1));
    }

    #[cfg(feature = "arrow")]
    #[test]
    fn ragged_short_row_yields_nulls_not_panic() {
        // A data row shorter than the heading count must null the missing
        // tail columns, never panic or misalign — the explorer has to
        // survive malformed real-world files.
        let bytes = b"\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n\
\"UNIT\",\"\",\"m\"\r\n\
\"TYPE\",\"ID\",\"2DP\"\r\n\
\"DATA\",\"BH01\"\r\n";
        let file = parse_bytes(bytes, encoding_rs::UTF_8).unwrap();
        let (arr, _) = column(&file, "LOCA", "LOCA_NATE");
        let a = arr.as_any().downcast_ref::<Float64Array>().unwrap();
        assert_eq!(a.len(), 1);
        assert!(a.is_null(0));
    }

    #[cfg(feature = "arrow")]
    #[test]
    fn arrow_ipc_keys_match_the_shared_golden_and_default_strips() {
        // SAME fixture + golden UUIDv8s as the Python (test_content_keys.py) and
        // Node (p3-content-keys.test.ts) tests — the ids come from the ONE shared
        // keychain, so matching here proves the wasm produces byte-identical keys
        // (a cross-surface parity check, ahead of Phase 6's full proof). (#303)
        const SRC: &[u8] = b"\"GROUP\",\"PROJ\"\r\n\"HEADING\",\"PROJ_ID\"\r\n\"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"P1\"\r\n\
\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\",\"PROJ_ID\"\r\n\"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"ID\"\r\n\"DATA\",\"BH1\",\"P1\"\r\n";
        let ds = ParsedDataset {
            parsed: parse_bytes(SRC, encoding_rs::UTF_8).expect("parses"),
        };

        // First-row string cell of `col` in an IPC stream, or None (missing col / null).
        let first = |ipc: &[u8], col: &str| -> Option<String> {
            let mut r =
                arrow::ipc::reader::StreamReader::try_new(std::io::Cursor::new(ipc.to_vec()), None)
                    .unwrap();
            let batch = r.next().unwrap().unwrap();
            let i = batch.schema().index_of(col).ok()?;
            let a = batch
                .column(i)
                .as_any()
                .downcast_ref::<StringArray>()
                .unwrap();
            if a.is_null(0) {
                None
            } else {
                Some(a.value(0).to_string())
            }
        };

        // keys=true → the golden UUIDv8s; child._parent_id links to parent._id;
        // a root group's _parent_id is NULL.
        // `unwrap_or_else(|_| panic!(…))`, NOT `.expect(…)`: the error is a
        // `JsError`, which does not implement `Debug`, so `.expect()` will not
        // compile. This read `.ok().expect(…)` for that reason and clippy's
        // `ok_expect` fires on it — but taking clippy's suggestion literally
        // breaks the build, so the escape is to drop the error explicitly.
        let proj = ds
            .arrow_ipc("PROJ", Some(true), None)
            .unwrap_or_else(|_| panic!("PROJ keyed"));
        let loca = ds
            .arrow_ipc("LOCA", Some(true), None)
            .unwrap_or_else(|_| panic!("LOCA keyed"));
        assert_eq!(
            first(&proj, "_id").as_deref(),
            Some("ac30a95d-e0ca-85f9-83c8-37a64af2762b"),
        );
        assert_eq!(
            first(&loca, "_id").as_deref(),
            Some("a7025a6f-d9b8-83b6-8fad-81c0c744edbc"),
        );
        assert_eq!(
            first(&loca, "_parent_id").as_deref(),
            Some("ac30a95d-e0ca-85f9-83c8-37a64af2762b"),
        );
        assert_eq!(first(&proj, "_parent_id"), None);

        // The default (no keys) strips: a plain frame carries no `_id` column.
        let plain = ds
            .arrow_ipc("PROJ", None, None)
            .unwrap_or_else(|_| panic!("PROJ plain"));
        assert!(
            first(&plain, "_id").is_none(),
            "default arrow_ipc must not carry _id",
        );
    }

    // ---------------------------------------------------------------
    // read_core + ParsedDataset
    // ---------------------------------------------------------------

    #[test]
    fn reading_with_an_unknown_encoding_is_refused() {
        let msg = err(read_core(CLEAN, Some("klingon-1")));
        assert!(!msg.is_empty(), "an unknown encoding must be reported");
    }

    #[test]
    fn an_unreadable_file_reports_the_validator_error_text() {
        // The ParseError -> ValidatorError bridge. It exists so an unparseable
        // file says the same thing here as it does from `validate`; without the
        // conversion the browser would show two different messages for one
        // problem depending on which door the user came through.
        let msg = err(read_core(b"nothing resembling ags4", None));
        let via_validate = ValidatorError::from(
            parse_bytes(b"nothing resembling ags4", encoding_rs::UTF_8)
                .expect_err("must not parse"),
        )
        .to_string();
        assert_eq!(
            msg, via_validate,
            "read and validate must agree on the text"
        );
    }

    #[test]
    fn group_codes_come_back_in_file_order() {
        // The explorer loads tables in this order, and PROJ must land before its
        // children — an alphabetical sort would put LOCA first and break the
        // foreign keys on insert.
        let ds = read_core(LOCA_A, None).expect("reads");
        assert_eq!(
            ds.group_codes(),
            vec!["PROJ".to_string(), "LOCA".to_string()]
        );
    }

    #[test]
    fn meta_is_none_for_a_group_the_file_lacks() {
        let ds = read_core(LOCA_A, None).expect("reads");
        assert!(ds.meta_core("SAMP").is_none());
    }

    #[test]
    fn meta_returns_four_arrays_of_equal_length() {
        let ds = read_core(LOCA_A, None).expect("reads");
        let m = ds.meta_core("LOCA").expect("LOCA is present");
        let n = m.headings.len();
        assert_eq!(n, 2);
        assert_eq!(m.units.len(), n);
        assert_eq!(m.types.len(), n);
        assert_eq!(m.sql_types.len(), n);
    }

    #[test]
    fn a_short_unit_or_type_row_is_padded_not_truncated() {
        // The parallel-array contract is what the UI indexes by, so a file whose
        // UNIT/TYPE rows are shorter than its HEADING row must still yield four
        // equal-length arrays. Truncating instead would silently mislabel every
        // column after the short one.
        let ragged: &[u8] = b"\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\",\"LOCA_REM\"\r\n\
\"UNIT\",\"\"\r\n\
\"TYPE\",\"ID\"\r\n\
\"DATA\",\"BH01\",\"1.00\",\"note\"\r\n";
        let ds = read_core(ragged, None).expect("reads");
        let m = ds.meta_core("LOCA").expect("LOCA is present");
        assert_eq!(m.headings.len(), 3);
        assert_eq!(m.units.len(), 3, "units must pad to the heading count");
        assert_eq!(m.types.len(), 3, "types must pad to the heading count");
        // A missing TYPE becomes "X" (free text), which is the safe assumption —
        // it casts to VARCHAR rather than guessing a numeric column.
        assert_eq!(m.types[2], "X");
        assert_eq!(m.units[2], "");
        assert_eq!(m.sql_types[2], sql_type("X"));
    }

    #[test]
    fn sql_types_are_derived_from_the_files_own_type_row() {
        // Parity by construction: the explorer must report the column types the
        // native DuckDB conversion would produce, and both read them off the
        // file's TYPE row through the same `sql_type`.
        let ds = read_core(LOCA_A, None).expect("reads");
        let m = ds.meta_core("LOCA").expect("LOCA is present");
        assert_eq!(m.types, vec!["ID".to_string(), "2DP".to_string()]);
        assert_eq!(
            m.sql_types,
            vec![sql_type("ID").to_string(), sql_type("2DP").to_string()]
        );
    }

    // --- rows_json: the non-Arrow read door (#330) ------------------------
    //
    // These are the tests that matter for the slim build: without `arrow` this
    // is the ONLY way data leaves `read`, so "born typed" has to be true here
    // and not merely inherited from the Arrow path's reputation.

    #[test]
    fn rows_json_names_the_group_it_could_not_find() {
        // Same contract as arrow_ipc_core's: name the code, don't return empty.
        let ds = read_core(LOCA_A, None).expect("reads");
        let msg = err(ds.rows_json_core("ZZZZ"));
        assert!(
            msg.contains("ZZZZ"),
            "the missing code must appear in the error, got: {msg}"
        );
    }

    #[test]
    fn rows_json_types_cells_off_the_files_own_type_row() {
        // The claim the slim package's docs make: a `2DP` heading arrives as a
        // JSON *number*, not the source string. If this ever regressed to
        // strings the demo would still render and every number would be text.
        let ds = read_core(LOCA_A, None).expect("reads");
        let rows: serde_json::Value =
            serde_json::from_str(&ds.rows_json_core("LOCA").expect("LOCA is present"))
                .expect("valid JSON");
        assert_eq!(
            rows,
            serde_json::json!([["BH01", 100.0], ["BH02", 200.0]]),
            "ID stays a string, 2DP becomes a number"
        );
    }

    #[test]
    fn rows_json_and_parse_value_cannot_disagree() {
        // Parity by construction, asserted rather than asserted-in-a-comment:
        // every cell must be exactly what `laterite_ags4_types::parse_value`
        // returns for that cell's declared type — the same function the native
        // DuckDB conversion and the Arrow cast both go through.
        let ds = read_core(LOCA_A, None).expect("reads");
        let rows: Vec<Vec<serde_json::Value>> =
            serde_json::from_str(&ds.rows_json_core("LOCA").expect("LOCA is present"))
                .expect("valid JSON");
        let group = ds.parsed.groups.get("LOCA").expect("LOCA is present");
        for (r, row) in rows.iter().enumerate() {
            for (c, cell) in row.iter().enumerate() {
                assert_eq!(
                    *cell,
                    laterite_ags4_types::parse_value(group.cell(c, r), &group.types[c]),
                    "cell ({r},{c}) drifted from parse_value"
                );
            }
        }
    }

    #[test]
    fn rows_json_pads_a_short_type_row_with_x_like_meta_does() {
        // `meta` pads a missing TYPE to "X"; this must agree, or the UI's
        // column header and the value under it would describe different types.
        // A ragged UNIT/TYPE row is common enough to reach the explorer.
        let ragged: &[u8] = b"\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\",\"LOCA_REM\"\r\n\
\"UNIT\",\"\"\r\n\
\"TYPE\",\"ID\"\r\n\
\"DATA\",\"BH01\",\"1.00\",\"note\"\r\n";
        let ds = read_core(ragged, None).expect("reads");
        let rows: serde_json::Value =
            serde_json::from_str(&ds.rows_json_core("LOCA").expect("LOCA is present"))
                .expect("valid JSON");
        // Under "X" (free text) the untyped columns stay strings — "1.00" must
        // NOT become 1.0, because nothing in the file said it was numeric.
        assert_eq!(rows, serde_json::json!([["BH01", "1.00", "note"]]));
    }

    #[test]
    fn rows_json_rows_are_as_wide_as_the_heading_row() {
        // The row arrays are positional against `meta().headings`, so a DATA
        // row shorter than the HEADING row must be padded (with nulls), never
        // returned short — a consumer zipping the two would otherwise shift
        // every value after the gap into the wrong column.
        let short: &[u8] = b"\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\",\"LOCA_REM\"\r\n\
\"UNIT\",\"\",\"m\",\"\"\r\n\
\"TYPE\",\"ID\",\"2DP\",\"X\"\r\n\
\"DATA\",\"BH01\"\r\n";
        let ds = read_core(short, None).expect("reads");
        let rows: serde_json::Value =
            serde_json::from_str(&ds.rows_json_core("LOCA").expect("LOCA is present"))
                .expect("valid JSON");
        assert_eq!(rows, serde_json::json!([["BH01", null, null]]));
    }

    #[test]
    fn rows_json_feeds_build_ags4_straight_back() {
        // The round trip the slim surface exists for (#334): read a file, edit
        // the tables, write it back. `meta()` gives headings/units/types and
        // this gives rows — together exactly `build_ags4`'s input shape. If
        // these two doors ever stopped composing, the demo would need a
        // hand-written adapter, which is the thing this asserts against.
        let ds = read_core(LOCA_A, None).expect("reads");
        let groups: Vec<serde_json::Value> = ds
            .group_codes()
            .iter()
            .map(|code| {
                let m = ds.meta_core(code).expect("a code the file listed");
                let rows: serde_json::Value =
                    serde_json::from_str(&ds.rows_json_core(code).expect("present"))
                        .expect("valid JSON");
                serde_json::json!({
                    "code": code, "headings": m.headings,
                    "units": m.units, "types": m.types, "rows": rows,
                })
            })
            .collect();
        let built = build_ags4_core(
            &serde_json::to_string(&groups).expect("plain data"),
            BuildOptions::default(),
        )
        .expect("builds");
        // Round-tripped through JSON and back out as AGS4, the values survive.
        assert!(built.text.contains("\"BH01\""), "got: {}", built.text);
        assert!(built.text.contains("\"100.00\""), "got: {}", built.text);
    }

    #[cfg(feature = "arrow")]
    #[test]
    fn arrow_ipc_names_the_group_it_could_not_find() {
        let ds = read_core(LOCA_A, None).expect("reads");
        let msg = err(ds.arrow_ipc_core("ZZZZ", false, false));
        assert!(
            msg.contains("ZZZZ"),
            "the missing code must appear in the error, got: {msg}"
        );
    }

    #[cfg(feature = "arrow")]
    #[test]
    fn keys_and_content_hash_are_off_by_default_and_add_columns_when_asked() {
        let ds = read_core(LOCA_A, None).expect("reads");
        let plain = ds.arrow_ipc_core("LOCA", false, false).expect("plain");
        let keyed = ds.arrow_ipc_core("LOCA", true, false).expect("keyed");
        let hashed = ds.arrow_ipc_core("LOCA", false, true).expect("hashed");

        // Asserted through the IPC bytes rather than a length comparison: the
        // column NAMES are the contract duckdb-wasm joins on.
        let names = |ipc: &[u8]| -> Vec<String> {
            let r = arrow::ipc::reader::StreamReader::try_new(std::io::Cursor::new(ipc), None)
                .expect("ipc reads");
            r.schema()
                .fields()
                .iter()
                .map(|f| f.name().clone())
                .collect()
        };
        let plain_names = names(&plain);
        assert!(
            !plain_names.iter().any(|n| n.starts_with('_')),
            "the default frame must carry no synthetic columns, got {plain_names:?}"
        );
        assert!(names(&keyed).contains(&"_id".to_string()));
        assert!(names(&hashed).contains(&"_content_hash".to_string()));
    }
}
