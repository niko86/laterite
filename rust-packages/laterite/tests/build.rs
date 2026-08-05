//! Building AGS4 from data the caller holds.
//!
//! Public API only. The properties worth nailing down are the ones that
//! separate this door from [`ags4::write`]: that a *typed* cell is formatted to
//! its heading's declared TYPE while a string is not, that a miscounted row is
//! refused rather than quietly padded, and that a document round-trips through
//! it unchanged.

use laterite::ErrorKind;
use laterite::ags4::{self, Cell, GroupData, WriteMode};

/// PROJ + LOCA, the smallest pair that exercises a typed heading: `LOCA_GL` is
/// `2DP` in every edition, so what the emitter does with a number is visible.
fn proj() -> GroupData {
    GroupData::new("PROJ", ["PROJ_ID", "PROJ_NAME"]).row(["P1", "A site"])
}

fn text_of(built: &ags4::Written) -> String {
    built.text().to_string()
}

#[test]
fn data_becomes_ags4_that_reads_back() {
    let loca = GroupData::new("LOCA", ["LOCA_ID", "LOCA_GL"])
        .row([Cell::from("BH01"), Cell::from(12.5)])
        .row([Cell::from("BH02"), Cell::from(3.0)]);

    let built = ags4::build(vec![proj(), loca]).run().unwrap();
    let doc = ags4::read_str(built.text()).run().unwrap();

    let loca = doc.group("LOCA").unwrap();
    assert_eq!(loca.len(), 2);
    assert_eq!(loca.row(0).unwrap().cell("LOCA_ID"), Some("BH01"));
    assert_eq!(doc.group("PROJ").unwrap().len(), 1);
}

/// The whole reason [`Cell`] is typed rather than a string.
///
/// Under `Report` nothing is fixed after the write, so what lands in the file is
/// exactly what the emitter formatted — a number goes through the heading's
/// declared TYPE (`LOCA_GL` is `2DP`), a string does not.
#[test]
fn a_number_is_formatted_to_its_type_and_a_string_is_not() {
    let typed =
        GroupData::new("LOCA", ["LOCA_ID", "LOCA_GL"]).row([Cell::from("BH01"), Cell::from(12.5)]);
    let stringly = GroupData::new("LOCA", ["LOCA_ID", "LOCA_GL"]).row(["BH01", "12.5"]);

    let from_number = ags4::build(vec![proj(), typed])
        .mode(WriteMode::Report)
        .run()
        .unwrap();
    let from_string = ags4::build(vec![proj(), stringly])
        .mode(WriteMode::Report)
        .run()
        .unwrap();

    assert!(
        text_of(&from_number).contains("\"12.50\""),
        "a 2DP heading should pad a number: {}",
        text_of(&from_number)
    );
    assert!(
        text_of(&from_string).contains("\"12.5\""),
        "a string is written verbatim: {}",
        text_of(&from_string)
    );
}

/// `AutoFix` reaches the same place by a different route — the numeric
/// reformatter pads the string afterwards. Pinned so the test above cannot be
/// read as "strings are never normalised": they are, just not by the emitter.
///
/// The count is asserted on BOTH modes. `> 0` alone is satisfied by a
/// `fixes_applied` hard-wired to 1, and `Report` is the mode that makes zero
/// the right answer — mutation testing found that gap.
#[test]
fn autofix_pads_the_string_the_emitter_left_alone() {
    let stringly = || GroupData::new("LOCA", ["LOCA_ID", "LOCA_GL"]).row(["BH01", "12.5"]);

    let built = ags4::build(vec![proj(), stringly()]).run().unwrap();
    assert!(text_of(&built).contains("\"12.50\""), "{}", text_of(&built));
    assert!(built.fixes_applied() > 0);

    let reported = ags4::build(vec![proj(), stringly()])
        .mode(WriteMode::Report)
        .run()
        .unwrap();
    assert_eq!(
        reported.fixes_applied(),
        0,
        "Report emits unchanged — nothing was fixed"
    );
}

#[test]
fn a_null_cell_is_the_empty_cell() {
    let loca = GroupData::new("LOCA", ["LOCA_ID", "LOCA_REM"])
        .row([Cell::from("BH01"), Cell::Null])
        .row([Cell::from("BH02"), Cell::from(None::<&str>)]);

    let built = ags4::build(vec![proj(), loca])
        .mode(WriteMode::Report)
        .run()
        .unwrap();
    let doc = ags4::read_str(built.text()).run().unwrap();
    let loca = doc.group("LOCA").unwrap();
    assert_eq!(loca.row(0).unwrap().cell("LOCA_REM"), Some(""));
    assert_eq!(loca.row(1).unwrap().cell("LOCA_REM"), Some(""));
}

