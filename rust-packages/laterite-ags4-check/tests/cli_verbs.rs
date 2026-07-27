//! End-to-end tests for the `lat` binary's core verbs — `validate`, `fix`,
//! `rules`, `diff` (coverage campaign, Rust phase). Spawns the built binary and
//! asserts the real exit code AND the substance of the output — the specific
//! findings, that the three render formats agree on the count, that `fix`
//! actually removes the defect it names, and the diff delta values — never
//! merely "it runs" / "it's valid JSON". Exit codes: 0 clean · 1 findings ·
//! 3 not-found/io · 4 parse · 5 bad-args/dict · 6 schema.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::Value;

/// The canonical hand-authored fixtures (referenced, not copied — a second copy
/// is a second thing to drift). They live in the validator crate.
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../laterite-ags4-validator/tests/fixtures")
        .join(name)
}

fn scratch() -> PathBuf {
    // A fresh dir per call. Tests run in parallel threads of ONE process, so a
    // pid-only dir would let two tests collide on a shared filename (e.g. two
    // `clean.ags`) — one test's `fs::copy` truncating the file mid-read of
    // another. The counter isolates every scratch to its own caller.
    static N: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "lat_verbs_{}_{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn lat<I, S>(args: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    Command::new(env!("CARGO_BIN_EXE_lat"))
        .args(args)
        .output()
        .expect("spawn lat")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}
fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

/// The `validate --json` shape is `{file, findings: {rule: [..], ..}}`; the total
/// finding count is the sum of the per-rule lists (what the plain "N finding(s)"
/// header and the ndjson line count must both agree with).
fn json_finding_count(v: &Value) -> usize {
    v["findings"].as_object().map_or(0, |m| {
        m.values().filter_map(|l| l.as_array()).map(Vec::len).sum()
    })
}

fn has_rule(v: &Value, rule: &str) -> bool {
    v["findings"].get(rule).is_some()
}

// A standalone LOCA group appended to make a file that differs by one group.
const LOCA_GROUP: &str = "\r\n\"GROUP\",\"LOCA\"\r\n\"HEADING\",\"LOCA_ID\"\r\n\"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"BH1\"\r\n";

// --- validate ---------------------------------------------------------------

#[test]
fn validate_clean_file_exits_0() {
    let o = lat(["validate", fixture("clean_minimal.ags").to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0), "stderr: {}", stderr(&o));
    assert!(stdout(&o).contains("clean (0 findings)"), "{}", stdout(&o));
}

#[test]
fn bare_file_is_validate_shorthand() {
    // `lat <file>` with no verb runs validate — same verdict as the explicit form.
    let bare = lat([fixture("clean_minimal.ags").to_str().unwrap()]);
    let explicit = lat(["validate", fixture("clean_minimal.ags").to_str().unwrap()]);
    assert_eq!(bare.status.code(), Some(0), "stderr: {}", stderr(&bare));
    assert_eq!(stdout(&bare), stdout(&explicit));
}

