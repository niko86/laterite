//! Reconciling several AGS4 deliveries of one project into one file.
//!
//! Public API only. What a merge can get quietly wrong is the interesting part:
//!
//! - order is meaning (a later file wins), and getting it backwards silently
//!   returns stale data,
//! - a re-sorted delivery must merge ONTO its prior self, not double it,
//! - a type clash is refused by default, because resolving two producers'
//!   declared types is not something to do on a caller's behalf unasked,
//! - a unit clash is refused in EVERY mode, and for a different reason.

use std::fs;

use laterite::ErrorKind;
use laterite::ags4::{self, TypeClash, WriteMode};

/// The first delivery: two boreholes.
const FIRST: &str = concat!(
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
    "\"DATA\",\"BH02\",\"9.00\"\r\n",
);

/// The second: BH01 revised, BH03 new — and the rows RE-SORTED, so a merge that
/// matched on position rather than KEY would double BH01 instead of revising it.
const SECOND: &str = concat!(
    "\"GROUP\",\"PROJ\"\r\n",
    "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n",
    "\"TYPE\",\"ID\",\"X\"\r\n",
    "\"DATA\",\"P1\",\"A site\"\r\n",
    "\"GROUP\",\"LOCA\"\r\n",
    "\"HEADING\",\"LOCA_ID\",\"LOCA_GL\"\r\n",
    "\"UNIT\",\"\",\"m\"\r\n",
    "\"TYPE\",\"ID\",\"2DP\"\r\n",
    "\"DATA\",\"BH03\",\"7.25\"\r\n",
    "\"DATA\",\"BH01\",\"13.75\"\r\n",
);

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("laterite-merge-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn level(merged: &ags4::Merged, id: &str) -> String {
    let doc = ags4::read_str(merged.text()).run().unwrap();
    let loca = doc.group("LOCA").expect("merged file has a LOCA");
    loca.rows()
        .find(|r| r.cell("LOCA_ID") == Some(id))
        .unwrap_or_else(|| panic!("{id} missing from the merge:\n{}", merged.text()))
        .cell("LOCA_GL")
        .unwrap()
        .to_string()
}

/// The headline. Three distinct boreholes out of two files that share one, with
/// the later file's value winning — and no duplicate, despite the re-sort.
#[test]
fn a_later_delivery_revises_rather_than_duplicates() {
    let merged = ags4::merge_bytes([FIRST.as_bytes(), SECOND.as_bytes()])
        .run()
        .unwrap();

    let doc = ags4::read_str(merged.text()).run().unwrap();
    let loca = doc.group("LOCA").unwrap();
    assert_eq!(
        loca.len(),
        3,
        "BH01 revised, not doubled:\n{}",
        merged.text()
    );

    assert_eq!(level(&merged, "BH01"), "13.75", "the later file wins");
    assert_eq!(
        level(&merged, "BH02"),
        "9.00",
        "carried over from the first"
    );
    assert_eq!(level(&merged, "BH03"), "7.25", "new in the second");
}

/// Order is meaning, so reversing it must reverse the outcome. Without this the
/// test above passes for a merge that always picks the FIRST file.
#[test]
fn reversing_the_order_reverses_who_wins() {
    let forwards = ags4::merge_bytes([FIRST.as_bytes(), SECOND.as_bytes()])
        .run()
        .unwrap();
    let backwards = ags4::merge_bytes([SECOND.as_bytes(), FIRST.as_bytes()])
        .run()
        .unwrap();

    assert_eq!(level(&forwards, "BH01"), "13.75");
    assert_eq!(level(&backwards, "BH01"), "12.50");
}

/// A merge has to be auditable — which row a later file changed, and what in it.
#[test]
fn a_revised_row_is_reported_with_what_changed() {
    let merged = ags4::merge_bytes([FIRST.as_bytes(), SECOND.as_bytes()])
        .run()
        .unwrap();

    let revision = merged
        .revisions()
        .iter()
        .find(|r| r.key() == ["BH01"])
        .unwrap_or_else(|| panic!("BH01's revision unreported: {:?}", merged.revisions()));

    assert_eq!(revision.group(), "LOCA");
    assert_eq!(revision.changed(), ["LOCA_GL"]);
    assert_eq!(revision.winner(), 1, "the second file supplied it");

    // With a third file the index has to MOVE, or `winner` could be the
    // constant 1 — which is what it was until mutation testing said so.
    let third = SECOND.replace("\"BH01\",\"13.75\"", "\"BH01\",\"20.00\"");
    let merged = ags4::merge_bytes([FIRST.as_bytes(), SECOND.as_bytes(), third.as_bytes()])
        .run()
        .unwrap();
    let last = merged
        .revisions()
        .iter()
        .filter(|r| r.key() == ["BH01"])
        .map(ags4::Revision::winner)
        .max()
        .expect("BH01 was revised twice");
    assert_eq!(last, 2, "the third file had the final say");
}