/// A miscounted row is the caller's mistake, and saying so beats correcting it.
///
/// The safe fix set pads short rows, so without this check a row the caller got
/// wrong would come back as a *finding about the output* rather than as the
/// error it is.
#[test]
fn a_row_that_does_not_match_its_headings_is_refused() {
    let loca = GroupData::new("LOCA", ["LOCA_ID", "LOCA_GL"]).row(["BH01"]);
    let err = ags4::build(vec![proj(), loca]).run().unwrap_err();

    assert_eq!(err.kind(), ErrorKind::InvalidArgument);
    let msg = err.to_string();
    assert!(msg.contains("LOCA"), "{msg}");
    assert!(msg.contains("row 0"), "{msg}");
}

#[test]
fn units_and_types_must_cover_every_heading() {
    let loca = GroupData::new("LOCA", ["LOCA_ID", "LOCA_GL"])
        .units(["", "m", "extra"])
        .row(["BH01", "1.00"]);
    let err = ags4::build(vec![proj(), loca]).run().unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidArgument);
    assert!(err.to_string().contains("units"), "{err}");

    let loca = GroupData::new("LOCA", ["LOCA_ID", "LOCA_GL"])
        .types(["ID"])
        .row(["BH01", "1.00"]);
    let err = ags4::build(vec![proj(), loca]).run().unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidArgument);
    assert!(err.to_string().contains("types"), "{err}");
}

/// A heading the standard dictionary has never heard of gets an empty unit —
/// unless the caller supplies one, which is what the override is for.
#[test]
fn an_explicit_unit_reaches_the_file() {
    let loca = GroupData::new("LOCA", ["LOCA_ID", "LOCA_XTRA"])
        .units(["", "kPa"])
        .types(["ID", "2DP"])
        .row(["BH01", "1.00"]);

    let built = ags4::build(vec![proj(), loca])
        .mode(WriteMode::Report)
        .run()
        .unwrap();
    let doc = ags4::read_str(built.text()).run().unwrap();
    let loca = doc.group("LOCA").unwrap();
    let i = loca
        .headings()
        .iter()
        .position(|h| *h == "LOCA_XTRA")
        .unwrap();
    assert_eq!(loca.units()[i], "kPa");
    assert_eq!(loca.types()[i], "2DP");
}

/// The handle door. A document read in and built back out keeps its cells —
/// cells cross as text, because a document's cells were already formatted when
/// the file that carried them was written.
#[test]
fn a_document_builds_back_into_itself() {
    let source = concat!(
        "\"GROUP\",\"PROJ\"\r\n",
        "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n",
        "\"UNIT\",\"\",\"\"\r\n",
        "\"TYPE\",\"ID\",\"X\"\r\n",
        "\"DATA\",\"P1\",\"A site\"\r\n",
        "\"GROUP\",\"LOCA\"\r\n",
        "\"HEADING\",\"LOCA_ID\",\"LOCA_GL\"\r\n",
        "\"UNIT\",\"\",\"m\"\r\n",
        "\"TYPE\",\"ID\",\"2DP\"\r\n",
        "\"DATA\",\"BH01\",\"12.50\"\r\n",
    );
    let doc = ags4::read_str(source).run().unwrap();
    let built = ags4::build_document(&doc)
        .mode(WriteMode::Report)
        .run()
        .unwrap();
    let again = ags4::read_str(built.text()).run().unwrap();

    assert_eq!(again.group("LOCA").unwrap().len(), 1);
    assert_eq!(
        again.group("LOCA").unwrap().row(0).unwrap().cell("LOCA_GL"),
        Some("12.50")
    );
    assert_eq!(
        again
            .group("PROJ")
            .unwrap()
            .row(0)
            .unwrap()
            .cell("PROJ_NAME"),
        Some("A site")
    );
}

/// An edit made on the document survives the trip — the point of having the
/// handle door at all.
#[test]
fn an_edited_document_builds_what_was_edited() {
    let source = concat!(
        "\"GROUP\",\"PROJ\"\r\n",
        "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n",
        "\"UNIT\",\"\",\"\"\r\n",
        "\"TYPE\",\"ID\",\"X\"\r\n",
        "\"DATA\",\"P1\",\"Old name\"\r\n",
    );
    let mut doc = ags4::read_str(source).run().unwrap();
    doc.set_cell("PROJ", 0, "PROJ_NAME", "New name").unwrap();

    let built = ags4::build_document(&doc).run().unwrap();
    let again = ags4::read_str(built.text()).run().unwrap();
    assert_eq!(
        again
            .group("PROJ")
            .unwrap()
            .row(0)
            .unwrap()
            .cell("PROJ_NAME"),
        Some("New name")
    );
}

