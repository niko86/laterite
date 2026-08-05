//! Mechanical repair of a delivered AGS4 file.
//!
//! Public API only. The properties worth nailing down are the ones a fixer can
//! get quietly wrong:
//!
//! - it is non-destructive unless asked — a `fix` that overwrote the source
//!   would be unrecoverable and the caller would find out afterwards,
//! - the risky tier stays withheld until asked for, and says how much it held,
//! - the output is always UTF-8, so repairing a cp1252 file normalises it
//!   rather than round-tripping the legacy encoding,
//! - the residual is what could NOT be fixed, not what was wrong to begin with.

use std::fs;

use laterite::ErrorKind;
use laterite::ags4;

/// LF line endings and a short row — two safe fixes (Rule 2a, Rule 4) on a file
/// that is otherwise ordinary.
const BROKEN: &str = concat!(
    "\"GROUP\",\"PROJ\"\n",
    "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\n",
    "\"UNIT\",\"\",\"\"\n",
    "\"TYPE\",\"ID\",\"X\"\n",
    "\"DATA\",\"P1\"\n",
);

/// A group declaring the same heading twice — repairable only by renaming the
/// second occurrence, which is a guess about the caller's intent, so it sits in
/// the risky tier.
const DOUBLED_HEADING: &str = concat!(
    "\"GROUP\",\"PROJ\"\r\n",
    "\"HEADING\",\"PROJ_ID\",\"PROJ_ID\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n",
    "\"TYPE\",\"ID\",\"ID\"\r\n",
    "\"DATA\",\"P1\",\"P2\"\r\n",
);

/// Correct AGS4 with nothing for the fixer to do — a fix run over it must be
/// byte-for-byte idempotent.
const TIDY: &str = concat!(
    "\"GROUP\",\"PROJ\"\r\n",
    "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n",
    "\"TYPE\",\"ID\",\"X\"\r\n",
    "\"DATA\",\"P1\",\"A site\"\r\n",
);

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("laterite-fix-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn the_safe_fixes_are_applied_and_listed() {
    let fixed = ags4::fix_str(BROKEN).run().unwrap();

    assert!(fixed.fixes_applied() > 0);
    assert_eq!(fixed.fixes_applied(), fixed.applied().len());
    assert!(fixed.text().contains("\r\n"), "{}", fixed.text());
    assert!(!fixed.text().contains("\"DATA\",\"P1\"\r\n"));

    let kinds: Vec<&str> = fixed.applied().iter().map(ags4::Repair::kind).collect();
    assert!(kinds.contains(&"normalize_crlf"), "{kinds:?}");
    assert!(kinds.contains(&"pad_short_row"), "{kinds:?}");
}

/// Every repair carries the rule it answers and a label describing it, so a
/// caller can cross-link back to the finding and show what was done.
///
/// The label is asserted on its CONTENT, not merely on being non-empty: a
/// getter returning the wrong string is exactly the defect a non-empty check
/// cannot see, and this ledger is what a UI puts in front of someone.
#[test]
fn a_repair_names_its_rule_and_describes_itself() {
    let fixed = ags4::fix_str(BROKEN).run().unwrap();
    let crlf = fixed
        .applied()
        .iter()
        .find(|r| r.kind() == "normalize_crlf")
        .unwrap();

    assert_eq!(crlf.rule(), "AGS Format Rule 2a");
    assert!(crlf.label().contains("CRLF"), "{}", crlf.label());
    assert!(crlf.label().contains("2a"), "{}", crlf.label());
    assert!(!crlf.is_risky());
}

/// A repair anchors to the line it applied at — and the whole-file repairs
/// (BOM, CRLF) have no line to anchor to.
///
/// Both halves, because either alone passes with `line()` hard-wired: `None`
/// for everything satisfies the CRLF case, and the short-row case is the one
/// that says the anchor is real.
#[test]
fn a_repair_anchors_to_its_line_unless_it_is_whole_file() {
    let fixed = ags4::fix_str(BROKEN).run().unwrap();
    let repair = |kind: &str| {
        fixed
            .applied()
            .iter()
            .find(|r| r.kind() == kind)
            .unwrap_or_else(|| panic!("no {kind} repair in {:?}", fixed.applied()))
    };

    // The short DATA row is line 5 of the fixture.
    assert_eq!(repair("pad_short_row").line(), Some(5));
    assert_eq!(repair("normalize_crlf").line(), None);
}

