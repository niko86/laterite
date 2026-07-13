//! Hardening tests: the gaps an adversarial review found in the POC, each with a
//! real fix (or, where the gap is architecturally inherent, a pinned behaviour).

use laterite_ags4_merge::{MergeOpts, TranStamp, TypeMismatchMode, merge_parsed};
use laterite_ags4_parse::{DataRow, ParsedFile, ParsedGroup, parse_str};

fn p(text: &str) -> ParsedFile {
    parse_str(text).unwrap()
}

fn reparse(bytes: &[u8]) -> ParsedFile {
    parse_str(std::str::from_utf8(bytes).unwrap()).unwrap()
}

fn lenient_no_tran() -> MergeOpts {
    MergeOpts {
        type_mismatch: TypeMismatchMode::Lenient,
        tran: None,
        ..Default::default()
    }
}

fn rows_with<'a>(g: &'a ParsedGroup, h: &str, v: &str) -> Vec<&'a DataRow> {
    let ci = g.headings.iter().position(|x| x == h);
    match ci {
        Some(ci) => g
            .rows
            .iter()
            .filter(|r| r.values.get(ci).map(String::as_str) == Some(v))
            .collect(),
        None => vec![],
    }
}

fn cell(g: &ParsedGroup, r: &DataRow, h: &str) -> Option<String> {
    let ci = g.headings.iter().position(|x| x == h)?;
    r.values.get(ci).cloned()
}

// --- Task #1: unkeyed groups don't explode under N-file schema widening ---
#[test]
fn unkeyed_group_dedups_exact_resends_across_widening_schema() {
    // LOGX is not a dictionary group → unkeyed. The same row is re-sent in all
    // three files while the schema WIDENS (f3 adds LOGX_C). A naive whole-row
    // match against an accumulating union would re-duplicate every fold; the
    // single-pass union-tuple identity collapses the exact re-sends instead.
    let f1 = p(
        "\"GROUP\",\"LOGX\"\n\"HEADING\",\"LOGX_A\",\"LOGX_B\"\n\"UNIT\",\"\",\"\"\n\"TYPE\",\"X\",\"X\"\n\"DATA\",\"r1\",\"hello\"\n",
    );
    let f2 = p(
        "\"GROUP\",\"LOGX\"\n\"HEADING\",\"LOGX_A\",\"LOGX_B\"\n\"UNIT\",\"\",\"\"\n\"TYPE\",\"X\",\"X\"\n\"DATA\",\"r1\",\"hello\"\n",
    );
    let f3 = p(
        "\"GROUP\",\"LOGX\"\n\"HEADING\",\"LOGX_A\",\"LOGX_B\",\"LOGX_C\"\n\"UNIT\",\"\",\"\",\"\"\n\"TYPE\",\"X\",\"X\",\"X\"\n\"DATA\",\"r1\",\"hello\",\"\"\n\"DATA\",\"r2\",\"new\",\"c2\"\n",
    );
    let res = merge_parsed(&[f1, f2, f3], &lenient_no_tran()).unwrap();
    let out = reparse(&res.bytes);
    let logx = &out.groups["LOGX"];
    // r1/hello was in all 3 files (with C blank in f3) → collapses to ONE row.
    assert_eq!(
        rows_with(logx, "LOGX_A", "r1").len(),
        1,
        "exact re-sends collapse, not 3×"
    );
    // Total is bounded: r1 + r2, no explosion.
    assert_eq!(logx.rows.len(), 2);
}

