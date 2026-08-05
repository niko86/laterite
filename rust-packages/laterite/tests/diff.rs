//! Comparing two revisions of an AGS4 file.
//!
//! Public API only. The two properties that make this worth having over a line
//! diff are the ones asserted hardest:
//!
//! - rows are matched by their dictionary KEY, so re-ordering a file is not a
//!   change,
//! - cells are compared through their declared TYPE, so re-formatting a number
//!   is not a change either.
//!
//! Both are things a text diff gets wrong, and both are things that would fail
//! silently — as a diff full of noise rather than as an error.

use std::fmt::Write as _;
use std::fs;

use laterite::ErrorKind;
use laterite::ags4::{self, Change};

/// Two boreholes, in order.
const BASELINE: &str = concat!(
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

/// The same two boreholes, **re-ordered**, with BH01's level genuinely changed
/// and BH02's merely re-spelled (`9.00` → `9.000`, the same number).
const REVISION: &str = concat!(
    "\"GROUP\",\"PROJ\"\r\n",
    "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n",
    "\"UNIT\",\"\",\"\"\r\n",
    "\"TYPE\",\"ID\",\"X\"\r\n",
    "\"DATA\",\"P1\",\"A site\"\r\n",
    "\"GROUP\",\"LOCA\"\r\n",
    "\"HEADING\",\"LOCA_ID\",\"LOCA_GL\"\r\n",
    "\"UNIT\",\"\",\"m\"\r\n",
    "\"TYPE\",\"ID\",\"2DP\"\r\n",
    "\"DATA\",\"BH02\",\"9.000\"\r\n",
    "\"DATA\",\"BH01\",\"13.75\"\r\n",
);

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("laterite-diff-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn loca(delta: &ags4::Delta) -> &ags4::GroupChange {
    delta
        .groups()
        .iter()
        .find(|g| g.code() == "LOCA")
        .unwrap_or_else(|| panic!("no LOCA change in {delta:?}"))
}

/// The headline: a re-ordered file with one real edit reports **one** change,
/// not two moves and a rewrite.
#[test]
fn rows_are_matched_by_key_not_by_position() {
    let delta = ags4::diff_bytes(BASELINE.as_bytes(), REVISION.as_bytes())
        .run()
        .unwrap();

    assert_eq!(delta.added(), 0, "{delta:?}");
    assert_eq!(delta.removed(), 0, "{delta:?}");
    assert_eq!(delta.changed(), 1, "{delta:?}");

    let g = loca(&delta);
    assert!(g.keyed(), "LOCA has a dictionary KEY");
    assert!(g.key_headings().contains(&"LOCA_ID".to_string()));

    let row = &g.rows()[0];
    assert_eq!(row.change(), Change::Changed);
    assert_eq!(row.key(), ["BH01"]);
}

/// The other half: `9.00` and `9.000` are the same number under `2DP`, so BH02
/// is not reported at all. A text diff would call it a change.
#[test]
fn a_reformatted_number_is_not_a_change() {
    let delta = ags4::diff_bytes(BASELINE.as_bytes(), REVISION.as_bytes())
        .run()
        .unwrap();

    let keys: Vec<&[String]> = loca(&delta)
        .rows()
        .iter()
        .map(ags4::RowChange::key)
        .collect();
    assert_eq!(keys.len(), 1, "only BH01 genuinely changed: {keys:?}");
    assert_eq!(keys[0], ["BH01"]);
}

#[test]
fn a_changed_cell_reports_both_sides_and_its_type() {
    let delta = ags4::diff_bytes(BASELINE.as_bytes(), REVISION.as_bytes())
        .run()
        .unwrap();
    let cells = loca(&delta).rows()[0].cells();

    assert_eq!(cells.len(), 1, "{cells:?}");
    assert_eq!(cells[0].heading(), "LOCA_GL");
    assert_eq!(cells[0].ags_type(), "2DP");
    assert_eq!(cells[0].baseline(), Some("12.50"));
    assert_eq!(cells[0].revision(), Some("13.75"));
}

#[test]
fn an_identical_pair_is_unchanged() {
    let delta = ags4::diff_bytes(BASELINE.as_bytes(), BASELINE.as_bytes())
        .run()
        .unwrap();

    assert!(delta.is_unchanged(), "{delta:?}");
    assert_eq!(delta.added() + delta.removed() + delta.changed(), 0);
    assert!(delta.groups().is_empty());
}

/// `is_unchanged` has to mean "no change of any shape", so a file that only
/// gains a whole group must not read as unchanged.
#[test]
fn a_new_group_alone_is_still_a_change() {
    let with_extra = format!(
        "{BASELINE}{}",
        concat!(
            "\"GROUP\",\"ABBR\"\r\n",
            "\"HEADING\",\"ABBR_HDNG\",\"ABBR_CODE\",\"ABBR_DESC\"\r\n",
            "\"UNIT\",\"\",\"\",\"\"\r\n",
            "\"TYPE\",\"X\",\"X\",\"X\"\r\n",
            "\"DATA\",\"LOCA_TYPE\",\"CP\",\"Cable percussion\"\r\n",
        )
    );
    let delta = ags4::diff_bytes(BASELINE.as_bytes(), with_extra.as_bytes())
        .run()
        .unwrap();

    assert!(!delta.is_unchanged(), "{delta:?}");
    assert_eq!(delta.groups_added(), ["ABBR"]);
    assert!(delta.groups_removed().is_empty());
}

/// A group dropped whole is **one** fact, not one per row it carried.
///
/// Its rows are deliberately not counted in `removed()` — the totals are over
/// the groups both sides have. Pinned because it is the surprising half of the
/// contract, and because summing the three totals is then the wrong way to ask
/// "did anything change": `is_unchanged()` is.
#[test]
fn a_dropped_group_is_reported_whole_and_not_row_by_row() {
    let delta = ags4::diff_bytes(
        BASELINE.as_bytes(),
        concat!(
            "\"GROUP\",\"PROJ\"\r\n",
            "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n",
            "\"UNIT\",\"\",\"\"\r\n",
            "\"TYPE\",\"ID\",\"X\"\r\n",
            "\"DATA\",\"P1\",\"A site\"\r\n",
        )
        .as_bytes(),
    )
    .run()
    .unwrap();

    assert_eq!(delta.groups_removed(), ["LOCA"]);
    assert_eq!(
        delta.removed(),
        0,
        "the two boreholes are not counted twice"
    );
    assert!(!delta.is_unchanged(), "dropping a group is still a change");
}

#[test]
fn an_added_and_a_removed_row_are_told_apart() {
    let revision = concat!(
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
        "\"DATA\",\"BH03\",\"7.25\"\r\n",
    );
    let delta = ags4::diff_bytes(BASELINE.as_bytes(), revision.as_bytes())
        .run()
        .unwrap();

    assert_eq!(delta.added(), 1);
    assert_eq!(delta.removed(), 1);
    assert_eq!(delta.changed(), 0);

    let g = loca(&delta);
    let verdict = |key: &str| {
        g.rows()
            .iter()
            .find(|r| r.key() == [key])
            .unwrap_or_else(|| panic!("no row for {key}"))
            .change()
    };
    assert_eq!(verdict("BH03"), Change::Added);
    assert_eq!(verdict("BH02"), Change::Removed);
}

/// A structural change is reported once, at the group, rather than as a change
/// on every row that now has an extra cell.
#[test]
fn a_new_heading_is_a_group_level_change() {
    let revision = concat!(
        "\"GROUP\",\"PROJ\"\r\n",
        "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n",
        "\"UNIT\",\"\",\"\"\r\n",
        "\"TYPE\",\"ID\",\"X\"\r\n",
        "\"DATA\",\"P1\",\"A site\"\r\n",
        "\"GROUP\",\"LOCA\"\r\n",
        "\"HEADING\",\"LOCA_ID\",\"LOCA_GL\",\"LOCA_REM\"\r\n",
        "\"UNIT\",\"\",\"m\",\"\"\r\n",
        "\"TYPE\",\"ID\",\"2DP\",\"X\"\r\n",
        "\"DATA\",\"BH01\",\"12.50\",\"\"\r\n",
        "\"DATA\",\"BH02\",\"9.00\",\"\"\r\n",
    );
    let delta = ags4::diff_bytes(BASELINE.as_bytes(), revision.as_bytes())
        .run()
        .unwrap();

    let g = loca(&delta);
    assert_eq!(g.headings_added(), ["LOCA_REM"]);
    assert!(g.headings_removed().is_empty());
    assert_eq!(g.changed(), 0, "no row's shared cells differ");
}

/// The cap bounds the DETAIL, never the counts — a summary that under-reported
/// the totals would be worse than no summary.
#[test]
fn the_row_cap_leaves_the_totals_alone() {
    let mut revision = String::from(
        "\"GROUP\",\"PROJ\"\r\n\
         \"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\
         \"UNIT\",\"\",\"\"\r\n\
         \"TYPE\",\"ID\",\"X\"\r\n\
         \"DATA\",\"P1\",\"A site\"\r\n\
         \"GROUP\",\"LOCA\"\r\n\
         \"HEADING\",\"LOCA_ID\",\"LOCA_GL\"\r\n\
         \"UNIT\",\"\",\"m\"\r\n\
         \"TYPE\",\"ID\",\"2DP\"\r\n",
    );
    for i in 0..10 {
        writeln!(revision, "\"DATA\",\"BH{i:02}\",\"1.00\"\r").unwrap();
    }

    let uncapped = ags4::diff_bytes(BASELINE.as_bytes(), revision.as_bytes())
        .run()
        .unwrap();
    let capped = ags4::diff_bytes(BASELINE.as_bytes(), revision.as_bytes())
        .max_rows_per_group(2)
        .run()
        .unwrap();

    assert_eq!(loca(&capped).added(), loca(&uncapped).added());
    assert_eq!(loca(&capped).removed(), loca(&uncapped).removed());
    assert!(loca(&uncapped).rows().len() > 2);
    assert_eq!(loca(&capped).rows().len(), 2);
}

/// A row's line numbers are how a caller gets from a verdict back to the file.
///
/// Both sides, and on all three verdicts: `None` for the side a row is not on,
/// and the real line for the side it is. Either assertion alone passes with the
/// accessor hard-wired — `None` for everything satisfies the added/removed
/// halves, and a constant satisfies the changed one.
#[test]
fn a_row_carries_the_line_it_sits_on_in_each_revision() {
    let revision = concat!(
        "\"GROUP\",\"PROJ\"\r\n",
        "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n",
        "\"UNIT\",\"\",\"\"\r\n",
        "\"TYPE\",\"ID\",\"X\"\r\n",
        "\"DATA\",\"P1\",\"A site\"\r\n",
        "\"GROUP\",\"LOCA\"\r\n",
        "\"HEADING\",\"LOCA_ID\",\"LOCA_GL\"\r\n",
        "\"UNIT\",\"\",\"m\"\r\n",
        "\"TYPE\",\"ID\",\"2DP\"\r\n",
        "\"DATA\",\"BH01\",\"13.75\"\r\n",
        "\"DATA\",\"BH03\",\"7.25\"\r\n",
    );
    let delta = ags4::diff_bytes(BASELINE.as_bytes(), revision.as_bytes())
        .run()
        .unwrap();
    let g = loca(&delta);
    let row = |key: &str| {
        g.rows()
            .iter()
            .find(|r| r.key() == [key])
            .unwrap_or_else(|| panic!("no row for {key}"))
    };

    // BH01 changed: on both sides, line 10 in each fixture.
    assert_eq!(row("BH01").line_baseline(), Some(10));
    assert_eq!(row("BH01").line_revision(), Some(10));

    // BH02 was dropped: a baseline line, no revision line.
    assert_eq!(row("BH02").line_baseline(), Some(11));
    assert_eq!(row("BH02").line_revision(), None);

    // BH03 is new: the mirror image.
    assert_eq!(row("BH03").line_baseline(), None);
    assert_eq!(row("BH03").line_revision(), Some(11));
}

/// The per-group counts, asserted against known values rather than against each
/// other. Comparing a capped run with an uncapped one — which the cap test does
/// — is satisfied by an accessor returning a constant.
///
/// Each count is deliberately ≥2 or distinct: the mutator substitutes `0` and
/// `1`, so a group that happens to remove exactly one row cannot tell a working
/// `removed()` from a hard-wired one. That is why this fixture has its own
/// three-borehole baseline rather than reusing the shared one.
#[test]
fn a_group_counts_its_own_changes() {
    let baseline = concat!(
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
        "\"DATA\",\"BH05\",\"4.00\"\r\n",
    );
    let revision = concat!(
        "\"GROUP\",\"PROJ\"\r\n",
        "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n",
        "\"UNIT\",\"\",\"\"\r\n",
        "\"TYPE\",\"ID\",\"X\"\r\n",
        "\"DATA\",\"P1\",\"A site\"\r\n",
        "\"GROUP\",\"LOCA\"\r\n",
        "\"HEADING\",\"LOCA_ID\",\"LOCA_GL\"\r\n",
        "\"UNIT\",\"\",\"m\"\r\n",
        "\"TYPE\",\"ID\",\"2DP\"\r\n",
        "\"DATA\",\"BH01\",\"13.75\"\r\n",
        "\"DATA\",\"BH03\",\"7.25\"\r\n",
        "\"DATA\",\"BH04\",\"1.00\"\r\n",
    );
    let delta = ags4::diff_bytes(baseline.as_bytes(), revision.as_bytes())
        .run()
        .unwrap();
    let g = loca(&delta);

    assert_eq!(g.added(), 2, "BH03 and BH04");
    assert_eq!(g.removed(), 2, "BH02 and BH05");
    assert_eq!(g.changed(), 1, "BH01");

    // And the file-level totals are the sum of them, for the one group that
    // changed — the two numbers are separate accessors on separate types.
    assert_eq!(delta.added(), 2);
    assert_eq!(delta.removed(), 2);
    assert_eq!(delta.changed(), 1);
}

/// A group with no dictionary KEY headings falls back to matching whole rows,
/// which changes how the result reads — an edit shows up as a remove and an add.
///
/// The counterpart to LOCA being keyed. Without a case where `keyed()` is false
/// the accessor could return `true` unconditionally.
#[test]
fn a_group_the_dictionary_does_not_know_is_matched_unkeyed() {
    let with_custom = |rows: &str, headings: &str, unit: &str, ty: &str| {
        format!(
            "\"GROUP\",\"PROJ\"\r\n\
             \"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"\r\n\
             \"UNIT\",\"\",\"\"\r\n\
             \"TYPE\",\"ID\",\"X\"\r\n\
             \"DATA\",\"P1\",\"A site\"\r\n\
             \"GROUP\",\"ZZZZ\"\r\n\
             {headings}{unit}{ty}{rows}"
        )
    };
    let a = with_custom(
        "\"DATA\",\"one\",\"first\"\r\n",
        "\"HEADING\",\"ZZZZ_A\",\"ZZZZ_B\"\r\n",
        "\"UNIT\",\"\",\"\"\r\n",
        "\"TYPE\",\"X\",\"X\"\r\n",
    );
    let b = with_custom(
        "\"DATA\",\"one\"\r\n",
        "\"HEADING\",\"ZZZZ_A\"\r\n",
        "\"UNIT\",\"\"\r\n",
        "\"TYPE\",\"X\"\r\n",
    );

    let delta = ags4::diff_bytes(a.as_bytes(), b.as_bytes()).run().unwrap();
    let g = delta
        .groups()
        .iter()
        .find(|g| g.code() == "ZZZZ")
        .unwrap_or_else(|| panic!("no ZZZZ change in {delta:?}"));

    assert!(!g.keyed(), "ZZZZ has no dictionary KEY to match on");
    assert!(g.key_headings().is_empty());
    // Dropping a heading is structural, and reported as such on the side it
    // left. The counterpart to `a_new_heading_is_a_group_level_change`, which
    // only ever exercises `headings_added`.
    assert_eq!(g.headings_removed(), ["ZZZZ_B"]);
    assert!(g.headings_added().is_empty());
    // Unkeyed, so the shortened row is a remove and an add, not a change.
    assert_eq!(g.changed(), 0);
    assert_eq!(g.added(), 1);
    assert_eq!(g.removed(), 1);
}

#[test]
fn paths_and_bytes_agree() {
    let dir = scratch("doors");
    let a = dir.join("a.ags");
    let b = dir.join("b.ags");
    fs::write(&a, BASELINE).unwrap();
    fs::write(&b, REVISION).unwrap();

    let from_paths = ags4::diff(&a, &b).run().unwrap();
    let from_bytes = ags4::diff_bytes(BASELINE.as_bytes(), REVISION.as_bytes())
        .run()
        .unwrap();

    assert_eq!(from_paths.changed(), from_bytes.changed());
    assert_eq!(
        loca(&from_paths).rows()[0].key(),
        loca(&from_bytes).rows()[0].key()
    );
}

/// The handle door compares the documents as they stand NOW. Diffing the files
/// they were read from would quietly ignore the edit, which is the one thing a
/// caller holding an edited handle cannot want.
#[test]
fn documents_are_compared_as_they_stand_including_edits() {
    let a = ags4::read_str(BASELINE).run().unwrap();
    let mut b = ags4::read_str(BASELINE).run().unwrap();

    let unedited = ags4::diff_documents(&a, &b).run().unwrap();
    assert!(unedited.is_unchanged(), "{unedited:?}");

    b.set_cell("LOCA", 0, "LOCA_GL", "99.00").unwrap();
    let edited = ags4::diff_documents(&a, &b).run().unwrap();

    assert_eq!(edited.changed(), 1, "{edited:?}");
    assert_eq!(loca(&edited).rows()[0].key(), ["BH01"]);
    assert_eq!(loca(&edited).rows()[0].cells()[0].revision(), Some("99.00"));
}

#[test]
fn a_missing_file_is_an_io_error() {
    let dir = scratch("missing");
    let a = dir.join("a.ags");
    fs::write(&a, BASELINE).unwrap();

    let err = ags4::diff(&a, dir.join("nope.ags")).run().unwrap_err();
    assert_eq!(err.kind(), ErrorKind::Io);
}

#[test]
fn an_unknown_edition_is_refused_by_name() {
    let err = ags4::diff_bytes(BASELINE.as_bytes(), REVISION.as_bytes())
        .edition("4.9")
        .run()
        .unwrap_err();
    assert_eq!(err.kind(), ErrorKind::BadDictionary);
    assert!(err.to_string().contains("4.9"), "{err}");
}

#[test]
fn an_unknown_encoding_is_refused_by_name() {
    let err = ags4::diff_bytes(BASELINE.as_bytes(), REVISION.as_bytes())
        .encoding("klingon-1")
        .run()
        .unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidArgument);
    assert!(err.to_string().contains("klingon-1"), "{err}");
}