/// Merging one file is a question with no answer. Returning it unchanged would
/// hide a caller who meant to pass more.
#[test]
fn fewer_than_two_sources_is_refused() {
    let err = ags4::merge_bytes([FIRST.as_bytes()]).run().unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidArgument);
    assert!(err.to_string().contains("two"), "{err}");

    let err = ags4::merge_bytes(Vec::<Vec<u8>>::new()).run().unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidArgument);
}

/// Two producers typing one column differently is refused by default — the
/// resolution is a judgement about their data, not a detail to settle silently.
#[test]
fn a_type_clash_is_refused_by_default() {
    let typed_as_text = FIRST.replace("\"TYPE\",\"ID\",\"2DP\"", "\"TYPE\",\"ID\",\"X\"");

    let err = ags4::merge_bytes([typed_as_text.as_bytes(), SECOND.as_bytes()])
        .run()
        .unwrap_err();

    assert_eq!(err.kind(), ErrorKind::TypeConflict);
    assert_eq!(
        err.kind().as_str(),
        "type_conflict",
        "the shared wire token"
    );
    let msg = err.to_string();
    assert!(msg.contains("LOCA_GL"), "{msg}");
    // The message has to carry the remedy, or the caller has nowhere to go.
    assert!(msg.contains("widen") || msg.contains("promote"), "{msg}");
}

/// A note has to say WHERE, not just that something happened. `type_widened`
/// is the one that carries a heading, so it is the case that can tell a real
/// accessor from one returning `None`.
///
/// Two different NON-`X` types, deliberately: `X` absorbs a typed column
/// silently, so an `X`-vs-`2DP` clash widens without a word and this test would
/// find no note at all. It is the pair that has no obvious winner that warns.
#[test]
fn a_note_names_the_group_and_heading_it_settled() {
    let three_sig_figs = FIRST.replace("\"TYPE\",\"ID\",\"2DP\"", "\"TYPE\",\"ID\",\"3SF\"");

    let merged = ags4::merge_bytes([three_sig_figs.as_bytes(), SECOND.as_bytes()])
        .on_type_clash(TypeClash::Widen)
        .run()
        .unwrap();

    let note = merged
        .notes()
        .iter()
        .find(|n| n.kind() == "type_widened")
        .unwrap_or_else(|| panic!("widening said nothing: {:?}", merged.notes()));

    assert_eq!(note.group(), Some("LOCA"));
    assert_eq!(note.heading(), Some("LOCA_GL"));
    // The message content, not merely its presence — a note whose text is empty
    // or wrong is a note that helps nobody, and only a content assertion sees
    // it. The WHERE lives in the fields above, so the message carries the what:
    // which two types disagreed and what they were settled to.
    let msg = note.message();
    assert!(msg.contains("2DP") && msg.contains("3SF"), "{msg}");
    assert!(msg.contains("widened"), "{msg}");
}

#[test]
fn widen_settles_a_type_clash_as_free_text() {
    let typed_as_text = FIRST.replace("\"TYPE\",\"ID\",\"2DP\"", "\"TYPE\",\"ID\",\"X\"");

    let merged = ags4::merge_bytes([typed_as_text.as_bytes(), SECOND.as_bytes()])
        .on_type_clash(TypeClash::Widen)
        .run()
        .unwrap();

    let doc = ags4::read_str(merged.text()).run().unwrap();
    let loca = doc.group("LOCA").unwrap();
    let i = loca
        .headings()
        .iter()
        .position(|h| *h == "LOCA_GL")
        .unwrap();
    assert_eq!(loca.types()[i], "X", "widened to free text");
}

