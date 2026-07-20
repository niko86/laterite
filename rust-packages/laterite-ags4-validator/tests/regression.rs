//! Regression harness.
//!
//! V0: there are no rule checks yet, so this only proves the
//! parse → dict → (empty) rules → findings pipeline runs end-to-end
//! through the public API on a hand-authored fixture, and that
//! un-validatable inputs surface the right error.
//!
//! From V1 each rule family adds `#[test]`s here that load a
//! purpose-built `tests/fixtures/<rule>.ags` and assert the expected
//! findings inline (rule + line + group — never exact `desc` wording,
//! since our wording is independent of python-ags4). Goldens are NOT
//! committed; the AGS4-spec is the authority and assertions live in
//! code. An optional `python_ags4` cross-check (skipped when the dep
//! is absent) can be added per-rule for parity confidence.

use std::path::PathBuf;

use laterite_ags4_validator::{
    CheckOptions, ValidatorError, check_file, findings, is_valid, parse,
};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn clean_minimal_fixture_parses_and_runs_pipeline() {
    let path = fixture("clean_minimal.ags");
    assert!(path.exists(), "missing fixture: {}", path.display());

    let found = check_file(&path, &CheckOptions::default())
        .expect("clean fixture should validate without a hard error");
    // V0 runs no rules, so a well-formed file has zero findings. When
    // V1+ land, this fixture stays clean and this assertion still holds.
    assert_eq!(
        findings::count(&found),
        0,
        "clean fixture produced findings: {found:?}"
    );
}

#[test]
fn clean_minimal_fixture_structure() {
    // Exercise the line-aware parser directly so the regression suite
    // also guards parsing, not just the rules dispatch.
    let path = fixture("clean_minimal.ags");
    let pf = parse::parse_file(&path).expect("parse clean fixture");
    // Conformant minimal file = the four mandatory groups (Rules
    // 13/14 PROJ/TRAN, 15 UNIT, 17 TYPE).
    assert_eq!(pf.group_order, vec!["PROJ", "TRAN", "UNIT", "TYPE"]);
    let proj = &pf.groups["PROJ"];
    assert_eq!(proj.group_line, 1);
    assert_eq!(proj.headings, vec!["PROJ_ID", "PROJ_NAME"]);
    assert_eq!(proj.rows.len(), 1);
    assert_eq!(proj.rows[0].values[0], "P1");
    // CRLF, 25 content lines (incl. 3 blank separators), no phantom
    // trailing blank. TRAN now carries its KEY (TRAN_ISNO) + REQUIRED
    // fields so the file is conformant through Rule 10a/10b (V7).
    assert_eq!(pf.total_lines, 25);
}

#[test]
fn missing_file_is_a_hard_error_not_a_finding() {
    let err = check_file(&fixture("does_not_exist.ags"), &CheckOptions::default()).unwrap_err();
    assert!(matches!(err, ValidatorError::NotFound(_)));
}

#[test]
fn is_valid_mirrors_zero_findings() {
    // The CLI exit code + `db-to-ags4 --validate` key off `is_valid`.
    // The conformant fixture is valid; a defect-bearing one is not, and
    // a missing file is still a hard error (not `false`).
    assert!(is_valid(&fixture("clean_minimal.ags"), &CheckOptions::default()).unwrap());
    assert!(!is_valid(&fixture("rule13_no_proj.ags"), &CheckOptions::default()).unwrap());
    assert!(is_valid(&fixture("does_not_exist.ags"), &CheckOptions::default()).is_err());
}

// ---- V1: line-level rules (1, 3, 5, 6) -----------------------------
// Expectations asserted inline (rule + line + group). Wording is never
// asserted — our `desc` is clean-room-independent of python-ags4.

fn findings_for(name: &str) -> laterite_ags4_validator::Findings {
    check_file(&fixture(name), &CheckOptions::default()).unwrap_or_else(|e| panic!("{name}: {e}"))
}

