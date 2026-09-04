//! POC acceptance tests for `merge_parsed`, all asserted against the actual
//! re-emitted bytes re-parsed via `parse_bytes` — proving the reconciliation
//! survives the real write path, not just an in-memory model.

use laterite_ags4_merge::{MergeError, MergeOpts, TranStamp, TypeClashMode, merge_parsed};
use laterite_ags4_parse::{DataRow, ParsedFile, ParsedGroup, parse_str};

fn load(name: &str) -> ParsedFile {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(&path).unwrap();
    parse_str(&text).unwrap()
}

fn stamp() -> TranStamp {
    TranStamp::new("3", "2024-03-01", "Merger", "Client", "Merged")
}

fn lenient() -> MergeOpts {
    MergeOpts {
        on_type_clash: TypeClashMode::Widen,
        tran: Some(stamp()),
        ..Default::default()
    }
}

fn row_by_key<'a>(g: &'a ParsedGroup, key_h: &str, key_v: &str) -> Option<&'a DataRow> {
    let ci = g.headings.iter().position(|h| h == key_h)?;
    g.rows.iter().find(|r| g.value_at(r, ci) == Some(key_v))
}

fn cell(g: &ParsedGroup, r: &DataRow, h: &str) -> Option<String> {
    let ci = g.headings.iter().position(|x| x == h)?;
    g.value_at(r, ci).map(str::to_string)
}

fn type_of(g: &ParsedGroup, h: &str) -> Option<String> {
    let ci = g.headings.iter().position(|x| x == h)?;
    g.types.get(ci).cloned()
}

/// Merge [a,b] leniently and re-parse the emitted bytes.
fn merged_ab() -> (ParsedFile, Vec<laterite_ags4_merge::MergeWarning>) {
    let (a, b) = (load("delivery_a.ags"), load("delivery_b.ags"));
    let res = merge_parsed(&[a, b], &lenient()).expect("lenient merge should succeed");
    let text = std::str::from_utf8(&res.bytes).expect("output is UTF-8");
    (parse_str(text).expect("output must re-parse"), res.warnings)
}

// --- 1. argument order wins on a revision ---------------------------------
#[test]
fn argument_order_wins_on_revision() {
    let (out, _) = merged_ab();
    let loca = &out.groups["LOCA"];
    let bh01 = row_by_key(loca, "LOCA_ID", "BH01").expect("BH01 present");
    // A had GL=10.00, B (later) has 11.50 → B wins.
    assert_eq!(cell(loca, bh01, "LOCA_GL").as_deref(), Some("11.50"));
}

// --- 2/3. recency cross-check --------------------------------------------
#[test]
fn no_recency_warning_when_argument_order_matches_dates() {
    let (_, warns) = merged_ab();
    assert!(
        !warns.iter().any(|w| w.kind == "recency_contradiction"),
        "b's TRAN_DATE is later, argument order [a,b] agrees → no contradiction: {warns:?}"
    );
}

#[test]
fn recency_contradiction_when_order_reversed() {
    let (a, b) = (load("delivery_a.ags"), load("delivery_b.ags"));
    // [b,a]: b (2024-02-15) first, then a (2024-01-15) is earlier → one warning.
    let res = merge_parsed(&[b, a], &lenient()).unwrap();
    let n = res
        .warnings
        .iter()
        .filter(|w| w.kind == "recency_contradiction")
        .count();
    assert_eq!(
        n, 1,
        "exactly one recency contradiction: {:?}",
        res.warnings
    );
}

// --- 4. column union ------------------------------------------------------
#[test]
fn column_union_adds_loca_lett_once_and_blank_for_a_only_rows() {
    let (out, _) = merged_ab();
    let loca = &out.groups["LOCA"];
    assert_eq!(
        loca.headings.iter().filter(|h| *h == "LOCA_LETT").count(),
        1,
        "LOCA_LETT (only in B) appears exactly once in the union schema"
    );
    // BH00 is only in A, which never carried LOCA_LETT → blank, not an error.
    let bh00 = row_by_key(loca, "LOCA_ID", "BH00").expect("BH00 survives");
    assert_eq!(cell(loca, bh00, "LOCA_LETT").as_deref(), Some(""));
}

// --- 5. group union -------------------------------------------------------
#[test]
fn group_union_keeps_b_only_abbr() {
    let (out, _) = merged_ab();
    let abbr = out.groups.get("ABBR").expect("ABBR (only in B) survives");
    assert!(row_by_key(abbr, "ABBR_CODE", "CP").is_some());
}

// --- 6. STRICT errors on the type mismatch -------------------------------
#[test]
fn strict_errors_on_type_mismatch() {
    let (a, b) = (load("delivery_a.ags"), load("delivery_b.ags"));
    let opts = MergeOpts {
        on_type_clash: TypeClashMode::Error,
        tran: Some(stamp()),
        ..Default::default()
    };
    match merge_parsed(&[a, b], &opts) {
        Err(MergeError::TypeConflict {
            group,
            heading,
            types,
        }) => {
            assert_eq!(group, "LOCA");
            assert_eq!(heading, "LOCA_NATE");
            assert!(
                types.contains(&"2DP".to_string()) && types.contains(&"X".to_string()),
                "{types:?}"
            );
        }
        other => panic!("expected TypeConflict, got {other:?}"),
    }
}

