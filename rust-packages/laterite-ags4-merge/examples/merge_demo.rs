//! Merge the two POC fixtures and print the result — a manual-inspection aid.
//! `cargo run -p laterite-ags4-merge --example merge_demo`

use laterite_ags4_merge::{MergeOpts, TranStamp, TypeMismatchMode, merge_parsed};
use laterite_ags4_parse::parse_str;

fn main() {
    let dir = env!("CARGO_MANIFEST_DIR");
    let a = parse_str(
        &std::fs::read_to_string(format!("{dir}/tests/fixtures/delivery_a.ags")).unwrap(),
    )
    .unwrap();
    let b = parse_str(
        &std::fs::read_to_string(format!("{dir}/tests/fixtures/delivery_b.ags")).unwrap(),
    )
    .unwrap();

    let opts = MergeOpts {
        type_mismatch: TypeMismatchMode::Lenient,
        tran: Some(TranStamp {
            isno: "3".into(),
            date: "2024-03-01".into(),
            prod: "Merger".into(),
            recv: "Client".into(),
            stat: "Merged".into(),
            ags: "4.1.1".into(),
        }),
        ..Default::default()
    };

    let res = merge_parsed(&[a, b], &opts).unwrap();
    println!("=== warnings ({}) ===", res.warnings.len());
    for w in &res.warnings {
        println!("  [{}] {}", w.kind, w.message);
    }
    println!("\n=== merged AGS4 ===");
    println!("{}", String::from_utf8_lossy(&res.bytes));
}
