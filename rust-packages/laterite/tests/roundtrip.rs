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

/// A group declared with no DATA rows — itself a Rule 2 violation, which is why
/// no valid fixture can supply the empty-group case.
const NO_ROWS: &str = concat!(
    "\"GROUP\",\"LOCA\"\r\n",
    "\"HEADING\",\"LOCA_ID\",\"LOCA_TYPE\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n",
    "\"TYPE\",\"ID\",\"X\"\r\n",
);

/// A group whose HEADING row declares nothing, so its DATA row has no cells.
/// The only route to an empty [`ags4::Row`] through the public surface.
const NO_HEADINGS: &str = concat!(
    "\"GROUP\",\"LOCA\"\r\n",
    "\"HEADING\"\r\n",
    "\"UNIT\"\r\n",
    "\"TYPE\"\r\n",
    "\"DATA\"\r\n",
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

    // Counted by walking to the end, because "it terminates" is half of what is
    // being claimed — but with a ceiling, so an iterator that never advances
    // fails HERE rather than spinning, which a bare `.count()` does.
    //
    // Note what this does not buy: a sweep still records `next` losing its
    // increment as a TIMEOUT rather than a failure, because
    // `walks_headings_and_rows` collects from `rows()` and hangs on the same
    // mutant, and one hung test holds the whole binary open however fast its
    // siblings fail. Bounding every consumer to change that would put a ceiling
    // in tests that are not about iteration, which reads worse than it repays —
    // and a timeout is still a kill, not a survivor.
    let mut seen = 0;
    for _ in loca.rows() {
        seen += 1;
        assert!(seen <= 100, "rows() did not terminate");
    }
    assert_eq!(seen, 2);
}

/// The remaining count goes DOWN as the iterator is stepped, and the values come
/// out in order.
///
/// The bounded walk above proves the iterator terminates on the right total; this
/// pins WHERE it is between steps, which is what `size_hint` promises a caller
/// pre-sizing a collection. Mutating `self.next += 1` to `*= 1` fails both, and
/// failed neither before (#377).
#[test]
fn stepping_rows_consumes_them() {
    let doc = sample();
    let loca = doc.group("LOCA").unwrap();
    let mut rows = loca.rows();

    assert_eq!(rows.len(), 2);
    assert_eq!(rows.next().unwrap().cell("LOCA_ID"), Some("BH01"));
    assert_eq!(rows.len(), 1, "the iterator did not advance");
    assert_eq!(rows.next().unwrap().cell("LOCA_ID"), Some("BH02"));
    assert_eq!(rows.len(), 0);
    assert!(rows.next().is_none(), "the iterator ran past its own end");
}

/// The `is_empty` half of every length accessor.
///
/// `len()` is asserted all over this file and `is_empty()` nowhere, so
/// `Document::is_empty`, `Group::is_empty` and `Row::is_empty` could each be
/// replaced by a constant (#377). Each needs BOTH answers to be reached, which is
/// the part that takes a fixture rather than a line.
#[test]
fn emptiness_is_asked_of_documents_groups_and_rows() {
    let mut doc = sample();

    // A document with groups, and then one without — `remove_group` is the only
    // way to reach the second state through the public surface.
    assert!(!doc.is_empty());
    assert!(doc.remove_group("PROJ"));
    assert!(doc.remove_group("LOCA"));
    assert!(doc.is_empty(), "every group was removed: {:?}", doc.codes());
    assert_eq!(doc.len(), 0);

    // A group with rows, and a group declared with none.
    let empty_doc = ags4::read_bytes(NO_ROWS.as_bytes()).run().expect("reads");
    let empty_group = empty_doc.group("LOCA").expect("LOCA is declared");
    assert!(empty_group.is_empty(), "the group has no DATA rows");
    assert_eq!(empty_group.len(), 0);
    assert!(empty_group.row(0).is_none());

    let full = sample();
    let loca = full.group("LOCA").unwrap();
    assert!(!loca.is_empty());

    // A row's own width — one cell per heading.
    let row = loca.row(0).unwrap();
    assert_eq!(row.len(), 2, "one cell per heading");
    assert!(!row.is_empty());

    // And a row with no cells at all. The first attempt at this test assumed that
    // was unreachable and asserted only the populated side, which left
    // `Row::is_empty -> false` alive. `NO_HEADINGS` parses.
    let bare = ags4::read_bytes(NO_HEADINGS.as_bytes())
        .run()
        .expect("reads");
    let bare_row = bare
        .group("LOCA")
        .expect("LOCA is declared")
        .row(0)
        .expect("the DATA row is there");
    assert_eq!(bare_row.len(), 0);
    assert!(bare_row.is_empty(), "a row with no headings has no cells");
}

