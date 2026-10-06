//! `row_order` on merge (#1008): the issue's two deliveries, interleaved by
//! arrival order, come out sorted by KEY — and sorting changes nothing but
//! the order rows are written in.

use std::fmt::Write as _;

use laterite_ags4_merge::{MergeOpts, MergeResult, RowOrder, merge_parsed};
use laterite_ags4_parse::{ParsedFile, parse_str};

/// One delivery: LOCA (with a ground level, so a re-sent location can be
/// revised) and SAMP, keyed on `LOCA_ID`, `SAMP_TOP`, `SAMP_REF`, `SAMP_TYPE`.
fn delivery(loca: &[(&str, &str)], samp: &[(&str, &str, &str, &str)]) -> ParsedFile {
    let mut s = String::from(
        "\"GROUP\",\"PROJ\"\n\"HEADING\",\"PROJ_ID\"\n\"UNIT\",\"\"\n\"TYPE\",\"ID\"\n\"DATA\",\"P1\"\n\n\
         \"GROUP\",\"LOCA\"\n\"HEADING\",\"LOCA_ID\",\"LOCA_GL\"\n\"UNIT\",\"\",\"m\"\n\"TYPE\",\"ID\",\"2DP\"\n",
    );
    for (id, gl) in loca {
        writeln!(s, "\"DATA\",\"{id}\",\"{gl}\"").unwrap();
    }
    s.push_str(
        "\n\"GROUP\",\"SAMP\"\n\"HEADING\",\"LOCA_ID\",\"SAMP_TOP\",\"SAMP_REF\",\"SAMP_TYPE\",\"SAMP_ID\"\n\
         \"UNIT\",\"\",\"m\",\"\",\"\",\"\"\n\"TYPE\",\"ID\",\"2DP\",\"X\",\"PA\",\"ID\"\n",
    );
    for (id, top, r, t) in samp {
        writeln!(s, "\"DATA\",\"{id}\",\"{top}\",\"{r}\",\"{t}\",\"\"").unwrap();
    }
    parse_str(&s).unwrap()
}

fn inputs() -> Vec<ParsedFile> {
    vec![
        delivery(
            &[("BH10", "5.00"), ("BH2", "10.00")],
            &[("BH10", "2.00", "2", "D"), ("BH2", "1.00", "1", "B")],
        ),
        delivery(
            &[("BH1", "4.00"), ("BH2", "11.00")],
            &[("BH2", "0.50", "1", "ES"), ("BH1", "3.00", "3", "B")],
        ),
    ]
}

fn merged(row_order: RowOrder) -> MergeResult {
    let opts = MergeOpts {
        row_order,
        ..MergeOpts::default()
    };
    merge_parsed(&inputs(), &opts).unwrap()
}

/// Every row of `code` as written, cells by heading order.
fn rows(res: &MergeResult, code: &str) -> Vec<Vec<String>> {
    let p = parse_str(std::str::from_utf8(&res.bytes).unwrap()).unwrap();
    let g = &p.groups[code];
    g.rows
        .iter()
        .map(|r| {
            (0..g.headings.len())
                .map(|i| g.value_at(r, i).unwrap_or_default().to_string())
                .collect()
        })
        .collect()
}

fn ids(rows: &[Vec<String>]) -> Vec<&str> {
    rows.iter().map(|r| r[0].as_str()).collect()
}

#[test]
fn key_order_un_interleaves_the_deliveries() {
    let res = merged(RowOrder::Key);
    assert_eq!(ids(&rows(&res, "LOCA")), ["BH1", "BH2", "BH10"]);
    let samp: Vec<(String, String, String)> = rows(&res, "SAMP")
        .into_iter()
        .map(|r| (r[0].clone(), r[1].clone(), r[3].clone()))
        .collect();
    let want = [
        ("BH1", "3.00", "B"),
        ("BH2", "0.50", "ES"),
        ("BH2", "1.00", "B"),
        ("BH10", "2.00", "D"),
    ]
    .map(|(a, b, c)| (a.to_string(), b.to_string(), c.to_string()));
    assert_eq!(samp, want);
}

#[test]
fn input_order_is_the_default_and_keeps_arrival_order() {
    let default = merge_parsed(&inputs(), &MergeOpts::default()).unwrap();
    assert_eq!(default.bytes, merged(RowOrder::Input).bytes);
    assert_eq!(ids(&rows(&default, "LOCA")), ["BH10", "BH2", "BH1"]);
}

#[test]
fn sorting_changes_only_the_order_rows_are_written_in() {
    let (input, key) = (merged(RowOrder::Input), merged(RowOrder::Key));
    assert_ne!(
        input.bytes, key.bytes,
        "the fixture must actually move rows"
    );
    // Reconciliation is untouched: the same winner (BH2 revised to 11.00),
    // the same revisions, the same warnings.
    assert!(!input.revisions.is_empty(), "the fixture must revise a row");
    assert_eq!(input.revisions, key.revisions);
    assert_eq!(input.warnings, key.warnings);
    // And the same rows, as a multiset, in every group.
    for code in ["PROJ", "LOCA", "SAMP"] {
        let (mut a, mut b) = (rows(&input, code), rows(&key, code));
        a.sort();
        b.sort();
        assert_eq!(a, b, "{code}");
    }
    assert!(rows(&key, "LOCA").contains(&vec!["BH2".to_string(), "11.00".to_string()]));
}