#[test]
fn rule1_non_ascii_flagged_at_its_line() {
    let f = findings_for("rule1_non_ascii.ags");
    let r1 = f.get("AGS Format Rule 1").expect("Rule 1 finding");
    assert_eq!(r1.len(), 1);
    assert_eq!(r1[0].line, Some(5)); // the DATA row with the smart quote
}

#[test]
fn rule3_bad_descriptor_flagged_at_its_line() {
    let f = findings_for("rule3_bad_descriptor.ags");
    let r3 = f.get("AGS Format Rule 3").expect("Rule 3 finding");
    assert!(
        r3.iter().any(|x| x.line == Some(6)),
        "expected a Rule 3 finding on line 6, got {r3:?}"
    );
}

#[test]
fn rule5_unquoted_field_flagged_at_its_line() {
    let f = findings_for("rule5_unquoted.ags");
    let r5 = f.get("AGS Format Rule 5").expect("Rule 5 finding");
    assert!(
        r5.iter().any(|x| x.line == Some(5)),
        "expected a Rule 5 finding on line 5, got {r5:?}"
    );
}

// ---- V2: group-structure rules (2, 2a, 2b, 4) ----------------------

#[test]
fn rule2_group_without_data_rows_flagged() {
    let f = findings_for("rule2_no_data_rows.ags");
    let r2 = f.get("AGS Format Rule 2").expect("Rule 2 finding");
    assert!(
        r2.iter().any(|x| x.group == "LOCA"),
        "expected LOCA flagged for no DATA rows, got {r2:?}"
    );
}

#[test]
fn rule2b_missing_unit_flagged() {
    let f = findings_for("rule2b_unit_missing.ags");
    let r2b = f.get("AGS Format Rule 2b").expect("Rule 2b finding");
    assert!(
        r2b.iter().any(|x| x.desc.contains("UNIT row missing")),
        "expected UNIT-missing, got {r2b:?}"
    );
}

#[test]
fn rule4_data_field_count_mismatch_flagged() {
    let f = findings_for("rule4_field_count.ags");
    let r4 = f.get("AGS Format Rule 4").expect("Rule 4 finding");
    // HEADING has 2 fields, the DATA row (line 5) has 3.
    assert!(
        r4.iter().any(|x| x.line == Some(5)),
        "expected a Rule 4 finding on line 5, got {r4:?}"
    );
}

// ---- V3: name-format rules (19, 19a, 19b) --------------------------

#[test]
fn rule19_bad_group_name_flagged() {
    let f = findings_for("rule19_bad_group_name.ags");
    let r19 = f.get("AGS Format Rule 19").expect("Rule 19 finding");
    assert!(
        r19.iter().any(|x| x.group == "TOOLONG"),
        "expected TOOLONG flagged, got {r19:?}"
    );
}

#[test]
fn rule19a_bad_heading_charset_flagged() {
    let f = findings_for("rule19a_bad_heading.ags");
    let r = f.get("AGS Format Rule 19a").expect("Rule 19a finding");
    assert!(
        r.iter().any(|x| x.desc.contains("PROJ_lc")),
        "expected PROJ_lc flagged for charset, got {r:?}"
    );
}

#[test]
fn rule19b_bad_prefix_flagged() {
    let f = findings_for("rule19b_bad_prefix.ags");
    let r = f.get("AGS Format Rule 19b").expect("Rule 19b finding");
    // "PRJ_ID" — 3-char prefix, not 4.
    assert!(
        r.iter().any(|x| x.line == Some(2)),
        "expected a Rule 19b finding on the HEADING line, got {r:?}"
    );
}

// ---- V4: dictionary-aware rules (7, 9) -----------------------------

#[test]
fn rule7_headings_out_of_dict_order_flagged() {
    let f = findings_for("rule7_out_of_order.ags");
    let r7 = f.get("AGS Format Rule 7").expect("Rule 7 finding");
    // PROJ_NAME before PROJ_ID — flagged on the HEADING line (2).
    assert!(
        r7.iter().any(|x| x.line == Some(2) && x.group == "PROJ"),
        "expected a Rule 7 order finding on line 2, got {r7:?}"
    );
}

