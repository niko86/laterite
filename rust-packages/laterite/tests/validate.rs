//! Validating AGS4 that never touches a filesystem.
//!
//! Deliberately uses nothing but `laterite::*` — same rule as `roundtrip.rs`.
//! The interesting cases here are all about Rule 20, whose on-disk half is the
//! only thing that distinguishes a path from bytes.

use laterite::ags4;

/// Declares one attachment. Nothing beside it on disk, which is the point: with
/// a path and `check_files` on, Rule 20's on-disk half has something to fail on;
/// from bytes it has nothing to look at at all.
const WITH_ATTACHMENT: &str = concat!(
    "\"GROUP\",\"PROJ\"\r\n",
    "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n",
    "\"TYPE\",\"ID\",\"X\"\r\n",
    "\"DATA\",\"P1\",\"Sidecar test\"\r\n",
    "\"GROUP\",\"FILE\"\r\n",
    "\"HEADING\",\"FILE_FSET\",\"FILE_NAME\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n",
    "\"TYPE\",\"ID\",\"X\"\r\n",
    "\"DATA\",\"FS1\",\"borehole-log.pdf\"\r\n",
);

/// The shared tail every well-formed fixture below needs: TRAN, UNIT and TYPE,
/// whose absence is three error-severity findings all by itself (Rules 14/15/17).
/// Without it every fixture here is invalid for a reason that has nothing to do
/// with what it is trying to demonstrate.
const WELL_FORMED_TAIL: &str = concat!(
    "\"GROUP\",\"TRAN\"\r\n",
    "\"HEADING\",\"TRAN_ISNO\",\"TRAN_DATE\",\"TRAN_PROD\",\"TRAN_STAT\",\"TRAN_AGS\",\"TRAN_RECV\"\r\n",
    "\"UNIT\",\"\",\"yyyy-mm-dd\",\"\",\"\",\"\",\"\"\r\n",
    "\"TYPE\",\"X\",\"DT\",\"X\",\"X\",\"X\",\"X\"\r\n",
    "\"DATA\",\"1\",\"2026-08-05\",\"Producer\",\"Draft\",\"4.1.1\",\"Recipient\"\r\n",
    "\"GROUP\",\"UNIT\"\r\n",
    "\"HEADING\",\"UNIT_UNIT\",\"UNIT_DESC\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n",
    "\"TYPE\",\"X\",\"X\"\r\n",
    "\"DATA\",\"yyyy-mm-dd\",\"Date\"\r\n",
    "\"GROUP\",\"TYPE\"\r\n",
    "\"HEADING\",\"TYPE_TYPE\",\"TYPE_DESC\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n",
    "\"TYPE\",\"X\",\"X\"\r\n",
    "\"DATA\",\"ID\",\"Unique identifier\"\r\n",
    "\"DATA\",\"X\",\"Text\"\r\n",
    "\"DATA\",\"DT\",\"Date\"\r\n",
);

/// Breaks nothing at all. The baseline the two below are read against.
const CLEAN: &str = concat!(
    "\"GROUP\",\"PROJ\"\r\n",
    "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n",
    "\"TYPE\",\"ID\",\"X\"\r\n",
    "\"DATA\",\"P1\",\"Site investigation\"\r\n",
);

/// Findings, but no ERROR-severity finding — the `é` trips the extended-ASCII
/// FYI and nothing else. This is the fixture that separates `is_valid` from
/// `is_empty`: a clean file and an error file agree about both, so neither can
/// show that they are different questions. Here they disagree, which is exactly
/// what `is_valid`'s doc comment claims.
const FYI_ONLY: &str = concat!(
    "\"GROUP\",\"PROJ\"\r\n",
    "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n",
    "\"TYPE\",\"ID\",\"X\"\r\n",
    "\"DATA\",\"P1\",\"Sité investigation\"\r\n",
);

/// Three errors and one FYI, so the three severity counts are three DIFFERENT
/// numbers. A fixture whose findings are all one severity cannot tell `count`
/// apart from "how many findings are there".
const MIXED: &str = concat!(
    "\"GROUP\",\"PROJ\"\r\n",
    "\"HEADING\",\"PROJ_ID\",\"PROJ_ZZZZ\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n",
    "\"TYPE\",\"ID\",\"X\"\r\n",
    "\"DATA\",\"P1\",\"Sité investigation\"\r\n",
);

