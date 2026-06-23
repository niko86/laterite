//! #168 parser convergence: an INDEPENDENT, hand-computed byte-offset oracle for
//! GROUP record starts — the third oracle that catches the "both parsers agree
//! but are wrong" mode a snapshot-vs-self gate cannot.
//!
//! GROUND_TRUTH = the TRUE line-start byte offset of each `"GROUP"` record,
//! verified out-of-band with `grep -abo '"GROUP"'`. As of Phase 4 the `.ags.idx`
//! byte index ([`index_ags4_bytes`]) sources these from the shared parse leaf's
//! source-true byte walk, so it matches GROUND_TRUTH for EVERY fixture —
//! including the two the retired csv reader got wrong: CRLF (it recorded a
//! non-first GROUP at the preceding `\n`, off by one) and leading blank lines (it
//! recorded the first GROUP at 0, absorbing the blanks into section 1). The owner
//! ratified that tightening; it is documented as O-40. (The two
//! `csv_index_is_loose_for_*` snapshots were retired here with the csv-based
//! index they described.)

use laterite_ags4_core::index::index_ags4_bytes;

/// (fixture stem, [(group code, TRUE start byte)]) — hand-computed, grep-verified.
const GROUND_TRUTH: &[(&str, &[(&str, u64)])] = &[
    ("two_group_lf", &[("PROJ", 0), ("TRAN", 65)]),
    ("two_group_crlf", &[("PROJ", 0), ("TRAN", 70)]),
    ("two_group_bom", &[("PROJ", 0), ("TRAN", 68)]),
    ("leading_blank", &[("PROJ", 2), ("TRAN", 67)]),
    ("quoted_newline", &[("PROJ", 0), ("TRAN", 67)]),
];

fn truth(name: &str) -> Vec<(String, u64)> {
    GROUND_TRUTH
        .iter()
        .find(|(n, _)| *n == name)
        .unwrap()
        .1
        .iter()
        .map(|(c, o)| (c.to_string(), *o))
        .collect()
}

/// The `.ags.idx` index's GROUP start offsets, in section order.
fn index_starts(name: &str) -> Vec<(String, u64)> {
    let bytes = std::fs::read(format!("tests/fixtures/{name}.ags")).unwrap();
    let idx = index_ags4_bytes(&bytes).unwrap();
    idx.order
        .iter()
        .map(|c| (c.clone(), idx.range(c).unwrap().0))
        .collect()
}

/// The leaf-sourced index records the TRUE line-start for EVERY fixture — LF,
/// CRLF, BOM, leading-blank, and quoted-embedded-newline alike. This is the
/// Phase-4 gate: byte-identical to the independent ground-truth oracle (NOT to
/// any self-snapshot), so it catches the "both parsers agree but are wrong" mode.
#[test]
fn index_matches_ground_truth_for_all_fixtures() {
    for (name, _) in GROUND_TRUTH {
        assert_eq!(index_starts(name), truth(name), "GROUP starts for {name}");
    }
}
