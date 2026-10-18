//! The per-rule findings cap (#1089): `CheckOptions::max_findings_per_rule`.
//!
//! What a cap may change is what a run KEEPS. What it may not change is what the
//! run CONCLUDES: the verdict, the per-tier counts and the total. Every test here
//! compares a capped run against the uncapped run of the same file, so the
//! uncapped engine is the oracle and no expected count is written down by hand.

use laterite_ags4_validator::findings::{Collected, Findings, Truncated};
use laterite_ags4_validator::verdict::Verdict;
use laterite_ags4_validator::{CheckOptions, WorldScope, check_parsed_with_dict, parse};

/// A small AGS4.1 file written with LF line endings, so Rule 2a fires once per
/// line, plus a non-standard heading with no DICT group, so Rule 9 fires and Rule
/// 18 reads it. That last pair is the one place a rule reads another rule's
/// result, which a cap of `0` must not change.
fn many_findings(n_loca: usize) -> String {
    let q = |cells: &[&str]| {
        cells
            .iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(",")
    };
    let mut lines = vec![
        q(&["GROUP", "PROJ"]),
        q(&["HEADING", "PROJ_ID", "PROJ_NAME"]),
        q(&["UNIT", "", ""]),
        q(&["TYPE", "ID", "X"]),
        q(&["DATA", "P1", "Synthetic"]),
        String::new(),
        q(&["GROUP", "TRAN"]),
        q(&["HEADING", "TRAN_ISNO", "TRAN_AGS", "TRAN_RCON", "TRAN_DLIM"]),
        q(&["UNIT", "", "", "", ""]),
        q(&["TYPE", "X", "X", "X", "X"]),
        q(&["DATA", "1", "4.1", "+", "|"]),
        String::new(),
        q(&["GROUP", "LOCA"]),
        q(&["HEADING", "LOCA_ID", "LOCA_XTRA"]),
        q(&["UNIT", "", ""]),
        q(&["TYPE", "ID", "X"]),
    ];
    for i in 0..n_loca {
        lines.push(q(&["DATA", &format!("BH{i:04}"), "x"]));
    }
    lines.join("\n") + "\n"
}

fn run(text: &str, cap: Option<usize>) -> Collected {
    let parsed = parse::parse_str(text).expect("parses");
    let mut opts = CheckOptions::default();
    opts.include_warnings = true;
    opts.include_fyi = true;
    opts.max_findings_per_rule = cap;
    check_parsed_with_dict(&parsed, &opts, &WorldScope::None)
        .expect("validates")
        .0
}

fn tiers(c: &Collected) -> (usize, usize, usize, bool) {
    let v = Verdict::of(&c.findings, &c.truncated, false);
    (v.errors, v.warnings, v.fyi, v.is_valid())
}

/// What `cap` must cut from `full`, worked out from the uncapped run alone.
fn expected_cut(full: &Findings, cap: usize) -> Vec<(String, usize, usize)> {
    full.iter()
        .filter(|(_, v)| v.len() > cap)
        .map(|(k, v)| (k.clone(), cap, v.len()))
        .collect()
}

fn cut(t: &Truncated) -> Vec<(String, usize, usize)> {
    t.iter()
        .map(|(k, c)| (k.clone(), c.shown, c.total))
        .collect()
}

#[test]
fn the_fixture_exercises_more_than_one_rule_and_the_rule_9_to_18_read() {
    // Guards the tests below against a fixture that stopped producing what they
    // rely on: a test of a cut needs a rule big enough to cut, a rule small enough
    // not to be, and the one rule that reads another.
    let full = run(&many_findings(50), None);
    assert!(full.truncated.is_empty(), "an uncapped run cuts nothing");
    assert!(full.findings["AGS Format Rule 2a"].len() > 50);
    assert!(full.findings.contains_key("AGS Format Rule 9"));
    assert_eq!(full.findings["AGS Format Rule 18"].len(), 1);
}

#[test]
fn a_cap_keeps_each_rules_first_n_and_names_exactly_the_rules_it_cut() {
    let text = many_findings(50);
    let full = run(&text, None);
    for cap in [1, 3, 10] {
        let capped = run(&text, Some(cap));
        assert_eq!(
            cut(&capped.truncated),
            expected_cut(&full.findings, cap),
            "cap {cap}"
        );
        for (rule, all) in &full.findings {
            let kept = &capped.findings[rule];
            // The FIRST n, in the order the rule found them, not any n.
            assert_eq!(kept[..], all[..all.len().min(cap)], "cap {cap}, {rule}");
        }
        assert_eq!(
            capped.findings.keys().collect::<Vec<_>>(),
            full.findings.keys().collect::<Vec<_>>(),
            "cap {cap}: a capped rule keeps its bucket"
        );
    }
}

#[test]
fn storage_never_exceeds_the_cap() {
    let capped = run(&many_findings(500), Some(7));
    for (rule, kept) in &capped.findings {
        assert!(kept.len() <= 7, "{rule} kept {}", kept.len());
    }
    assert_eq!(capped.truncated["AGS Format Rule 2a"].shown, 7);
}

#[test]
fn the_verdict_and_every_count_are_the_same_under_any_cap() {
    let text = many_findings(50);
    let full = run(&text, None);
    for cap in [Some(0), Some(1), Some(3), Some(10_000)] {
        let capped = run(&text, cap);
        assert_eq!(tiers(&capped), tiers(&full), "cap {cap:?}");
        assert_eq!(capped.count(), full.count(), "cap {cap:?}");
    }
    assert_eq!(
        full.count(),
        laterite_ags4_validator::findings::count(&full.findings)
    );
}

#[test]
fn a_cap_of_zero_keeps_no_findings_and_still_counts_them_all() {
    let text = many_findings(50);
    let full = run(&text, None);
    let counts_only = run(&text, Some(0));
    assert!(
        counts_only.findings.is_empty(),
        "{:?}",
        counts_only.findings
    );
    assert_eq!(
        cut(&counts_only.truncated),
        expected_cut(&full.findings, 0),
        "every rule that fired is named, shown 0"
    );
    // Rule 18 asks whether Rule 9 fired. Under a cap of 0 Rule 9 stored nothing,
    // and Rule 18 must still see it.
    assert_eq!(counts_only.truncated["AGS Format Rule 18"].total, 1);
}

#[test]
fn a_cap_no_rule_reaches_changes_nothing() {
    let text = many_findings(5);
    let full = run(&text, None);
    let roomy = run(&text, Some(10_000));
    assert_eq!(roomy, full);
}
