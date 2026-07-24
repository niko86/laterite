//! Write-path benches — the "export data" half of the pipeline.
//!
//! Two layers, benched separately because they cost very differently and the
//! aggregate hides which one moved:
//!
//!   * `write_ags4` — the byte writer alone: quote every cell, double embedded
//!     quotes, CRLF, pad UNIT/TYPE. Pure formatting, no dictionary, no rules.
//!   * `emit_ags4` — the orchestrator every surface actually calls: dictionary
//!     UNIT/TYPE fill + per-cell `ags4_str` formatting + the write + the chosen
//!     `EmitMode`.
//!
//! The gap between them IS the orchestrator's overhead, and `EmitMode` is
//! benched as a ladder because `AutoFix` re-parses and re-validates the bytes
//! it just produced, and then optionally mints missing metadata on top — costs
//! worth seeing separately, since that mode is the DEFAULT.
//!
//! Input is synthesised here rather than read from a fixture: the write path
//! takes cell data, not a file, so a fixture would only add a parse step to
//! every measurement.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use laterite_ags4_emit::{EmitGroup, EmitMode, EmitOpts, GroupInput, emit_ags4, write_ags4};
use serde_json::Value;

const HEADINGS: [&str; 8] = [
    "LOCA_ID",
    "SAMP_TOP",
    "SAMP_REF",
    "SAMP_TYPE",
    "SAMP_DESC",
    "SAMP_DATE",
    "SAMP_BASE",
    "SAMP_REM",
];
const TYPES: [&str; 8] = ["ID", "2DP", "ID", "PA", "X", "DT", "2DP", "X"];

/// One group of `n_rows`, with a cell mix that exercises the quoter: plain
/// codes, numerics, and free text containing an embedded `"` (the escaping
/// path, which is the writer's only branchy per-cell work).
fn rows(n_rows: usize) -> Vec<Vec<String>> {
    (0..n_rows)
        .map(|r| {
            vec![
                format!("BH{r:04}"),
                format!("{}.00", r % 40),
                format!("S{r}"),
                "D".to_string(),
                r#"Firm grey slightly sandy CLAY with rare 2" gravel"#.to_string(),
                "2024-01-15".to_string(),
                format!("{}.50", r % 40),
                String::new(),
            ]
        })
        .collect()
}

fn bench_writer(c: &mut Criterion) {
    let mut g = c.benchmark_group("emit/write_ags4");
    for n in [1_000usize, 20_000] {
        let data = rows(n);
        let group = EmitGroup {
            code: "SAMP",
            headings: HEADINGS.to_vec(),
            units: vec!["", "m", "", "", "", "yyyy-mm-dd", "m", ""],
            types: TYPES.to_vec(),
            rows: data,
        };
        // Bytes written, not rows: makes this comparable to the parse side's
        // MB/s so read and write can be read on the same axis.
        let mut sizing = Vec::new();
        write_ags4(&mut sizing, std::slice::from_ref(&group)).expect("writes");
        g.throughput(Throughput::Bytes(sizing.len() as u64));
        g.bench_with_input(BenchmarkId::from_parameter(n), &group, |b, group| {
            b.iter(|| {
                let mut out = Vec::with_capacity(sizing.len());
                write_ags4(&mut out, std::slice::from_ref(black_box(group))).expect("writes");
                out
            });
        });
    }
    g.finish();
}

fn bench_orchestrator(c: &mut Criterion) {
    let mut g = c.benchmark_group("emit/emit_ags4");
    let n = 20_000usize;
    let input = vec![GroupInput {
        code: "SAMP".to_string(),
        headings: HEADINGS.iter().map(ToString::to_string).collect(),
        units: None, // dictionary fill — the hybrid path a real caller hits
        types: None,
        rows: rows(n)
            .into_iter()
            .map(|r| r.into_iter().map(Value::String).collect())
            .collect(),
    }];

    // A STAGE LADDER, not two points. Each rung adds exactly one stage, so a
    // subtraction prices that stage instead of leaving one lump labelled
    // "AutoFix overhead":
    //
    //   report              = dictionary fill + per-cell ags4_str + write + validate
    //   autofix-no-synth    = the above + compute_fixes/apply_fixes
    //   autofix-with-synth  = the above + step 2.5 metadata synthesis
    //
    // and `write_ags4` from the group above is the floor (bytes only), so
    // `report - write` is the fill+format+validate cost.
    //
    // Splitting synthesis out matters because it is the DEFAULT and it grew
    // after the mode did: the default was accepted 2026-06-12 meaning
    // validate+safe-fix, and 2026-06-25 added metadata minting to it. Without
    // this rung there is no way to tell which of those a caller is paying for.
    let ladder = [
        ("report", EmitMode::Report, false),
        ("autofix-no-synth", EmitMode::AutoFix, false),
        ("autofix-with-synth", EmitMode::AutoFix, true),
    ];
    g.sample_size(20);
    for (label, mode, synth) in ladder {
        let opts = EmitOpts {
            mode,
            synthesize_metadata: synth,
            ..EmitOpts::default()
        };
        g.bench_function(label, |b| {
            b.iter(|| emit_ags4(black_box(&input), &opts).expect("emits"));
        });
    }
    g.finish();
}

criterion_group!(benches, bench_writer, bench_orchestrator);
criterion_main!(benches);
