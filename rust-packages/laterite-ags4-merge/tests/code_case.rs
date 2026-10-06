//! ABBR codes that differ only by letter case across (or within) the inputs
//! (#1010): always warned, rewritten only on request — by `recode`, or by
//! `on_code_case = standard` when exactly one standard code matches — and a
//! rewrite that would merge two distinct rows is refused.

use std::fmt::Write as _;

use laterite_ags4_emit::{Cell, DictVersion, EmitMode, EmitOpts, GroupInput, emit_ags4};
use laterite_ags4_merge::{
    CodeCaseMode, MergeError, MergeOpts, MergeResult, MergeWarning, Recode, TranStamp, merge_parsed,
};
use laterite_ags4_parse::{ParsedFile, parse_str};

fn group(code: &str, headings: &[&str], rows: &[Vec<&str>]) -> GroupInput {
    GroupInput {
        code: code.to_string(),
        headings: headings.iter().map(|h| (*h).to_string()).collect(),
        units: None,
        types: None,
        rows: rows
            .iter()
            .map(|r| r.iter().map(|c| Cell::Text((*c).to_string())).collect())
            .collect(),
    }
}

const SAMP_H: [&str; 5] = ["LOCA_ID", "SAMP_TOP", "SAMP_REF", "SAMP_TYPE", "SAMP_ID"];

/// The issue's delivery: two TRIG specimens of BH1 whose `TRIG_COND` is
/// `cond`, with SAMP types `types`, built with synthesised metadata so it is
/// valid on its own (its ABBR rows are made from the PA cells it carries).
fn delivery(cond: &str, types: [&str; 2]) -> ParsedFile {
    let samp = |i: usize| {
        let (top, r, id) = [("1.00", "1", "S1"), ("2.00", "2", "S2")][i];
        vec!["BH1", top, r, types[i], id]
    };
    let trig = |i: usize| {
        let mut row = samp(i);
        row.extend(["1", ["1.00", "2.00"][i], "UU", cond]);
        row
    };
    let mut trig_h = SAMP_H.to_vec();
    trig_h.extend(["SPEC_REF", "SPEC_DPTH", "TRIG_TYPE", "TRIG_COND"]);
    let groups = [
        group("PROJ", &["PROJ_ID"], &[vec!["P1"]]),
        group("LOCA", &["LOCA_ID"], &[vec!["BH1"]]),
        group("SAMP", &SAMP_H, &[samp(0), samp(1)]),
        group("TRIG", &trig_h, &[trig(0), trig(1)]),
    ];
    let opts = EmitOpts {
        mode: EmitMode::AutoFix,
        edition: DictVersion::V4_1_1,
        tran: Some(TranStamp::new("1", "2026-01-01", "A", "B", "Draft")),
        synthesise_metadata: true,
        ..EmitOpts::default()
    };
    let out = emit_ags4(&groups, &opts).unwrap();
    assert!(out.findings.is_empty(), "valid alone: {:?}", out.findings);
    parse_str(std::str::from_utf8(&out.bytes).unwrap()).unwrap()
}

/// Strict, so `Ok` is itself the assertion that the merged file validates.
fn opts(mode: CodeCaseMode, recode: Recode) -> MergeOpts {
    MergeOpts {
        edition: DictVersion::V4_1_1,
        emit_mode: EmitMode::Strict,
        tran: Some(TranStamp::new("2", "2026-01-02", "A", "B", "Draft")),
        on_code_case: mode,
        recode,
        ..MergeOpts::default()
    }
}

fn recode(entries: &[(&str, &str, &str)]) -> Recode {
    let mut m = Recode::new();
    for (h, from, to) in entries {
        m.entry((*h).to_string())
            .or_default()
            .insert((*from).to_string(), (*to).to_string());
    }
    m
}

fn the_issue() -> [ParsedFile; 2] {
    [
        delivery("Undisturbed", ["U", "D"]),
        delivery("UNDISTURBED", ["U", "D"]),
    ]
}