/// `push_row` refuses a heading the group does not have — the counterpart of
/// `set_cell_refuses_what_it_cannot_do`, which covers only `set_cell`.
///
/// Its heading check was free: inverting the comparison inside it survived
/// (#377), because inverting `any(|x| x == h)` on a group with two headings is
/// true for every `h`, so the guard stops rejecting anything at all.
#[test]
fn push_row_refuses_a_heading_the_group_does_not_have() {
    let mut doc = sample();

    let err = doc
        .push_row("LOCA", &[("LOCA_TYPO", "x")])
        .expect_err("a heading that does not exist must be refused");
    assert_eq!(err.kind(), laterite::ErrorKind::InvalidArgument);
    assert!(
        err.to_string().contains("LOCA_TYPO"),
        "the message must name the heading it rejected: {err}"
    );

    // A real heading alongside a bogus one is still refused — the check is per
    // cell, not "did any of these look right".
    assert!(
        doc.push_row("LOCA", &[("LOCA_ID", "BH03"), ("LOCA_TYPO", "x")])
            .is_err()
    );
    assert_eq!(
        doc.group("LOCA").unwrap().len(),
        2,
        "a refused push must not have appended anything"
    );

    let err = doc
        .push_row("NOPE", &[("LOCA_ID", "BH03")])
        .expect_err("a group that does not exist must be refused");
    assert_eq!(err.kind(), laterite::ErrorKind::InvalidArgument);
}

/// The four `Debug` impls print SHAPE, never contents.
///
/// That is a deliberate choice — the module's own comment says a `dbg!` of a
/// document should not print a delivery — and nothing enforced it: all four
/// survived being stubbed to an empty rendering (#377). An empty rendering also
/// satisfies "does not leak", so the assertion has to be that each one is
/// informative AND that the cell values are absent.
#[test]
fn the_debug_renderings_show_shape_and_not_contents() {
    let doc = sample();
    let loca = doc.group("LOCA").unwrap();

    let rendered = format!("{doc:?}");
    assert!(rendered.starts_with("Document {"), "got: {rendered}");
    for field in ["groups", "codes", "source_bytes", "encoding", "sliced"] {
        assert!(rendered.contains(field), "{field} missing from: {rendered}");
    }
    assert!(
        !rendered.contains("BH01"),
        "a document's Debug must not print the delivery: {rendered}"
    );

    let rendered = format!("{loca:?}");
    assert!(rendered.starts_with("Group {"), "got: {rendered}");
    for field in ["code", "headings", "rows"] {
        assert!(rendered.contains(field), "{field} missing from: {rendered}");
    }
    assert!(rendered.contains("LOCA"), "got: {rendered}");
    assert!(!rendered.contains("BH01"), "got: {rendered}");

    let rendered = format!("{:?}", loca.row(0).unwrap());
    assert!(rendered.starts_with("Row {"), "got: {rendered}");
    assert!(rendered.contains("cells"), "got: {rendered}");
    assert!(
        !rendered.contains("BH01"),
        "a row's Debug must not print its own cells: {rendered}"
    );

    let rendered = format!("{:?}", loca.rows());
    assert!(rendered.starts_with("Rows {"), "got: {rendered}");
    for field in ["group", "remaining"] {
        assert!(rendered.contains(field), "{field} missing from: {rendered}");
    }
    assert!(!rendered.contains("BH01"), "got: {rendered}");
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