/// No `TRAN` is ever invented. Rule 14 reports the absence instead, which is
/// the honest answer — see `Build::transmission`.
#[test]
fn no_transmission_means_no_tran_group() {
    let built = ags4::build(vec![proj()]).run().unwrap();
    assert!(!built.text().contains("\"TRAN\""), "{}", built.text());
    assert!(
        built.findings().iter().any(|f| f.rule().contains("14")),
        "{:?}",
        built
            .findings()
            .iter()
            .map(ags4::Finding::rule)
            .collect::<Vec<_>>()
    );
}

#[test]
fn a_stated_transmission_is_written() {
    let built = ags4::build(vec![proj()])
        .transmission("1", "2026-08-05", "Producer", "Recipient", "Draft")
        .run()
        .unwrap();

    let doc = ags4::read_str(built.text()).run().unwrap();
    let tran = doc.group("TRAN").unwrap();
    assert_eq!(tran.row(0).unwrap().cell("TRAN_PROD"), Some("Producer"));
    assert_eq!(tran.row(0).unwrap().cell("TRAN_DATE"), Some("2026-08-05"));
}

#[test]
fn strict_refuses_data_that_would_be_invalid() {
    // PROJ alone is short of TRAN/UNIT/TYPE, so strict has something to refuse.
    let err = ags4::build(vec![proj()])
        .mode(WriteMode::Strict)
        .run()
        .unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Emit);
}

/// LOCA rather than PROJ alone, because a UNIT catalogue is only synthesised
/// when the data uses a unit at all — `LOCA_GL` carries `m` and PROJ carries
/// nothing, so PROJ on its own gets a TYPE catalogue and no UNIT, which reads
/// like the flag failing when it is working.
#[test]
fn synthesise_metadata_off_writes_no_catalogues() {
    let loca = || GroupData::new("LOCA", ["LOCA_ID", "LOCA_GL"]).row(["BH01", "12.50"]);

    let with = ags4::build(vec![proj(), loca()]).run().unwrap();
    let without = ags4::build(vec![proj(), loca()])
        .synthesise_metadata(false)
        .run()
        .unwrap();

    for group in ["\"GROUP\",\"UNIT\"", "\"GROUP\",\"TYPE\""] {
        assert!(with.text().contains(group), "{}", with.text());
        assert!(!without.text().contains(group), "{}", without.text());
    }
}

#[test]
fn an_unknown_edition_is_refused_by_name() {
    let err = ags4::build(vec![proj()]).edition("4.9").run().unwrap_err();
    assert_eq!(err.kind(), ErrorKind::BadDictionary);
    assert!(err.to_string().contains("4.9"), "{err}");
}

/// Every `From` the door advertises, exercised through the door rather than by
/// asserting on the enum — a conversion that compiles but writes the wrong cell
/// is the failure worth catching.
#[test]
fn every_cell_conversion_reaches_the_file() {
    let loca = GroupData::new(
        "LOCA",
        ["LOCA_ID", "LOCA_GL", "LOCA_REM", "LOCA_PURP", "LOCA_TERM"],
    )
    .row([
        Cell::from("BH01"),
        Cell::from(1.5),
        Cell::from(String::from("a remark")),
        Cell::from(7i32),
        Cell::from(true),
    ]);

    let built = ags4::build(vec![proj(), loca])
        .mode(WriteMode::Report)
        .run()
        .unwrap();
    let doc = ags4::read_str(built.text()).run().unwrap();
    let row = doc.group("LOCA").unwrap().row(0).unwrap();

    assert_eq!(row.cell("LOCA_ID"), Some("BH01"));
    assert_eq!(row.cell("LOCA_REM"), Some("a remark"));
    assert_eq!(row.cell("LOCA_PURP"), Some("7"));
    assert_eq!(row.cell("LOCA_TERM"), Some("true"));
}

/// `Debug` shows the shape, never the values — the same rule the read handles
/// follow, and it matters more here because the values are the caller's data.
#[test]
fn debug_shows_shape_not_data() {
    let loca = GroupData::new("LOCA", ["LOCA_ID"]).row(["a-secret-borehole-id"]);
    let rendered = format!("{:?}", ags4::build(vec![loca]));

    assert!(rendered.contains("LOCA x1"), "{rendered}");
    assert!(!rendered.contains("a-secret-borehole-id"), "{rendered}");
}

#[test]
fn a_group_reports_its_own_shape() {
    let empty = GroupData::new("LOCA", ["LOCA_ID"]);
    assert_eq!(empty.code(), "LOCA");
    assert!(empty.is_empty());
    assert_eq!(empty.len(), 0);

    let one = empty.row(["BH01"]);
    assert!(!one.is_empty());
    assert_eq!(one.len(), 1);
}
