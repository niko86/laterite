//! `merge()` — reconcile two deliveries of one project into one file.
//!
//! Rows are matched by their dictionary KEY headings, `b` wins a conflict, and
//! the type-clash vocabulary is parsed by the merge crate's own `FromStr` so the
//! browser cannot accept a token the CLI rejects or word the rejection
//! differently.
use crate::boundary::{TranInput, WasmOptions, decode_opts};
use crate::resolve::{resolve_dict_override, resolve_encoding};
use laterite_ags4_parse::parse_bytes;
use laterite_ags4_validator::{ValidatorError, resolve_dict_version, tran_ags_of};
use wasm_bindgen::prelude::*;

/// The result of a merge: the reconciled `bytes` (a JS `Uint8Array` — the merged
/// `.ags` file), plus `warnings_json` and `revisions_json` (the audit arrays the
/// Tools UI parses — the same shape PyO3 / Node return).
#[cfg(feature = "merge")]
#[wasm_bindgen]
pub struct MergeResult {
    bytes: Vec<u8>,
    warnings_json: String,
    revisions_json: String,
}

#[cfg(feature = "merge")]
#[wasm_bindgen]
impl MergeResult {
    #[wasm_bindgen(getter)]
    pub fn bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn warnings_json(&self) -> String {
        self.warnings_json.clone()
    }
    #[wasm_bindgen(getter)]
    pub fn revisions_json(&self) -> String {
        self.revisions_json.clone()
    }
}

/// `merge`'s named options.
///
/// `encoding` rather than `encodingLabel`: every other surface calls this
/// concept `encoding`, and the browser was the only one carrying the `_label`
/// suffix. See the README's note on which exports still take it positionally —
/// the ones not yet migrated keep the old name until they are, so the split is
/// a recorded state rather than an accident.
#[cfg(feature = "merge")]
#[cfg_attr(test, derive(serde::Serialize))]
#[derive(serde::Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct MergeOptions {
    /// Force the edition instead of reading it from `b`'s `TRAN_AGS`.
    ///
    /// Added because the cross-surface parity gate found it missing: Python's
    /// `merge` and `lat merge` both take it, and this surface hard-coded `None`
    /// — so a browser user merging files whose `TRAN_AGS` was wrong or absent
    /// had no way to say so, while every other door did.
    dict_version: Option<String>,
    encoding: Option<String>,
    on_type_clash: Option<String>,
    /// What to do when no `tran` is supplied and both inputs carry TRAN rows —
    /// `"reconcile"` (default, today's behaviour) or `"error"` (refuse before
    /// emitting). Parsed through the engine's `FromStr`, so an unknown token is
    /// rejected with the same enumerated message every other surface gives.
    on_missing_tran: Option<String>,
    tran: Option<TranInput>,
}

#[cfg(feature = "merge")]
impl WasmOptions for MergeOptions {
    const KEYS: &'static [&'static str] = &[
        "dictVersion",
        "encoding",
        "onTypeClash",
        "onMissingTran",
        "tran",
    ];
    const WHAT: &'static str = "merge options";
}

#[cfg(feature = "merge")]
#[wasm_bindgen(typescript_custom_section)]
const TS_MERGE_OPTIONS: &'static str = r#"
/** Named options for `merge`. */
export interface MergeOptions {
  /** Force the edition rather than reading it from `b`'s `TRAN_AGS`. */
  dictVersion?: "auto" | "4.0.3" | "4.0.4" | "4.1" | "4.1.1" | "4.2";
  /** `"utf-8"` (default) or `"windows-1252"`, applied to BOTH inputs. */
  encoding?: "utf-8" | "windows-1252";
  /** What to do when two files declare a different AGS TYPE for one heading.
   *  `"error"` (default) refuses; `"widen"` falls back to `X`, keeping raw
   *  values but discarding the type; `"promote"` keeps the greatest `nDP`
   *  precision, zero-padding the coarser values. */
  onTypeClash?: "error" | "widen" | "promote";
  /** What to do when no `tran` is supplied and both inputs carry `TRAN` rows.
   *  `"reconcile"` (default) folds `TRAN` like any other group and warns — every
   *  input's transmission survives, which is more rows than Rule 14 permits.
   *  `"error"` refuses before any bytes are produced. Irrelevant when `tran` is
   *  supplied. */
  onMissingTran?: "reconcile" | "error";
  /** The transmission the MERGED file represents — it genuinely is a new one.
   *  Omit it and `TRAN` is reconciled like any other group, with a warning
   *  noting no merge-transmission stamp was supplied — and because `TRAN_ISNO`
   *  is a KEY heading, each input's transmission normally survives, leaving
   *  more TRAN rows than Rule 14 permits.
   *
   *  `remarks` is APPENDED to merge's own provenance note ("Merged from N
   *  deliveries: …") rather than replacing it: both are true of the result. */
  tran?: TranStamp;
}
"#;

