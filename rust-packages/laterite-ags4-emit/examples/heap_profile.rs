//! Allocation-site attribution for the emit pipeline's per-cell copies.
//!
//! `build_ags4` was measured (by a downstream consumer of the wheel) peaking
//! at ~30x the size of the file it writes, ~244 bytes per cell, and the
//! attribution there was structural: three owned-String copies read off the
//! source at `arrow_in.rs` (`cell_value`'s `.to_string()`),
//! `emit.rs` (`OwnedGroup`'s formatted `Vec<Vec<String>>`), and the parse-back
//! in `laterite-ags4-parse` (`DataRow.values`). This harness replaces that
//! read-off with a measurement: dhat as the global allocator, the pipeline
//! entered at the same seam the PyO3 host enters it — `group_from_arrow`,
//! typed Arrow batches in — and the live bytes at global peak attributed by
//! call site.
//!
//! Two things the structural account could not see, which are the reason this
//! exists:
//!
//!   * `cell_value` only allocates for STRING and fallback columns — a
//!     `Float64` cell crosses as a typed numeric cell, no String. So
//!     "copy 1" is per string cell, not per cell.
//!   * `emit_ags4`'s step 3 builds its `EmitGroup` views with
//!     `rows: g.rows.clone()` — a full fourth copy of every formatted cell,
//!     co-resident with the other three, under a comment saying the struct
//!     borrows.
//!
//! The workload is a downstream build's `TREL`: the same 22 columns
//! polars hands over (7 Utf8, 12 Float64, 3 Int64), the same cell widths, the
//! group declared by an in-file DICT the way the real delivery declares it.
//! One deliberate departure: the harness's DICT declares no parent for TREL
//! and ships no TRET/TREG rows, so the relational rules have nothing to walk —
//! the real delivery validates clean, and a harness drowning the profile in
//! orphan findings would be measuring its own scaffolding. Findings are
//! printed so a non-clean run is visible.
//!
//! What dhat reports is REQUESTED bytes live at peak — not RSS. Allocator
//! retention and fragmentation (mimalloc's in the shipped wheel) are on top of
//! this; peak RSS stays measured separately, on the wheel, with
//! `/usr/bin/time -l`. Keep the two claims apart.
//!
//! Run (from `rust-packages/`):
//!
//! ```sh
//! cargo run --release -p laterite-ags4-emit --features arrow \
//!     --example heap_profile -- 296600 autofix dhat-heap.json
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use arrow::array::{ArrayRef, Float64Array, Int64Array, RecordBatch, StringArray};
use arrow::datatypes::{DataType, Field, Schema};
use laterite_ags4_emit::Cell;
use laterite_ags4_emit::{EmitMode, EmitOpts, GroupInput, emit_ags4_owned, group_from_arrow};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

/// The downstream build's TREL frame: heading, Arrow type, AGS4 TYPE, UNIT —
/// the same 22 columns it fills, in its order, with the TYPE/UNIT its
/// declarations pass alongside them (transcribed from the AGS-L working
/// group's own DICT files).
#[derive(Clone, Copy)]
enum Col {
    Str,
    F64,
    I64,
}

const TREL: [(&str, Col, &str, &str, &str); 22] = [
    ("LOCA_ID", Col::Str, "ID", "", "KEY"),
    ("SAMP_TOP", Col::F64, "2DP", "m", "KEY"),
    ("SAMP_REF", Col::Str, "X", "", "KEY"),
    ("SAMP_TYPE", Col::Str, "PA", "", "KEY"),
    ("SAMP_ID", Col::Str, "ID", "", "KEY"),
    ("SPEC_REF", Col::Str, "X", "", "KEY"),
    ("SPEC_DPTH", Col::F64, "2DP", "m", "KEY"),
    ("TRET_TESN", Col::Str, "X", "", "KEY"),
    ("TREL_MNUM", Col::I64, "0DP", "", "KEY"),
    ("TREL_TTIM", Col::F64, "1DP", "s", "OTHER"),
    ("TREL_STIM", Col::F64, "1DP", "s", "OTHER"),
    ("TREL_STGN", Col::I64, "0DP", "", "OTHER"),
    ("TREL_STGD", Col::Str, "X", "", "OTHER"),
    ("TREL_CYCN", Col::I64, "0DP", "", "OTHER"),
    ("TREL_CELL", Col::F64, "1DP", "kPa", "OTHER"),
    ("TREL_BACK", Col::F64, "1DP", "kPa", "OTHER"),
    ("TREL_PWP", Col::F64, "1DP", "kPa", "OTHER"),
    ("TREL_SZT", Col::F64, "1DP", "kPa", "OTHER"),
    ("TREL_SRT", Col::F64, "1DP", "kPa", "OTHER"),
    ("TREL_SZE", Col::F64, "1DP", "kPa", "OTHER"),
    ("TREL_SRE", Col::F64, "1DP", "kPa", "OTHER"),
    ("TREL_EZES", Col::F64, "3DP", "%", "OTHER"),
];