// --- 7. LENIENT widens the merged type to X ------------------------------
#[test]
fn lenient_widens_merged_type_to_x() {
    let (out, _) = merged_ab();
    let loca = &out.groups["LOCA"];
    assert_eq!(
        type_of(loca, "LOCA_NATE").as_deref(),
        Some("X"),
        "LOCA_NATE (2DP in A, X in B) widens to X"
    );
    // LOCA_GL agreed (2DP in both) → keeps its type.
    assert_eq!(type_of(loca, "LOCA_GL").as_deref(), Some("2DP"));
}

// --- 8. typed-vs-X widening is silent ------------------------------------
#[test]
fn typed_vs_x_widen_is_silent() {
    let (_, warns) = merged_ab();
    assert!(
        !warns.iter().any(|w| w.kind == "type_widened"),
        "2DP-vs-X is the trivial widen → no warning: {warns:?}"
    );
}

// --- 9. non-X-vs-non-X warns (lenient) and errors (strict) ---------------
fn two_files_typing(h_type_a: &str, h_type_b: &str) -> (ParsedFile, ParsedFile) {
    let mk = |t: &str, v: &str| {
        parse_str(&format!(
            "\"GROUP\",\"LOCA\"\n\"HEADING\",\"LOCA_ID\",\"LOCA_FOO\"\n\
             \"UNIT\",\"\",\"\"\n\"TYPE\",\"ID\",\"{t}\"\n\"DATA\",\"BH1\",\"{v}\"\n"
        ))
        .unwrap()
    };
    (mk(h_type_a, "1.0"), mk(h_type_b, "1.00"))
}

#[test]
fn non_x_vs_non_x_warns_under_lenient() {
    let (a, b) = two_files_typing("2DP", "1DP");
    let res = merge_parsed(
        &[a, b],
        &MergeOpts {
            on_type_clash: TypeClashMode::Widen,
            ..Default::default()
        },
    )
    .unwrap();
    let w = res
        .warnings
        .iter()
        .find(|w| w.kind == "type_widened")
        .expect("a widen warning");
    assert_eq!(w.heading.as_deref(), Some("LOCA_FOO"));
    assert!(
        w.message.contains("2DP") && w.message.contains("1DP"),
        "{}",
        w.message
    );
}

#[test]
fn non_x_vs_non_x_errors_under_strict() {
    let (a, b) = two_files_typing("2DP", "1DP");
    let err = merge_parsed(
        &[a, b],
        &MergeOpts {
            on_type_clash: TypeClashMode::Error,
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(matches!(err, MergeError::TypeConflict { .. }));
}

// --- 10. byte fidelity on an unchanged row -------------------------------
#[test]
fn byte_fidelity_on_unchanged_row_bh02() {
    let (out, _) = merged_ab();
    let loca = &out.groups["LOCA"];
    let bh02 = row_by_key(loca, "LOCA_ID", "BH02").unwrap();
    // Identical in both files → values preserved verbatim through the write path.
    assert_eq!(cell(loca, bh02, "LOCA_NATE").as_deref(), Some("120.00"));
    assert_eq!(cell(loca, bh02, "LOCA_GL").as_deref(), Some("12.00"));
    // And it picked up LOCA_LETT from B (union), not a conflict.
    assert_eq!(cell(loca, bh02, "LOCA_LETT").as_deref(), Some("GRID"));
}

// --- 12 (critic HIGH). silence is not deletion: A-only row + group survive
#[test]
fn silence_not_deletion_row_bh00_survives() {
    let (out, _) = merged_ab();
    let loca = &out.groups["LOCA"];
    let bh00 = row_by_key(loca, "LOCA_ID", "BH00").expect("BH00 (only in A) must survive");
    assert_eq!(cell(loca, bh00, "LOCA_GL").as_deref(), Some("5.00"));
}

#[test]
fn silence_not_deletion_group_myun_survives() {
    let (out, _) = merged_ab();
    let myun = out
        .groups
        .get("MYUN")
        .expect("MYUN (only in A, unkeyed) must survive");
    assert_eq!(myun.rows.len(), 1);
    assert_eq!(cell(myun, &myun.rows[0], "MYUN_A").as_deref(), Some("only"));
}

#[test]
fn new_row_bh03_added() {
    let (out, _) = merged_ab();
    assert!(row_by_key(&out.groups["LOCA"], "LOCA_ID", "BH03").is_some());
}

// --- merged TRAN is a synthesised single row with provenance --------------
#[test]
fn merged_tran_is_synthesised_with_provenance() {
    let (out, _) = merged_ab();
    let tran = out.groups.get("TRAN").expect("merged file has a TRAN");
    assert_eq!(
        tran.rows.len(),
        1,
        "one synthesised TRAN row, not the union of inputs"
    );
    let r = &tran.rows[0];
    assert_eq!(
        cell(tran, r, "TRAN_ISNO").as_deref(),
        Some("3"),
        "the merge stamp's ISNO"
    );
    let rem = cell(tran, r, "TRAN_REM").unwrap_or_default();
    assert!(rem.contains("Merged from 2"), "provenance recorded: {rem}");
}
