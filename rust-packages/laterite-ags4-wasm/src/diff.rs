//! `diff()` — the KEY-aware, type-aware revision comparison.
//!
//! The comparison itself lives in the shared `laterite-ags4-diff` leaf (PyO3 and
//! the CLI reuse it); this parses both sides, decides which edition's KEY
//! headings identify a row, and hands the result back. That decision is the
//! only real one here, and getting it wrong silently changes what counts as
//! "the same row".
use crate::boundary::{WasmOptions, decode_opts, to_js};
use crate::resolve::resolve_encoding;
use laterite_ags4_parse::parse_bytes;
use laterite_ags4_validator::{
    ValidatorError, dict::Dictionary, resolve_dict_version, tran_ags_of,
};
use wasm_bindgen::prelude::*;

// The `diff` result. The shapes are `laterite-ags4-diff`'s, not this crate's —
// but publishing them here is still right: this is the door a JS caller comes
// through, and the alternative (what the web app did) is every consumer keeping
// its own copy. `ts_interfaces_match_the_serde_structs` binds these to the leaf's
// real structs, so "owned elsewhere" does not mean "unchecked here".
ts_section! {
    #[cfg(feature = "diff")]
    TS_DIFF_RESULT,
    TS_DIFF_RESULT_SECTION,
    r#"
/** One changed cell of a row matched on both sides. */
export interface CellDelta {
  heading: string;
  /** The AGS TYPE code the two cells were compared AS — a numeric compare is
   *  value-wise, so `"1.50"` and `"1.5"` are equal under `2DP` but differ as
   *  raw text. */
  type: string;
  /** Raw value on each side; `null` when that side's row is shorter than the
   *  heading list. */
  a: string | null;
  b: string | null;
}

/** One row's verdict. */
export interface RowDelta {
  kind: "added" | "removed" | "changed";
  /** The KEY values identifying the row — or the whole-row tuple when the
   *  group has no dictionary KEY headings (see `GroupDelta.keyed`). */
  key: string[];
  line_a: number | null;
  line_b: number | null;
  /** Populated only for `kind === "changed"`. */
  cells: CellDelta[];
}

/** One group's change summary. */
export interface GroupDelta {
  code: string;
  /** TRUE totals, independent of any `maxRowsPerGroup` cap — so `rows.length`
   *  may be smaller than `added + removed + changed`. */
  added: number;
  removed: number;
  changed: number;
  /** Structural change: headings present on only one side. */
  headings_added: string[];
  headings_removed: string[];
  /** `false` ⇒ rows were matched on the whole-row tuple because the dictionary
   *  gave this group no KEY headings. Matching is weaker; a row that changed
   *  in every cell reads as one removal plus one addition. */
  keyed: boolean;
  key_headings: string[];
  rows: RowDelta[];
}

/** The `diff` result: a KEY-aware, type-aware comparison of two files. */
export interface RevisionDelta {
  /** Groups with at least one row or heading change, in `b`'s file order,
   *  then the groups only `a` had. */
  groups: GroupDelta[];
  groups_added: string[];
  groups_removed: string[];
  total_added: number;
  total_removed: number;
  total_changed: number;
}
"#
}

#[cfg(feature = "diff")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "RevisionDelta")]
    pub type RevisionDeltaJs;
}

/// `diff`'s named options. `encoding`, not `encodingLabel` — see [`MergeOptions`].
#[cfg(feature = "diff")]
#[cfg_attr(test, derive(serde::Serialize))]
#[derive(serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct DiffOptions {
    encoding: Option<String>,
    max_rows_per_group: Option<u32>,
}

#[cfg(feature = "diff")]
impl WasmOptions for DiffOptions {
    const KEYS: &'static [&'static str] = &["encoding", "maxRowsPerGroup"];
    const WHAT: &'static str = "diff options";
}

#[cfg(feature = "diff")]
#[wasm_bindgen(typescript_custom_section)]
const TS_DIFF_OPTIONS: &'static str = r#"
/** Named options for `diff`. */
export interface DiffOptions {
  /** `"utf-8"` (default) or `"windows-1252"`, applied to BOTH inputs. */
  encoding?: "utf-8" | "windows-1252";
  /** Cap how many per-row deltas each group SERIALISES. The
   *  `added`/`removed`/`changed` counts stay true totals either way, so a cap
   *  bounds the payload without lying about the size of the change. Omit for
   *  everything. */
  maxRowsPerGroup?: number;
}
"#;

#[cfg(feature = "diff")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "DiffOptions")]
    pub type DiffOptionsJs;
}

/// Compare two AGS4 files.
///
/// * `opts` — a `DiffOptions` object; every field optional, so `diff(a, b)`
///   is a complete call. An unrecognised key is refused by name.
///
/// Behind the `diff` feature (#330).
#[cfg(feature = "diff")]
#[wasm_bindgen]
pub fn diff(a: &[u8], b: &[u8], opts: Option<DiffOptionsJs>) -> Result<RevisionDeltaJs, JsError> {
    console_error_panic_hook::set_once();
    let o: DiffOptions = decode_opts(opts.map(JsValue::from)).map_err(|m| JsError::new(&m))?;
    let delta = diff_core(a, b, &o).map_err(|m| JsError::new(&m))?;
    to_js(&delta)
}