#[test]
fn rule9_unknown_heading_flagged() {
    let f = findings_for("rule9_unknown_heading.ags");
    let r9 = f.get("AGS Format Rule 9").expect("Rule 9 finding");
    // PROJ_QQQQ is in neither the standard dict nor a DICT group.
    assert!(
        r9.iter().any(|x| x.line == Some(2) && x.group == "PROJ"),
        "expected a Rule 9 finding on line 2, got {r9:?}"
    );
}

// ---- V5: typed-value rule (8) --------------------------------------

#[test]
fn rule8_wrong_decimal_precision_flagged() {
    let f = findings_for("rule8_dp_wrong_precision.ags");
    let r8 = f.get("AGS Format Rule 8").expect("Rule 8 finding");
    // "10.5" declared 2DP — flagged on its DATA line (5).
    assert!(
        r8.iter().any(|x| x.line == Some(5) && x.group == "LOCA"),
        "expected a Rule 8 finding on line 5, got {r8:?}"
    );
}

#[test]
fn rule8_invalid_date_flagged() {
    let f = findings_for("rule8_dt_bad.ags");
    let r8 = f.get("AGS Format Rule 8").expect("Rule 8 finding");
    // "2023-13-45" is structurally yyyy-mm-dd but not a real date.
    assert!(
        r8.iter().any(|x| x.line == Some(5) && x.group == "LOCA"),
        "expected a Rule 8 finding on line 5, got {r8:?}"
    );
}

#[test]
fn rule8_empty_unit_dt_flags_like_python() {
    // O-31: a DT field with an EMPTY UNIT. python-ags4 builds an empty
    // per-char regex and `''.fullmatch("2025-02-24")` fails, so it
    // flags Rule 8 ("…the specified format () …"). We now match —
    // closing the O-12 degenerate gap: a structurally-valid date with
    // no declared format must flag.
    let f = findings_for("rule8_dt_empty_unit.ags");
    let r8 = f.get("AGS Format Rule 8").expect("Rule 8 finding");
    assert!(
        r8.iter().any(|x| x.line == Some(5) && x.group == "LOCA"),
        "empty-UNIT DT value must flag Rule 8 (python parity), got {r8:?}"
    );
}

#[test]
fn rule8_date_out_of_pandas_range_flagged() {
    // O-33: "0018-06-03" is structurally yyyy-mm-dd AND a valid
    // proleptic-Gregorian date (chrono accepts it), but outside
    // pandas' Timestamp range — python's pd.to_datetime NaTs it and
    // flags Rule 8. We now match (the dogfood `LOCA_STAR=0018-06-03`
    // defect): a corrupt year must flag like python.
    let f = findings_for("rule8_dt_out_of_range.ags");
    let r8 = f.get("AGS Format Rule 8").expect("Rule 8 finding");
    assert!(
        r8.iter().any(|x| x.line == Some(5) && x.group == "LOCA"),
        "out-of-pandas-range date must flag Rule 8 (python parity), got {r8:?}"
    );
}

// ---- V6: mandatory / definition groups (13, 14, 15, 16, 17, 18) ----

#[test]
fn rule13_missing_proj_flagged() {
    let f = findings_for("rule13_no_proj.ags");
    let r13 = f.get("AGS Format Rule 13").expect("Rule 13 finding");
    assert!(
        r13.iter().any(|x| x.group == "PROJ" && x.line.is_none()),
        "expected PROJ-not-found, got {r13:?}"
    );
}

#[test]
fn rule15_undefined_unit_flagged() {
    let f = findings_for("rule15_unit_undef.ags");
    let r15 = f.get("AGS Format Rule 15").expect("Rule 15 finding");
    // 'm' is used by LOCA_FDEP but the UNIT group only defines 'mm'.
    assert!(
        r15.iter()
            .any(|x| x.desc.contains("\"m\"") && x.group == "UNIT"),
        "expected unit 'm' undefined, got {r15:?}"
    );
}

