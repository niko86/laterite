//! Core read-path benches: the AGS4 read codec and the `.ags.idx` index build.
//!
//! Both take raw bytes and both sit on the shared parse leaf since the #168
//! convergence, so the interesting number is not either one alone — it is what
//! each ADDS on top of `laterite-ags4-parse`'s own `parse_bytes` bench:
//!
//!   * `read_ags4_bytes` — parse + the re-trim / UNIT-TYPE pad the codec applies
//!     to stay byte-identical to the historical reader.
//!   * `index_ags4_bytes` — the byte-offset walk behind the certificate index.
//!     It should be markedly cheaper than a full read (it records GROUP offsets
//!     rather than materialising rows); if it isn't, the index is doing work it
//!     doesn't need to.
//!
//! Read the two side by side with `parse/parse_bytes` at the same rung.
//!
//! Fixtures: `tools/gen-bench-fixtures.sh`. Absent → skip, so a clean checkout
//! can still run `cargo bench`.

use std::hint::black_box;
use std::path::PathBuf;
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use laterite_ags4_core::ags4_codec::read_ags4_bytes;
use laterite_ags4_core::index::index_ags4_bytes;

fn fixture(label: &str) -> Option<PathBuf> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../output/bench-fixtures")
        .join(format!("{label}.ags"));
    p.exists().then_some(p)
}

fn rungs() -> Vec<(&'static str, Vec<u8>)> {
    ["small", "medium", "large"]
        .into_iter()
        .filter_map(|l| fixture(l).map(|p| (l, std::fs::read(p).expect("fixture readable"))))
        .collect()
}

fn bench_read(c: &mut Criterion) {
    let data = rungs();
    if data.is_empty() {
        eprintln!("core: no fixtures — run tools/gen-bench-fixtures.sh");
        return;
    }
    let mut g = c.benchmark_group("core/read_ags4_bytes");
    g.sample_size(10).measurement_time(Duration::from_secs(20));
    for (label, bytes) in &data {
        g.throughput(Throughput::Bytes(bytes.len() as u64));
        g.bench_with_input(BenchmarkId::from_parameter(label), bytes, |b, bytes| {
            b.iter(|| read_ags4_bytes(black_box(bytes)).expect("fixture reads"));
        });
    }
    g.finish();

    let mut g = c.benchmark_group("core/index_ags4_bytes");
    g.sample_size(10).measurement_time(Duration::from_secs(20));
    for (label, bytes) in &data {
        g.throughput(Throughput::Bytes(bytes.len() as u64));
        g.bench_with_input(BenchmarkId::from_parameter(label), bytes, |b, bytes| {
            b.iter(|| index_ags4_bytes(black_box(bytes)).expect("fixture indexes"));
        });
    }
    g.finish();
}

criterion_group!(benches, bench_read);
criterion_main!(benches);