/// The host-testable core of [`diff`]: decode both files, resolve the edition,
/// and run the shared comparison.
///
/// Which edition is a real decision and it is made here — KEY headings come
/// from the dictionary, and picking the wrong one silently changes what counts
/// as "the same row". It reads `b`'s `TRAN_AGS` (the newer file) and falls back
/// to the standard, and neither half of that could be reached from a test while
/// it sat behind `RevisionDeltaJs`.
#[cfg(feature = "diff")]
fn diff_core(
    a: &[u8],
    b: &[u8],
    o: &DiffOptions,
) -> Result<laterite_ags4_diff::RevisionDelta, String> {
    let encoding = resolve_encoding(o.encoding.as_deref())?;
    let pa = parse_bytes(a, encoding).map_err(|e| ValidatorError::from(e).to_string())?;
    let pb = parse_bytes(b, encoding).map_err(|e| ValidatorError::from(e).to_string())?;

    // KEY headings come from the dictionary; pick the edition from the
    // revision's TRAN_AGS (the "new" file), falling back to the standard.
    let dv = resolve_dict_version(None, tran_ags_of(&pb).as_deref())
        .map(|(dv, _)| dv)
        .unwrap_or(laterite_ags4_validator::dict::FALLBACK);
    let dict = Dictionary::bundled(dv);
    let cap = o.max_rows_per_group.map(|c| c as usize);

    // The KEY-aware/type-aware comparison itself lives in the shared
    // laterite-ags4-diff leaf (so PyO3 + the CLI reuse it); this only parses,
    // resolves the dictionary, and hands the result back.
    Ok(laterite_ags4_diff::diff_parsed(&pa, &pb, &dict, cap))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testdata::{LOCA_A, LOCA_B};

    // ---------------------------------------------------------------
    // diff_core
    // ---------------------------------------------------------------

    #[cfg(feature = "diff")]
    #[test]
    fn diffing_with_an_unknown_encoding_is_refused() {
        let o = DiffOptions {
            encoding: Some("klingon-1".into()),
            ..Default::default()
        };
        assert!(diff_core(LOCA_A, LOCA_B, &o).is_err());
    }

    #[cfg(feature = "diff")]
    #[test]
    fn an_unparseable_side_is_reported() {
        let o = DiffOptions::default();
        assert!(diff_core(b"junk", LOCA_B, &o).is_err());
        assert!(diff_core(LOCA_A, b"junk", &o).is_err());
    }

    #[cfg(feature = "diff")]
    #[test]
    fn a_file_diffed_against_itself_reports_nothing() {
        let d = diff_core(LOCA_A, LOCA_A, &DiffOptions::default()).expect("diffs");
        assert_eq!((d.total_added, d.total_removed, d.total_changed), (0, 0, 0));
        assert!(
            d.groups.is_empty(),
            "no group should be reported as changed"
        );
    }

    #[cfg(feature = "diff")]
    #[test]
    fn rows_are_matched_by_key_not_position() {
        // BH01 changed value, BH02 is gone, BH03 is new. Matching by position
        // instead would report two changes and no add/remove.
        let d = diff_core(LOCA_A, LOCA_B, &DiffOptions::default()).expect("diffs");
        assert_eq!(d.total_changed, 1, "BH01's coordinate changed");
        assert_eq!(d.total_removed, 1, "BH02 is gone");
        assert_eq!(d.total_added, 1, "BH03 is new");
        let loca = d.groups.iter().find(|g| g.code == "LOCA").expect("LOCA");
        assert!(loca.keyed, "LOCA has dictionary KEY headings");
        assert!(loca.key_headings.contains(&"LOCA_ID".to_string()));
    }

    #[cfg(feature = "diff")]
    #[test]
    fn a_row_cap_bounds_the_payload_without_lying_about_the_totals() {
        // The documented contract: `maxRowsPerGroup` caps what each group
        // SERIALISES, and the added/removed/changed counts stay true totals. A
        // cap that also truncated the counts would tell the user a three-row
        // change was a one-row change.
        let capped = DiffOptions {
            max_rows_per_group: Some(1),
            ..Default::default()
        };
        let d = diff_core(LOCA_A, LOCA_B, &capped).expect("diffs");
        let loca = d.groups.iter().find(|g| g.code == "LOCA").expect("LOCA");
        assert_eq!(loca.rows.len(), 1, "the cap must bound the serialised rows");
        assert_eq!(
            loca.added + loca.removed + loca.changed,
            3,
            "the totals must survive the cap"
        );
        assert_eq!(d.total_added + d.total_removed + d.total_changed, 3);
    }
}