#[test]
fn rule17_missing_type_group_flagged() {
    let f = findings_for("rule17_no_type.ags");
    let r17 = f.get("AGS Format Rule 17").expect("Rule 17 finding");
    assert!(
        r17.iter()
            .any(|x| x.group == "TYPE" && x.desc.contains("not found")),
        "expected TYPE-group-not-found, got {r17:?}"
    );
}

// ---- V7: relational rules (10a, 10b, 10c, 11a/11b/11c) -------------

#[test]
fn rule10a_duplicate_key_flagged() {
    let f = findings_for("rule10a_dup_key.ags");
    let r10a = f.get("AGS Format Rule 10a").expect("Rule 10a finding");
    // Two LOCA rows share LOCA_ID "BH1" → both flagged (lines 5 & 6).
    assert!(
        r10a.iter().filter(|x| x.desc.contains("Duplicate")).count() >= 2,
        "expected duplicate-KEY findings, got {r10a:?}"
    );
}

#[test]
fn rule10c_orphan_child_flagged() {
    let f = findings_for("rule10c_orphan_child.ags");
    let r10c = f.get("AGS Format Rule 10c").expect("Rule 10c finding");
    // SAMP row with LOCA_ID "BH9" has no parent in LOCA.
    assert!(
        r10c.iter()
            .any(|x| x.group == "SAMP" && x.desc.contains("BH9")),
        "expected orphan SAMP/BH9, got {r10c:?}"
    );
}

#[test]
fn rule11c_invalid_record_link_flagged() {
    let f = findings_for("rule11c_bad_rl.ags");
    let r11c = f.get("AGS Format Rule 11c").expect("Rule 11c finding");
    // SAMP_LINK = "LOCA|BH404" — no such LOCA record.
    assert!(
        r11c.iter()
            .any(|x| x.group == "SAMP" && x.desc.contains("no such record")),
        "expected invalid record link, got {r11c:?}"
    );
}

// ---- V8: cross-reference rules (19b borrow, 20) --------------------

#[test]
fn rule19b_unknown_borrowed_prefix_flagged() {
    let f = findings_for("rule19b_borrowed_bad.ags");
    let r = f.get("AGS Format Rule 19b").expect("Rule 19b finding");
    // ZZZZ_FOO in SAMP — ZZZZ is not a defined group.
    assert!(
        r.iter()
            .any(|x| x.group == "SAMP" && x.desc.contains("ZZZZ")),
        "expected a borrowed-prefix Rule 19b finding, got {r:?}"
    );
}

#[test]
fn rule20_undefined_file_fset_flagged() {
    let f = findings_for("rule20_undefined_fset.ags");
    let r20 = f.get("AGS Format Rule 20").expect("Rule 20 finding");
    // LOCA uses FILE_FSET "FS9"; FILE group only defines "FS1".
    assert!(
        r20.iter()
            .any(|x| x.group == "LOCA" && x.desc.contains("FS9")),
        "expected an undefined-FILE_FSET Rule 20 finding, got {r20:?}"
    );
}

#[test]
fn clean_minimal_still_clean() {
    // Regression guard: the known-good fixture must stay finding-free
    // with every rule family wired (V1–V8). It is a fully
    // conformant file: PROJ + TRAN single rows with all KEY/REQUIRED
    // fields filled, UNIT + TYPE definition groups, all headings
    // standard and in dictionary order, values matching their TYPEs,
    // every unit/type used is defined, no orphan rows.
    let f = findings_for("clean_minimal.ags");
    assert_eq!(findings::count(&f), 0, "clean fixture regressed: {f:?}");
}

#[test]
fn non_ags4_input_is_a_hard_error() {
    // A real file that isn't AGS4 (this source file itself).
    let me = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("regression.rs");
    let err = check_file(&me, &CheckOptions::default()).unwrap_err();
    assert!(matches!(err, ValidatorError::NotAgs4(_)));
}

// ---- O-32: invalid-encoding input is decoded lossily, not refused --