const TESTS: usize = 20;
const READINGS_PER_CYCLE: usize = 200;

/// The TREL batch, cell widths matching the downstream build's: eight-character
/// borehole ids, kilopascals to one decimal, strains to three.
fn trel_batch(rows: usize) -> (Arc<Schema>, RecordBatch) {
    let per_test = rows.div_ceil(TESTS);
    let fields: Vec<Field> = TREL
        .iter()
        .map(|(name, col, ..)| {
            let dt = match col {
                Col::Str => DataType::Utf8,
                Col::F64 => DataType::Float64,
                Col::I64 => DataType::Int64,
            };
            Field::new(*name, dt, false)
        })
        .collect();
    let schema = Arc::new(Schema::new(fields));

    let columns: Vec<ArrayRef> = TREL
        .iter()
        .map(|(name, col, ..)| -> ArrayRef {
            match col {
                Col::Str => Arc::new(StringArray::from_iter_values((0..rows).map(|r| {
                    let test = r / per_test;
                    match *name {
                        "LOCA_ID" => format!("P3-BH{:03}", test * 2 + 1),
                        "SAMP_REF" => "S3".to_string(),
                        "SAMP_TYPE" => "UT".to_string(),
                        "SAMP_ID" => format!("P3-BH{:03}-S3", test * 2 + 1),
                        "SPEC_REF" => "2".to_string(),
                        "TRET_TESN" => "1".to_string(),
                        _ => "CYCLIC LOADING".to_string(),
                    }
                }))),
                Col::F64 => Arc::new(Float64Array::from_iter_values((0..rows).map(|r| {
                    let test = r / per_test;
                    let within = r % per_test;
                    let t = within as f64 / READINGS_PER_CYCLE as f64;
                    match *name {
                        "SAMP_TOP" | "SPEC_DPTH" => 5.31 + test as f64 * 0.7,
                        "TREL_TTIM" | "TREL_STIM" => (t * 10.0).round() / 10.0,
                        "TREL_CELL" => 620.0 + test as f64,
                        "TREL_BACK" => 300.0,
                        "TREL_PWP" => 300.0 + (within % 70) as f64 / 10.0,
                        "TREL_SZT" => 350.0 + (within % 130) as f64 / 10.0,
                        "TREL_SRT" => 320.0 + (within % 110) as f64 / 10.0,
                        "TREL_SZE" => 55.0 + (within % 90) as f64 / 10.0,
                        _ => 0.001 * (within % 2500) as f64, // SRE-ish / EZES
                    }
                }))),
                Col::I64 => Arc::new(Int64Array::from_iter_values((0..rows).map(|r| {
                    let within = (r % per_test) as i64;
                    match *name {
                        "TREL_MNUM" => within + 1,
                        "TREL_STGN" => 1,
                        _ => within / READINGS_PER_CYCLE as i64 + 1, // TREL_CYCN
                    }
                }))),
            }
        })
        .collect();

    let batch = RecordBatch::try_new(schema.clone(), columns).expect("a coherent batch");
    (schema, batch)
}

/// The file's own account of TREL, as the real delivery carries one (Rule 18).
/// DICT is a standard group, so its own UNIT/TYPE fill from the dictionary.
fn dict_group() -> GroupInput {
    let headings = [
        "DICT_TYPE",
        "DICT_GRP",
        "DICT_HDNG",
        "DICT_STAT",
        "DICT_DTYP",
        "DICT_DESC",
        "DICT_UNIT",
        "DICT_PGRP",
        "DICT_REM",
    ];
    let rem = "AGS-L draft, publish 2026";
    let mut rows: Vec<Vec<Cell>> = vec![
        [
            "GROUP",
            "TREL",
            "",
            "",
            "",
            "Triaxial Tests - Effective Stress - Logged Data",
            "",
            "",
            rem,
        ]
        .iter()
        .map(|s| Cell::Text((*s).to_string()))
        .collect(),
    ];
    for (name, _, dtyp, unit, stat) in TREL {
        rows.push(
            ["HEADING", "TREL", name, stat, dtyp, name, unit, "", rem]
                .iter()
                .map(|s| Cell::Text((*s).to_string()))
                .collect(),
        );
    }
    GroupInput {
        code: "DICT".to_string(),
        headings: headings.iter().map(ToString::to_string).collect(),
        units: None,
        types: None,
        rows,
    }
}