#[test]
fn validate_dirty_file_names_the_specific_rules() {
    // rule5_unquoted.ags carries exactly four findings: a missing TRAN/UNIT/TYPE
    // group (Rules 14/15/17) plus the unquoted DATA field (Rule 5, line 5).
    let o = lat(["validate", fixture("rule5_unquoted.ags").to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(1), "stderr: {}", stderr(&o));
    let out = stdout(&o);
    assert!(out.contains("4 finding(s)"), "{out}");
    // the RIGHT problem is named, not just "some finding"
    assert!(
        out.contains("not enclosed in double quotes"),
        "Rule 5 desc missing: {out}"
    );
    for rule in ["5", "14", "15", "17"] {
        assert!(out.contains(rule), "rule {rule} absent from table: {out}");
    }
}

#[test]
fn validate_formats_agree_on_the_finding_set() {
    // The plain header count, the --json total, and the --ndjson line count must
    // all describe the SAME finding set — the contract that keeps a scripted
    // consumer and a human reading the same file from disagreeing.
    let f = fixture("rule5_unquoted.ags");
    let plain = lat(["validate", f.to_str().unwrap()]);
    let json = lat(["validate", "--json", f.to_str().unwrap()]);
    let ndjson = lat(["validate", "--ndjson", f.to_str().unwrap()]);
    assert_eq!(plain.status.code(), Some(1));

    let v: Value = serde_json::from_str(&stdout(&json)).expect("valid JSON");
    let json_n = json_finding_count(&v);
    let ndjson_n = stdout(&ndjson)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .count();

    assert_eq!(json_n, 4, "json findings: {}", stdout(&json));
    assert_eq!(ndjson_n, 4, "ndjson lines: {}", stdout(&ndjson));
    assert!(stdout(&plain).contains("4 finding(s)"));
    // the Rule 5 finding is in the structured output with its real line number
    assert!(has_rule(&v, "AGS Format Rule 5"));
    assert_eq!(v["findings"]["AGS Format Rule 5"][0]["line"], 5);
}

#[test]
fn validate_warnings_are_on_by_default_and_no_warnings_drops_them() {
    // rule18_malformed_dict.ags carries a warning-tier finding on top of its
    // errors. Warnings are ON by default (so the default run sees it) and
    // --no-warnings drops exactly that one — pinning the flag → CheckOptions
    // .include_warnings wiring that mutation testing showed was unasserted.
    let f = fixture("rule18_malformed_dict.ags");
    let default: Value =
        serde_json::from_str(&stdout(&lat(["validate", "--json", f.to_str().unwrap()]))).unwrap();
    let quiet: Value = serde_json::from_str(&stdout(&lat([
        "validate",
        "--no-warnings",
        "--json",
        f.to_str().unwrap(),
    ])))
    .unwrap();
    assert_eq!(json_finding_count(&default), 4, "warnings on by default");
    assert_eq!(
        json_finding_count(&quiet),
        3,
        "--no-warnings drops the warning"
    );
}

#[test]
fn validate_missing_file_exits_3() {
    let o = lat(["validate", "/no/such/file_xyz.ags"]);
    assert_eq!(o.status.code(), Some(3));
    assert!(stderr(&o).contains("read"), "{}", stderr(&o));
}

#[test]
fn validate_out_writes_the_same_report_it_would_print() {
    // --out redirects the plain report to a file; the file must carry the real
    // report (the findings), not a stub — and stdout keeps only the confirmation.
    let d = scratch();
    let out = d.join("report.txt");
    let o = lat([
        "validate",
        fixture("rule5_unquoted.ags").to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert_eq!(o.status.code(), Some(1));
    assert!(stdout(&o).contains("wrote 4 finding(s)"), "{}", stdout(&o));
    let written = std::fs::read_to_string(&out).unwrap();
    assert!(
        written.contains("not enclosed in double quotes") && written.contains("4 finding(s)"),
        "redirected report is not the real report: {written}"
    );
}

#[test]
fn validate_json_out_tees_the_same_json_as_stdout() {
    // The tee'd artifact must be byte-identical to what `--json` prints — one
    // report, two destinations, no divergence.
    let d = scratch();
    let j = d.join("report.json");
    let teed = lat([
        "validate",
        fixture("rule5_unquoted.ags").to_str().unwrap(),
        "--json-out",
        j.to_str().unwrap(),
    ]);
    assert_eq!(teed.status.code(), Some(1));
    assert!(
        stderr(&teed).contains("JSON written to"),
        "{}",
        stderr(&teed)
    );
    let stdout_json = lat([
        "validate",
        "--json",
        fixture("rule5_unquoted.ags").to_str().unwrap(),
    ]);
    let artifact: Value = serde_json::from_str(&std::fs::read_to_string(&j).unwrap()).unwrap();
    let printed: Value = serde_json::from_str(&stdout(&stdout_json)).unwrap();
    assert_eq!(artifact, printed, "tee'd JSON differs from --json stdout");
}

// --- fix --------------------------------------------------------------------

#[test]
fn fix_clean_file_applies_nothing_and_stays_clean() {
    let d = scratch();
    let src = d.join("clean.ags");
    std::fs::copy(fixture("clean_minimal.ags"), &src).unwrap();
    let o = lat(["fix", src.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0), "stderr: {}", stderr(&o));
    assert!(stdout(&o).contains("no fixes applicable"), "{}", stdout(&o));
    assert!(stdout(&o).contains("clean (0 findings)"), "{}", stdout(&o));
    // the sibling is a faithful copy of an already-clean file
    let sibling = std::fs::read(d.join("clean.fixed.ags")).unwrap();
    assert_eq!(sibling, std::fs::read(&src).unwrap());
}

#[test]
fn fix_repairs_the_defect_it_names() {
    // rule8_dp_wrong_precision.ags has a 2DP value at the wrong precision — a
    // mechanically-safe reformat. Assert fix (a) names that exact fix, and
    // (b) actually removes the Rule 8 finding from the repaired file.
    let d = scratch();
    let src = d.join("r8.ags");
    std::fs::copy(fixture("rule8_dp_wrong_precision.ags"), &src).unwrap();

    // before: the file HAS a Rule 8 finding
    let before: Value =
        serde_json::from_str(&stdout(&lat(["validate", "--json", src.to_str().unwrap()]))).unwrap();
    assert!(has_rule(&before, "AGS Format Rule 8"), "precondition");

    let fixed = lat(["fix", "--json", "--in-place", src.to_str().unwrap()]);
    let report: Value = serde_json::from_str(&stdout(&fixed)).expect("fix --json");
    assert_eq!(report["applied"][0]["kind"], "reformat_numeric");
    assert_eq!(report["applied"][0]["rule"], "AGS Format Rule 8");

    // after: the repaired file no longer trips Rule 8
    let after: Value =
        serde_json::from_str(&stdout(&lat(["validate", "--json", src.to_str().unwrap()]))).unwrap();
    assert!(
        !has_rule(&after, "AGS Format Rule 8"),
        "fix did not remove the Rule 8 defect"
    );
}

#[test]
fn fix_in_place_overwrites_source_without_a_sibling() {
    let d = scratch();
    let src = d.join("inplace.ags");
    std::fs::copy(fixture("rule8_dp_wrong_precision.ags"), &src).unwrap();
    let before = std::fs::read(&src).unwrap();
    let o = lat(["fix", "--in-place", src.to_str().unwrap()]);
    assert!(
        o.status.code() == Some(0) || o.status.code() == Some(1),
        "stderr: {}",
        stderr(&o)
    );
    assert!(
        !d.join("inplace.fixed.ags").exists(),
        "no sibling on --in-place"
    );
    // the source itself was rewritten (the reformat landed)
    assert_ne!(std::fs::read(&src).unwrap(), before, "source unchanged");
}

#[test]
fn fix_missing_file_exits_3() {
    let o = lat(["fix", "/no/such/file_xyz.ags"]);
    assert_eq!(o.status.code(), Some(3));
    assert!(stderr(&o).contains("error"), "{}", stderr(&o));
}

#[test]
fn fix_out_write_failure_exits_3() {
    // write_atomic creates missing parent dirs, so a genuine failure needs a
    // parent that is a regular FILE (creating a dir under it is ENOTDIR).
    let d = scratch();
    let src = d.join("in2.ags");
    std::fs::copy(fixture("clean_minimal.ags"), &src).unwrap();
    let blocker = d.join("blocker");
    std::fs::write(&blocker, b"i am a file, not a dir").unwrap();
    let o = lat([
        "fix",
        src.to_str().unwrap(),
        "--fix-out",
        blocker.join("out.ags").to_str().unwrap(),
    ]);
    assert_eq!(o.status.code(), Some(3), "stderr: {}", stderr(&o));
    assert!(stderr(&o).contains("writing"), "{}", stderr(&o));
}

#[test]
fn fix_json_residual_counts_warnings_and_drives_the_exit_code() {
    // rule18_malformed_dict carries three errors + one warning, none mechanically
    // fixable, so fix leaves all four. `fix` runs with include_warnings=true, so
    // the residual is 4 (not 3) and the exit is 1; a clean file leaves 0 and exits
    // 0. Pins both the warnings-in-residual wiring and the sign of the --json exit
    // code — two live behaviours mutation testing showed were unasserted.
    let d = scratch();
    let dirty = d.join("r18.ags");
    std::fs::copy(fixture("rule18_malformed_dict.ags"), &dirty).unwrap();
    let o = lat(["fix", "--json", dirty.to_str().unwrap()]);
    let report: Value = serde_json::from_str(&stdout(&o)).expect("fix --json");
    assert_eq!(
        report["residual"],
        4,
        "residual must count the warning tier: {}",
        stdout(&o)
    );
    assert_eq!(o.status.code(), Some(1), "residual > 0 must exit 1");

    let clean = d.join("clean.ags");
    std::fs::copy(fixture("clean_minimal.ags"), &clean).unwrap();
    let c = lat(["fix", "--json", clean.to_str().unwrap()]);
    let creport: Value = serde_json::from_str(&stdout(&c)).expect("fix --json clean");
    assert_eq!(creport["residual"], 0, "a clean file has no residual");
    assert_eq!(c.status.code(), Some(0), "residual == 0 must exit 0");
}

#[test]
fn fix_risky_hint_shows_only_when_risky_fixes_are_withheld() {
    // rule1_non_ascii has an intent-guessing (risky) transliteration fix that a
    // plain `fix` withholds — the hint must name it and its count. A clean file
    // has no withheld fix and must NOT print the hint. Pins the
    // `!risky && risky_available > 0` gate the sweep found unasserted.
    let d = scratch();
    let risky = d.join("r1.ags");
    std::fs::copy(fixture("rule1_non_ascii.ags"), &risky).unwrap();
    let o = lat(["fix", risky.to_str().unwrap()]);
    assert!(
        stdout(&o).contains("1 more fixable with --fix-risky"),
        "risky hint missing: {}",
        stdout(&o)
    );

    let clean = d.join("clean.ags");
    std::fs::copy(fixture("clean_minimal.ags"), &clean).unwrap();
    let c = lat(["fix", clean.to_str().unwrap()]);
    assert!(
        !stdout(&c).contains("fix-risky"),
        "hint shown for a file with no withheld fix: {}",
        stdout(&c)
    );
}

// --- rules ------------------------------------------------------------------

#[test]
fn rules_table_and_json_agree_on_the_catalogue() {
    let table = lat(["rules"]);
    let json = lat(["rules", "--json"]);
    assert_eq!(table.status.code(), Some(0));
    assert_eq!(json.status.code(), Some(0));

    let v: Value = serde_json::from_str(&stdout(&json)).expect("rules --json");
    let rules = v["rules"].as_array().expect("rules array");
    assert!(
        rules.len() > 20,
        "catalogue looks truncated: {}",
        rules.len()
    );

    // the human table is the same catalogue: its header, a distinctive rule title
    // (Rule 1 "Character Set"), and the multi-part rule ids (2a/10a) all appear.
    let table_out = stdout(&table);
    assert!(table_out.contains("Rule") && table_out.contains("Severity"));
    let r1_title = rules[0]["title"].as_str().unwrap();
    assert!(
        table_out.contains(r1_title),
        "table missing '{r1_title}': {table_out}"
    );
    for id in ["2a", "10a"] {
        assert!(
            rules.iter().any(|r| r["rule"] == id),
            "json missing rule {id}"
        );
        assert!(table_out.contains(id), "table missing rule {id}");
    }
    // Rule 8 (numeric precision) is fixable — the flag the `fix` verb relies on,
    // and which `fix_repairs_the_defect_it_names` exercises end-to-end.
    let r8 = rules
        .iter()
        .find(|r| r["rule"] == "8")
        .expect("Rule 8 in catalogue");
    assert_eq!(r8["fixable"], true);
}

// --- diff -------------------------------------------------------------------

fn write_pair() -> (PathBuf, PathBuf, PathBuf) {
    let d = scratch();
    let base = d.join("base.ags");
    let plus = d.join("plus.ags");
    let clean = std::fs::read(fixture("clean_minimal.ags")).unwrap();
    std::fs::write(&base, &clean).unwrap();
    let mut extended = clean.clone();
    extended.extend_from_slice(LOCA_GROUP.as_bytes());
    std::fs::write(&plus, &extended).unwrap();
    (d, base, plus)
}

#[test]
fn diff_identical_files_reports_zero_delta() {
    let (_d, base, _plus) = write_pair();
    let o = lat(["diff", base.to_str().unwrap(), base.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0), "stderr: {}", stderr(&o));
    let out = stdout(&o);
    assert!(out.contains("total: +0 added"), "{out}");
    assert!(
        !out.contains("groups added:"),
        "no groups should differ: {out}"
    );
}

#[test]
fn diff_names_the_added_and_removed_group() {
    let (_d, base, plus) = write_pair();
    // base → plus adds LOCA; plus → base removes it. The delta is symmetric.
    let added = lat(["diff", base.to_str().unwrap(), plus.to_str().unwrap()]);
    let a = stdout(&added);
    assert_eq!(added.status.code(), Some(0));
    assert!(a.contains("groups added:   LOCA"), "{a}");
    assert!(!a.contains("groups removed:"), "{a}");

    let removed = lat(["diff", plus.to_str().unwrap(), base.to_str().unwrap()]);
    let r = stdout(&removed);
    assert!(r.contains("groups removed: LOCA"), "{r}");
    assert!(!r.contains("groups added:"), "{r}");
}

#[test]
fn diff_json_carries_the_group_delta() {
    let (_d, base, plus) = write_pair();
    let o = lat([
        "diff",
        "--json",
        base.to_str().unwrap(),
        plus.to_str().unwrap(),
    ]);
    assert_eq!(o.status.code(), Some(0));
    let v: Value = serde_json::from_str(&stdout(&o)).expect("diff --json");
    // the structured delta agrees with the human summary: LOCA was added
    let added = v["groups_added"].as_array().expect("groups_added");
    assert_eq!(added.iter().filter(|g| *g == "LOCA").count(), 1, "{v}");
}

#[test]
fn diff_missing_file_exits_3() {
    let (_d, base, _plus) = write_pair();
    let o = lat(["diff", base.to_str().unwrap(), "/no/such/file_xyz.ags"]);
    assert_eq!(o.status.code(), Some(3));
}

#[test]
fn diff_unparseable_file_exits_4() {
    let d = scratch();
    let garbage = d.join("garbage.ags");
    std::fs::write(&garbage, "not ags4 at all\r\n").unwrap();
    let (_d, base, _plus) = write_pair();
    let o = lat(["diff", base.to_str().unwrap(), garbage.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(4), "stderr: {}", stderr(&o));
}

// --- read -------------------------------------------------------------------

#[test]
fn read_lists_group_codes_in_source_order() {
    // No group named → the file's group codes, one per line, in source order.
    let o = lat(["read", fixture("clean_minimal.ags").to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0), "stderr: {}", stderr(&o));
    assert_eq!(stdout(&o), "PROJ\nTRAN\nUNIT\nTYPE\n");
}

#[test]
fn read_json_lists_groups_as_an_array() {
    let o = lat([
        "read",
        "--json",
        fixture("clean_minimal.ags").to_str().unwrap(),
    ]);
    assert_eq!(o.status.code(), Some(0));
    let v: Value = serde_json::from_str(&stdout(&o)).expect("read --json");
    let codes: Vec<&str> = v
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap())
        .collect();
    assert_eq!(codes, ["PROJ", "TRAN", "UNIT", "TYPE"]);
}

#[test]
fn read_renders_a_named_groups_cells() {
    // A named group → its rows; the projected cells (the id + the project name)
    // appear, in the group's heading order.
    let o = lat([
        "read",
        fixture("clean_minimal.ags").to_str().unwrap(),
        "PROJ",
    ]);
    assert_eq!(o.status.code(), Some(0), "stderr: {}", stderr(&o));
    let out = stdout(&o);
    assert!(
        out.contains("PROJ_ID") && out.contains("PROJ_NAME"),
        "headers: {out}"
    );
    assert!(
        out.contains("P1") && out.contains("Clean minimal AGS4 fixture"),
        "cells: {out}"
    );
}

#[test]
fn read_csv_quotes_the_comma_bearing_cell() {
    // --csv is RFC-4180: the header row is the headings, and the project name
    // (which contains commas) is double-quoted — real CSV, not a naive join.
    let o = lat([
        "read",
        "--csv",
        fixture("clean_minimal.ags").to_str().unwrap(),
        "PROJ",
    ]);
    assert_eq!(o.status.code(), Some(0));
    let out = stdout(&o);
    assert_eq!(out.lines().next(), Some("PROJ_ID,PROJ_NAME"));
    assert!(
        out.contains("P1,\"Clean minimal"),
        "row not RFC-4180 quoted: {out}"
    );
}

#[test]
fn read_unknown_group_exits_4_and_lists_whats_present() {
    // A named-but-absent group is a schema miss (exit 4) that names the groups
    // that ARE present, so the user can retry — not a bare failure.
    let o = lat([
        "read",
        fixture("clean_minimal.ags").to_str().unwrap(),
        "ZZZZ",
    ]);
    assert_eq!(o.status.code(), Some(4), "stderr: {}", stderr(&o));
    let err = stderr(&o);
    assert!(err.contains("not found"), "{err}");
    assert!(err.contains("PROJ"), "present-list missing: {err}");
}

#[test]
fn read_missing_file_exits_3_not_4() {
    // A genuine I/O miss is exit 3 — read pre-checks existence so it isn't mapped
    // to the codec's schema-error 4.
    let o = lat(["read", "/no/such/file_xyz.ags"]);
    assert_eq!(o.status.code(), Some(3), "stderr: {}", stderr(&o));
    assert!(stderr(&o).contains("not found"), "{}", stderr(&o));
}

#[test]
fn read_out_writes_the_body_and_keeps_stdout_clean() {
    // --out sends the rendered body to a file (with a stderr note); stdout stays
    // empty so a pipeline sees only what it redirected.
    let d = scratch();
    let out = d.join("proj.csv");
    let o = lat([
        "read",
        "--csv",
        fixture("clean_minimal.ags").to_str().unwrap(),
        "PROJ",
        "--out",
        out.to_str().unwrap(),
    ]);
    assert_eq!(o.status.code(), Some(0), "stderr: {}", stderr(&o));
    assert!(stdout(&o).is_empty(), "stdout not clean: {}", stdout(&o));
    assert!(stderr(&o).contains("written to"), "{}", stderr(&o));
    let written = std::fs::read_to_string(&out).unwrap();
    assert_eq!(written.lines().next(), Some("PROJ_ID,PROJ_NAME"));
}

// --- merge ------------------------------------------------------------------

/// A base file and a revision of it: the revision changes PROJ P1's `PROJ_NAME`
/// (a non-KEY field), so a KEY-aware merge reports exactly one row revision, won
/// by the second (newer) file.
fn write_merge_pair() -> (PathBuf, PathBuf, PathBuf) {
    let dir = scratch();
    let base_path = dir.join("a.ags");
    let rev_path = dir.join("b.ags");
    let base = std::fs::read_to_string(fixture("clean_minimal.ags")).unwrap();
    let revised = base.replace(
        "Clean minimal AGS4 fixture (hand-authored, MIT, ours)",
        "Revised project name",
    );
    assert_ne!(
        base, revised,
        "fixture project name changed — update this test"
    );
    std::fs::write(&base_path, &base).unwrap();
    std::fs::write(&rev_path, &revised).unwrap();
    (dir, base_path, rev_path)
}

#[test]
fn merge_reconciles_a_revision_last_wins() {
    let (dir, base, rev) = write_merge_pair();
    let out = dir.join("merged.ags");
    let o = lat([
        "merge",
        base.to_str().unwrap(),
        rev.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert_eq!(o.status.code(), Some(0), "stderr: {}", stderr(&o));
    let summary = stdout(&o);
    assert!(summary.contains("merged 2 files"), "{summary}");
    assert!(
        summary.contains("1 row revision"),
        "revision not reported: {summary}"
    );
    assert!(
        summary.contains("PROJ") && summary.contains("PROJ_NAME"),
        "revision detail: {summary}"
    );
    // last wins: the merged file carries the revision's name, not the base's.
    let merged = std::fs::read_to_string(&out).unwrap();
    assert!(
        merged.contains("Revised project name"),
        "last-wins not applied: {merged}"
    );
    assert!(
        !merged.contains("hand-authored"),
        "old value survived: {merged}"
    );
}

#[test]
fn merge_json_reports_the_revision_and_the_tran_warning() {
    let (dir, base, rev) = write_merge_pair();
    let out = dir.join("merged.ags");
    let o = lat([
        "merge",
        "--json",
        base.to_str().unwrap(),
        rev.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert_eq!(o.status.code(), Some(0), "stderr: {}", stderr(&o));
    let v: Value = serde_json::from_str(&stdout(&o)).expect("merge --json");
    // exactly the PROJ P1 revision, won by the second (index 1) file
    let revs = v["revisions"].as_array().expect("revisions");
    assert_eq!(revs.len(), 1, "{v}");
    assert_eq!(revs[0]["group"], "PROJ");
    assert_eq!(revs[0]["key"][0], "P1");
    assert_eq!(revs[0]["changed"][0], "PROJ_NAME");
    assert_eq!(revs[0]["winner_file"], 1);
    // no TRAN stamp supplied → the documented fallback warning
    let warns = v["warnings"].as_array().expect("warnings");
    assert!(
        warns
            .iter()
            .any(|entry| entry["kind"] == "tran_not_stamped"),
        "{v}"
    );
    // the reported byte count is the file actually written
    assert_eq!(
        v["bytes"].as_u64().unwrap(),
        std::fs::metadata(&out).unwrap().len()
    );
}

#[test]
fn merge_missing_file_exits_3() {
    let (dir, base, _rev) = write_merge_pair();
    let out = dir.join("merged.ags");
    let o = lat([
        "merge",
        base.to_str().unwrap(),
        "/no/such/file_xyz.ags",
        "--out",
        out.to_str().unwrap(),
    ]);
    assert_eq!(o.status.code(), Some(3), "stderr: {}", stderr(&o));
}

#[test]
fn merge_unparseable_file_exits_4() {
    let (dir, base, _rev) = write_merge_pair();
    let garbage = dir.join("garbage.ags");
    std::fs::write(&garbage, "not ags4 at all\r\n").unwrap();
    let out = dir.join("merged.ags");
    let o = lat([
        "merge",
        base.to_str().unwrap(),
        garbage.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert_eq!(o.status.code(), Some(4), "stderr: {}", stderr(&o));
}

#[test]
fn merge_synthesises_a_tran_stamp_when_issue_and_date_are_given() {
    // With BOTH --tran-issue and --tran-date, merge writes a fresh merge-TRAN — so
    // the "not stamped" fallback warning disappears and the date + a "Merged from
    // N deliveries" note land in the file's TRAN row. Without both, no stamp is
    // made. Pins the (Some, Some) synthesis arm and the `tran` MergeOpts field the
    // sweep found unasserted.
    let (dir, base, rev) = write_merge_pair();
    let out = dir.join("stamped.ags");
    let o = lat([
        "merge",
        "--json",
        base.to_str().unwrap(),
        rev.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
        "--tran-issue",
        "7",
        "--tran-date",
        "2026-07-27",
    ]);
    assert_eq!(o.status.code(), Some(0), "stderr: {}", stderr(&o));
    let v: Value = serde_json::from_str(&stdout(&o)).expect("merge --json");
    // a supplied stamp silences the fallback warning...
    let warns = v["warnings"].as_array().expect("warnings");
    assert!(
        !warns
            .iter()
            .any(|entry| entry["kind"] == "tran_not_stamped"),
        "a supplied stamp must silence the fallback warning: {v}"
    );
    // ...and lands in the merged file's TRAN row.
    let stamped = std::fs::read_to_string(&out).unwrap();
    assert!(
        stamped.contains("2026-07-27"),
        "TRAN date not stamped: {stamped}"
    );
    assert!(
        stamped.contains("Merged from 2 deliveries"),
        "merge TRAN_REM missing: {stamped}"
    );
}

// --- transport (pack / unpack / lock / unlock) ------------------------------

fn pw_file(dir: &Path, name: &str, pw: &str) -> PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, pw).unwrap();
    p
}

#[test]
fn pack_then_unpack_round_trips() {
    // The zstd envelope is lossless: unpack(pack(x)) == x, byte for byte.
    let dir = scratch();
    let src = fixture("clean_minimal.ags");
    let packed = dir.join("c.lat.zst");
    let out = dir.join("c.ags");
    let p = lat(["pack", src.to_str().unwrap(), packed.to_str().unwrap()]);
    assert_eq!(p.status.code(), Some(0), "stderr: {}", stderr(&p));
    assert!(stderr(&p).contains("packed"), "{}", stderr(&p));
    let u = lat(["unpack", packed.to_str().unwrap(), out.to_str().unwrap()]);
    assert_eq!(u.status.code(), Some(0), "stderr: {}", stderr(&u));
    assert_eq!(
        std::fs::read(&src).unwrap(),
        std::fs::read(&out).unwrap(),
        "pack → unpack changed the bytes"
    );
}

#[test]
fn lock_then_unlock_round_trips_with_a_passphrase() {
    // The age passphrase envelope: unlock(lock(x, pw), pw) == x. --log-n 2 keeps
    // the scrypt KDF fast for the test (18 is the shipped default).
    let dir = scratch();
    let src = fixture("clean_minimal.ags");
    let pw = pw_file(&dir, "pw.txt", "hunter2");
    let locked = dir.join("c.lat.age");
    let out = dir.join("c.ags");
    let l = lat([
        "lock",
        src.to_str().unwrap(),
        locked.to_str().unwrap(),
        "--password-file",
        pw.to_str().unwrap(),
        "--log-n",
        "2",
    ]);
    assert_eq!(l.status.code(), Some(0), "stderr: {}", stderr(&l));
    let u = lat([
        "unlock",
        locked.to_str().unwrap(),
        out.to_str().unwrap(),
        "--password-file",
        pw.to_str().unwrap(),
    ]);
    assert_eq!(u.status.code(), Some(0), "stderr: {}", stderr(&u));
    assert_eq!(
        std::fs::read(&src).unwrap(),
        std::fs::read(&out).unwrap(),
        "lock → unlock changed the bytes"
    );
    // the locked file is genuinely encrypted, not the plaintext copied through
    assert_ne!(
        std::fs::read(&locked).unwrap(),
        std::fs::read(&src).unwrap()
    );
}

#[test]
fn unlock_with_the_wrong_passphrase_exits_6() {
    let dir = scratch();
    let src = fixture("clean_minimal.ags");
    let right = pw_file(&dir, "right.txt", "correct-horse");
    let wrong = pw_file(&dir, "wrong.txt", "battery-staple");
    let locked = dir.join("c.lat.age");
    let out = dir.join("c.ags");
    let l = lat([
        "lock",
        src.to_str().unwrap(),
        locked.to_str().unwrap(),
        "--password-file",
        right.to_str().unwrap(),
        "--log-n",
        "2",
    ]);
    assert_eq!(l.status.code(), Some(0), "stderr: {}", stderr(&l));
    let u = lat([
        "unlock",
        locked.to_str().unwrap(),
        out.to_str().unwrap(),
        "--password-file",
        wrong.to_str().unwrap(),
    ]);
    assert_eq!(u.status.code(), Some(6), "stderr: {}", stderr(&u));
    assert!(stderr(&u).contains("decrypt"), "{}", stderr(&u));
}

#[test]
fn pack_missing_input_exits_3() {
    let dir = scratch();
    let out = dir.join("x.zst");
    let p = lat(["pack", "/no/such/file_xyz.ags", out.to_str().unwrap()]);
    assert_eq!(p.status.code(), Some(3), "stderr: {}", stderr(&p));
    assert!(stderr(&p).contains("not found"), "{}", stderr(&p));
}

#[test]
fn unpack_a_non_envelope_exits_6() {
    // A plain .ags is not a zstd frame → a schema/transport error, not a crash.
    let dir = scratch();
    let out = dir.join("x.ags");
    let u = lat([
        "unpack",
        fixture("clean_minimal.ags").to_str().unwrap(),
        out.to_str().unwrap(),
    ]);
    assert_eq!(u.status.code(), Some(6), "stderr: {}", stderr(&u));
}

#[test]
fn lock_reads_the_passphrase_from_the_env_var() {
    // With no --password-file, the passphrase comes from $LAT_TRANSPORT_PASSWORD.
    // Set per-child via Command::env (NOT env::set_var), so there is no cross-test
    // race. Round-trips through unlock reading the same env var.
    let dir = scratch();
    let src = fixture("clean_minimal.ags");
    let locked = dir.join("c.lat.age");
    let out = dir.join("c.ags");
    let lock = Command::new(env!("CARGO_BIN_EXE_lat"))
        .args([
            "lock",
            src.to_str().unwrap(),
            locked.to_str().unwrap(),
            "--log-n",
            "2",
        ])
        .env("LAT_TRANSPORT_PASSWORD", "hunter2")
        .output()
        .expect("spawn lat");
    assert_eq!(lock.status.code(), Some(0), "stderr: {}", stderr(&lock));
    let unlock = Command::new(env!("CARGO_BIN_EXE_lat"))
        .args(["unlock", locked.to_str().unwrap(), out.to_str().unwrap()])
        .env("LAT_TRANSPORT_PASSWORD", "hunter2")
        .output()
        .expect("spawn lat");
    assert_eq!(unlock.status.code(), Some(0), "stderr: {}", stderr(&unlock));
    assert_eq!(
        std::fs::read(&src).unwrap(),
        std::fs::read(&out).unwrap(),
        "env-passphrase round-trip differs"
    );
}

// --- certify / cert (the .ags.idx certificate) ------------------------------

#[test]
fn certify_a_clean_file_writes_a_valid_cert() {
    let dir = scratch();
    let src = dir.join("c.ags");
    std::fs::copy(fixture("clean_minimal.ags"), &src).unwrap();
    let o = lat(["certify", src.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(0), "stderr: {}", stderr(&o));
    assert!(
        stdout(&o).contains("certificate written to"),
        "{}",
        stdout(&o)
    );
    // the sidecar is real: format 2, the file's true size, the laterite validator
    let cert = dir.join("c.ags.idx");
    let v: Value =
        serde_json::from_str(&std::fs::read_to_string(&cert).unwrap()).expect("cert is JSON");
    assert_eq!(v["version"], 2);
    assert_eq!(v["file"]["size"], 726);
    assert_eq!(v["validation"]["validator"], "laterite_ags4");
}

#[test]
fn certify_a_file_with_errors_is_not_certifiable() {
    let dir = scratch();
    let src = dir.join("dirty.ags");
    std::fs::copy(fixture("rule5_unquoted.ags"), &src).unwrap();
    let o = lat(["certify", src.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(1), "stderr: {}", stderr(&o));
    let err = stderr(&o);
    assert!(
        err.contains("cannot certify") && err.contains("error-severity"),
        "{err}"
    );
    assert!(
        err.contains("lat validate"),
        "no pointer to validate: {err}"
    );
    // no certificate is minted for an uncertifiable file
    assert!(
        !dir.join("dirty.ags.idx").exists(),
        "a cert was written despite errors"
    );
}

#[test]
fn certify_missing_file_exits_3() {
    let o = lat(["certify", "/no/such/file_xyz.ags"]);
    assert_eq!(o.status.code(), Some(3), "stderr: {}", stderr(&o));
}

#[test]
fn validate_index_with_a_fresh_cert_skips_the_engine() {
    let dir = scratch();
    let src = dir.join("c.ags");
    std::fs::copy(fixture("clean_minimal.ags"), &src).unwrap();
    let cert = dir.join("c.ags.idx");
    assert_eq!(
        lat(["certify", src.to_str().unwrap()]).status.code(),
        Some(0)
    );

    let o = lat([
        "validate",
        "--index",
        cert.to_str().unwrap(),
        src.to_str().unwrap(),
    ]);
    assert_eq!(o.status.code(), Some(0), "stderr: {}", stderr(&o));
    // the engine was skipped on the certificate's authority, and the run says so
    assert!(
        stderr(&o).contains("certified clean by") && stderr(&o).contains("rule engine skipped"),
        "no skip note: {}",
        stderr(&o)
    );
    assert!(stdout(&o).contains("clean (0 findings)"), "{}", stdout(&o));
}

#[test]
fn validate_index_with_a_stale_cert_revalidates() {
    let dir = scratch();
    let src = dir.join("c.ags");
    std::fs::copy(fixture("clean_minimal.ags"), &src).unwrap();
    let cert = dir.join("c.ags.idx");
    assert_eq!(
        lat(["certify", src.to_str().unwrap()]).status.code(),
        Some(0)
    );

    // change the file after minting: the cert is now stale and must NOT be trusted
    let mut bytes = std::fs::read(&src).unwrap();
    bytes.extend_from_slice(b"\r\n\"GROUP\",\"XXXX\"\r\n");
    std::fs::write(&src, &bytes).unwrap();

    let o = lat([
        "validate",
        "--index",
        cert.to_str().unwrap(),
        src.to_str().unwrap(),
    ]);
    // the engine runs for real (the junk group trips findings) — the cert was
    // refused with the "stale" reason, not silently honoured
    assert_eq!(o.status.code(), Some(1), "stderr: {}", stderr(&o));
    assert!(
        stderr(&o).contains("stale") && stderr(&o).contains("not used"),
        "no stale note: {}",
        stderr(&o)
    );
    assert!(
        !stdout(&o).contains("clean (0 findings)"),
        "a stale cert was trusted: {}",
        stdout(&o)
    );
}

// --- dict / encoding (shared flag folding: commands/common.rs) ---------------

/// The `--dict` overlay changes the verdict, and a forced base (`--dict-version`)
/// is a legitimate companion to it — not the `--dict-replace` conflict.
///
/// Bare, the bespoke `XTRA` group (hung off `SAMP`) is unknown and flagged across
/// the delivery. With the overlay forced onto its own detected base (4.2), `XTRA`
/// is first-class and those findings vanish — while the delivery's unrelated
/// findings (a missing TRAN key) keep the exit at 1. The forced-base run must NOT
/// trip the `--dict-replace`/`--dict-version` conflict guard, which would exit 5:
/// the guard is `dict_replace && dict_version.is_some()`, so `--dict-version`
/// alone alongside `--dict` is allowed.
#[test]
fn dict_overlay_makes_a_bespoke_group_known_without_tripping_the_conflict_guard() {
    let delivery = fixture("custom_dict/delivery_with_xtra.ags");
    let dict = fixture("custom_dict/xtra.dict.json");

    // bare: XTRA is an unknown group, flagged across the delivery
    let bare = lat(["validate", delivery.to_str().unwrap(), "--json"]);
    assert_eq!(bare.status.code(), Some(1), "stderr: {}", stderr(&bare));
    assert!(
        stdout(&bare).contains("XTRA"),
        "the bundled dictionary should flag the unknown XTRA group: {}",
        stdout(&bare)
    );

    // overlay + forced base: no conflict (exit stays 1, not 5), XTRA now known
    let over = lat([
        "validate",
        "--dict",
        dict.to_str().unwrap(),
        "--dict-version",
        "4.2",
        delivery.to_str().unwrap(),
        "--json",
    ]);
    assert_eq!(
        over.status.code(),
        Some(1),
        "a forced base must validate, not trip the conflict guard: {}",
        stderr(&over)
    );
    assert!(
        !stderr(&over).contains("cannot be combined"),
        "--dict-version alongside --dict is not the --dict-replace conflict: {}",
        stderr(&over)
    );
    assert!(
        !stdout(&over).contains("XTRA"),
        "the overlay should make XTRA a recognised group: {}",
        stdout(&over)
    );
}

/// `--encoding` resolves a valid label and rejects an unknown one (exit 5). A
/// clean file read as utf-8 still validates clean; a bogus label is named and
/// refused. Guards `resolve_encoding` against returning a blanket `None`, which
/// would make even `utf-8` unrecognised and turn every `--encoding` into an exit 5.
#[test]
fn encoding_flag_accepts_a_valid_label_and_rejects_an_unknown_one() {
    let clean = fixture("clean_minimal.ags");

    let ok = lat(["validate", "--encoding", "utf-8", clean.to_str().unwrap()]);
    assert_eq!(ok.status.code(), Some(0), "stderr: {}", stderr(&ok));

    let bad = lat([
        "validate",
        "--encoding",
        "no-such-encoding",
        clean.to_str().unwrap(),
    ]);
    assert_eq!(bad.status.code(), Some(5), "stderr: {}", stderr(&bad));
    assert!(
        stderr(&bad).contains("not recognised"),
        "the unknown label should be named: {}",
        stderr(&bad)
    );
}

// --- excel (AGS4 <-> XLSX round-trip) ---------------------------------------

/// `lat excel` round-trips AGS4 → XLSX → AGS4, with `--no-format-numeric` toggling
/// the numeric re-formatting on import.
///
/// Direction is inferred from the output extension (`.xlsx` ⇒ export, `.ags` ⇒
/// import). On import, a DATA cell is re-formatted to its column's TYPE precision
/// by default: a 3DP column holding `523145.1` becomes the canonical `523145.100`.
/// `--no-format-numeric` leaves it raw. This pins the default-on wiring
/// (`!args.no_format_numeric`) — a flipped default would silently stop padding.
#[test]
fn excel_round_trip_reformats_numerics_by_default_and_no_format_numeric_opts_out() {
    let dir = scratch();
    let src = dir.join("in.ags");
    // a 3DP column carrying an under-precise value — its canonical form is 523145.100
    std::fs::write(
        &src,
        "\"GROUP\",\"LOCA\"\r\n\
         \"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n\
         \"UNIT\",\"\",\"m\"\r\n\
         \"TYPE\",\"ID\",\"3DP\"\r\n\
         \"DATA\",\"BH01\",\"523145.1\"\r\n",
    )
    .unwrap();
    let xlsx = dir.join("mid.xlsx");

    // export (direction inferred from the .xlsx output)
    let exp = lat(["excel", src.to_str().unwrap(), xlsx.to_str().unwrap()]);
    assert_eq!(exp.status.code(), Some(0), "stderr: {}", stderr(&exp));
    assert!(xlsx.exists(), "no xlsx written");

    // import, default: the 3DP cell is padded to its column precision
    let fmt = dir.join("out_fmt.ags");
    let imp = lat(["excel", xlsx.to_str().unwrap(), fmt.to_str().unwrap()]);
    assert_eq!(imp.status.code(), Some(0), "stderr: {}", stderr(&imp));
    let fmt_body = std::fs::read_to_string(&fmt).unwrap();
    assert!(
        fmt_body.contains("523145.100"),
        "default import should re-format to the 3DP column precision: {fmt_body}"
    );

    // import with --no-format-numeric: the raw value is kept as-is
    let raw = dir.join("out_raw.ags");
    let imp2 = lat([
        "excel",
        "--no-format-numeric",
        xlsx.to_str().unwrap(),
        raw.to_str().unwrap(),
    ]);
    assert_eq!(imp2.status.code(), Some(0), "stderr: {}", stderr(&imp2));
    let raw_body = std::fs::read_to_string(&raw).unwrap();
    assert!(
        raw_body.contains("523145.1\"") && !raw_body.contains("523145.100"),
        "--no-format-numeric should keep the raw value, unpadded: {raw_body}"
    );
}
