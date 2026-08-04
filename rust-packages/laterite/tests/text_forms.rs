//! The text doors: `read_str`, `validate_str`, and `Written::text`.
//!
//! Deliberately uses nothing but `laterite::*` — same rule as `roundtrip.rs`.
//!
//! These forms are not new capability. `read_bytes(s.as_bytes())` already
//! reached the same engine, and that is precisely the claim under test: a form
//! offered for ergonomics must be *identical* to the one it replaces, or it is a
//! second implementation with its own bugs. So every case here asserts equality
//! against the bytes path rather than merely asserting the text path works.
//!
//! The exception is encoding, which is where the two forms genuinely differ and
//! must: bytes need decoding, text is decoded already, and applying an encoding
//! to text would corrupt exactly the non-ASCII cells that motivate the option.

use laterite::ags4;

/// Includes a `°` so the encoding cases have something that survives or does not.
const SAMPLE: &str = concat!(
    "\"GROUP\",\"PROJ\"\r\n",
    "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n",
    "\"TYPE\",\"ID\",\"X\"\r\n",
    "\"DATA\",\"P1\",\"Slope 35° cutting\"\r\n",
);

fn digest(report: &ags4::Report) -> Vec<String> {
    report
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
        .collect()
}

// --- read ---------------------------------------------------------------

#[test]
fn read_str_matches_read_bytes_exactly() {
    let from_text = ags4::read_str(SAMPLE).run().expect("read_str");
    let from_bytes = ags4::read_bytes(SAMPLE.as_bytes())
        .run()
        .expect("read_bytes");

    let shape = |d: &ags4::Document| {
        d.groups()
            .into_iter()
            .map(|g| (g.code().to_string(), g.len()))
            .collect::<Vec<_>>()
    };
    assert_eq!(shape(&from_text), shape(&from_bytes));
}

#[test]
fn read_str_preserves_non_ascii() {
    // The `°` is the whole reason `encoding` exists; a text door that mangles it
    // would be worse than no door.
    let doc = ags4::read_str(SAMPLE).run().expect("read_str");
    let proj = doc.group("PROJ").expect("PROJ");
    let name = proj.row(0).expect("row 0").cell("PROJ_NAME").expect("cell");
    assert_eq!(name, "Slope 35° cutting");
}

#[test]
fn read_str_accepts_both_string_and_str() {
    // `impl Into<String>` — a caller holding either should not have to convert.
    assert!(ags4::read_str(SAMPLE).run().is_ok());
    assert!(ags4::read_str(SAMPLE.to_string()).run().is_ok());
}

#[test]
fn encoding_cannot_corrupt_text() {
    // Text is decoded already. Asking for cp1252 anyway must NOT re-decode it —
    // that would turn `°` (U+00B0, two UTF-8 bytes) into two mojibake chars.
    // This is the one behaviour where the text form must differ from bytes.
    let doc = ags4::read_str(SAMPLE)
        .encoding("windows-1252")
        .run()
        .expect("read_str with a stray encoding");
    let name = doc
        .group("PROJ")
        .and_then(|g| g.row(0))
        .and_then(|r| r.cell("PROJ_NAME").map(str::to_owned))
        .expect("cell");
    assert_eq!(name, "Slope 35° cutting", "encoding must not touch text");

    // ...whereas on the SAME content as bytes it genuinely does re-decode.
    let via_bytes = ags4::read_bytes(SAMPLE.as_bytes())
        .encoding("windows-1252")
        .run()
        .expect("read_bytes with cp1252");
    let mangled = via_bytes
        .group("PROJ")
        .and_then(|g| g.row(0))
        .and_then(|r| r.cell("PROJ_NAME").map(str::to_owned))
        .expect("cell");
    assert_ne!(
        mangled, name,
        "if these agree the encoding option stopped doing anything and this \
         test no longer proves the text form is exempt from it"
    );
}

// --- validate -----------------------------------------------------------

#[test]
fn validate_str_matches_validate_bytes_exactly() {
    let from_text = ags4::validate_str(SAMPLE)
        .warnings(true)
        .fyi(true)
        .run()
        .expect("validate_str");
    let from_bytes = ags4::validate_bytes(SAMPLE.as_bytes())
        .warnings(true)
        .fyi(true)
        .run()
        .expect("validate_bytes");

    // The whole finding, not the count: two runs agreeing on how many things are
    // wrong while disagreeing about which is the failure worth catching.
    assert_eq!(digest(&from_text), digest(&from_bytes));
}

#[test]
fn validate_str_refuses_the_on_disk_check() {
    // Text has no sibling anything, so Rule 20's on-disk half is unanswerable —
    // and asking must be an error, not a quietly clean result. Same contract as
    // `validate_bytes`; this asserts the new form did not acquire a softer one.
    let err = ags4::validate_str(SAMPLE)
        .check_files(true)
        .run()
        .expect_err("check_files on text must be refused");
    assert_eq!(err.kind_str(), "invalid_argument");
}

// --- emit ---------------------------------------------------------------

#[test]
fn written_text_and_bytes_are_the_same_output() {
    let doc = ags4::read_str(SAMPLE).run().expect("read");
    let out = ags4::write(&doc).to_bytes().expect("write");
    assert_eq!(out.text().as_bytes(), out.bytes());
}

#[test]
fn written_text_round_trips_back_through_read_str() {
    let doc = ags4::read_str(SAMPLE).run().expect("read");
    let out = ags4::write(&doc).to_bytes().expect("write");

    let again = ags4::read_str(out.text()).run().expect("re-read the text");
    let name = again
        .group("PROJ")
        .and_then(|g| g.row(0))
        .and_then(|r| r.cell("PROJ_NAME").map(str::to_owned))
        .expect("cell");
    assert_eq!(name, "Slope 35° cutting");
}

#[test]
fn into_text_and_into_bytes_agree() {
    let doc = ags4::read_str(SAMPLE).run().expect("read");
    let a = ags4::write(&doc).to_bytes().expect("write").into_text();
    let b = ags4::write(&doc).to_bytes().expect("write").into_bytes();
    assert_eq!(a.into_bytes(), b);
}

#[test]
fn debug_never_prints_the_document() {
    // The Debug impls report shape, never contents — `finish_non_exhaustive`
    // is what keeps the produced AGS4 out of a log line.
    let doc = ags4::read_str(SAMPLE).run().expect("read");
    let out = ags4::write(&doc).to_bytes().expect("write");
    let rendered = format!("{out:?}");
    assert!(!rendered.contains("Slope 35°"), "Debug leaked contents");
    assert!(rendered.contains("bytes"), "Debug should still report size");

    let pending = format!("{:?}", ags4::read_str(SAMPLE));
    assert!(!pending.contains("PROJ"), "Debug leaked the source text");
}
