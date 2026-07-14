//! UNIT reconciliation (laterite#501).
//!
//! Merge used to take the FIRST non-empty UNIT per heading and discard the rest,
//! silently. With `LOCA_GL` declared `m` in one delivery and `mm` in another,
//! both values survive under the surviving `m` label — and because both are
//! valid `2DP` numbers, *nothing downstream can catch it*. A borehole's ground
//! level silently becomes 10,500 metres.
//!
//! The asymmetry these tests pin: **TYPE has a universal absorber (`X`), UNIT has
//! none.** So a unit clash is fatal in EVERY mode, widen and promote included — the one
//! place merge is deliberately less forgiving than it is about types.

use laterite_ags4_merge::{MergeError, MergeOpts, TypeClashMode, merge_parsed};
use laterite_ags4_parse::{ParsedFile, parse_str};

fn p(text: &str) -> ParsedFile {
    parse_str(text).unwrap()
}

/// One LOCA row, with `unit` / `ty` declared for LOCA_GL.
fn loca(unit: &str, ty: &str, id: &str, gl: &str) -> String {
    [
        r#""GROUP","PROJ""#.to_string(),
        r#""HEADING","PROJ_ID""#.to_string(),
        r#""UNIT","""#.to_string(),
        r#""TYPE","ID""#.to_string(),
        r#""DATA","P1""#.to_string(),
        r#""GROUP","LOCA""#.to_string(),
        r#""HEADING","LOCA_ID","LOCA_GL""#.to_string(),
        format!(r#""UNIT","","{unit}""#),
        format!(r#""TYPE","ID","{ty}""#),
        format!(r#""DATA","{id}","{gl}""#),
        String::new(),
    ]
    .join("\r\n")
}

fn opts(mode: TypeClashMode) -> MergeOpts {
    MergeOpts {
        on_type_clash: mode,
        tran: None,
        ..Default::default()
    }
}

/// THE bug. Metres and millimetres, identical TYPE — so no type-clash path is
/// entered at all — must not silently produce a file claiming both are metres.
#[test]
fn metres_and_millimetres_are_fatal_not_silently_relabelled() {
    let a = p(&loca("m", "2DP", "BH01", "10.00"));
    let b = p(&loca("mm", "2DP", "BH02", "10500.00"));

    for mode in [
        TypeClashMode::Error,
        TypeClashMode::Widen,
        TypeClashMode::Promote,
    ] {
        let err = merge_parsed(&[a.clone(), b.clone()], &opts(mode))
            .expect_err("a unit clash must be fatal in EVERY mode — widen and promote included");
        match err {
            MergeError::UnitConflict {
                group,
                heading,
                units,
            } => {
                assert_eq!(group, "LOCA");
                assert_eq!(heading, "LOCA_GL");
                assert!(units.contains(&"m".to_string()) && units.contains(&"mm".to_string()));
            }
            other => panic!("expected UnitConflict, got {other:?}"),
        }
    }
}

/// The message must not send the user in a circle. `--lenient` cannot absorb a
/// unit clash, so the error must not imply it can.
#[test]
fn the_error_does_not_offer_a_lenient_escape_it_cannot_honour() {
    let err = merge_parsed(
        &[
            p(&loca("m", "2DP", "BH01", "10.00")),
            p(&loca("mm", "2DP", "BH02", "10500.00")),
        ],
        &opts(TypeClashMode::Widen),
    )
    .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("will not convert units"), "got: {msg}");
    assert!(
        !msg.to_lowercase().contains("pass --lenient"),
        "must not suggest a mode that cannot fix it: {msg}"
    );
}

/// The regression guard on the fix itself: BLANK means "unspecified", not a
/// competing claim. Over-erroring here would break every ordinary merge, since a
/// sparse delivery routinely leaves UNIT empty.
#[test]
fn a_blank_unit_is_not_a_conflict_the_declared_one_wins() {
    let merged = merge_parsed(
        &[
            p(&loca("", "2DP", "BH01", "10.00")),
            p(&loca("m", "2DP", "BH02", "12.00")),
        ],
        &opts(TypeClashMode::Error),
    )
    .expect("blank vs `m` is not a disagreement");
    let out = String::from_utf8(merged.bytes).unwrap();
    assert!(
        out.contains(r#""UNIT","","m""#),
        "the declared unit must survive:\n{out}"
    );
}

/// All-blank stays blank — emit then fills UNIT from the dictionary.
#[test]
fn all_blank_units_stay_blank_for_emit_to_fill() {
    let merged = merge_parsed(
        &[
            p(&loca("", "2DP", "BH01", "10.00")),
            p(&loca("", "2DP", "BH02", "12.00")),
        ],
        &opts(TypeClashMode::Error),
    )
    .expect("no units declared anywhere is not a conflict");
    assert!(!merged.bytes.is_empty());
}

/// Identical units are obviously fine — the fix must not fire on agreement.
#[test]
fn identical_units_merge_cleanly() {
    merge_parsed(
        &[
            p(&loca("m", "2DP", "BH01", "10.00")),
            p(&loca("m", "2DP", "BH02", "12.00")),
        ],
        &opts(TypeClashMode::Error),
    )
    .expect("agreeing units must merge");
}

/// A `DT` column's FORMAT lives in the UNIT row, so a date-format clash IS a unit
/// clash — and is fixed by this rule with no DT-specific logic. (Unlike the
/// numeric case this one would at least trip Rule 8 on the merged file; catching
/// it at merge time is still strictly better than shipping a broken file.)
#[test]
fn a_dt_format_clash_is_a_unit_clash() {
    let a = p(&loca("yyyy-mm-dd", "DT", "BH01", "2026-03-05"));
    let b = p(&loca("dd/mm/yyyy", "DT", "BH02", "05/03/2026"));
    let err = merge_parsed(&[a, b], &opts(TypeClashMode::Widen))
        .expect_err("two date formats are a unit conflict");
    assert!(matches!(err, MergeError::UnitConflict { .. }), "{err:?}");
}

/// N-way: two files agree, a third differs. The conflict must still be caught and
/// must name the offending heading (a 2-file-only check would miss this).
#[test]
fn a_third_file_disagreeing_is_still_caught() {
    let err = merge_parsed(
        &[
            p(&loca("m", "2DP", "BH01", "10.00")),
            p(&loca("m", "2DP", "BH02", "12.00")),
            p(&loca("mm", "2DP", "BH03", "13000.00")),
        ],
        &opts(TypeClashMode::Widen),
    )
    .expect_err("a disagreeing third file must still be fatal");
    match err {
        MergeError::UnitConflict { heading, .. } => assert_eq!(heading, "LOCA_GL"),
        other => panic!("expected UnitConflict, got {other:?}"),
    }
}