/// The message has to say WHICH side failed — "cannot read as AGS4" with two
/// inputs sends the caller looking at both.
#[test]
fn a_side_that_is_not_ags4_is_named() {
    let err = ags4::diff_bytes(BASELINE.as_bytes(), b"not a delivery file".to_vec())
        .run()
        .unwrap_err();
    assert_eq!(err.kind(), ErrorKind::NotAgs4);
    assert!(err.to_string().contains("revision"), "{err}");

    let err = ags4::diff_bytes(b"not a delivery file".to_vec(), BASELINE.as_bytes())
        .run()
        .unwrap_err();
    assert!(err.to_string().contains("baseline"), "{err}");
}

#[test]
fn debug_reports_shape_and_never_the_data() {
    let rendered = format!(
        "{:?}",
        ags4::diff_bytes(BASELINE.as_bytes(), REVISION.as_bytes()).max_rows_per_group(5)
    );
    assert!(rendered.contains("Diff"), "{rendered}");
    assert!(rendered.contains("bytes"), "{rendered}");
    assert!(rendered.contains('5'), "{rendered}");
    assert!(
        !rendered.contains("BH01"),
        "Debug leaked the data: {rendered}"
    );

    let a = ags4::read_str(BASELINE).run().unwrap();
    let rendered = format!("{:?}", ags4::diff_documents(&a, &a));
    assert!(rendered.contains("document of 2 groups"), "{rendered}");
    assert!(
        !rendered.contains("BH01"),
        "Debug leaked the data: {rendered}"
    );
}