/// The three doors are one engine. A file, its bytes and its text must produce
/// identical output, or "which door did you use" becomes a thing a caller has
/// to reason about.
#[test]
fn path_bytes_and_text_agree() {
    let dir = scratch("doors");
    let path = dir.join("broken.ags");
    fs::write(&path, BROKEN).unwrap();

    let from_path = ags4::fix(&path).run().unwrap();
    let from_bytes = ags4::fix_bytes(BROKEN.as_bytes()).run().unwrap();
    let from_text = ags4::fix_str(BROKEN).run().unwrap();

    assert_eq!(from_path.bytes(), from_bytes.bytes());
    assert_eq!(from_path.bytes(), from_text.bytes());
    assert_eq!(from_path.fixes_applied(), from_text.fixes_applied());
}

#[test]
fn a_run_does_not_touch_the_source_file() {
    let dir = scratch("nondestructive");
    let path = dir.join("broken.ags");
    fs::write(&path, BROKEN).unwrap();

    let fixed = ags4::fix(&path).run().unwrap();

    assert_ne!(fixed.bytes(), BROKEN.as_bytes());
    assert_eq!(fs::read(&path).unwrap(), BROKEN.as_bytes());
}

#[test]
fn to_path_writes_what_the_result_carries() {
    let dir = scratch("to-path");
    let src = dir.join("broken.ags");
    let out = dir.join("repaired.ags");
    fs::write(&src, BROKEN).unwrap();

    let fixed = ags4::fix(&src).to_path(&out).unwrap();

    assert_eq!(fs::read(&out).unwrap(), fixed.bytes());
    assert_eq!(fs::read(&src).unwrap(), BROKEN.as_bytes());
}

/// In place is the source path named as the destination — no separate flag,
/// because there is nothing a flag would express that the path does not.
#[test]
fn naming_the_source_repairs_it_in_place() {
    let dir = scratch("in-place");
    let path = dir.join("broken.ags");
    fs::write(&path, BROKEN).unwrap();

    let fixed = ags4::fix(&path).to_path(&path).unwrap();

    assert_eq!(fs::read(&path).unwrap(), fixed.bytes());
    assert_ne!(fs::read(&path).unwrap(), BROKEN.as_bytes());
}

#[test]
fn save_writes_a_result_held_in_memory() {
    let dir = scratch("save");
    let out = dir.join("repaired.ags");

    let fixed = ags4::fix_str(BROKEN).run().unwrap();
    fixed.save(&out).unwrap();

    assert_eq!(fs::read(&out).unwrap(), fixed.bytes());
}

/// Nothing to fix means nothing changed — the same bytes back, not merely
/// equivalent ones.
#[test]
fn a_tidy_file_comes_back_untouched() {
    let fixed = ags4::fix_str(TIDY).run().unwrap();

    assert_eq!(fixed.bytes(), TIDY.as_bytes());
    assert_eq!(fixed.fixes_applied(), 0);
    assert_eq!(fixed.risky_available(), 0);
}

/// A duplicate heading is repairable only by guessing — the fixer renames the
/// second occurrence, which is a decision about the caller's data. So it is
/// withheld, and the withholding is reported.
#[test]
fn the_risky_tier_is_withheld_until_asked_for() {
    let doubled = DOUBLED_HEADING;

    let safe = ags4::fix_str(doubled).run().unwrap();
    assert!(
        safe.risky_available() > 0,
        "the safe run should say what it held back"
    );

    let risky = ags4::fix_str(doubled).risky(true).run().unwrap();
    assert!(risky.fixes_applied() > safe.fixes_applied());
    assert_eq!(
        risky.risky_available(),
        0,
        "nothing is withheld once risky is on"
    );
    assert!(risky.applied().iter().any(ags4::Repair::is_risky));
}

#[test]
fn only_narrows_the_repairs_and_exclude_removes_from_them() {
    let all = ags4::fix_str(BROKEN).run().unwrap();
    assert!(all.fixes_applied() > 1);

    let crlf_only = ags4::fix_str(BROKEN).only(["2a"]).run().unwrap();
    let kinds: Vec<&str> = crlf_only.applied().iter().map(ags4::Repair::kind).collect();
    assert!(kinds.contains(&"normalize_crlf"), "{kinds:?}");
    assert!(!kinds.contains(&"pad_short_row"), "{kinds:?}");

    let without_crlf = ags4::fix_str(BROKEN).exclude(["2a"]).run().unwrap();
    let kinds: Vec<&str> = without_crlf
        .applied()
        .iter()
        .map(ags4::Repair::kind)
        .collect();
    assert!(!kinds.contains(&"normalize_crlf"), "{kinds:?}");
    assert!(kinds.contains(&"pad_short_row"), "{kinds:?}");
}