/// Promote keeps the column numeric at the greatest precision, zero-padding the
/// coarser file — and never the other way round, which would round data away.
#[test]
fn promote_takes_the_greater_precision_and_pads_the_coarser() {
    let coarse = FIRST
        .replace("\"TYPE\",\"ID\",\"2DP\"", "\"TYPE\",\"ID\",\"1DP\"")
        .replace("\"12.50\"", "\"12.5\"")
        .replace("\"9.00\"", "\"9.0\"");

    let merged = ags4::merge_bytes([coarse.as_bytes(), SECOND.as_bytes()])
        .on_type_clash(TypeClash::Promote)
        .run()
        .unwrap();

    let doc = ags4::read_str(merged.text()).run().unwrap();
    let loca = doc.group("LOCA").unwrap();
    let i = loca
        .headings()
        .iter()
        .position(|h| *h == "LOCA_GL")
        .unwrap();
    assert_eq!(loca.types()[i], "2DP", "max(n), never min");
    assert_eq!(
        level(&merged, "BH02"),
        "9.00",
        "the coarser file's cell padded"
    );
}

/// A unit disagreement is fatal in EVERY mode, unlike a type clash — no
/// resolution absorbs it, because picking one would silently mislabel the other
/// file's values. Asserted under `Widen` precisely because that is the mode a
/// caller would expect to make conflicts go away.
#[test]
fn a_unit_clash_is_refused_even_when_a_resolution_was_chosen() {
    let in_feet = FIRST.replace("\"UNIT\",\"\",\"m\"", "\"UNIT\",\"\",\"ft\"");

    let err = ags4::merge_bytes([in_feet.as_bytes(), SECOND.as_bytes()])
        .on_type_clash(TypeClash::Widen)
        .run()
        .unwrap_err();

    assert_eq!(err.kind(), ErrorKind::UnitConflict);
    assert_eq!(err.kind().as_str(), "unit_conflict");
    assert_ne!(
        err.kind(),
        ErrorKind::TypeConflict,
        "the two need different fixes and must be distinguishable"
    );
    let msg = err.to_string();
    assert!(msg.contains("LOCA_GL"), "{msg}");
}

/// A merged file is a NEW transmission, so an unstamped merge that had TRANs to
/// reconcile says so rather than passing one off as the merge's own.
///
/// The note only exists because there was a `TRAN` to keep — see the companion
/// below for the other case. Getting that wrong is how I wrote this test the
/// first time.
#[test]
fn keeping_an_input_tran_unstamped_is_reported() {
    let tran = |isno: &str, date: &str| {
        format!(
            concat!(
                "\"GROUP\",\"TRAN\"\r\n",
                "\"HEADING\",\"TRAN_ISNO\",\"TRAN_DATE\",\"TRAN_PROD\",\"TRAN_STAT\",\"TRAN_AGS\",\"TRAN_RECV\"\r\n",
                "\"UNIT\",\"\",\"yyyy-mm-dd\",\"\",\"\",\"\",\"\"\r\n",
                "\"TYPE\",\"X\",\"DT\",\"X\",\"X\",\"X\",\"X\"\r\n",
                "\"DATA\",\"{}\",\"{}\",\"Producer\",\"Draft\",\"4.1.1\",\"Recipient\"\r\n",
            ),
            isno, date
        )
    };
    let first = format!("{FIRST}{}", tran("1", "2026-08-01"));
    let second = format!("{SECOND}{}", tran("2", "2026-08-05"));

    let merged = ags4::merge_bytes([first.as_bytes(), second.as_bytes()])
        .run()
        .unwrap();

    let note = merged
        .notes()
        .iter()
        .find(|n| n.kind() == "tran_not_stamped")
        .unwrap_or_else(|| panic!("unstamped merge said nothing: {:?}", merged.notes()));
    assert_eq!(note.group(), Some("TRAN"));

    // What actually survives is BOTH input transmissions, because TRAN_ISNO is a
    // KEY heading and ordinary reconciliation keeps rows with distinct keys.
    // Nothing was invented, which is the promise.
    let doc = ags4::read_str(merged.text()).run().unwrap();
    let issues: Vec<&str> = doc
        .group("TRAN")
        .unwrap()
        .rows()
        .filter_map(|r| r.cell("TRAN_ISNO"))
        .collect();
    assert_eq!(issues, ["1", "2"], "both inputs' transmissions survive");

    // #729: the note and the file must not disagree. The message used to say it
    // "kept the newest input's TRAN", which reads as one row, while the file
    // carried both — the BEHAVIOUR was pinned above and the CLAIM was not, so
    // the two contradicted each other for months with nothing red. These two
    // assertions tie the sentence to the outcome, in both directions.
    let msg = note.message();
    assert!(
        !msg.contains("newest input"),
        "the note promises one surviving row again: {msg}"
    );
    // It says the result breaks Rule 14. Verify that against the validator
    // rather than against the sentence, so a message that stops being true
    // fails here instead of misleading a caller.
    assert!(
        msg.contains("Rule 14"),
        "the note no longer names the consequence: {msg}"
    );
    let report = ags4::validate_str(merged.text()).run().unwrap();
    assert!(
        report
            .findings()
            .iter()
            .any(|f| f.rule() == "AGS Format Rule 14"),
        "the note claims Rule 14 is broken and the validator disagrees: {:?}",
        report
            .findings()
            .iter()
            .map(laterite::ags4::Finding::rule)
            .collect::<Vec<_>>()
    );

    // Stamping is what collapses them into the merge's own single transmission,
    // which is the reason the note exists at all.
    let stamped = ags4::merge_bytes([first.as_bytes(), second.as_bytes()])
        .transmission("3", "2026-08-06", "Merger", "Recipient", "Final")
        .run()
        .unwrap();
    assert!(
        !stamped
            .notes()
            .iter()
            .any(|n| n.kind() == "tran_not_stamped"),
        "{:?}",
        stamped.notes()
    );
    let doc = ags4::read_str(stamped.text()).run().unwrap();
    let tran = doc.group("TRAN").unwrap();
    assert_eq!(tran.len(), 1, "one merge transmission, not one per input");
    assert_eq!(tran.row(0).unwrap().cell("TRAN_ISNO"), Some("3"));
}

