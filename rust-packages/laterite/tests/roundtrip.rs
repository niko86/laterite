//! Read → modify → write, through the public surface only.
//!
//! Deliberately uses nothing but `laterite::*`. If a test here needs an engine
//! crate to express itself, the facade has a hole — that is the point of not
//! importing one.

use laterite::ags4;

/// A minimal valid-ish AGS4 file. Written out longhand rather than generated,
/// so a change in how the writer quotes or orders things shows up as a diff
/// here rather than being regenerated into agreement with itself.
const SAMPLE: &str = concat!(
    "\"GROUP\",\"PROJ\"\r\n",
    "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n",
    "\"TYPE\",\"ID\",\"X\"\r\n",
    "\"DATA\",\"P1\",\"Old name\"\r\n",
    "\"GROUP\",\"LOCA\"\r\n",
    "\"HEADING\",\"LOCA_ID\",\"LOCA_TYPE\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n",
    "\"TYPE\",\"ID\",\"X\"\r\n",
    "\"DATA\",\"BH01\",\"CP\"\r\n",
    "\"DATA\",\"BH02\",\"CP\"\r\n",
);

fn sample() -> ags4::Document {
    ags4::read_bytes(SAMPLE.as_bytes()).run().expect("reads")
}

#[test]
fn reads_groups_in_file_order() {
    let doc = sample();
    assert_eq!(doc.codes(), ["PROJ", "LOCA"]);
    assert_eq!(doc.len(), 2);
    assert!(doc.contains("LOCA"));
    assert!(!doc.contains("SAMP"));
}

#[test]
fn walks_headings_and_rows() {
    let doc = sample();
    let loca = doc.group("LOCA").expect("LOCA is present");
    assert_eq!(loca.code(), "LOCA");
    assert_eq!(loca.headings(), ["LOCA_ID", "LOCA_TYPE"]);
    assert_eq!(loca.types(), ["ID", "X"]);
    assert_eq!(loca.len(), 2);

    let ids: Vec<&str> = loca.rows().filter_map(|r| r.cell("LOCA_ID")).collect();
    assert_eq!(ids, ["BH01", "BH02"]);
}

/// `Rows` reports its length without being consumed — the property that makes
/// it usable for pre-sizing a collection.
#[test]
fn rows_is_an_exact_size_iterator() {
    let doc = sample();
    let loca = doc.group("LOCA").unwrap();
    assert_eq!(loca.rows().len(), 2);
    assert_eq!(loca.rows().count(), 2);
}

#[test]
fn a_missing_heading_reads_as_none_not_empty() {
    let doc = sample();
    let row = doc.group("LOCA").unwrap().row(0).unwrap();
    assert_eq!(row.cell("LOCA_ID"), Some("BH01"));
    assert_eq!(
        row.cell("LOCA_NATE"),
        None,
        "an absent heading and a heading holding an empty string are different \
         facts about a delivery, and the API must not conflate them"
    );
}

#[test]
fn set_cell_changes_one_value() {
    let mut doc = sample();
    doc.set_cell("PROJ", 0, "PROJ_NAME", "New name").unwrap();
    let v = doc.group("PROJ").unwrap().row(0).unwrap().cell("PROJ_NAME");
    assert_eq!(v, Some("New name"));
}

#[test]
fn set_cell_refuses_what_it_cannot_do() {
    let mut doc = sample();
    for (group, row, heading) in [
        ("NOPE", 0, "PROJ_NAME"),
        ("PROJ", 99, "PROJ_NAME"),
        ("PROJ", 0, "PROJ_TYPO"),
    ] {
        let err = doc
            .set_cell(group, row, heading, "x")
            .expect_err("must refuse");
        assert_eq!(
            err.kind(),
            laterite::ErrorKind::InvalidArgument,
            "{group}/{row}/{heading}"
        );
    }
}

#[test]
fn push_row_appends_and_fills_unnamed_headings() {
    let mut doc = sample();
    doc.push_row("LOCA", &[("LOCA_ID", "BH03")]).unwrap();
    let loca = doc.group("LOCA").unwrap();
    assert_eq!(loca.len(), 3);
    let added = loca.row(2).unwrap();
    assert_eq!(added.cell("LOCA_ID"), Some("BH03"));
    assert_eq!(
        added.cell("LOCA_TYPE"),
        Some(""),
        "an unnamed heading is written empty, not omitted — a short row is a \
         Rule 4 violation and the writer would emit it"
    );
}

#[test]
fn remove_group_drops_it_from_order_too() {
    let mut doc = sample();
    assert!(doc.remove_group("LOCA"));
    assert!(!doc.remove_group("LOCA"), "second removal is a no-op");
    assert_eq!(doc.codes(), ["PROJ"]);
}

#[test]
fn writes_ags4_that_reads_back_identically() {
    let mut doc = sample();
    doc.set_cell("PROJ", 0, "PROJ_NAME", "Round trip").unwrap();

    let written = ags4::write(&doc).to_bytes().expect("writes");
    let back = ags4::read_bytes(written.bytes()).run().expect("re-reads");

    assert_eq!(
        back.group("PROJ")
            .unwrap()
            .row(0)
            .unwrap()
            .cell("PROJ_NAME"),
        Some("Round trip"),
        "the edit survives a write/read cycle"
    );
    assert_eq!(
        back.group("LOCA").unwrap().len(),
        2,
        "untouched data survives too"
    );
}