fn column(res: &MergeResult, code: &str, heading: &str) -> Vec<String> {
    let p = parse_str(std::str::from_utf8(&res.bytes).unwrap()).unwrap();
    let g = &p.groups[code];
    let c = g.col(heading).unwrap();
    g.rows
        .iter()
        .map(|r| g.value_at(r, c).unwrap_or_default().to_string())
        .collect()
}

/// The `(code, description)` ABBR rows under `heading`.
fn abbr(res: &MergeResult, heading: &str) -> Vec<(String, String)> {
    let h = column(res, "ABBR", "ABBR_HDNG");
    let c = column(res, "ABBR", "ABBR_CODE");
    let d = column(res, "ABBR", "ABBR_DESC");
    (0..h.len())
        .filter(|&i| h[i] == heading)
        .map(|i| (c[i].clone(), d[i].clone()))
        .collect()
}

fn case_warnings(res: &MergeResult) -> Vec<&MergeWarning> {
    res.warnings
        .iter()
        .filter(|w| w.kind == "abbr_code_case")
        .collect()
}

#[test]
fn keep_warns_and_changes_nothing_else() {
    let res = merge_parsed(&the_issue(), &opts(CodeCaseMode::Keep, Recode::new())).unwrap();
    let w = case_warnings(&res);
    assert_eq!(w.len(), 1, "{:?}", res.warnings);
    assert_eq!(w[0].heading.as_deref(), Some("TRIG_COND"));
    let set = w[0].case.as_ref().unwrap();
    let named: Vec<(&str, &[usize])> = set
        .spellings
        .iter()
        .map(|s| (s.code.as_str(), s.inputs.as_slice()))
        .collect();
    assert_eq!(
        named,
        vec![("Undisturbed", &[0][..]), ("UNDISTURBED", &[1][..])]
    );
    assert_eq!(set.suggested.as_deref(), Some("UNDISTURBED"));
    assert_eq!(set.resolved, None);
    assert!(
        w[0].message.contains("\"Undisturbed\" (input 0)"),
        "{}",
        w[0].message
    );
    // Today's output: both spellings survive and the case change is a revision.
    let codes: Vec<String> = abbr(&res, "TRIG_COND").into_iter().map(|r| r.0).collect();
    assert_eq!(codes, ["Undisturbed", "UNDISTURBED"]);
    let trig: Vec<_> = res.revisions.iter().filter(|r| r.group == "TRIG").collect();
    assert_eq!(trig.len(), 2);
    assert!(trig.iter().all(|r| r.changed == ["TRIG_COND"]));
}

#[test]
fn keep_is_the_default() {
    let a = merge_parsed(&the_issue(), &opts(CodeCaseMode::Keep, Recode::new())).unwrap();
    let mut o = opts(CodeCaseMode::Keep, Recode::new());
    o.on_code_case = MergeOpts::default().on_code_case;
    let b = merge_parsed(&the_issue(), &o).unwrap();
    assert_eq!(a.bytes, b.bytes);
}

#[test]
fn standard_settles_on_the_one_standard_spelling_before_reconciling() {
    let res = merge_parsed(&the_issue(), &opts(CodeCaseMode::Standard, Recode::new()))
        .expect("the merged file validates");
    let rows = abbr(&res, "TRIG_COND");
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0].0, "UNDISTURBED");
    assert!(
        column(&res, "TRIG", "TRIG_COND")
            .iter()
            .all(|c| c == "UNDISTURBED")
    );
    assert!(
        res.revisions.iter().all(|r| r.group != "TRIG"),
        "a settled case difference is not a revision: {:?}",
        res.revisions
    );
    let w = case_warnings(&res);
    assert_eq!(w.len(), 1, "the warning fires in every mode");
    assert_eq!(
        w[0].case.as_ref().unwrap().resolved.as_deref(),
        Some("UNDISTURBED")
    );
}

