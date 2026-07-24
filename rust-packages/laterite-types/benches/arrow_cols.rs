//! Typed-column build benches — the "process data" step.
//!
//! `build_record_batch` is where AGS4 strings become real Arrow types, and it
//! runs on every typed read on every surface (the PyO3 read boundary, the Node
//! IPC framing, the browser explorer). It is also where per-cell casting cost
//! lives: each cell goes through `parse_value` for its AGS type.
//!
//! Benched per TYPE FAMILY, not just in aggregate, because the families do
//! very different work — `ID`/`X` are string passthrough, `2DP`/`3SF` parse a
//! float, `DT` parses a datetime — and an aggregate number cannot tell you
//! which one to optimise. A file dominated by `DT` columns and one dominated
//! by `ID` columns are different workloads wearing the same name.
//!
//! Input is synthesised here: this leaf takes cells, not files, so reading a
//! fixture would only bolt a parse step onto every measurement.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use laterite_types::arrow_cols::build_record_batch;

const N_ROWS: usize = 50_000;

/// (`ags_type`, a value that type actually parses) — one column per family so
/// each bench measures that family's caster in isolation.
const FAMILIES: [(&str, &str); 5] = [
    ("ID", "BH0001"),              // string passthrough
    ("X", "Firm grey sandy CLAY"), // string passthrough, longer
    ("2DP", "12.34"),              // decimal-places float
    ("3SF", "1.23"),               // significant-figures float
    ("DT", "2024-01-15T09:30:00"), // datetime parse — the costliest caster
];

fn bench_families(c: &mut Criterion) {
    let mut g = c.benchmark_group("types/build_record_batch");
    g.sample_size(20);

    for (ags_type, value) in FAMILIES {
        // Four columns of one family — enough that per-column fixed costs don't
        // dominate the per-cell casting we're actually measuring.
        let headings: Vec<String> = (0..4).map(|i| format!("COL_{i}")).collect();
        let types: Vec<String> = vec![ags_type.to_string(); 4];
        g.throughput(Throughput::Elements((N_ROWS * 4) as u64));
        g.bench_with_input(
            BenchmarkId::from_parameter(ags_type),
            &(headings, types),
            |b, (headings, types)| {
                b.iter(|| {
                    build_record_batch(
                        black_box(headings),
                        black_box(types),
                        N_ROWS,
                        |_col, _row| Some(value),
                    )
                    .expect("builds")
                });
            },
        );
    }
    g.finish();
}

/// A realistic mixed group — the shape a real read actually hands over, so the
/// per-family numbers above have something to be weighed against.
fn bench_mixed(c: &mut Criterion) {
    let headings: Vec<String> = FAMILIES.iter().map(|(t, _)| format!("C_{t}")).collect();
    let types: Vec<String> = FAMILIES.iter().map(|(t, _)| t.to_string()).collect();
    let values: Vec<&str> = FAMILIES.iter().map(|(_, v)| *v).collect();

    let mut g = c.benchmark_group("types/build_record_batch");
    g.sample_size(20);
    g.throughput(Throughput::Elements((N_ROWS * FAMILIES.len()) as u64));
    g.bench_function("mixed", |b| {
        b.iter(|| {
            build_record_batch(
                black_box(&headings),
                black_box(&types),
                N_ROWS,
                |col, _row| values.get(col).copied(),
            )
            .expect("builds")
        });
    });
    g.finish();
}

criterion_group!(benches, bench_families, bench_mixed);
criterion_main!(benches);