/// A temp `.ags` copy of `clean_minimal.ags` with `inject` spliced into
/// the `PROJ_NAME` value (after the literal "Clean"). Temp, never
/// `tests/fixtures/`: a non-UTF-8 file there would trip corpus-qa's
/// e2e `hard_error==0` assertion *and* defeat the behaviour under
/// test. `PROJ_NAME` is a non-KEY OTHER field, so splicing there can't
/// perturb any relational rule — only Rule 1 reacts.
fn clean_minimal_with_injected_bytes(inject: &[u8]) -> tempfile::NamedTempFile {
    use std::io::Write;
    let mut bytes = std::fs::read(fixture("clean_minimal.ags")).unwrap();
    let at = bytes.windows(5).position(|w| w == b"Clean").unwrap() + 5;
    bytes.splice(at..at, inject.iter().copied());
    let mut f = tempfile::Builder::new().suffix(".ags").tempfile().unwrap();
    f.write_all(&bytes).unwrap();
    f.flush().unwrap();
    f
}

#[test]
fn invalid_utf8_input_flags_rule1_not_hard_error() {
    // O-32: a raw cp1252 byte (0xB0) is invalid UTF-8 → decoded
    // lossily to U+FFFD (code point 65533 > 255), exactly as
    // python-ags4's `errors="replace"` does. The file MUST validate
    // (no NotUtf8 black hole) and Rule 1 MUST fire on *default* opts
    // (the >255 arm, independent of --show-fyi) — this is what makes
    // the 12 dogfood files AGREE with python on a Rule 1 error.
    let tf = clean_minimal_with_injected_bytes(&[0xB0]);
    let f = check_file(tf.path(), &CheckOptions::default())
        .expect("invalid encoding must NOT be a hard error any more");
    assert!(
        f.contains_key("AGS Format Rule 1"),
        "lossy U+FFFD must surface as a Rule 1 finding, got {f:?}"
    );
}

#[test]
fn valid_utf8_extended_char_is_fyi_only_not_rule1() {
    // Correct-encoding-rewarded side of O-32: a *properly*
    // UTF-8-encoded `°` (0xC2 0xB0 → U+00B0, ≤255) is the tolerated
    // extended-ASCII case — suppressed by default (parity with
    // `ags4_cli check`), surfacing only as the Rule 1 FYI key under
    // include_fyi. It must never become the >255 Rule 1 error.
    let tf = clean_minimal_with_injected_bytes("°".as_bytes());

    let default = check_file(tf.path(), &CheckOptions::default()).unwrap();
    assert!(
        !default.contains_key("AGS Format Rule 1"),
        "valid extended-ASCII must not be a Rule 1 error, got {default:?}"
    );

    let with_fyi = check_file(
        tf.path(),
        &CheckOptions {
            include_fyi: true,
            ..CheckOptions::default()
        },
    )
    .unwrap();
    assert!(
        with_fyi.contains_key("FYI (Related to Rule 1)"),
        "valid extended-ASCII must surface as the Rule 1 FYI, got {with_fyi:?}"
    );
}

#[test]
fn rule_16_nonstandard_self_declared_abbr_is_fyi_only() {
    // A file using SAMP_TYPE="ZZ", self-declared in ABBR. Rule 16 (error) is
    // satisfied — ZZ IS in the file's ABBR — but ZZ is not a standard SAMP_TYPE
    // code, which surfaces ONLY as the Rule 16 FYI under include_fyi (O-43).
    let path = fixture("rule16_fyi_nonstandard_abbr.ags");
    assert!(path.exists(), "missing fixture: {}", path.display());

    let default = check_file(&path, &CheckOptions::default()).unwrap();
    assert!(
        !default.contains_key("AGS Format Rule 16"),
        "ZZ is defined in ABBR, so Rule 16 (error) must stay silent: {default:?}"
    );
    assert!(
        !default.contains_key("FYI (Related to Rule 16)"),
        "the FYI must be suppressed by default (opt-in): {default:?}"
    );

    let with_fyi = check_file(
        &path,
        &CheckOptions {
            include_fyi: true,
            ..CheckOptions::default()
        },
    )
    .unwrap();
    let fyi = with_fyi
        .get("FYI (Related to Rule 16)")
        .expect("the non-standard-abbr FYI under include_fyi");
    assert!(
        fyi.iter()
            .any(|f| f.desc.contains("\"ZZ\"") && f.desc.contains("not a recognised standard")),
        "expected the non-standard SAMP_TYPE FYI, got {fyi:?}"
    );
}

