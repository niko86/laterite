//! Perf guard for `check_file` on the real 23 MB / ~107 k-row
//! workspace fixture. python-ags4 takes ~30 s on it (pandas
//! row-by-row); the Rust target is well under 3 s.
//!
//! The fixture (`examples/output/large.ags`) is gitignored working
//! space — when it's absent the bench registers nothing and Criterion
//! exits cleanly, so `cargo bench` never fails for lack of it.

use std::path::PathBuf;
use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use laterite_ags4_validator::{CheckOptions, check_file};

fn fixture() -> Option<PathBuf> {
    // rust-packages/laterite-ags4-validator → repo root → examples/...
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/output/large.ags");
    p.exists().then_some(p)
}

fn bench(c: &mut Criterion) {
    let Some(path) = fixture() else {
        eprintln!("validate_large_fixture: examples/output/large.ags absent — skipping");
        return;
    };
    let opts = CheckOptions::default();
    let mut g = c.benchmark_group("validate");
    // The file is big; a handful of samples with a generous window is
    // plenty to catch a >50% regression.
    g.sample_size(10).measurement_time(Duration::from_secs(30));
    g.bench_function("large.ags", |b| {
        b.iter(|| check_file(&path, &opts).expect("validates"));
    });
    g.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