// --- Task #2: a within-file duplicate KEY is surfaced, not silently absorbed --
#[test]
fn within_file_duplicate_key_warns() {
    let f = p(
        "\"GROUP\",\"LOCA\"\n\"HEADING\",\"LOCA_ID\",\"LOCA_GL\"\n\"UNIT\",\"\",\"m\"\n\"TYPE\",\"ID\",\"2DP\"\n\"DATA\",\"BH1\",\"10.00\"\n\"DATA\",\"BH1\",\"11.00\"\n",
    );
    let res = merge_parsed(&[f], &lenient_no_tran()).unwrap();
    assert!(
        res.warnings
            .iter()
            .any(|w| w.kind == "duplicate_key_in_file"),
        "a source file with two BH1 rows is a data-quality error worth surfacing: {:?}",
        res.warnings
    );
    // Last-wins collapse: one BH1 row, GL=11.00.
    let out = reparse(&res.bytes);
    let loca = &out.groups["LOCA"];
    let bh1 = rows_with(loca, "LOCA_ID", "BH1");
    assert_eq!(bh1.len(), 1);
    assert_eq!(cell(loca, bh1[0], "LOCA_GL").as_deref(), Some("11.00"));
}

// --- Task #4: revisions are reported (typed), formatting-only changes are not --
#[test]
fn revision_report_records_typed_changes_only() {
    let f1 = p(
        "\"GROUP\",\"LOCA\"\n\"HEADING\",\"LOCA_ID\",\"LOCA_GL\"\n\"UNIT\",\"\",\"m\"\n\"TYPE\",\"ID\",\"2DP\"\n\"DATA\",\"BH1\",\"10.00\"\n\"DATA\",\"BH2\",\"20.00\"\n",
    );
    // f2 revises BH1 (10.00 → 11.50, a real change) and re-sends BH2 with a
    // formatting-only difference (20.00 → 20.0, typed-equal under 2DP).
    let f2 = p(
        "\"GROUP\",\"LOCA\"\n\"HEADING\",\"LOCA_ID\",\"LOCA_GL\"\n\"UNIT\",\"\",\"m\"\n\"TYPE\",\"ID\",\"2DP\"\n\"DATA\",\"BH1\",\"11.50\"\n\"DATA\",\"BH2\",\"20.0\"\n",
    );
    let res = merge_parsed(&[f1, f2], &lenient_no_tran()).unwrap();
    assert_eq!(
        res.revisions.len(),
        1,
        "only BH1 is a real revision: {:?}",
        res.revisions
    );
    let r = &res.revisions[0];
    assert_eq!(r.group, "LOCA");
    assert_eq!(r.key, vec!["BH1".to_string()]);
    assert_eq!(r.changed, vec!["LOCA_GL".to_string()]);
    assert_eq!(r.winner_file, 1);
}

// --- Task #3: same-day TRAN_DATE with different times does NOT false-warn ------
#[test]
fn same_day_different_time_does_not_warn() {
    let mk = |dt: &str| {
        p(&format!(
            "\"GROUP\",\"TRAN\"\n\"HEADING\",\"TRAN_ISNO\",\"TRAN_DATE\"\n\"UNIT\",\"\",\"yyyy-mm-dd\"\n\"TYPE\",\"X\",\"DT\"\n\"DATA\",\"1\",\"{dt}\"\n"
        ))
    };
    // f2 is later in argument order but earlier in the DAY — different times, same
    // calendar day → date-granular comparison must not flag a contradiction.
    let res = merge_parsed(
        &[mk("2024-01-15T14:00:00"), mk("2024-01-15T09:00:00")],
        &MergeOpts {
            tran: None,
            ..Default::default()
        },
    )
    .unwrap();
    assert!(
        !res.warnings
            .iter()
            .any(|w| w.kind == "recency_contradiction"),
        "same calendar day → no contradiction: {:?}",
        res.warnings
    );
}