/// With no `TRAN` on either side and no stamp, none is invented — and there is
/// nothing to report, because nothing was kept or discarded.
#[test]
fn no_transmission_anywhere_invents_none() {
    let merged = ags4::merge_bytes([FIRST.as_bytes(), SECOND.as_bytes()])
        .run()
        .unwrap();

    assert!(!merged.text().contains("\"TRAN\""), "{}", merged.text());
    assert!(
        !merged
            .notes()
            .iter()
            .any(|n| n.kind() == "tran_not_stamped"),
        "nothing was kept, so there is nothing to note: {:?}",
        merged.notes()
    );
}

#[test]
fn a_stated_transmission_is_written_into_the_merge() {
    let merged = ags4::merge_bytes([FIRST.as_bytes(), SECOND.as_bytes()])
        .transmission("3", "2026-08-06", "Merger", "Recipient", "Final")
        .run()
        .unwrap();

    let doc = ags4::read_str(merged.text()).run().unwrap();
    let tran = doc.group("TRAN").expect("a stated transmission is written");
    assert_eq!(tran.len(), 1, "one merge-TRAN row, not one per input");
    assert_eq!(tran.row(0).unwrap().cell("TRAN_PROD"), Some("Merger"));
}

#[test]
fn paths_bytes_and_documents_agree() {
    let dir = scratch("doors");
    let a = dir.join("a.ags");
    let b = dir.join("b.ags");
    fs::write(&a, FIRST).unwrap();
    fs::write(&b, SECOND).unwrap();

    let from_paths = ags4::merge([&a, &b]).run().unwrap();
    let from_bytes = ags4::merge_bytes([FIRST.as_bytes(), SECOND.as_bytes()])
        .run()
        .unwrap();

    let doc_a = ags4::read_str(FIRST).run().unwrap();
    let doc_b = ags4::read_str(SECOND).run().unwrap();
    let from_docs = ags4::merge_documents([&doc_a, &doc_b]).run().unwrap();

    assert_eq!(from_paths.bytes(), from_bytes.bytes());
    for merged in [&from_paths, &from_docs] {
        assert_eq!(level(merged, "BH01"), "13.75");
        assert_eq!(level(merged, "BH03"), "7.25");
    }
}

/// The handle door merges the documents as they stand, edits included — the
/// same rule `diff_documents` follows.
#[test]
fn an_edited_document_merges_what_was_edited() {
    let doc_a = ags4::read_str(FIRST).run().unwrap();
    let mut doc_b = ags4::read_str(SECOND).run().unwrap();
    doc_b.set_cell("LOCA", 1, "LOCA_GL", "42.00").unwrap();

    let merged = ags4::merge_documents([&doc_a, &doc_b]).run().unwrap();
    assert_eq!(level(&merged, "BH01"), "42.00");
}