/// No `TRAN` unless the caller states one — the writer does not invent a claim
/// about who transferred what to whom.
#[test]
fn tran_is_absent_until_stated_and_present_after() {
    let doc = sample();

    let plain = ags4::write(&doc).to_bytes().unwrap();
    let plain_doc = ags4::read_bytes(plain.bytes()).run().unwrap();
    assert!(
        !plain_doc.contains("TRAN"),
        "a TRAN nobody supplied would be a fabricated provenance record"
    );

    let stamped = ags4::write(&doc)
        .transmission("1", "2026-08-01", "Producer", "Recipient", "Final")
        .to_bytes()
        .unwrap();
    let stamped_doc = ags4::read_bytes(stamped.bytes()).run().unwrap();
    let tran = stamped_doc.group("TRAN").expect("TRAN was stated");
    assert_eq!(tran.row(0).unwrap().cell("TRAN_PROD"), Some("Producer"));
}

#[test]
fn synthesise_metadata_derives_the_type_catalogue_and_can_be_turned_off() {
    let doc = sample();

    let with = ags4::read_bytes(ags4::write(&doc).to_bytes().unwrap().bytes())
        .run()
        .unwrap();
    assert!(
        with.contains("TYPE"),
        "the TYPE catalogue is derived from the types in use: {:?}",
        with.codes()
    );

    let without = ags4::read_bytes(
        ags4::write(&doc)
            .synthesise_metadata(false)
            .to_bytes()
            .unwrap()
            .bytes(),
    )
    .run()
    .unwrap();
    assert!(
        !without.contains("TYPE"),
        "turning it off has to actually reach the engine: {:?}",
        without.codes()
    );
}

/// UNIT is synthesised only when a unit is actually used — not unconditionally.
///
/// Worth pinning, because the obvious expectation ("metadata synthesis emits
/// UNIT and TYPE") is wrong and the reason is not obvious: a group with no DATA
/// rows is itself a Rule 2 error, so minting an empty UNIT catalogue would trade
/// a Rule 15 finding for a Rule 2 one and call that synthesis.
#[test]
fn unit_catalogue_appears_only_when_a_unit_is_in_use() {
    const UNITLESS: &str = concat!(
        "\"GROUP\",\"PROJ\"\r\n",
        "\"HEADING\",\"PROJ_ID\"\r\n",
        "\"UNIT\",\"\"\r\n",
        "\"TYPE\",\"ID\"\r\n",
        "\"DATA\",\"P1\"\r\n",
    );
    const WITH_UNIT: &str = concat!(
        "\"GROUP\",\"LOCA\"\r\n",
        "\"HEADING\",\"LOCA_ID\",\"LOCA_FDEP\"\r\n",
        "\"UNIT\",\"\",\"m\"\r\n",
        "\"TYPE\",\"ID\",\"2DP\"\r\n",
        "\"DATA\",\"BH01\",\"12.50\"\r\n",
    );

    for (src, expect_unit) in [(UNITLESS, false), (WITH_UNIT, true)] {
        let doc = ags4::read_bytes(src.as_bytes()).run().unwrap();
        let out = ags4::read_bytes(ags4::write(&doc).to_bytes().unwrap().bytes())
            .run()
            .unwrap();
        assert_eq!(
            out.contains("UNIT"),
            expect_unit,
            "expected UNIT present={expect_unit}, got {:?}",
            out.codes()
        );
    }
}

#[test]
fn unknown_edition_and_encoding_are_named_not_swallowed() {
    let doc = sample();
    let err = ags4::write(&doc).edition("9.9").to_bytes().unwrap_err();
    assert_eq!(err.kind(), laterite::ErrorKind::BadDictionary);
    assert!(
        err.to_string().contains("9.9"),
        "the message must name what was rejected: {err}"
    );

    let err = ags4::read_bytes(SAMPLE.as_bytes())
        .encoding("klingon-1")
        .run()
        .unwrap_err();
    assert_eq!(err.kind(), laterite::ErrorKind::InvalidArgument);
}

#[test]
fn editions_are_reported_and_usable() {
    let all = ags4::editions();
    assert!(!all.is_empty(), "zero is a bad witness");
    let doc = sample();
    for edition in all {
        ags4::write(&doc)
            .edition(edition)
            .to_bytes()
            .unwrap_or_else(|e| panic!("edition {edition} was advertised but rejected: {e}"));
    }
}

#[test]
fn a_duplicate_heading_is_refused_by_default_and_recoverable_on_request() {
    const DUPED: &str = concat!(
        "\"GROUP\",\"LOCA\"\r\n",
        "\"HEADING\",\"LOCA_ID\",\"LOCA_ID\"\r\n",
        "\"UNIT\",\"\",\"\"\r\n",
        "\"TYPE\",\"ID\",\"ID\"\r\n",
        "\"DATA\",\"BH01\",\"BH02\"\r\n",
    );
    assert!(
        ags4::read_bytes(DUPED.as_bytes()).run().is_err(),
        "read must refuse rather than silently drop a column"
    );

    let doc = ags4::read_bytes(DUPED.as_bytes())
        .recover_duplicate_headings(true)
        .run()
        .expect("recovery was requested");
    let loca = doc.group("LOCA").unwrap();
    assert_eq!(
        loca.headings().len(),
        2,
        "both columns survive recovery: {:?}",
        loca.headings()
    );
}