/// The vocabulary `only`/`exclude` speak — asserted against repairs the fixer
/// actually made, not against itself.
///
/// Round-tripping each label through `only()` proves nothing: an unrecognised
/// label simply selects nothing and the run succeeds, so a list of `"xyzzy"`
/// passes that check exactly as well as the real one. What the list has to be
/// true about is the labels the fixer emits, which is what this compares.
#[test]
fn the_advertised_labels_are_the_ones_the_fixer_emits() {
    let advertised = ags4::fixable_rules();
    assert!(!advertised.is_empty());

    let mut seen = Vec::new();
    for source in [BROKEN, DOUBLED_HEADING] {
        for repair in ags4::fix_str(source).risky(true).run().unwrap().applied() {
            seen.push(
                repair
                    .rule()
                    .trim_start_matches("AGS Format Rule ")
                    .to_string(),
            );
        }
    }
    assert!(
        seen.len() >= 3,
        "the fixtures should produce repairs: {seen:?}"
    );

    for label in &seen {
        assert!(
            advertised.contains(&label.as_str()),
            "the fixer emitted a repair for rule {label:?} that `fixable_rules()` \
             does not advertise: {advertised:?}"
        );
    }
    // And the advertised set is a vocabulary of rule labels, not of anything
    // else — `"2a"`, `"11b"`, never a word.
    for label in &advertised {
        let (digits, suffix) = label.split_at(
            label
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(label.len()),
        );
        assert!(!digits.is_empty(), "{label:?} is not a rule label");
        assert!(
            suffix.is_empty() || suffix.chars().all(|c| c.is_ascii_lowercase()),
            "{label:?} is not a rule label"
        );
    }
}

/// The output is UTF-8 whatever went in. A `°` that arrived as cp1252 comes
/// back as its UTF-8 self, not as the two bytes cp1252 spells it with.
#[test]
fn a_legacy_encoding_is_normalised_on_the_way_out() {
    let mut cp1252: Vec<u8> = Vec::new();
    cp1252.extend_from_slice(b"\"GROUP\",\"PROJ\"\n");
    cp1252.extend_from_slice(b"\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\n");
    cp1252.extend_from_slice(b"\"UNIT\",\"\",\"\"\n");
    cp1252.extend_from_slice(b"\"TYPE\",\"ID\",\"X\"\n");
    cp1252.extend_from_slice(b"\"DATA\",\"P1\",\"45\xb0 slope\"\n");

    let fixed = ags4::fix_bytes(cp1252)
        .encoding("windows-1252")
        .run()
        .unwrap();

    assert!(fixed.text().contains("45° slope"), "{}", fixed.text());
    assert_eq!(std::str::from_utf8(fixed.bytes()).unwrap(), fixed.text());
}

/// `fix_str` takes text that is decoded already, so an encoding label must not
/// reach it — transcoding decoded text is what corrupts the `°` this option
/// exists to rescue. The same structural guard `read_str` carries.
#[test]
fn encoding_cannot_corrupt_text() {
    let text = concat!(
        "\"GROUP\",\"PROJ\"\r\n",
        "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n",
        "\"UNIT\",\"\",\"\"\r\n",
        "\"TYPE\",\"ID\",\"X\"\r\n",
        "\"DATA\",\"P1\",\"45° slope\"\r\n",
    );

    let plain = ags4::fix_str(text).run().unwrap();
    let mislabelled = ags4::fix_str(text).encoding("windows-1252").run().unwrap();

    assert_eq!(plain.bytes(), mislabelled.bytes());
    assert!(mislabelled.text().contains("45° slope"));
}

/// The residual is what survived the repair — the CRLF and short-row faults
/// were fixed, so they are gone from it; the missing UNIT/TYPE catalogues were
/// not mechanically fixable and remain.
#[test]
fn the_residual_is_what_could_not_be_fixed() {
    let fixed = ags4::fix_str(BROKEN).run().unwrap();

    assert!(fixed.fixes_applied() > 0);
    let rules: Vec<&str> = fixed.findings().iter().map(ags4::Finding::rule).collect();
    assert!(!rules.contains(&"AGS Format Rule 2a"), "{rules:?}");
    assert!(!rules.contains(&"AGS Format Rule 4"), "{rules:?}");
    assert!(rules.iter().any(|r| r.contains("15")), "{rules:?}");
}

