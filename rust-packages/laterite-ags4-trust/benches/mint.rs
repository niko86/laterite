//! Prices candidate #5 (T4): `mint` parses the file to validate it, then
//! `Sidecar::assemble` walks the same bytes AGAIN via `index_ags4_bytes` to
//! build the byte-offset index — offsets the first parse already computed in
//! `group_records`. This is the first bench of the mint/certify path at all
//! (the trust crate had none), so it establishes the baseline AND the prize:
//!
//!   * `mint`       — the whole certify operation (parse + rules + assemble),
//!     the number a caller actually pays.
//!   * `index_walk` — `index_ags4_bytes` alone, the redundant second walk #5
//!     removes. `index_walk / mint` is the ceiling on the #5 win.

use std::hint::black_box;
use std::path::PathBuf;
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use laterite_ags4_core::index::index_ags4_bytes;
use laterite_ags4_trust::mint;
use laterite_ags4_validator::CheckOptions;

fn fixture(label: &str) -> Option<PathBuf> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../output/bench-fixtures")
        .join(format!("{label}.ags"));
    p.exists().then_some(p)
}

fn bench(c: &mut Criterion) {
    let mut g = c.benchmark_group("trust/mint");
    g.sample_size(10).measurement_time(Duration::from_secs(20));

    for label in ["large"] {
        let Some(path) = fixture(label) else { continue };
        let bytes = std::fs::read(&path).expect("fixture readable");
        g.throughput(Throughput::Bytes(bytes.len() as u64));

        g.bench_with_input(BenchmarkId::new("mint", label), &bytes, |b, bytes| {
            b.iter(|| {
                black_box(
                    mint(
                        bytes,
                        &CheckOptions::default(),
                        "2026-01-01T00:00:00Z".to_string(),
                        None,
                    )
                    .expect("mints (fixture is validation-clean)"),
                )
            });
        });
        g.bench_with_input(BenchmarkId::new("index_walk", label), &bytes, |b, bytes| {
            b.iter(|| black_box(index_ags4_bytes(bytes).expect("indexes")));
        });
    }
    g.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