#[test]
fn rule_18_malformed_dict_is_warning_only() {
    // A DICT group with a HEADING-type row that names no heading (blank
    // DICT_HDNG). It is a WARNING under include_warnings (O-44), suppressed by
    // default, and never the error-tier "AGS Format Rule 18".
    let path = fixture("rule18_malformed_dict.ags");
    assert!(path.exists(), "missing fixture: {}", path.display());

    let default = check_file(&path, &CheckOptions::default()).unwrap();
    assert!(
        !default.contains_key("Warning (Related to Rule 18)"),
        "the DICT warning must be opt-in: {default:?}"
    );

    let with_warn = check_file(
        &path,
        &CheckOptions {
            include_warnings: true,
            ..CheckOptions::default()
        },
    )
    .unwrap();
    let w = with_warn
        .get("Warning (Related to Rule 18)")
        .expect("the malformed-DICT warning under include_warnings");
    assert!(
        w.iter().any(|f| f.desc.contains("DICT_HDNG is blank")),
        "expected the blank-DICT_HDNG warning, got {w:?}"
    );
    assert_eq!(w[0].severity, findings::Severity::Warning);
}

#[test]
fn rule_labels_inventory_is_grounded_against_real_emissions() {
    // Grounds catalogue::RULE_LABELS against what the engine ACTUALLY emits:
    // run every fixture (FYI on) and assert no numbered rule label escapes the
    // inventory, and no unexpected top-level FYI bucket appears. Catches a new
    // rule emitting a label the catalogue (and `--list-rules`) doesn't know —
    // the drift direction the in-crate `metadata_covers_exactly_the_inventory`
    // gate (meta == RULE_LABELS) can't see on its own.
    use std::collections::BTreeSet;
    let inventory: BTreeSet<&str> = laterite_ags4_validator::RULE_LABELS
        .iter()
        .copied()
        .collect();
    // The non-numbered FYI / WARNING buckets are a DELIBERATELY separate label
    // space (see catalogue.rs) — enumerated here so a NEW one also trips this gate.
    let known_non_numbered: BTreeSet<&str> = [
        "FYI",
        "FYI (Related to Rule 1)",
        "FYI (Related to Rule 16)",
        "Warning (Related to Rule 18)",
    ]
    .into_iter()
    .collect();

    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");
    let opts = CheckOptions {
        include_fyi: true,
        include_warnings: true,
        ..CheckOptions::default()
    };
    let mut seen_numbered: BTreeSet<String> = BTreeSet::new();
    for entry in std::fs::read_dir(&dir).expect("read fixtures dir") {
        let path = entry.expect("dir entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("ags") {
            continue;
        }
        let Ok(found) = check_file(&path, &opts) else {
            continue; // a hard-error fixture (not-utf8/not-ags4) emits no findings
        };
        for label in found.keys() {
            if let Some(suffix) = label.strip_prefix("AGS Format Rule ") {
                assert!(
                    inventory.contains(suffix),
                    "{}: emits {label:?} but {suffix:?} is not in RULE_LABELS",
                    path.display()
                );
                seen_numbered.insert(suffix.to_string());
            } else {
                assert!(
                    known_non_numbered.contains(label.as_str()),
                    "{}: emits an unknown non-numbered label {label:?}",
                    path.display()
                );
            }
        }
    }
    // Sanity: the corpus exercises a meaningful slice of the inventory (so this
    // test isn't silently a no-op if fixtures vanish).
    assert!(
        seen_numbered.len() >= 15,
        "fixtures should exercise many rules, saw {}",
        seen_numbered.len()
    );
}