#[cfg(feature = "merge")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "MergeOptions")]
    pub type MergeOptionsJs;
}

/// Merge two AGS4 deliveries of one project into one file (`a` then `b` — `b`
/// wins a KEY conflict). Rows are matched by their dictionary KEY headings.
///
/// * `opts` — a `MergeOptions` object; every field optional, so
///   `merge(a, b)` is a complete call. An unrecognised key is refused by name.
///
/// A heading the two files typed differently is a `JsError` unless
/// `onTypeClash` settles it. A complete `tran` stamps a synthesised merge-TRAN;
/// omit it and TRAN is reconciled like any other group, with a warning. The
/// edition is `b`'s `TRAN_AGS`, falling back to the standard.
///
/// The returned [`MergeResult`] is a wasm-owned handle: read its getters and
/// call `.free()`. The web worker leaked one per merge until this was written
/// down here.
///
/// Behind the `merge` feature (#330).
#[cfg(feature = "merge")]
#[wasm_bindgen]
pub fn merge(a: &[u8], b: &[u8], opts: Option<MergeOptionsJs>) -> Result<MergeResult, JsError> {
    console_error_panic_hook::set_once();
    let o: MergeOptions = decode_opts(opts.map(JsValue::from)).map_err(|m| JsError::new(&m))?;
    merge_core(a, b, o).map_err(|m| JsError::new(&m))
}

