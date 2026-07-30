//! Typed-column build benches — the "process data" step.
//!
//! `build_record_batch` is where AGS4 strings become real Arrow types, and it
//! runs on every typed read on every surface (the PyO3 read boundary, the Node
//! IPC framing, the browser explorer). It is also where per-cell casting cost
//! lives: each cell goes through `parse_value` for its AGS type.
//!
//! Two tiers, answering different questions:
//!
//!   * **per type FAMILY** (`bench_families`) — synthetic cells, one column per
//!     family, so each caster is measured in isolation: `ID`/`X` string
//!     passthrough, `2DP`/`3SF` float, `0DP` integer, `YN` bool, `DT` datetime.
//!     An aggregate number cannot say which one to optimise. A `null` rung is
//!     included because a real sparse delivery takes the `append_null` branch
//!     constantly, and the constant-cell rungs never do.
//!   * **file scale** (`bench_file_typed`) — parses a forge fixture ONCE in
//!     setup, then times `build_record_batch` over that file's REAL cells across
//!     every group, with byte throughput. This is the rung that places the typed
//!     build on the same MB/s axis as `parse_bytes` in [[core-perf-baseline]] —
//!     the per-family rungs are reproducible but not comparable with the
//!     whole-file stages. The parse is setup, not measured.
//!
//! The sibling builders the surfaces actually call — `build_record_batch_compat`
//! (the parity-oracle drop-in shape), `build_record_batch_with_ids` (the keyed
//! relational builder) and the `ipc` framing (node + wasm) — are benched too, so
//! none rides invisibly on the typed builder's number.

use std::hint::black_box;
use std::path::PathBuf;
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use laterite_ags4_types::arrow_cols::{
    build_record_batch, build_record_batch_compat, build_record_batch_with_ids,
};
use laterite_ags4_types::ipc::build_group_ipc;

const N_ROWS: usize = 50_000;

/// (`ags_type`, a value that type actually parses) — one column per family so
/// each bench measures that family's caster in isolation. Covers all five of
/// `build_column`'s arms: String (`ID`/`X`), Decimal (`2DP`/`3SF`), Integer
/// (`0DP`), Bool (`YN`), Datetime (`DT`).
const FAMILIES: [(&str, &str); 7] = [
    ("ID", "BH0001"),              // string passthrough
    ("X", "Firm grey sandy CLAY"), // string passthrough, longer
    ("2DP", "12.34"),              // decimal-places float
    ("3SF", "1.23"),               // significant-figures float
    ("0DP", "42"),                 // integer — the only Integer code
    ("YN", "Y"),                   // bool — the only Bool code
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

    // The null/empty branch: a real sparse delivery hits `append_null` on most
    // cells, and every rung above returns a constant non-empty cell, so that
    // branch was never timed. Mix DT (the costliest caster's null path) with a
    // string column, returning None for ~half the rows.
    let headings: Vec<String> = (0..4).map(|i| format!("COL_{i}")).collect();
    let types: Vec<String> = ["DT", "X", "DT", "X"]
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    g.throughput(Throughput::Elements((N_ROWS * 4) as u64));
    g.bench_function("null-half", |b| {
        b.iter(|| {
            build_record_batch(
                black_box(&headings),
                black_box(&types),
                N_ROWS,
                |_col, row| {
                    if row % 2 == 0 {
                        None
                    } else {
                        Some("2024-01-15T09:30:00")
                    }
                },
            )
            .expect("builds")
        });
    });

    g.finish();
}