#[test]
fn a_recode_to_the_standard_spelling_matches_standard() {
    let std = merge_parsed(&the_issue(), &opts(CodeCaseMode::Standard, Recode::new())).unwrap();
    let rc = recode(&[("TRIG_COND", "Undisturbed", "UNDISTURBED")]);
    let named = merge_parsed(&the_issue(), &opts(CodeCaseMode::Keep, rc)).unwrap();
    assert_eq!(std.bytes, named.bytes);
    assert_eq!(std.revisions, named.revisions);
}

#[test]
fn a_recode_to_a_non_standard_target_works() {
    let rc = recode(&[("TRIG_COND", "UNDISTURBED", "Undisturbed")]);
    let res = merge_parsed(&the_issue(), &opts(CodeCaseMode::Keep, rc)).unwrap();
    let rows = abbr(&res, "TRIG_COND");
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0].0, "Undisturbed");
    assert!(
        column(&res, "TRIG", "TRIG_COND")
            .iter()
            .all(|c| c == "Undisturbed")
    );
}

#[test]
fn recode_takes_precedence_over_standard() {
    let rc = recode(&[("TRIG_COND", "UNDISTURBED", "Undisturbed")]);
    let res = merge_parsed(&the_issue(), &opts(CodeCaseMode::Standard, rc)).unwrap();
    assert!(
        column(&res, "TRIG", "TRIG_COND")
            .iter()
            .all(|c| c == "Undisturbed")
    );
}

#[test]
fn a_concatenated_value_is_rewritten_part_by_part() {
    // The non-standard spelling comes LAST, so it would win unrewritten.
    let files = [
        delivery("UNDISTURBED+B", ["U", "D"]),
        delivery("Undisturbed+B", ["U", "D"]),
    ];
    let res = merge_parsed(&files, &opts(CodeCaseMode::Standard, Recode::new()))
        .expect("the merged file validates");
    assert!(
        column(&res, "TRIG", "TRIG_COND")
            .iter()
            .all(|c| c == "UNDISTURBED+B"),
        "{:?}",
        column(&res, "TRIG", "TRIG_COND")
    );
}

/// A hand-written delivery declaring `codes` under `heading` in ABBR — for
/// sets the standard cannot settle, which need no data rows to be found.
fn abbr_only(heading: &str, codes: &[&str]) -> ParsedFile {
    let mut s = String::from(
        "\"GROUP\",\"PROJ\"\n\"HEADING\",\"PROJ_ID\"\n\"UNIT\",\"\"\n\"TYPE\",\"ID\"\n\
         \"DATA\",\"P1\"\n\n\"GROUP\",\"ABBR\"\n\
         \"HEADING\",\"ABBR_HDNG\",\"ABBR_CODE\",\"ABBR_DESC\"\n\
         \"UNIT\",\"\",\"\",\"\"\n\"TYPE\",\"X\",\"X\",\"X\"\n",
    );
    for c in codes {
        writeln!(s, "\"DATA\",\"{heading}\",\"{c}\",\"d\"").unwrap();
    }
    parse_str(&s).unwrap()
}

#[test]
fn standard_leaves_a_set_with_no_standard_match_and_still_warns() {
    let files = [
        abbr_only("TRIG_COND", &["Wobbly"]),
        abbr_only("TRIG_COND", &["WOBBLY"]),
    ];
    let o = MergeOpts {
        on_code_case: CodeCaseMode::Standard,
        ..MergeOpts::default()
    };
    let res = merge_parsed(&files, &o).unwrap();
    let codes: Vec<String> = abbr(&res, "TRIG_COND").into_iter().map(|r| r.0).collect();
    assert_eq!(codes, ["Wobbly", "WOBBLY"]);
    let w = case_warnings(&res);
    assert_eq!(w.len(), 1);
    let set = w[0].case.as_ref().unwrap();
    assert_eq!(
        (set.suggested.as_deref(), set.resolved.as_deref()),
        (None, None)
    );
    // The field is on the wire even when empty, so a caller can rely on it.
    let wire = serde_json::to_value(w[0]).unwrap();
    assert!(wire["case"]["suggested"].is_null(), "{wire}");
}