fn report_for(head: &str) -> ags4::Report {
    let mut text = String::from(head);
    text.push_str(WELL_FORMED_TAIL);
    ags4::validate_str(&text)
        .warnings(true)
        .fyi(true)
        .run()
        .expect("fixture validates")
}

/// A per-test directory, so the on-disk cases cannot see each other's leftovers
/// when the harness runs them in parallel. No `tempfile` dev-dependency for the
/// sake of four tests — this crate currently has none at all, and a published
/// crate's dependency list is worth more than the convenience.
fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("laterite-validate-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

fn write_sample(name: &str) -> std::path::PathBuf {
    let path = scratch(name).join("delivery.ags");
    std::fs::write(&path, WITH_ATTACHMENT).expect("write fixture");
    path
}

/// Render a report as comparable text — the whole finding, not just the count,
/// because two runs agreeing on how many things are wrong while disagreeing
/// about which is exactly the failure this is here to catch.
fn digest(report: &ags4::Report) -> Vec<String> {
    let mut out: Vec<String> = report
        .findings()
        .iter()
        .map(|f| {
            format!(
                "{}|{}|{}|{}",
                f.rule(),
                f.group(),
                f.severity().as_str(),
                f.description()
            )
        })
        .collect();
    out.sort();
    out
}

#[test]
fn bytes_are_validated_without_a_filesystem() {
    let report = ags4::validate_bytes(WITH_ATTACHMENT.as_bytes())
        .run()
        .expect("bytes validate");
    // The fixture has no TRAN group, so the rules have something to say. The
    // assertion is that the engine RAN, not that this file is clean.
    assert!(
        !report.findings().is_empty(),
        "the rule engine did not run over the bytes at all"
    );
}

/// The one that matters. The engine's own notes record surfaces judging a single
/// file against two different dictionaries depending on whether it arrived as a
/// path or as bytes — same file, same flags, two answers. This pins that shut for
/// this crate.
#[test]
fn a_path_and_its_own_bytes_produce_identical_findings() {
    let path = write_sample("agreement");
    let from_path = ags4::validate(&path)
        .warnings(true)
        .fyi(true)
        .run()
        .unwrap();
    let from_bytes = ags4::validate_bytes(WITH_ATTACHMENT.as_bytes())
        .warnings(true)
        .fyi(true)
        .run()
        .unwrap();

    // Zero is a bad witness: two empty reports are trivially equal, and would let
    // this pass while comparing nothing.
    assert!(
        !digest(&from_path).is_empty(),
        "fixture produced no findings, so the comparison below proves nothing"
    );
    assert_eq!(
        digest(&from_path),
        digest(&from_bytes),
        "the same file read two ways must not produce two verdicts"
    );
}

/// Asking for the on-disk check without anything to check against must fail, not
/// come back clean. A false clean here is worse than an error: it reports that the
/// attachments were verified when nothing was ever looked at.
#[test]
fn requesting_the_on_disk_check_on_bytes_is_refused() {
    let err = ags4::validate_bytes(WITH_ATTACHMENT.as_bytes())
        .check_files(true)
        .run()
        .expect_err("must refuse rather than report Rule 20 clean");
    assert_eq!(err.kind(), laterite::ErrorKind::InvalidArgument);

    // The refusal has to say what the caller did wrong. `cannot validate 210
    // bytes` is true, useless, and was what this printed to begin with.
    let msg = err.to_string();
    for expected in ["check_files", "validate_bytes", "Rule 20"] {
        assert!(
            msg.contains(expected),
            "the message must name {expected} so the caller can act on it: {msg}"
        );
    }

    // ...once. The engine's own wording says the same thing, so carrying it as a
    // cause made `{:#}` and every anyhow chain print the explanation twice.
    assert_eq!(
        format!("{err:#}"),
        msg,
        "the explanation is already in the message; a cause here only repeats it"
    );
}

/// Every engine error token this crate maps has to be REACHED by a test, not
/// merely listed in the mapping.
///
/// `validator_kind` translates the engine's own kind tokens into our
/// `ErrorKind`, and two of its arms are only ever produced by the engine — a
/// caller cannot cause them directly the way a bad encoding label or a bad
/// edition string goes through this crate's own checks first. Deleting either
/// arm survived the suite until this test existed, because the tests that look
/// like they cover them (a missing file, a bad edition) reach `ErrorKind::Io`
/// and `ErrorKind::BadDictionary` by a different route entirely.
#[test]
fn engine_error_tokens_reach_the_kinds_they_map_to() {
    // `not_found` — the engine's, from `check_file` on a path that is not there.
    // (`fix`'s missing-file error comes from this crate's own `fs::read`.)
    let err = ags4::validate("no-such-file-anywhere.ags")
        .run()
        .expect_err("missing file");
    assert_eq!(err.kind(), laterite::ErrorKind::Io);

    // `unsupported_edition` — AGS **3**, which the engine refuses outright
    // rather than validating against a 4.x dictionary.
    let ags3 = concat!(
        "\"**PROJ\"\r\n",
        "\"*PROJ_ID\",\"*PROJ_NAME\"\r\n",
        "\"<UNITS>\",\"\"\r\n",
        "\"P1\",\"An AGS3 file\"\r\n",
    );
    let err = ags4::validate_str(ags3)
        .run()
        .expect_err("AGS3 is not validated against an AGS4 dictionary");
    assert_eq!(err.kind(), laterite::ErrorKind::BadDictionary);
}

/// `{}` stays terse so wrappers do not print the cause twice; `{:#}` shows it.
#[test]
fn the_alternate_format_appends_the_cause() {
    let err = ags4::validate("no-such-file-anywhere.ags")
        .run()
        .expect_err("missing file");

    let terse = format!("{err}");
    let full = format!("{err:#}");
    assert!(
        full.starts_with(&terse),
        "the alternate form must extend the terse one, not replace it: {full}"
    );
    assert!(
        full.len() > terse.len(),
        "`{{:#}}` added nothing, so the cause is still invisible: {full}"
    );
}

/// The error chain is walkable through the std trait, not only through `{:#}`.
///
/// These are different mechanisms and only the second was covered: `{:#}` reads
/// `self.source` directly inside `Display`, while `anyhow`, `eyre` and every
/// generic reporter call `std::error::Error::source()`. Returning `None` from it
/// survived the suite (#377) — the alternate-format test above stays green,
/// because it never touches the trait method, so a chain that renders in `{:#}`
/// could report nothing at all to a wrapper.
#[test]
fn the_cause_is_reachable_through_the_std_error_trait() {
    use std::error::Error as _;

    let err = ags4::validate("no-such-file-anywhere.ags")
        .run()
        .expect_err("missing file");

    let source = err
        .source()
        .expect("the engine detail is kept, so it must be reachable as a source");

    // It has to carry the detail, not merely exist. The terse `Display` of the
    // error deliberately omits it, so an empty source is indistinguishable from
    // none for any reporter that walks the chain.
    let detail = source.to_string();
    assert!(
        !detail.is_empty(),
        "the source rendered as nothing, so a chain shows the caller nothing"
    );
    assert!(
        format!("{err:#}").contains(&detail),
        "`{{:#}}` and the walked source disagree about the cause: {detail}"
    );

    // One link, and it ends. A cycle here would hang any reporter that walks it.
    assert!(source.source().is_none(), "the chain does not terminate");
}

/// `Debug` on an error shows its three fields — including the source, whose own
/// `Debug` is only ever reached through this one.
///
/// Both impls survived being stubbed to an empty rendering (#377): nothing had
/// ever read `{:?}` of either type, so a `dbg!` or an `unwrap()` panic on a
/// laterite error could have printed nothing useful at all.
#[test]
fn the_error_debug_rendering_shows_its_fields_and_its_cause() {
    use std::error::Error as _;

    let err = ags4::validate("no-such-file-anywhere.ags")
        .run()
        .expect_err("missing file");

    let rendered = format!("{err:?}");
    assert!(rendered.starts_with("Error {"), "got: {rendered}");
    for field in ["kind", "message", "source"] {
        assert!(rendered.contains(field), "{field} missing from: {rendered}");
    }
    assert!(
        rendered.contains("Io"),
        "the kind has to be in it, or Debug cannot tell two errors apart: {rendered}"
    );

    // `Source`'s own `Debug` writes the engine text, and this is the only place
    // it is reachable from — a stub there leaves `source:` rendering as nothing
    // while every assertion above still passes.
    let detail = err.source().expect("has a source").to_string();
    assert!(
        rendered.contains(&detail),
        "the source rendered empty inside the error: {rendered}"
    );

    // An error with no cause still renders — and says so rather than inventing one.
    let bare = ags4::validate_bytes(WITH_ATTACHMENT.as_bytes())
        .check_files(true)
        .run()
        .expect_err("refused");
    let rendered = format!("{bare:?}");
    assert!(rendered.starts_with("Error {"), "got: {rendered}");
    assert!(rendered.contains("None"), "got: {rendered}");
}

/// The counterpart: from a path the same request is answerable, and answers.
/// Without this, the test above would also pass if `check_files` had simply
/// stopped working everywhere.
#[test]
fn the_on_disk_check_runs_from_a_path_and_finds_the_missing_sidecar() {
    let path = write_sample("sidecar");

    let without = ags4::validate(&path).warnings(true).run().unwrap();
    let with = ags4::validate(&path)
        .warnings(true)
        .check_files(true)
        .run()
        .unwrap();

    let extra: Vec<_> = digest(&with)
        .into_iter()
        .filter(|f| !digest(&without).contains(f))
        .collect();
    assert!(
        !extra.is_empty(),
        "check_files(true) added no finding, so Rule 20's on-disk half never ran \
         even though FILE/FS1/borehole-log.pdf is absent"
    );
}

/// The service shape end to end: take bytes, edit, write, validate the result —
/// never once touching a path.
#[test]
fn a_document_can_be_edited_and_revalidated_entirely_in_memory() {
    let mut doc = ags4::read_bytes(WITH_ATTACHMENT.as_bytes()).run().unwrap();
    doc.set_cell("PROJ", 0, "PROJ_NAME", "Edited in memory")
        .unwrap();

    let written = ags4::write(&doc).to_bytes().unwrap();
    let report = ags4::validate_bytes(written.bytes()).run().unwrap();

    assert_eq!(
        ags4::read_bytes(written.bytes())
            .run()
            .unwrap()
            .group("PROJ")
            .unwrap()
            .row(0)
            .unwrap()
            .cell("PROJ_NAME"),
        Some("Edited in memory"),
        "the edit survived the write that was validated"
    );
    // Whatever the verdict, it has to be a verdict — not a panic and not an error.
    let _ = report.is_valid();
}

/// `is_valid` asks about ERROR severity, and `is_empty` asks whether anything was
/// found at all. Nothing here had ever asserted the first one FALSE.
///
/// Its three call sites in this crate were two `assert!(report.is_valid())` in
/// `certificates.rs` and one `let _ =` above, so `is_valid -> true` and the
/// `==`→`!=` in its severity comparison both survived a sweep (#377): the
/// predicate could be stubbed to "everything is valid" with the suite green.
///
/// The FYI-only fixture is what makes this more than a second `assert!`. On a
/// clean file and on an error file the two accessors agree, so either could be
/// the other; here they must disagree.
#[test]
fn a_valid_file_and_an_empty_report_are_different_questions() {
    let clean = report_for(CLEAN);
    assert!(clean.is_valid(), "the baseline fixture is not clean");
    assert!(clean.is_empty(), "the baseline fixture produced findings");

    let fyi = report_for(FYI_ONLY);
    assert!(
        fyi.is_valid(),
        "an FYI is not an error, so the file is still valid AGS4"
    );
    assert!(
        !fyi.is_empty(),
        "the FYI fixture produced nothing, so it cannot separate the two"
    );

    let mixed = report_for(MIXED);
    assert!(
        !mixed.is_valid(),
        "a file breaking Rule 9 is not valid, whatever else it carries"
    );
    assert!(!mixed.is_empty());
}

/// Each count is its own number. Asserted on a fixture where all three differ,
/// so no constant and no swapped comparison satisfies them together.
#[test]
fn the_severity_counts_are_counted_per_severity() {
    let mixed = report_for(MIXED);

    assert_eq!(mixed.count(ags4::Severity::Error), 3, "errors");
    assert_eq!(mixed.count(ags4::Severity::Fyi), 1, "fyi");
    assert_eq!(
        mixed.count(ags4::Severity::Warning),
        0,
        "nothing in this fixture is a warning"
    );

    // The counts have to account for the findings, or `count` is filtering on
    // something other than severity.
    assert_eq!(mixed.findings().len(), 4);
}

/// The severity tokens are a cross-surface contract — the same strings Python,
/// Node and `lat` emit — so they are pinned to literals here rather than
/// inferred from a report.
///
/// This needs no fixture at all, which is why the gap lasted: the only reads of
/// `as_str` in the suite were inside `digest`, whose output is only ever compared
/// with another `digest`, so `""` and `"xyzzy"` both survived (#377).
#[test]
fn the_severity_tokens_are_the_documented_ones() {
    assert_eq!(ags4::Severity::Error.as_str(), "error");
    assert_eq!(ags4::Severity::Warning.as_str(), "warning");
    assert_eq!(ags4::Severity::Fyi.as_str(), "fyi");
}

/// A finding's fields, compared against LITERALS.
///
/// `digest` exists and does its job — catching two runs that agree on how many
/// findings there are while disagreeing about which — but every use compares one
/// digest with another, so both sides move together and nothing inside the format
/// string is pinned. `group`, `description` and `line` all survived on that
/// account.
#[test]
fn a_findings_fields_are_the_engine_s_own_values() {
    let mixed = report_for(MIXED);
    let find = |rule: &str| {
        mixed
            .findings()
            .iter()
            .find(|f| f.rule() == rule)
            .unwrap_or_else(|| panic!("no {rule} finding in {:?}", digest(&mixed)))
    };

    // Rule 9 names the offending heading, and knows which line it was on.
    let r9 = find("AGS Format Rule 9");
    assert_eq!(r9.group(), "PROJ");
    assert_eq!(r9.severity(), ags4::Severity::Error);
    assert_eq!(r9.line(), Some(2), "the HEADING row is line 2");
    assert!(
        r9.description().contains("PROJ_ZZZZ"),
        "the description must name the heading it rejected: {}",
        r9.description()
    );

    // A finding about the file rather than a group carries an EMPTY group and a
    // line — the case `group`'s doc comment calls out, and the one a fixture of
    // group-scoped findings alone would never reach.
    let fyi = find("FYI (Related to Rule 1)");
    assert_eq!(fyi.group(), "");
    assert_eq!(fyi.severity(), ags4::Severity::Fyi);
    assert_eq!(fyi.line(), Some(5), "the DATA row carrying the `é`");

    // And a finding with no line at all, so `line` cannot be a constant `Some`.
    let r18 = find("AGS Format Rule 18");
    assert_eq!(r18.line(), None, "a whole-file rule has no one line");
    assert_eq!(r18.group(), "DICT");
}

/// Both `Debug` impls, which nothing had ever read.
///
/// Same reasoning as the transport renderings in #273: what is worth asserting
/// about an impl carrying no secret is that it is INFORMATIVE. Stubbed to
/// `Ok(Default::default())` — an empty rendering — both survived.
#[test]
fn the_debug_renderings_show_what_they_are_supposed_to_show() {
    let mixed = report_for(MIXED);

    // `Report` deliberately renders COUNTS rather than the findings, because a
    // report over a bad file can carry thousands and a panic message is the wrong
    // place for them. So the counts are what has to be in it.
    let rendered = format!("{mixed:?}");
    assert!(rendered.starts_with("Report {"), "got: {rendered}");
    for field in ["valid", "errors", "warnings", "fyi"] {
        assert!(rendered.contains(field), "{field} missing from: {rendered}");
    }
    assert!(
        rendered.contains("false") && rendered.contains('3'),
        "the rendering must carry the verdict and the counts, not just their \
         labels: {rendered}"
    );
    assert!(
        !rendered.contains("PROJ_ZZZZ"),
        "Report must not render the findings themselves: {rendered}"
    );

    let one = format!(
        "{:?}",
        mixed
            .findings()
            .iter()
            .find(|f| f.rule() == "AGS Format Rule 9")
            .expect("Rule 9 finding")
    );
    assert!(one.starts_with("Finding {"), "got: {one}");
    for field in ["rule", "group", "line", "severity", "description"] {
        assert!(one.contains(field), "{field} missing from: {one}");
    }
    assert!(one.contains("PROJ_ZZZZ"), "got: {one}");
}