/// A realistic mixed group — the shape a real read actually hands over, so the
/// per-family numbers above have something to be weighed against. Also benches
/// the three sibling builders on the SAME cells, so the gap between them
/// (typing vs raw-string compat vs the keyed builder vs the IPC framing) is
/// visible in one place.
fn bench_mixed(c: &mut Criterion) {
    let headings: Vec<String> = FAMILIES.iter().map(|(t, _)| format!("C_{t}")).collect();
    let types: Vec<String> = FAMILIES.iter().map(|(t, _)| t.to_string()).collect();
    let units: Vec<String> = vec![String::new(); FAMILIES.len()];
    let values: Vec<&str> = FAMILIES.iter().map(|(_, v)| *v).collect();
    let cell = |col: usize, _row: usize| values.get(col).copied();
    // Dummy ids for the keyed builder — the id computation lives in the caller
    // (keychain), out of this leaf, so a plausible shape is enough to price the
    // two extra Utf8 columns + null-parent handling.
    let ids: Vec<(String, Option<String>)> = (0..N_ROWS)
        .map(|i| {
            (
                format!("id{i}"),
                if i == 0 {
                    None
                } else {
                    Some(format!("id{}", i - 1))
                },
            )
        })
        .collect();

    let mut g = c.benchmark_group("types/build_record_batch");
    g.sample_size(20);
    g.throughput(Throughput::Elements((N_ROWS * FAMILIES.len()) as u64));

    g.bench_function("mixed", |b| {
        b.iter(|| {
            build_record_batch(black_box(&headings), black_box(&types), N_ROWS, cell)
                .expect("builds")
        });
    });
    g.bench_function("mixed-compat", |b| {
        b.iter(|| {
            build_record_batch_compat(
                black_box(&headings),
                black_box(&units),
                black_box(&types),
                N_ROWS,
                cell,
            )
            .expect("builds")
        });
    });
    g.bench_function("mixed-with-ids", |b| {
        b.iter(|| {
            build_record_batch_with_ids(
                black_box(&ids),
                black_box(&headings),
                black_box(&types),
                N_ROWS,
                cell,
            )
            .expect("builds")
        });
    });
    g.bench_function("mixed-ipc", |b| {
        b.iter(|| {
            build_group_ipc(black_box(&headings), black_box(&types), N_ROWS, cell).expect("frames")
        });
    });
    g.finish();
}

/// `output/bench-fixtures/<label>.ags`, or None if it hasn't been generated.
fn fixture(label: &str) -> Option<PathBuf> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../output/bench-fixtures")
        .join(format!("{label}.ags"));
    p.exists().then_some(p)
}

/// The typed build over a REAL file's cells, at file scale, with byte
/// throughput — the rung that lands on the baseline table beside `parse_bytes`.
/// The parse is setup (once, outside the timed loop); the loop times only
/// `build_record_batch` across every group, exactly as the wheel's typed read
/// does after it has a parsed file.
fn bench_file_typed(c: &mut Criterion) {
    let mut g = c.benchmark_group("types/typed_read_file");
    g.sample_size(10).measurement_time(Duration::from_secs(20));

    let mut ran = false;
    for label in ["small", "medium", "large"] {
        let Some(path) = fixture(label) else { continue };
        let bytes = std::fs::read(&path).expect("fixture readable");
        // Parse ONCE. This is the `ParsedFile` the wheel hands to the typed
        // build; keeping it out of the timed loop isolates the build cost.
        let pf =
            laterite_ags4_parse::parse_bytes(&bytes, encoding_rs::UTF_8).expect("fixture parses");
        ran = true;
        g.throughput(Throughput::Bytes(bytes.len() as u64));
        g.bench_with_input(BenchmarkId::from_parameter(label), &pf, |b, pf| {
            b.iter(|| {
                for code in &pf.group_order {
                    let Some(grp) = pf.groups.get(code) else {
                        continue;
                    };
                    let batch = build_record_batch(
                        &grp.headings,
                        &grp.types,
                        grp.rows.len(),
                        |col, row| grp.cell(col, row),
                    )
                    .expect("builds");
                    black_box(batch);
                }
            });
        });
    }
    if !ran {
        eprintln!("arrow_cols: no fixtures — run tools/gen-bench-fixtures.sh");
    }
    g.finish();
}

criterion_group!(benches, bench_families, bench_mixed, bench_file_typed);
criterion_main!(benches);