#[test]
fn standard_leaves_a_set_with_several_standard_matches() {
    // The standard lists both "CONSTANT HEAD" and "Constant Head".
    let files = [
        abbr_only("PTST_TYPE", &["constant head"]),
        abbr_only("PTST_TYPE", &["CONSTANT HEAD"]),
    ];
    let o = MergeOpts {
        on_code_case: CodeCaseMode::Standard,
        ..MergeOpts::default()
    };
    let res = merge_parsed(&files, &o).unwrap();
    let codes: Vec<String> = abbr(&res, "PTST_TYPE").into_iter().map(|r| r.0).collect();
    assert_eq!(codes, ["constant head", "CONSTANT HEAD"]);
    let w = case_warnings(&res);
    assert_eq!(w.len(), 1);
    assert_eq!(w[0].case.as_ref().unwrap().suggested, None);
}

#[test]
fn a_single_input_case_collision_is_warned() {
    let files = [
        abbr_only("TRIG_COND", &["Undisturbed", "UNDISTURBED"]),
        abbr_only("TRIG_COND", &[]),
    ];
    let res = merge_parsed(&files, &MergeOpts::default()).unwrap();
    let w = case_warnings(&res);
    assert_eq!(w.len(), 1);
    assert!(
        w[0].case
            .as_ref()
            .unwrap()
            .spellings
            .iter()
            .all(|s| s.inputs == [0])
    );
}

#[test]
fn a_rewrite_that_merges_two_distinct_rows_is_refused() {
    // Two SAMP rows that differ only in SAMP_TYPE "U" / "u": distinct KEYs as
    // written, one KEY once the case is settled.
    let files = [
        delivery("UNDISTURBED", ["U", "D"]),
        delivery("UNDISTURBED", ["u", "D"]),
    ];
    for o in [
        opts(CodeCaseMode::Standard, Recode::new()),
        opts(CodeCaseMode::Keep, recode(&[("SAMP_TYPE", "u", "U")])),
    ] {
        match merge_parsed(&files, &o) {
            Err(e @ MergeError::KeyCollision { .. }) => {
                let MergeError::KeyCollision { group, rows, .. } = &e else {
                    unreachable!()
                };
                assert_eq!(group, "SAMP");
                assert_eq!(rows.len(), 2, "{e}");
                assert!(e.to_string().contains("SAMP"), "{e}");
            }
            other => panic!("expected a key collision, got {other:?}"),
        }
    }
    // Keep merges them as today: two rows, case and all. (Not strict: the two
    // spellings are the inputs' own, and judging them is not this test's job.)
    let mut keep = opts(CodeCaseMode::Keep, Recode::new());
    keep.emit_mode = EmitMode::AutoFix;
    let res = merge_parsed(&files, &keep).unwrap();
    let types = column(&res, "SAMP", "SAMP_TYPE");
    assert!(types.contains(&"U".to_string()) && types.contains(&"u".to_string()));
}

#[test]
fn a_recode_the_inputs_cannot_honour_is_refused_by_name() {
    for (rc, named) in [
        (recode(&[("SAMP_REF", "1", "2")]), "SAMP_REF"),
        (recode(&[("TRIG_COND", "Disturbed", "D")]), "Disturbed"),
    ] {
        match merge_parsed(&the_issue(), &opts(CodeCaseMode::Keep, rc)) {
            Err(e @ MergeError::Recode(_)) => {
                assert!(e.to_string().contains(named), "{e}");
            }
            other => panic!("expected a recode error, got {other:?}"),
        }
    }
}