/// Errors AND warnings, not errors alone. A fix run's residual is an account of
/// what it could not put right, and an errors-only account under-reports that —
/// which is the tier every other surface settled on for the same reason.
///
/// `BROKEN` has no warning to find, so this needs its own fixture: an
/// unrecognised `TRAN_AGS` is warned about rather than errored on.
#[test]
fn the_residual_carries_warnings_as_well_as_errors() {
    const ODD_EDITION: &str = concat!(
        "\"GROUP\",\"PROJ\"\n",
        "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\n",
        "\"UNIT\",\"\",\"\"\n",
        "\"TYPE\",\"ID\",\"X\"\n",
        "\"DATA\",\"P1\"\n",
        "\"GROUP\",\"TRAN\"\n",
        "\"HEADING\",\"TRAN_ISNO\",\"TRAN_DATE\",\"TRAN_PROD\",\"TRAN_STAT\",\"TRAN_AGS\",\"TRAN_RECV\"\n",
        "\"UNIT\",\"\",\"yyyy-mm-dd\",\"\",\"\",\"\",\"\"\n",
        "\"TYPE\",\"X\",\"DT\",\"X\",\"X\",\"X\",\"X\"\n",
        "\"DATA\",\"1\",\"2026-08-05\",\"Producer\",\"Draft\",\"4.5\",\"Recipient\"\n",
    );

    let fixed = ags4::fix_str(ODD_EDITION).run().unwrap();

    assert!(
        fixed
            .findings()
            .iter()
            .any(|f| f.severity() == ags4::Severity::Warning),
        "{:?}",
        fixed.findings()
    );
    assert!(
        fixed
            .findings()
            .iter()
            .any(|f| f.severity() == ags4::Severity::Error),
        "{:?}",
        fixed.findings()
    );
}

#[test]
fn the_edition_is_reported_whether_pinned_or_derived() {
    let derived = ags4::fix_str(BROKEN).run().unwrap();
    assert!(!derived.edition().is_empty());

    let pinned = ags4::fix_str(BROKEN).edition("4.0.4").run().unwrap();
    assert_eq!(pinned.edition(), "4.0.4");
}

#[test]
fn a_missing_file_is_an_io_error() {
    let err = ags4::fix("no/such/file.ags").run().unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Io);
}

#[test]
fn an_unknown_encoding_is_refused_by_name() {
    let err = ags4::fix_bytes(BROKEN.as_bytes())
        .encoding("klingon-1")
        .run()
        .unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidArgument);
    assert!(err.to_string().contains("klingon-1"), "{err}");
}

#[test]
fn an_unknown_edition_is_refused_by_name() {
    let err = ags4::fix_str(BROKEN).edition("4.9").run().unwrap_err();
    assert_eq!(err.kind(), ErrorKind::BadDictionary);
    assert!(err.to_string().contains("4.9"), "{err}");
}

#[test]
fn input_that_is_not_ags4_is_refused() {
    let err = ags4::fix_str("this is not a delivery file")
        .run()
        .unwrap_err();
    assert_eq!(err.kind(), ErrorKind::NotAgs4);
}

#[test]
fn into_bytes_and_into_text_hand_over_the_same_content() {
    let fixed = ags4::fix_str(BROKEN).run().unwrap();
    let text = fixed.text().to_string();

    assert_eq!(ags4::fix_str(BROKEN).run().unwrap().into_text(), text);
    assert_eq!(
        ags4::fix_str(BROKEN).run().unwrap().into_bytes(),
        text.into_bytes()
    );
}

/// `Debug` shows the settings, never the file — the same rule the read handles
/// follow.
#[test]
fn debug_shows_settings_not_contents() {
    let rendered = format!("{:?}", ags4::fix_str(BROKEN).risky(true).only(["2a"]));
    assert!(rendered.contains("characters"), "{rendered}");
    assert!(!rendered.contains("PROJ_ID"), "{rendered}");

    let rendered = format!("{:?}", ags4::fix_str(BROKEN).run().unwrap());
    assert!(rendered.contains("Fixed"), "{rendered}");
    assert!(!rendered.contains("PROJ_ID"), "{rendered}");

    // A repair, unlike the file, IS its own description — so its Debug carries
    // the whole record rather than a summary.
    let fixed = ags4::fix_str(BROKEN).run().unwrap();
    let rendered = format!("{:?}", fixed.applied()[0]);
    for field in ["Repair", "kind", "label", "rule", "line", "risky"] {
        assert!(rendered.contains(field), "{field} missing from {rendered}");
    }
}
