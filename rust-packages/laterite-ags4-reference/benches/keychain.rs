//! Keychain hot-path bench — `group_row_ids` mints `_id`/`_parent_id` for every
//! row of a keyed read (the opt-in relational / `to_duckdb` persist path). This
//! is the one bench that isolates that cost: the end-to-end keyed read folds it
//! into SHA-256 + the Arrow key-column build + IPC framing, which mask a change
//! confined to the per-row key-chain construction.
//!
//! SAMP is the deepest-key group — a 5-entry denormalised chain
//! (`LOCA_ID, SAMP_TOP, SAMP_REF, SAMP_TYPE, SAMP_ID`) plus a reconstructed
//! parent chain — so it exercises the per-row hashing hardest.

use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use laterite_ags4_reference::keychain::group_row_ids;
use laterite_ags4_reference::union::registry;

fn bench_group_row_ids(c: &mut Criterion) {
    let reg = registry();
    let desc = reg.get("SAMP").expect("SAMP is in the union registry");

    // A realistic wide keyed group: SAMP's full KEY chain, then filler non-key
    // columns (a real delivery carries many more headings than KEYs).
    let mut headings: Vec<String> = desc.key_headings().map(|h| h.name.clone()).collect();
    headings.extend(
        ["SAMP_DESC", "SAMP_DTIM", "SAMP_UBLK", "SAMP_BASE"]
            .iter()
            .map(|s| (*s).to_string()),
    );

    let n_rows = 10_000usize;
    // Distinct per-row values so every id is unique — the real workload (a keyed
    // read of a genuine file, not a degenerate all-equal one).
    let vals: Vec<Vec<String>> = (0..n_rows)
        .map(|r| {
            (0..headings.len())
                .map(|col| format!("v{r}-{col}"))
                .collect()
        })
        .collect();

    let mut g = c.benchmark_group("keychain");
    g.throughput(Throughput::Elements(n_rows as u64));
    g.bench_function("group_row_ids/SAMP-10k", |b| {
        b.iter(|| {
            let ids = group_row_ids(reg, "SAMP", &headings, n_rows, |col, row| {
                vals.get(row).and_then(|r| r.get(col)).map(String::as_str)
            });
            black_box(ids);
        });
    });
    g.finish();
}

criterion_group!(benches, bench_group_row_ids);
criterion_main!(benches);
