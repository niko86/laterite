//! Property-based invariants for merge, over generated LOCA deliveries.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt::Write as _;

use laterite_ags4_merge::{MergeOpts, TypeClashMode, merge_parsed};
use laterite_ags4_parse::{ParsedFile, parse_str};
use proptest::prelude::*;

fn opts() -> MergeOpts {
    MergeOpts {
        on_type_clash: TypeClashMode::Widen,
        tran: None,
        ..Default::default()
    }
}

/// Build a LOCA file from unique (id, gl) pairs, formatting GL with `decimals`
/// places — so the SAME data can be emitted with different byte formatting.
fn build_loca(rows: &BTreeMap<u8, i32>, decimals: usize) -> ParsedFile {
    let mut s = String::from(
        "\"GROUP\",\"LOCA\"\n\"HEADING\",\"LOCA_ID\",\"LOCA_GL\"\n\"UNIT\",\"\",\"m\"\n\"TYPE\",\"ID\",\"2DP\"\n",
    );
    for (id, gl) in rows {
        let _ = writeln!(s, "\"DATA\",\"BH{id}\",\"{:.*}\"", decimals, f64::from(*gl));
    }
    parse_str(&s).unwrap()
}

fn reparse(bytes: &[u8]) -> ParsedFile {
    parse_str(std::str::from_utf8(bytes).unwrap()).unwrap()
}

fn loca_ids(f: &ParsedFile) -> BTreeSet<String> {
    match f.groups.get("LOCA") {
        Some(g) => {
            let ci = g.headings.iter().position(|h| h == "LOCA_ID").unwrap();
            g.rows
                .iter()
                .filter_map(|r| r.values.get(ci).cloned())
                .collect()
        }
        None => BTreeSet::new(),
    }
}

fn rows_strategy() -> impl Strategy<Value = BTreeMap<u8, i32>> {
    // Small id space (0..6) so two files genuinely overlap; values 0..1000.
    prop::collection::btree_map(0u8..6, 0i32..1000, 0..6)
}

proptest! {
    /// The merged set of KEYs is the UNION of both files' KEYs, regardless of
    /// argument order — union is commutative on membership even though which
    /// file WINS a shared key's content is order-dependent.
    #[test]
    fn merged_key_set_is_order_independent(a in rows_strategy(), b in rows_strategy()) {
        let m_ab = merge_parsed(&[build_loca(&a, 2), build_loca(&b, 2)], &opts()).unwrap();
        let m_ba = merge_parsed(&[build_loca(&b, 2), build_loca(&a, 2)], &opts()).unwrap();
        let ids_ab = loca_ids(&reparse(&m_ab.bytes));
        let ids_ba = loca_ids(&reparse(&m_ba.bytes));
        prop_assert_eq!(&ids_ab, &ids_ba);
        // And it really is the union of the inputs.
        let expected: BTreeSet<String> =
            a.keys().chain(b.keys()).map(|k| format!("BH{k}")).collect();
        prop_assert_eq!(ids_ab, expected);
    }

    /// Merging a delivery with a COSMETICALLY REFORMATTED copy of itself (same
    /// values, GL emitted with a different number of decimals) collapses via
    /// TYPED matching: exactly the same rows, and ZERO reported revisions —
    /// proving matching is type-aware, not byte-dedup.
    #[test]
    fn reformatted_resend_collapses_with_no_revisions(a in rows_strategy()) {
        let f = build_loca(&a, 2);   // "10.00"
        let f_reformatted = build_loca(&a, 1); // "10.0" — same value, different bytes
        let res = merge_parsed(&[f, f_reformatted], &opts()).unwrap();
        prop_assert!(res.revisions.is_empty(), "formatting-only, no revisions: {:?}", res.revisions);
        let out = reparse(&res.bytes);
        prop_assert_eq!(loca_ids(&out).len(), a.len());
    }
}
