//! The type-clash lattice (laterite#500).
//!
//! `widen` resolves every TYPE disagreement to `X`, which is byte-faithful but
//! throws the type away — and `X` is the *least* informative answer available. For
//! a `2DP` vs `5DP` clash we can do better: keep `5DP`, the greatest precision
//! declared, and zero-pad the lower-precision file's cells.
//!
//! Two constraints make that non-trivial, and these tests pin both:
//!
//! - **Rule 8** requires a value to match its declared TYPE *exactly*, so promoting
//!   the TYPE without rewriting the values yields an invalid file. Promote is
//!   therefore the one mode in which merge rewrites a cell.
//! - Padding must never round. `max(n)` is the only admissible direction, which is
//!   also why the outcome cannot depend on argument order.
//!
//! Every test here emits under [`EmitMode::Strict`] — which refuses to write a file
//! that breaks any error-severity rule — so a merge that *returns bytes at all* has
//! already proved the merged output validates clean.

use laterite_ags4_emit::EmitMode;
use laterite_ags4_merge::{MergeError, MergeOpts, MergeResult, TypeClashMode, merge_parsed};
use laterite_ags4_parse::{ParsedFile, parse_str};
use laterite_ags4_reference::keychain::content_hash;

/// A COMPLETE, spec-valid AGS4 file carrying one LOCA row whose `LOCA_GL` is
/// declared `ty`. Complete — the TRAN/UNIT/TYPE abbreviation groups Rule 14/15/17
/// demand — precisely so `EmitMode::Strict` can be the validity oracle. Each file
/// declares its own TYPE code in the TYPE group, as a compliant producer would.
fn file(ty: &str, id: &str, gl: &str) -> String {
    [
        r#""GROUP","PROJ""#.into(),
        r#""HEADING","PROJ_ID","PROJ_NAME""#.into(),
        r#""UNIT","","""#.into(),
        r#""TYPE","ID","X""#.into(),
        r#""DATA","P1","Promote fixture""#.into(),
        String::new(),
        r#""GROUP","TRAN""#.into(),
        r#""HEADING","TRAN_ISNO","TRAN_DATE","TRAN_PROD","TRAN_STAT","TRAN_AGS","TRAN_RECV","TRAN_DLIM","TRAN_RCON""#.into(),
        r#""UNIT","","yyyy-mm-dd","","","","","","""#.into(),
        r#""TYPE","X","DT","X","X","X","X","X","X""#.into(),
        r#""DATA","1","2020-08-18","Producer","Draft","4.1","Recipient","|","+""#.into(),
        String::new(),
        r#""GROUP","UNIT""#.into(),
        r#""HEADING","UNIT_UNIT","UNIT_DESC""#.into(),
        r#""UNIT","","""#.into(),
        r#""TYPE","X","X""#.into(),
        r#""DATA","yyyy-mm-dd","year month day""#.into(),
        r#""DATA","m","metres""#.into(),
        String::new(),
        r#""GROUP","TYPE""#.into(),
        r#""HEADING","TYPE_TYPE","TYPE_DESC""#.into(),
        r#""UNIT","","""#.into(),
        r#""TYPE","X","X""#.into(),
        r#""DATA","ID","Unique identifier""#.into(),
        r#""DATA","X","Text""#.into(),
        r#""DATA","DT","Date and time""#.into(),
        format!(r#""DATA","{ty}","Number""#),
        String::new(),
        r#""GROUP","LOCA""#.into(),
        r#""HEADING","LOCA_ID","LOCA_GL""#.into(),
        r#""UNIT","","m""#.into(),
        format!(r#""TYPE","ID","{ty}""#),
        format!(r#""DATA","{id}","{gl}""#),
        String::new(),
    ]
    .join("\r\n")
}

fn p(ty: &str, id: &str, gl: &str) -> ParsedFile {
    parse_str(&file(ty, id, gl)).unwrap()
}

/// Merge under `mode`, emitting STRICT — so `Ok` means "and the output is valid AGS4".
fn merge(files: &[ParsedFile], mode: TypeClashMode) -> Result<MergeResult, MergeError> {
    merge_parsed(
        files,
        &MergeOpts {
            on_type_clash: mode,
            emit_mode: EmitMode::Strict,
            ..Default::default()
        },
    )
}

/// The merged file's `LOCA_GL`: its declared TYPE, and `LOCA_ID -> value`.
fn loca_gl(r: &MergeResult) -> (String, Vec<(String, String)>) {
    let text = String::from_utf8(r.bytes.clone()).expect("merged output is UTF-8");
    let f = parse_str(&text).unwrap();
    let g = f.groups.get("LOCA").expect("LOCA survived the merge");
    let gl = g
        .headings
        .iter()
        .position(|h| h == "LOCA_GL")
        .expect("LOCA_GL survived");
    let id = g.headings.iter().position(|h| h == "LOCA_ID").unwrap();
    let rows = g
        .rows
        .iter()
        .map(|r| (r.values[id].clone(), r.values[gl].clone()))
        .collect();
    (g.types[gl].clone(), rows)
}

// ---------------------------------------------------------------------------
// The Rule-8 matrix: what each mode resolves a clash to.
// ---------------------------------------------------------------------------

/// THE case. `2DP` + `5DP` → `5DP`, and the 2DP file's value is zero-padded so the
/// merged file is Rule-8 clean. Strict emit proves the "validates clean" half.
#[test]
fn two_dp_and_five_dp_promote_to_five_dp_and_the_output_is_valid() {
    let r = merge(
        &[p("2DP", "BH01", "10.00"), p("5DP", "BH02", "20.12345")],
        TypeClashMode::Promote,
    )
    .expect("promote yields a VALID file (EmitMode::Strict would refuse otherwise)");

    let (ty, rows) = loca_gl(&r);
    assert_eq!(ty, "5DP", "the merged column keeps the greatest precision");
    assert_eq!(
        rows,
        vec![
            ("BH01".to_string(), "10.00000".to_string()), // padded: 2 → 5 places
            ("BH02".to_string(), "20.12345".to_string()), // already 5DP: untouched
        ]
    );

    let w = r
        .warnings
        .iter()
        .find(|w| w.kind == "type_promoted")
        .unwrap();
    assert_eq!(w.heading.as_deref(), Some("LOCA_GL"));
    assert!(w.message.contains("5DP"), "warning names the promoted type");
}

/// `0DP` is in the family too: an integer column promotes and gains a decimal point.
#[test]
fn zero_dp_and_two_dp_promote_to_two_dp() {
    let r = merge(
        &[p("0DP", "BH01", "10"), p("2DP", "BH02", "20.50")],
        TypeClashMode::Promote,
    )
    .expect("valid");
    let (ty, rows) = loca_gl(&r);
    assert_eq!(ty, "2DP");
    assert_eq!(rows[0], ("BH01".to_string(), "10.00".to_string()));
    assert_eq!(rows[1], ("BH02".to_string(), "20.50".to_string()));
}

/// **Significant figures must NOT promote.** Padding `3SF` to `5SF` would assert two
/// digits of measured precision the instrument never resolved — a lie about the
/// data, not a reformatting. So an nSF clash falls back to `X`, values untouched.
#[test]
fn significant_figures_fall_back_to_x_and_are_never_padded() {
    let r = merge(
        &[p("3SF", "BH01", "10.0"), p("5SF", "BH02", "20.123")],
        TypeClashMode::Promote,
    )
    .expect("valid");
    let (ty, rows) = loca_gl(&r);
    assert_eq!(ty, "X", "nSF has no lossless join — widen, don't promote");
    assert_eq!(rows[0].1, "10.0", "bytes untouched");
    assert_eq!(rows[1].1, "20.123");
    assert!(
        r.warnings.iter().any(|w| w.kind == "type_widened"),
        "the fallback is reported as a widen, not a promote"
    );
    assert!(!r.warnings.iter().any(|w| w.kind == "type_promoted"));
}

/// Anything involving `X`, or crossing type families, has no numeric join → `X`.
#[test]
fn x_and_cross_family_clashes_still_widen() {
    for (a, b, va, vb) in [
        ("2DP", "X", "10.00", "about ten"),   // typed vs free text
        ("2DP", "DT", "10.00", "2020-08-18"), // cross-family
        ("2DP", "2SCI", "10.00", "1.0E+01"),  // nDP vs nSCI — different precision claim
    ] {
        let r = merge(
            &[p(a, "BH01", va), p(b, "BH02", vb)],
            TypeClashMode::Promote,
        )
        .unwrap_or_else(|e| panic!("{a} vs {b} should widen, not fail: {e}"));
        let (ty, rows) = loca_gl(&r);
        assert_eq!(ty, "X", "{a} vs {b} must widen to X");
        assert_eq!(rows[0].1, va, "{a} vs {b}: bytes untouched");
        assert_eq!(rows[1].1, vb);
    }
}

/// The default is unchanged: a clash is still an error unless a mode is chosen.
#[test]
fn error_is_still_the_default_and_still_refuses() {
    let files = [p("2DP", "BH01", "10.00"), p("5DP", "BH02", "20.12345")];
    let err = merge(&files, TypeClashMode::Error).expect_err("default refuses");
    let MergeError::TypeConflict { heading, types, .. } = &err else {
        panic!("expected TypeConflict, got {err:?}");
    };
    assert_eq!(heading, "LOCA_GL");
    assert_eq!(types, &["2DP".to_string(), "5DP".to_string()]);
    // The message must point at BOTH escape hatches, not just the lossy one.
    let msg = err.to_string();
    assert!(msg.contains("promote"), "message offers promote: {msg}");
    assert!(msg.contains("widen"), "message offers widen: {msg}");
}

// ---------------------------------------------------------------------------
// Promote, never demote.
// ---------------------------------------------------------------------------

/// The outcome must not depend on argument order — `max` guarantees it. (Contrast
/// the KEY-conflict rule, where later-argument deliberately wins.) If the join were
/// "last file's type wins", `[5DP, 2DP]` would demote and ROUND `20.12345` away.
#[test]
fn never_demotes_whatever_the_argument_order() {
    let hi = || p("5DP", "BH02", "20.12345");
    let lo = || p("2DP", "BH01", "10.00");

    for (label, files) in [
        ("low precision first", vec![lo(), hi()]),
        ("high precision first", vec![hi(), lo()]),
    ] {
        let r = merge(&files, TypeClashMode::Promote).expect("valid");
        let (ty, rows) = loca_gl(&r);
        assert_eq!(ty, "5DP", "{label}: max(n) wins, not the last file");
        let gl: Vec<&str> = rows.iter().map(|(_, v)| v.as_str()).collect();
        assert!(
            gl.contains(&"20.12345"),
            "{label}: the precise value survives un-rounded — got {gl:?}"
        );
        assert!(
            gl.contains(&"10.00000"),
            "{label}: the coarse value is padded"
        );
    }
}

/// A value merge cannot pad LOSSLESSLY is kept byte-for-byte and reported — never
/// rounded. Two ways that happens, and neither may be silently "tidied":
///
/// - more decimal places than the promoted type (`10.0000012` is 7 places, the join
///   is 5) — padding is impossible, and shortening would *round*;
/// - not a number at all.
///
/// Both were already invalid for the TYPE their own file declared, so the merged
/// file inherits a Rule 8 error it did not create — hence REPORT, not Strict.
#[test]
fn an_unpaddable_value_is_kept_verbatim_and_warned_never_rounded() {
    for bad in ["10.0000012", "N/A"] {
        let r = merge_parsed(
            &[p("2DP", "BH01", bad), p("5DP", "BH02", "20.12345")],
            &MergeOpts {
                on_type_clash: TypeClashMode::Promote,
                emit_mode: EmitMode::Report,
                ..Default::default()
            },
        )
        .expect("report mode emits");

        let (ty, rows) = loca_gl(&r);
        assert_eq!(ty, "5DP");
        assert_eq!(
            rows[0].1, bad,
            "{bad:?} kept verbatim — rounding it would be the very data loss promote exists \
             to avoid"
        );
        let w = r
            .warnings
            .iter()
            .find(|w| w.kind == "promote_value_kept_verbatim")
            .unwrap_or_else(|| {
                panic!("{bad:?}: merge must say so, not let it surface as a bare Rule 8 error")
            });
        assert_eq!(w.heading.as_deref(), Some("LOCA_GL"));
        assert!(w.message.contains('1'), "counts the affected values");
    }
}

/// The happy converse: a cell with more places than its OWN declared type but fewer
/// than the promoted join pads cleanly — so promote incidentally *repairs* it. Here
/// `10.005` is Rule-8-invalid as `2DP`, and lands as a valid `10.00500` under `5DP`
/// with every digit intact. Emitted STRICT, so validity is asserted, not assumed.
#[test]
fn promote_can_incidentally_repair_a_value_that_was_invalid_at_source() {
    let r = merge(
        &[p("2DP", "BH01", "10.005"), p("5DP", "BH02", "20.12345")],
        TypeClashMode::Promote,
    )
    .expect("the padded file is VALID — Strict emit would refuse otherwise");

    let (ty, rows) = loca_gl(&r);
    assert_eq!(ty, "5DP");
    assert_eq!(
        rows[0].1, "10.00500",
        "padded, not rounded — the 5 survives"
    );
    assert!(
        !r.warnings
            .iter()
            .any(|w| w.kind == "promote_value_kept_verbatim"),
        "it padded fine, so there is nothing to warn about"
    );
}

/// Blank means "no opinion", not zero. Padding it into `0.00000` would invent data.
#[test]
fn a_blank_cell_is_not_padded_into_a_zero() {
    let r = merge(
        &[p("2DP", "BH01", ""), p("5DP", "BH02", "20.12345")],
        TypeClashMode::Promote,
    )
    .expect("valid");
    let (_, rows) = loca_gl(&r);
    assert_eq!(rows[0].1, "", "blank stays blank");
    assert!(
        !r.warnings
            .iter()
            .any(|w| w.kind == "promote_value_kept_verbatim"),
        "a blank is not an unpaddable value — it must not warn"
    );
}

// ---------------------------------------------------------------------------
// Composition: the reason the mode exists.
// ---------------------------------------------------------------------------

/// **The motivating property.** `_content_hash` canonicalises through the declared
/// TYPE, so `10.00` hashes as a *number* under `2DP` but as a *string* under `X`.
/// That means a WIDENED merge does not value-dedup against its own typed inputs —
/// while a PROMOTED one does. Hashes are taken from the real merged bytes.
#[test]
fn a_promoted_row_still_content_hashes_equal_to_its_typed_source() {
    let source = content_hash(
        "LOCA",
        &[("LOCA_ID", "ID", "BH01"), ("LOCA_GL", "2DP", "10.00")],
    );

    let promoted = merge(
        &[p("2DP", "BH01", "10.00"), p("5DP", "BH02", "20.12345")],
        TypeClashMode::Promote,
    )
    .unwrap();
    let (pty, prows) = loca_gl(&promoted);
    let ph = content_hash(
        "LOCA",
        &[
            ("LOCA_ID", "ID", "BH01"),
            ("LOCA_GL", &pty, &prows[0].1), // 5DP / "10.00000"
        ],
    );
    assert_eq!(
        ph, source,
        "promote keeps the column numeric, so 10.00 and 10.00000 canonicalise to the \
         same number and the merged row still dedups against its typed source"
    );

    let widened = merge(
        &[p("2DP", "BH01", "10.00"), p("X", "BH02", "about twenty")],
        TypeClashMode::Widen,
    )
    .unwrap();
    let (wty, wrows) = loca_gl(&widened);
    let wh = content_hash(
        "LOCA",
        &[("LOCA_ID", "ID", "BH01"), ("LOCA_GL", &wty, &wrows[0].1)], // X / "10.00"
    );
    assert_ne!(
        wh, source,
        "the widen sharp edge, pinned: identical bytes, but X makes it a STRING where \
         2DP made it a number — so a widened merge does NOT dedup against its inputs"
    );
}

/// Zero-padding changes raw bytes but not the value, and a revision requires BOTH to
/// differ — so promote cannot manufacture a false revision.
#[test]
fn padding_is_not_reported_as_a_revision() {
    // Same borehole, same ground level, two precisions. Nothing was revised.
    let r = merge(
        &[p("2DP", "BH01", "10.00"), p("5DP", "BH01", "10.00000")],
        TypeClashMode::Promote,
    )
    .expect("valid");
    assert!(
        r.revisions.is_empty(),
        "re-stating 10.00 as 10.00000 is a formatting change, not a revision: {:?}",
        r.revisions
    );

    // ...but a genuine change still is one.
    let r = merge(
        &[p("2DP", "BH01", "10.00"), p("5DP", "BH01", "11.00000")],
        TypeClashMode::Promote,
    )
    .expect("valid");
    assert_eq!(r.revisions.len(), 1, "a real value change IS a revision");
    assert_eq!(r.revisions[0].changed, vec!["LOCA_GL".to_string()]);
}

/// Promote must not open a hole in the UNIT gate (#501): metres vs millimetres is
/// still fatal, in the new mode as in every other.
#[test]
fn promote_does_not_weaken_the_unit_conflict_gate() {
    let m = parse_str(&file("2DP", "BH01", "10.00")).unwrap();
    let mm = parse_str(&file("5DP", "BH02", "10500.00000").replace(r#""m""#, r#""mm""#)).unwrap();
    let err = merge(&[m, mm], TypeClashMode::Promote)
        .expect_err("a unit clash is fatal in EVERY mode, promote included");
    assert!(
        matches!(err, MergeError::UnitConflict { .. }),
        "got {err:?}"
    );
}
