//! Read-path benches for the shared parse leaf.
//!
//! This crate is on EVERY read: the validator, the core codec, the index
//! builder and all four surfaces funnel through `parse_bytes`, so a percent
//! here is a percent everywhere. Split deliberately into two tiers, because
//! they answer different questions:
//!
//!   * whole-file (`parse_bytes`) — where the read path's time actually goes,
//!     with throughput so the number is comparable across fixture sizes;
//!   * per-line (`split_ags_line` / `tokenize_spans` / `field_span`) — the
//!     tokenizers the whole-file walk calls once per line. A regression here
//!     is invisible at the file level until it is already large.
//!
//! Fixtures come from `tools/gen-bench-fixtures.sh` (forge-synthesised,
//! deterministic). Absent fixtures SKIP rather than fail, so `cargo bench`
//! stays runnable on a clean checkout — but note that a skipped bench measures
//! nothing, which is exactly how the old single bench sat silently dead.

use std::path::PathBuf;
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use laterite_ags4_parse::{field_span, parse_bytes, split_ags_line, tokenize_spans};

/// `output/bench-fixtures/<label>.ags`, or None if it hasn't been generated.
fn fixture(label: &str) -> Option<PathBuf> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../output/bench-fixtures")
        .join(format!("{label}.ags"));
    p.exists().then_some(p)
}

fn bench_parse_bytes(c: &mut Criterion) {
    let mut g = c.benchmark_group("parse/parse_bytes");
    // Generous window, few samples: the large rung is 25 MB and we care about a
    // shifted median, not a tight confidence interval.
    g.sample_size(10).measurement_time(Duration::from_secs(20));

    let mut ran = false;
    for label in ["small", "medium", "large"] {
        let Some(path) = fixture(label) else { continue };
        let bytes = std::fs::read(&path).expect("fixture readable");
        ran = true;
        // Throughput turns three different-sized rungs into one comparable
        // MB/s number — the only way to see whether cost is linear in input.
        g.throughput(Throughput::Bytes(bytes.len() as u64));
        g.bench_with_input(BenchmarkId::from_parameter(label), &bytes, |b, bytes| {
            // UTF-8 explicitly: the fixtures are forge-emitted UTF-8, and
            // pinning it keeps the decode leg out of the measurement's variance.
            b.iter(|| parse_bytes(bytes, encoding_rs::UTF_8).expect("fixture parses"));
        });
    }
    if !ran {
        eprintln!("parse: no fixtures — run tools/gen-bench-fixtures.sh");
    }
    g.finish();
}

/// One representative DATA line: quoted fields, an embedded doubled quote, and
/// a mix of short codes and longer free text. Deliberately hand-written rather
/// than pulled from a fixture — a per-line bench should measure the tokenizer,
/// not whichever line a generator happened to emit first.
const LINE: &str = r#""DATA","BH001","1.00","2.50","CLAY","Firm becoming stiff grey slightly sandy CLAY with rare 2"" gravel","2024-01-15","","0.00","MADE GROUND""#;

fn bench_line_tokenizers(c: &mut Criterion) {
    let mut g = c.benchmark_group("parse/per-line");
    g.throughput(Throughput::Bytes(LINE.len() as u64));

    // The validator's line walk: allocates a Vec<String> per line.
    g.bench_function("split_ags_line", |b| {
        b.iter(|| split_ags_line(std::hint::black_box(LINE)));
    });
    // The browser editor's offset-preserving tokenizer (also a Vec, plus spans).
    g.bench_function("tokenize_spans", |b| {
        b.iter(|| tokenize_spans(std::hint::black_box(LINE)));
    });
    // The allocation-free single-field probe — the shape the other two would
    // ideally converge toward for callers that want one field.
    g.bench_function("field_span", |b| {
        b.iter(|| field_span(std::hint::black_box(LINE), 5));
    });
    g.finish();
}

criterion_group!(benches, bench_parse_bytes, bench_line_tokenizers);
criterion_main!(benches);