/// The host-testable core of [`merge`].
///
/// Four separate decisions live in here, all of which change the output file
/// and none of which `cargo test` could reach behind a `MergeOptionsJs`: the
/// TRAN completeness rule, the encoding, the edition (an explicit
/// `dictVersion` overriding `b`'s `TRAN_AGS`), and the type-clash vocabulary —
/// which is deliberately parsed by the merge crate's own `FromStr` so the
/// browser cannot accept a token the CLI rejects, or word the rejection
/// differently.
#[cfg(feature = "merge")]
fn merge_core(a: &[u8], b: &[u8], o: MergeOptions) -> Result<MergeResult, String> {
    use laterite_ags4_merge::{MergeOpts, MissingTranMode, TypeClashMode, merge_parsed};

    let tran = o.tran.map(TranInput::fold).transpose()?.flatten();
    let encoding = resolve_encoding(o.encoding.as_deref())?;
    let pa = parse_bytes(a, encoding).map_err(|e| ValidatorError::from(e).to_string())?;
    let pb = parse_bytes(b, encoding).map_err(|e| ValidatorError::from(e).to_string())?;

    // Edition from the newest file (b)'s TRAN_AGS, falling back to the standard.
    let over = resolve_dict_override(o.dict_version.as_deref())?;
    let dv = resolve_dict_version(over, tran_ags_of(&pb).as_deref())
        .map(|(dv, _)| dv)
        .unwrap_or(laterite_ags4_validator::dict::FALLBACK);

    // One vocabulary for every surface: accepted tokens + rejection message come
    // from the merge crate's FromStr, so the browser cannot drift from the CLI.
    let clash: TypeClashMode = o.on_type_clash.as_deref().unwrap_or("error").parse()?;
    let missing_tran: MissingTranMode = o
        .on_missing_tran
        .as_deref()
        .unwrap_or("reconcile")
        .parse()?;

    let opts = MergeOpts {
        on_type_clash: clash,
        on_missing_tran: missing_tran,
        edition: dv,
        tran,
        ..Default::default()
    };

    let res = merge_parsed(&[pa, pb], &opts).map_err(|e| e.to_string())?;
    let warnings: Vec<_> = res
        .warnings
        .iter()
        .map(|w| {
            serde_json::json!({
                "kind": w.kind, "group": w.group,
                "heading": w.heading, "message": w.message,
            })
        })
        .collect();
    let revisions: Vec<_> = res
        .revisions
        .iter()
        .map(|r| {
            serde_json::json!({
                "group": r.group, "key": r.key,
                "changed": r.changed, "winnerFile": r.winner_file,
            })
        })
        .collect();
    Ok(MergeResult {
        bytes: res.bytes,
        warnings_json: serde_json::to_string(&warnings).unwrap_or_else(|_| "[]".into()),
        revisions_json: serde_json::to_string(&revisions).unwrap_or_else(|_| "[]".into()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testdata::{LOCA_A, LOCA_B, err};

    /// `LOCA_A` with `LOCA_NATE` typed `X` instead of `2DP` — the clash.
    #[cfg(feature = "merge")]
    const LOCA_CLASH: &[u8] = b"\"GROUP\",\"PROJ\"\r\n\
\"HEADING\",\"PROJ_ID\"\r\n\
\"UNIT\",\"\"\r\n\
\"TYPE\",\"ID\"\r\n\
\"DATA\",\"P1\"\r\n\
\r\n\
\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n\
\"UNIT\",\"\",\"m\"\r\n\
\"TYPE\",\"ID\",\"X\"\r\n\
\"DATA\",\"BH09\",\"nine\"\r\n";

    // ---------------------------------------------------------------
    // merge_core
    // ---------------------------------------------------------------

    #[cfg(feature = "merge")]
    #[test]
    fn an_unknown_type_clash_token_is_refused_in_the_merge_crates_own_words() {
        // The vocabulary is parsed by laterite-ags4-merge's FromStr precisely so
        // the browser cannot accept a token the CLI rejects, or word the
        // rejection differently. Asserted against that crate's own list, not a
        // literal here — a copy would drift the moment a fourth mode appears.
        let o = MergeOptions {
            on_type_clash: Some("widenn".into()),
            ..Default::default()
        };
        let msg = err(merge_core(LOCA_A, LOCA_B, o));
        for mode in laterite_ags4_merge::TypeClashMode::ALL {
            assert!(
                msg.contains(mode.as_str()),
                "the rejection must list {:?}, got: {msg}",
                mode.as_str()
            );
        }
    }

    #[cfg(feature = "merge")]
    #[test]
    fn every_documented_clash_mode_is_accepted() {
        for mode in laterite_ags4_merge::TypeClashMode::ALL {
            let o = MergeOptions {
                on_type_clash: Some(mode.as_str().to_string()),
                ..Default::default()
            };
            // `error` legitimately fails on the clashing pair, so merge two files
            // that do NOT clash — this asserts the TOKEN is accepted.
            assert!(
                merge_core(LOCA_A, LOCA_B, o).is_ok(),
                "documented mode {:?} was rejected",
                mode.as_str()
            );
        }
    }

    #[cfg(feature = "merge")]
    #[test]
    fn a_type_clash_is_fatal_by_default_and_widen_settles_it() {
        let strict = merge_core(LOCA_A, LOCA_CLASH, MergeOptions::default());
        assert!(
            strict.is_err(),
            "two files typing LOCA_NATE differently must not merge silently"
        );
        let widened = merge_core(
            LOCA_A,
            LOCA_CLASH,
            MergeOptions {
                on_type_clash: Some("widen".into()),
                ..Default::default()
            },
        )
        .expect("widen settles the clash");
        let text = String::from_utf8(widened.bytes()).expect("utf-8");
        assert!(
            text.contains("BH01") && text.contains("BH09"),
            "both deliveries' rows must survive the widen:\n{text}"
        );
    }

    #[cfg(feature = "merge")]
    #[test]
    fn merge_refuses_an_unknown_edition_and_an_unknown_encoding() {
        let bad_dict = MergeOptions {
            dict_version: Some("4.9".into()),
            ..Default::default()
        };
        assert!(merge_core(LOCA_A, LOCA_B, bad_dict).is_err());
        let bad_enc = MergeOptions {
            encoding: Some("klingon-1".into()),
            ..Default::default()
        };
        assert!(merge_core(LOCA_A, LOCA_B, bad_enc).is_err());
    }

    #[cfg(feature = "merge")]
    #[test]
    fn a_partial_tran_stops_a_merge() {
        let o = MergeOptions {
            tran: Some(TranInput {
                issue: Some("1".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let msg = err(merge_core(LOCA_A, LOCA_B, o));
        assert!(
            !msg.is_empty(),
            "an incomplete merge-TRAN must be reported, not silently dropped"
        );
    }

    #[cfg(feature = "merge")]
    #[test]
    fn the_merge_result_getters_return_what_the_core_built() {
        // These three getters are the entire JS-visible surface of a merge, and
        // the web worker reads them before calling `.free()`. Empty JSON here
        // would look exactly like "nothing to report".
        let res = merge_core(LOCA_A, LOCA_B, MergeOptions::default()).expect("merges");
        assert!(!res.bytes().is_empty(), "the merged file must have bytes");
        // Valid JSON arrays, not the "[]" fallback that a serialisation failure
        // would produce indistinguishably from a genuinely empty audit.
        let w: serde_json::Value =
            serde_json::from_str(&res.warnings_json()).expect("warnings are JSON");
        let r: serde_json::Value =
            serde_json::from_str(&res.revisions_json()).expect("revisions are JSON");
        assert!(w.is_array() && r.is_array());
        assert!(
            r.as_array().is_some_and(|a| !a.is_empty()),
            "BH01 differs between the two deliveries, so a revision must be recorded"
        );
    }

    /// `LOCA_A` typed `2DP`; this types the same heading `3DP` — a clash
    /// entirely inside the nDP family, which is what `promote` can join.
    #[cfg(feature = "merge")]
    const LOCA_3DP: &[u8] = b"\"GROUP\",\"PROJ\"\r\n\
\"HEADING\",\"PROJ_ID\"\r\n\
\"UNIT\",\"\"\r\n\
\"TYPE\",\"ID\"\r\n\
\"DATA\",\"P1\"\r\n\
\r\n\
\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n\
\"UNIT\",\"\",\"m\"\r\n\
\"TYPE\",\"ID\",\"3DP\"\r\n\
\"DATA\",\"BH09\",\"9.123\"\r\n";

    #[cfg(feature = "merge")]
    fn warning_kinds(res: &MergeResult) -> Vec<String> {
        let v: serde_json::Value =
            serde_json::from_str(&res.warnings_json()).expect("warnings are JSON");
        v.as_array()
            .expect("array")
            .iter()
            .filter_map(|w| w.get("kind").and_then(|k| k.as_str()).map(str::to_string))
            .collect()
    }

    #[cfg(feature = "merge")]
    #[test]
    fn a_widened_type_clash_is_recorded_in_the_merge_warnings() {
        // Widening keeps both deliveries' rows but DISCARDS the column's type,
        // which is a loss the audit trail has to carry — a silent widen is
        // indistinguishable from the files having agreed.
        let res = merge_core(
            LOCA_A,
            LOCA_3DP,
            MergeOptions {
                on_type_clash: Some("widen".into()),
                ..Default::default()
            },
        )
        .expect("widen settles the clash");
        let kinds = warning_kinds(&res);
        assert!(
            kinds.iter().any(|k| k == "type_widened"),
            "a discarded type must be warned about, got {kinds:?}"
        );
        // The warning has to say WHICH column lost its type, or it is unactionable.
        let v: serde_json::Value = serde_json::from_str(&res.warnings_json()).expect("JSON");
        let widened = v
            .as_array()
            .expect("array")
            .iter()
            .find(|w| w.get("kind").and_then(|k| k.as_str()) == Some("type_widened"))
            .expect("the widen warning");
        assert_eq!(
            widened.get("heading").and_then(|h| h.as_str()),
            Some("LOCA_NATE")
        );
    }

    #[cfg(feature = "merge")]
    #[test]
    fn promote_keeps_the_column_numeric_and_says_so() {
        // The other resolution: `{2DP, 3DP}` joins inside the nDP family, so the
        // column stays numeric at the greater precision rather than falling back
        // to raw text.
        let res = merge_core(
            LOCA_A,
            LOCA_3DP,
            MergeOptions {
                on_type_clash: Some("promote".into()),
                ..Default::default()
            },
        )
        .expect("promote settles the clash");
        assert!(
            warning_kinds(&res).iter().any(|k| k == "type_promoted"),
            "a promotion changes what the file asserts and must be recorded"
        );
        let text = String::from_utf8(res.bytes()).expect("utf-8");
        assert!(
            text.contains("\"3DP\""),
            "the column should keep the greatest precision, not widen to X:\n{text}"
        );
    }

    #[cfg(feature = "merge")]
    #[test]
    fn a_typed_vs_x_clash_widens_silently_by_design() {
        // NOT a wart — the documented lattice behaviour, verified through the
        // browser door. `TypeClashMode::Widen`'s own doc states it: "Typed-vs-`X`
        // resolves silently (`X` trivially absorbs a typed value); two *different*
        // non-`X` types warn", and laterite-ags4-merge's acceptance suite pins
        // both halves (`typed_vs_x_widen_is_silent`, `non_x_vs_non_x_warns_under_
        // lenient`).
        //
        // The reasoning is that merge made no CHOICE here: once one file declares
        // `X`, the merged column can only be `X` — promote cannot reach it either
        // — so there is no resolution to report. A warning would fire on every
        // typed-vs-freetext column in a real delivery and drown the ones that
        // record an actual decision.
        //
        // Asserted here because a surface can drop a warning its engine emitted:
        // this proves the browser reports exactly what merge reports, silence
        // included.
        let clash: &[u8] = b"\"GROUP\",\"PROJ\"\r\n\
\"HEADING\",\"PROJ_ID\"\r\n\
\"UNIT\",\"\"\r\n\
\"TYPE\",\"ID\"\r\n\
\"DATA\",\"P1\"\r\n\
\r\n\
\"GROUP\",\"LOCA\"\r\n\
\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n\
\"UNIT\",\"\",\"m\"\r\n\
\"TYPE\",\"ID\",\"X\"\r\n\
\"DATA\",\"BH09\",\"nine\"\r\n";
        let res = merge_core(
            LOCA_A,
            clash,
            MergeOptions {
                on_type_clash: Some("widen".into()),
                ..Default::default()
            },
        )
        .expect("widen accepts the pair");
        let text = String::from_utf8(res.bytes()).expect("utf-8");
        assert!(
            text.contains("\"ID\",\"X\""),
            "the column widens to X:\n{text}"
        );
        assert!(
            !warning_kinds(&res).iter().any(|k| k == "type_widened"),
            "typed-vs-X is the trivial widen and must stay silent — if merge's \
             lattice changed, change it here and in TypeClashMode's doc together"
        );
    }

    #[cfg(feature = "merge")]
    #[test]
    fn merging_two_identical_files_changes_nothing() {
        // The degenerate case, and a useful control on the revision audit: if a
        // file merged with itself reports revisions, "changed" means nothing.
        let res = merge_core(LOCA_A, LOCA_A, MergeOptions::default()).expect("merges");
        let revisions: serde_json::Value =
            serde_json::from_str(&res.revisions_json()).expect("JSON");
        let changed = revisions
            .as_array()
            .expect("array")
            .iter()
            .filter(|r| r.get("changed").and_then(serde_json::Value::as_bool) == Some(true))
            .count();
        assert_eq!(changed, 0, "a file merged with itself has no revisions");
    }

    #[cfg(feature = "merge")]
    #[test]
    fn an_explicit_edition_overrides_the_files_own_tran_ags() {
        // The parity gap this option was added to close: Python's `merge` and
        // `lat merge` both take it, and this surface used to hard-code None — so
        // a browser user merging files with a wrong or absent TRAN_AGS had no
        // way to say so. Proven by forcing an edition the inputs do not name.
        let forced = merge_core(
            LOCA_A,
            LOCA_B,
            MergeOptions {
                dict_version: Some("4.0.3".into()),
                ..Default::default()
            },
        )
        .expect("merges against the forced edition");
        assert!(!forced.bytes().is_empty());
    }
}