#[test]
fn more_than_two_sources_merge_in_order() {
    let third = SECOND.replace("\"BH01\",\"13.75\"", "\"BH01\",\"20.00\"");

    let merged = ags4::merge_bytes([FIRST.as_bytes(), SECOND.as_bytes(), third.as_bytes()])
        .run()
        .unwrap();

    assert_eq!(level(&merged, "BH01"), "20.00", "the last file wins");
}

#[test]
fn save_writes_the_merged_bytes() {
    let dir = scratch("save");
    let out = dir.join("merged.ags");

    let merged = ags4::merge_bytes([FIRST.as_bytes(), SECOND.as_bytes()])
        .run()
        .unwrap();
    merged.save(&out).unwrap();

    // Against the TEXT, not against `bytes()` — `save` writes `bytes()`, so
    // comparing the two only proves the accessor agrees with itself, and both
    // sides move together if it returns a constant.
    let written = fs::read(&out).unwrap();
    assert_eq!(written, merged.text().as_bytes());
    assert!(!written.is_empty());
    assert!(
        String::from_utf8(written)
            .unwrap()
            .contains("\"GROUP\",\"LOCA\""),
        "the merged file really is the merged AGS4"
    );
}

#[test]
fn into_bytes_and_into_text_hand_over_the_same_content() {
    let make = || {
        ags4::merge_bytes([FIRST.as_bytes(), SECOND.as_bytes()])
            .run()
            .unwrap()
    };
    let text = make().into_text();
    assert_eq!(make().into_bytes(), text.clone().into_bytes());
    assert_eq!(make().text(), text);
}

#[test]
fn a_missing_file_is_an_io_error() {
    let dir = scratch("missing");
    let a = dir.join("a.ags");
    fs::write(&a, FIRST).unwrap();

    let err = ags4::merge([a, dir.join("nope.ags")]).run().unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Io);
}

#[test]
fn an_unknown_edition_is_refused_by_name() {
    let err = ags4::merge_bytes([FIRST.as_bytes(), SECOND.as_bytes()])
        .edition("4.9")
        .run()
        .unwrap_err();
    assert_eq!(err.kind(), ErrorKind::BadDictionary);
    assert!(err.to_string().contains("4.9"), "{err}");
}

#[test]
fn an_unknown_encoding_is_refused_by_name() {
    let err = ags4::merge_bytes([FIRST.as_bytes(), SECOND.as_bytes()])
        .encoding("klingon-1")
        .run()
        .unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidArgument);
    assert!(err.to_string().contains("klingon-1"), "{err}");
}

/// Which source failed has to be in the message — "cannot read as AGS4" with N
/// inputs names none of them.
#[test]
fn a_source_that_is_not_ags4_is_named_by_position() {
    let err = ags4::merge_bytes([FIRST.as_bytes(), b"not a delivery file".as_slice()])
        .run()
        .unwrap_err();

    assert_eq!(err.kind(), ErrorKind::NotAgs4);
    assert!(err.to_string().contains('1'), "{err}");
}

#[test]
fn the_write_mode_reaches_the_merged_output() {
    let strict = ags4::merge_bytes([FIRST.as_bytes(), SECOND.as_bytes()])
        .mode(WriteMode::Strict)
        .run();

    // The inputs carry no TRAN/UNIT/TYPE catalogues, so strict has something to
    // refuse — which is what proves the mode reached the emitter at all.
    assert!(strict.is_err(), "strict should refuse an incomplete merge");
    assert_eq!(strict.unwrap_err().kind(), ErrorKind::Emit);
}

#[test]
fn debug_reports_shape_and_never_the_data() {
    let rendered = format!(
        "{:?}",
        ags4::merge_bytes([FIRST.as_bytes(), SECOND.as_bytes()]).on_type_clash(TypeClash::Promote)
    );
    assert!(rendered.contains("Merge"), "{rendered}");
    assert!(rendered.contains("Promote"), "{rendered}");
    assert!(rendered.contains("bytes"), "{rendered}");
    assert!(
        !rendered.contains("BH01"),
        "Debug leaked the data: {rendered}"
    );

    let merged = ags4::merge_bytes([FIRST.as_bytes(), SECOND.as_bytes()])
        .run()
        .unwrap();
    let rendered = format!("{merged:?}");
    assert!(rendered.contains("Merged"), "{rendered}");
    assert!(rendered.contains("revisions"), "{rendered}");
    assert!(
        !rendered.contains("BH01"),
        "Debug leaked the data: {rendered}"
    );
}
