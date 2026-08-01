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
