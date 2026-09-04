//! AGS4 ↔ XLSX conversion, behind the `excel` feature.
//!
//! Public API only. The properties worth nailing down are the ones a converter
//! can get quietly wrong:
//!
//! - the UNIT/TYPE pseudo-rows must SURVIVE the workbook round trip — losing
//!   them still yields a plausible-looking file, just one whose typed columns
//!   have silently become text,
//! - declared TYPE formatting drives the way back: a spreadsheet holds numbers
//!   as floats, so `523145.10` only comes home under its `2DP` if the TYPE row
//!   made the trip and the re-formatter ran,
//! - the recovery and formatting knobs must actually reach the engine — a
//!   builder that dropped one would still compile and still convert,
//! - what cannot be converted errors under the kind the caller can act on.

#![cfg(feature = "excel")]

use std::fs;

use laterite::ErrorKind;
use laterite::ags4;

/// Two groups, three DATA rows. `LOCA_NATE` is `2DP` and its values are
/// deliberately NOT 2DP-formatted, so the round trip's formatting step is
/// observable rather than a no-op.
const DELIVERY: &str = concat!(
    "\"GROUP\",\"PROJ\"\r\n",
    "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n",
    "\"TYPE\",\"ID\",\"X\"\r\n",
    "\"DATA\",\"P1\",\"A site\"\r\n",
    "\r\n",
    "\"GROUP\",\"LOCA\"\r\n",
    "\"HEADING\",\"LOCA_ID\",\"LOCA_NATE\"\r\n",
    "\"UNIT\",\"\",\"m\"\r\n",
    "\"TYPE\",\"ID\",\"2DP\"\r\n",
    "\"DATA\",\"BH01\",\"523145.1\"\r\n",
    "\"DATA\",\"BH02\",\"523200\"\r\n",
);

/// A group declaring the same heading twice — refused by default, exactly as
/// [`ags4::read`] refuses it, and for the same reason: read naively, the second
/// column silently overwrites the first.
const DOUBLED_HEADING: &str = concat!(
    "\"GROUP\",\"PROJ\"\r\n",
    "\"HEADING\",\"PROJ_ID\",\"PROJ_ID\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n",
    "\"TYPE\",\"ID\",\"ID\"\r\n",
    "\"DATA\",\"P1\",\"P2\"\r\n",
);

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("laterite-excel-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn the_round_trip_reformats_typed_columns() {
    let workbook = ags4::to_excel_bytes(DELIVERY).run().unwrap();
    assert_eq!(workbook.sheets_written(), 2);
    assert_eq!(workbook.rows_written(), 3);
    assert!(workbook.warnings().is_empty(), "{:?}", workbook.warnings());

    let back = ags4::from_excel_bytes(workbook.bytes()).run().unwrap();
    assert_eq!(back.sheets_written(), 2);
    assert_eq!(back.rows_written(), 3);
    // The 2DP TYPE survived the trip AND drove the re-format — `523145.1`
    // comes home canonical. Both values, so a formatter that only handled
    // fractional input would still fail here on the integer one.
    assert!(back.text().contains("\"523145.10\""), "{}", back.text());
    assert!(back.text().contains("\"523200.00\""), "{}", back.text());
    // The text and byte views are one value.
    assert_eq!(back.text().as_bytes(), back.bytes());
}

#[test]
fn numeric_formatting_can_be_declined() {
    let workbook = ags4::to_excel_bytes(DELIVERY).run().unwrap();
    let raw = ags4::from_excel_bytes(workbook.bytes())
        .format_numeric_columns(false)
        .run()
        .unwrap();
    // The spreadsheet held the float `523145.1`; declining the re-format keeps
    // it that way. This is the assertion that proves the knob reaches the
    // engine — with it dropped, the default re-format runs and `523145.10`
    // appears.
    assert!(raw.text().contains("\"523145.1\""), "{}", raw.text());
    assert!(!raw.text().contains("\"523145.10\""), "{}", raw.text());
}

#[test]
fn to_path_writes_what_it_returns() {
    let dir = scratch("to-path");
    let src = dir.join("delivery.ags");
    fs::write(&src, DELIVERY).unwrap();

    let xlsx = dir.join("delivery.xlsx");
    let workbook = ags4::to_excel(&src).to_path(&xlsx).unwrap();
    assert_eq!(fs::read(&xlsx).unwrap(), workbook.bytes());

    let back_path = dir.join("back.ags");
    let converted = ags4::from_excel(&xlsx).to_path(&back_path).unwrap();
    assert_eq!(fs::read(&back_path).unwrap(), converted.bytes());
    assert!(converted.text().contains("\"523145.10\""));
}

#[test]
fn a_doubled_heading_is_refused_unless_recovery_is_asked_for() {
    let err = ags4::to_excel_bytes(DOUBLED_HEADING).run().unwrap_err();
    assert_eq!(err.kind(), ErrorKind::NotAgs4);

    let workbook = ags4::to_excel_bytes(DOUBLED_HEADING)
        .recover_duplicate_headings(true)
        .run()
        .unwrap();
    assert_eq!(workbook.sheets_written(), 1);
}

#[test]
fn a_legacy_encoding_is_decoded_when_named() {
    // `°` as cp1252's 0xB0 — invalid UTF-8, the byte that makes legacy
    // delivery files legacy.
    let cp1252: &[u8] = b"\"GROUP\",\"PROJ\"\r\n\
\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\
\"UNIT\",\"\",\"\"\r\n\
\"TYPE\",\"ID\",\"X\"\r\n\
\"DATA\",\"P1\",\"5\xB0 slope\"\r\n";

    let workbook = ags4::to_excel_bytes(cp1252)
        .encoding("windows-1252")
        .run()
        .unwrap();
    let back = ags4::from_excel_bytes(workbook.bytes()).run().unwrap();
    assert!(back.text().contains("5° slope"), "{}", back.text());
}

#[test]
fn an_unknown_encoding_label_is_refused_by_name() {
    let err = ags4::to_excel_bytes(DELIVERY)
        .encoding("not-a-charset")
        .run()
        .unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidArgument);
}

#[test]
fn input_that_is_not_ags4_errors_as_such() {
    let err = ags4::to_excel_bytes("no AGS4 here").run().unwrap_err();
    assert_eq!(err.kind(), ErrorKind::NotAgs4);
}

#[test]
fn a_workbook_that_is_not_xlsx_carries_the_engine_message() {
    let err = ags4::from_excel_bytes("not a workbook").run().unwrap_err();
    // `Other` on purpose — a broken workbook is a domain the shared ErrorKind
    // vocabulary does not name; the engine's own message is the useful part.
    assert_eq!(err.kind(), ErrorKind::Other);
}

#[test]
fn a_missing_path_errors_as_io() {
    let dir = scratch("missing");
    let err = ags4::to_excel(dir.join("absent.ags")).run().unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Io);
}