fn proj_group() -> GroupInput {
    GroupInput {
        code: "PROJ".to_string(),
        headings: ["PROJ_ID", "PROJ_NAME", "PROJ_LOC"]
            .iter()
            .map(ToString::to_string)
            .collect(),
        units: None,
        types: None,
        rows: vec![
            ["121415", "Clacton Marine GI", "Clacton-on-Sea"]
                .iter()
                .map(|s| Cell::Text((*s).to_string()))
                .collect(),
        ],
    }
}

fn stats(label: &str) {
    let s = dhat::HeapStats::get();
    println!(
        "{label:<28} live {:>7.0} MB in {:>10} blocks   (max so far {:>7.0} MB)",
        s.curr_bytes as f64 / 1e6,
        s.curr_blocks,
        s.max_bytes as f64 / 1e6,
    );
}

fn main() {
    let mut args = std::env::args().skip(1);
    let rows: usize = args
        .next()
        .map_or(296_600, |a| a.parse().expect("rows: an integer"));
    let mode = match args.next().as_deref() {
        None | Some("autofix") => EmitMode::AutoFix,
        Some("report") => EmitMode::Report,
        Some(other) => panic!("mode: report|autofix, not {other}"),
    };
    let out = args.next().unwrap_or_else(|| "dhat-heap.json".to_string());

    let _profiler = dhat::Profiler::builder()
        .file_name(&out)
        .trim_backtraces(Some(24))
        .build();

    // --- the polars stand-in: typed Arrow columns ----------------------
    let (schema, batch) = trel_batch(rows);
    stats("arrow batch built");

    // --- copy 1: the Arrow -> Cell transpose (the PyO3 host's seam) ---
    let units: HashMap<String, String> = TREL
        .iter()
        .map(|(n, _, _, u, _)| ((*n).to_string(), (*u).to_string()))
        .collect();
    let types: HashMap<String, String> = TREL
        .iter()
        .map(|(n, _, t, _, _)| ((*n).to_string(), (*t).to_string()))
        .collect();
    let trel = laterite_ags4_emit::group_from_arrow_with_meta(
        "TREL".to_string(),
        &schema,
        std::slice::from_ref(&batch),
        Some(&units),
        Some(&types),
    );
    // The downstream caller goes through `_at_edition`, which for TREL resolves
    // no DT units (no temporal columns, and the heading is not in the standard
    // dictionary) — `group_from_arrow_with_meta` is the identical path.
    let _ = group_from_arrow; // referenced so the doc comment's seam is checked
    stats("copy 1: GroupInput");

    let groups = vec![proj_group(), dict_group(), trel];

    // --- copies 2, 2b, 3: format, clone for the writer, parse back -----
    let opts = EmitOpts {
        mode,
        ..EmitOpts::default()
    };
    let result = emit_ags4_owned(groups, &opts).expect("emits");
    stats("emit_ags4 returned");

    let cells = rows * TREL.len();
    let findings: usize = result.findings.values().map(Vec::len).sum();
    println!("---");
    println!(
        "rows {rows}   cells {cells}   output {:.1} MB   {:.1} bytes/cell in the file",
        result.bytes.len() as f64 / 1e6,
        result.bytes.len() as f64 / cells as f64,
    );
    let peak = dhat::HeapStats::get().max_bytes;
    println!(
        "peak live (requested) {:.0} MB   {:.1} bytes/cell   {:.1}x the output",
        peak as f64 / 1e6,
        peak as f64 / cells as f64,
        peak as f64 / result.bytes.len() as f64,
    );
    println!(
        "findings {findings} across rules {:?}   fixes applied {}",
        result.findings.keys().collect::<Vec<_>>(),
        result.fixes_applied,
    );
    println!("attribution: {out} (view with dh_view.html or dhat-to-flamegraph)");
}