#[test]
fn different_days_with_times_still_warn() {
    // Proves the day comparison is real (parses the timestamp), not just skipped.
    let mk = |dt: &str| {
        p(&format!(
            "\"GROUP\",\"TRAN\"\n\"HEADING\",\"TRAN_ISNO\",\"TRAN_DATE\"\n\"UNIT\",\"\",\"yyyy-mm-dd\"\n\"TYPE\",\"X\",\"DT\"\n\"DATA\",\"1\",\"{dt}\"\n"
        ))
    };
    let res = merge_parsed(
        &[mk("2024-02-15T09:00:00"), mk("2024-01-15T23:00:00")],
        &MergeOpts {
            tran: None,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        res.warnings
            .iter()
            .filter(|w| w.kind == "recency_contradiction")
            .count(),
        1,
        "Feb→Jan across days is a real contradiction: {:?}",
        res.warnings
    );
}

// --- Task #6: non-ASCII content survives merge → emit → re-parse --------------
#[test]
fn non_ascii_round_trips() {
    let f = p(
        "\"GROUP\",\"PROJ\"\n\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\n\"UNIT\",\"\",\"\"\n\"TYPE\",\"ID\",\"X\"\n\"DATA\",\"P1\",\"Café Motörhead — Ø Δ\"\n",
    );
    let res = merge_parsed(&[f], &lenient_no_tran()).unwrap();
    let out = reparse(&res.bytes);
    let proj = &out.groups["PROJ"];
    assert_eq!(
        cell(proj, &proj.rows[0], "PROJ_NAME").as_deref(),
        Some("Café Motörhead — Ø Δ")
    );
}

// --- Task #7: a KEY-value correction is (inherently) two rows, not a revision --
// PINNED behaviour, not a fix: KEY-based identity cannot tell a typo-fix in a KEY
// field from a genuinely new row. Both persist. Only a supersede/delete primitive
// (deferred, owner-decision 7) could change this. Documented in the module doc.
#[test]
fn key_value_correction_yields_two_rows_pinned() {
    let f1 = p(
        "\"GROUP\",\"LOCA\"\n\"HEADING\",\"LOCA_ID\",\"LOCA_GL\"\n\"UNIT\",\"\",\"m\"\n\"TYPE\",\"ID\",\"2DP\"\n\"DATA\",\"BH1\",\"10.00\"\n",
    );
    // "BH1" was a typo; corrected to "BH01" — but LOCA_ID IS the key, so this is a
    // different identity, not a revision.
    let f2 = p(
        "\"GROUP\",\"LOCA\"\n\"HEADING\",\"LOCA_ID\",\"LOCA_GL\"\n\"UNIT\",\"\",\"m\"\n\"TYPE\",\"ID\",\"2DP\"\n\"DATA\",\"BH01\",\"10.00\"\n",
    );
    let res = merge_parsed(&[f1, f2], &lenient_no_tran()).unwrap();
    let out = reparse(&res.bytes);
    let loca = &out.groups["LOCA"];
    assert_eq!(
        loca.rows.len(),
        2,
        "both the typo'd and corrected KEY persist"
    );
    assert_eq!(rows_with(loca, "LOCA_ID", "BH1").len(), 1);
    assert_eq!(rows_with(loca, "LOCA_ID", "BH01").len(), 1);
    // And there is no revision — the correction is not detectable as one.
    assert!(res.revisions.is_empty());
}

// --- Task #5: a revised parent with children in the merge is flagged for review
#[test]
fn revised_parent_flags_child_groups() {
    // SAMP is a dictionary child of LOCA. f1 supplies both; f2 revises LOCA BH1's
    // GL but does NOT re-supply SAMP → the SAMP rows still reference the old GL.
    let f1 = p(
        "\"GROUP\",\"LOCA\"\n\"HEADING\",\"LOCA_ID\",\"LOCA_GL\"\n\"UNIT\",\"\",\"m\"\n\"TYPE\",\"ID\",\"2DP\"\n\"DATA\",\"BH1\",\"10.00\"\n\n\"GROUP\",\"SAMP\"\n\"HEADING\",\"LOCA_ID\",\"SAMP_TOP\",\"SAMP_REF\",\"SAMP_TYPE\",\"SAMP_ID\"\n\"UNIT\",\"\",\"m\",\"\",\"\",\"\"\n\"TYPE\",\"ID\",\"2DP\",\"X\",\"PA\",\"ID\"\n\"DATA\",\"BH1\",\"1.00\",\"S1\",\"U\",\"SA1\"\n",
    );
    let f2 = p(
        "\"GROUP\",\"LOCA\"\n\"HEADING\",\"LOCA_ID\",\"LOCA_GL\"\n\"UNIT\",\"\",\"m\"\n\"TYPE\",\"ID\",\"2DP\"\n\"DATA\",\"BH1\",\"11.50\"\n",
    );
    let res = merge_parsed(&[f1, f2], &lenient_no_tran()).unwrap();
    let w = res
        .warnings
        .iter()
        .find(|w| w.kind == "parent_revised_check_children")
        .expect("LOCA revised while SAMP present → flag it");
    assert_eq!(w.group.as_deref(), Some("LOCA"));
    assert!(
        w.message.contains("SAMP"),
        "names the child group: {}",
        w.message
    );
}

// A type WIDEN over an identical raw value is not a revision. LOCA_NATE is 2DP in
// f1 and X in f2 with the SAME bytes "100.00"; only LOCA_GL actually changes. The
// widened column must not make equal bytes compare unequal across the type boundary.
#[test]
fn type_widen_over_identical_value_is_not_a_revision() {
    let f1 = p(
        "\"GROUP\",\"LOCA\"\n\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\",\"LOCA_GL\"\n\"UNIT\",\"\",\"m\",\"m\"\n\"TYPE\",\"ID\",\"2DP\",\"2DP\"\n\"DATA\",\"BH1\",\"100.00\",\"10.00\"\n",
    );
    let f2 = p(
        "\"GROUP\",\"LOCA\"\n\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\",\"LOCA_GL\"\n\"UNIT\",\"\",\"m\",\"m\"\n\"TYPE\",\"ID\",\"X\",\"2DP\"\n\"DATA\",\"BH1\",\"100.00\",\"11.50\"\n",
    );
    let res = merge_parsed(&[f1, f2], &lenient_no_tran()).unwrap();
    assert_eq!(
        res.revisions.len(),
        1,
        "only LOCA_GL changed: {:?}",
        res.revisions
    );
    assert_eq!(res.revisions[0].changed, vec!["LOCA_GL".to_string()]);
}

// A supplied merge-TRAN stamp must reach the output even when NO input file
// carries a TRAN group. The synthesis was gated behind "TRAN present in the input
// union", so a stamp for TRAN-less inputs was silently dropped and emit injected
// its own generic placeholder (ISNO=1, date 1900-01-01) instead of the caller's.
#[test]
fn merge_tran_stamp_lands_even_when_no_input_has_tran() {
    let f = "\"GROUP\",\"LOCA\"\n\"HEADING\",\"LOCA_ID\",\"LOCA_GL\"\n\"UNIT\",\"\",\"m\"\n\"TYPE\",\"ID\",\"2DP\"\n\"DATA\",\"BH1\",\"10.00\"\n";
    let opts = MergeOpts {
        type_mismatch: TypeMismatchMode::Lenient,
        tran: Some(TranStamp {
            isno: "9".into(),
            date: "2024-05-01".into(),
            prod: "Merger".into(),
            recv: String::new(),
            stat: String::new(),
            ags: "4.1.1".into(),
        }),
        ..Default::default()
    };
    let res = merge_parsed(&[p(f), p(f)], &opts).unwrap();
    let out = reparse(&res.bytes);
    let tran = out
        .groups
        .get("TRAN")
        .expect("a stamped merge-TRAN is emitted even with TRAN-less inputs");
    assert_eq!(tran.rows.len(), 1);
    assert_eq!(cell(tran, &tran.rows[0], "TRAN_ISNO").as_deref(), Some("9"));
    assert_eq!(
        cell(tran, &tran.rows[0], "TRAN_DATE").as_deref(),
        Some("2024-05-01")
    );
    assert_eq!(
        cell(tran, &tran.rows[0], "TRAN_PROD").as_deref(),
        Some("Merger")
    );
}
