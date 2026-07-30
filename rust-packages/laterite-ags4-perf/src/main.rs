//! `laterite-ags4-perf` — the rust leg of the cross-surface performance matrix.
//!
//! Times the two read operations every shipped surface performs —
//! `validate` and `parse-to-typed` — over the forge-generated size ladder
//! (`tools/perf-ladder.py` → `output/perf-ladder/manifest.json`), and
//! writes the matrix's *uniform* result schema
//! (`{surface, results:[{op, rung, median_ms, throughput_mb_s}]}`) that
//! `tools/perf-matrix.py` merges with the other surfaces. Every surface
//! emits this same shape, so the aggregator is a dumb merger rather than a
//! pile of per-tool format parsers.
//!
//! `parse-to-typed` reads the file bytes once *outside* the timed loop and
//! measures parse + materialise-every-group-to-Arrow — the same work the
//! wasm/node/python hosts do on in-memory bytes (file I/O is the duckdb /
//! path-only surface's concern, flagged in the report). `validate` goes
//! through `check_file(path)` (the public API reads the path itself); the
//! OS page cache makes the repeated read negligible, noted as a caveat.
//!
//! Usage: `laterite-ags4-perf [--manifest <p>] [--out <p>] [--iters N]`
//! (defaults: `output/perf-ladder/manifest.json` → `output/perf-results/rust.json`, 10 iters).

use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::Instant;

use laterite_ags4_types::arrow_cols::build_record_batch;
use laterite_ags4_validator::{CheckOptions, check_file};
use serde::{Deserialize, Serialize};

/// The forge ladder manifest (only the fields this harness reads).
#[derive(Deserialize)]
struct Manifest {
    rungs: Vec<Rung>,
}

#[derive(Deserialize)]
struct Rung {
    label: String,
    path: String,
}

/// The matrix's uniform per-surface result file.
#[derive(Serialize)]
struct Output {
    schema: u32,
    surface: &'static str,
    tool: &'static str,
    iters: usize,
    results: Vec<Measurement>,
}

#[derive(Serialize)]
struct Measurement {
    op: &'static str,
    rung: String,
    bytes: u64,
    median_ms: f64,
    throughput_mb_s: f64,
}

/// Warm up `warmup` untimed runs, then return the median wall time (ms)
/// over `iters` timed runs — the matrix's headline metric.
fn median_ms<F: FnMut()>(warmup: usize, iters: usize, mut f: F) -> f64 {
    for _ in 0..warmup {
        f();
    }
    let mut samples = Vec::with_capacity(iters);
    for _ in 0..iters {
        let t = Instant::now();
        f();
        samples.push(t.elapsed().as_secs_f64() * 1000.0);
    }
    samples.sort_by(|a, b| a.partial_cmp(b).expect("no NaN durations"));
    samples[samples.len() / 2]
}

/// Decimal MB/s (MB = 1e6 bytes, matching forge's `parse_size`): the
/// cross-surface throughput headline.
fn throughput_mb_s(bytes: u64, median_ms: f64) -> f64 {
    if median_ms <= 0.0 {
        return 0.0;
    }
    bytes as f64 / (median_ms * 1000.0)
}

/// Time `validate` on a rung: the public `check_file` on the path.
fn bench_validate(path: &Path, bytes: u64, label: &str, iters: usize) -> Measurement {
    let opts = CheckOptions::default();
    let ms = median_ms(1, iters, || {
        black_box(check_file(path, &opts).expect("ladder file validates"));
    });
    Measurement {
        op: "validate",
        rung: label.to_string(),
        bytes,
        median_ms: ms,
        throughput_mb_s: throughput_mb_s(bytes, ms),
    }
}

/// Time `parse-to-typed` on a rung — the EXACT path the shipped bindings
/// (py/node/wasm) take: the validator's parser + the shared
/// `build_record_batch` fed the positional `ParsedGroup::cell` accessor. So
/// the rust surface materialises types identically to the hosts it's compared
/// against (not the DuckDB-ingest codec in `laterite-ags4-core`). The file is
/// read to a `String` once, outside the timed loop.
fn bench_parse_typed(text: &str, bytes: u64, label: &str, iters: usize) -> Measurement {
    let ms = median_ms(2, iters, || {
        let parsed = laterite_ags4_parse::parse_str(text).expect("ladder file parses");
        for g in parsed.groups.values() {
            let batch = build_record_batch(&g.headings, &g.types, g.rows.len(), |col, row| {
                g.cell(col, row)
            })
            .expect("typed batch");
            black_box(&batch);
        }
    });
    Measurement {
        op: "parse-to-typed",
        rung: label.to_string(),
        bytes,
        median_ms: ms,
        throughput_mb_s: throughput_mb_s(bytes, ms),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manifest_path = PathBuf::from("output/perf-ladder/manifest.json");
    let mut out_path = PathBuf::from("output/perf-results/rust.json");
    let mut iters = 10usize;

    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--manifest" => manifest_path = args.next().ok_or("--manifest needs a value")?.into(),
            "--out" => out_path = args.next().ok_or("--out needs a value")?.into(),
            "--iters" => iters = args.next().ok_or("--iters needs a value")?.parse()?,
            "-h" | "--help" => {
                eprintln!("usage: laterite-ags4-perf [--manifest <p>] [--out <p>] [--iters N]");
                return Ok(());
            }
            other => return Err(format!("unknown arg: {other}").into()),
        }
    }

    let manifest: Manifest =
        serde_json::from_slice(&std::fs::read(&manifest_path).map_err(|e| {
            format!(
                "read ladder manifest {}: {e} — run `python tools/perf-ladder.py` first",
                manifest_path.display()
            )
        })?)?;

    let mut results = Vec::new();
    for rung in &manifest.rungs {
        let path = Path::new(&rung.path);
        let Ok(meta) = std::fs::metadata(path) else {
            eprintln!(
                "laterite-ags4-perf: rung {} missing ({}) — skipping",
                rung.label, rung.path
            );
            continue;
        };
        let bytes = meta.len();
        let text = std::fs::read_to_string(path)?;
        eprintln!(
            "laterite-ags4-perf: {} ({bytes} bytes) × {iters} iters",
            rung.label
        );
        results.push(bench_validate(path, bytes, &rung.label, iters));
        results.push(bench_parse_typed(&text, bytes, &rung.label, iters));
    }

    let output = Output {
        schema: 1,
        surface: "rust",
        tool: "laterite-ags4-perf",
        iters,
        results,
    };
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&out_path, serde_json::to_vec_pretty(&output)?)?;
    eprintln!(
        "laterite-ags4-perf: wrote {} measurements → {}",
        output.results.len(),
        out_path.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn throughput_is_decimal_mb_per_s() {
        // 5 MB in 10 ms → 500 MB/s (decimal MB, matching the report).
        assert!((throughput_mb_s(5_000_000, 10.0) - 500.0).abs() < 1e-9);
        // Degenerate timing never divides by zero: `throughput_mb_s` returns the
        // literal `0.0` (a guarded early return, not a computed value) for any
        // non-positive `median_ms`, so this is an exact-value check, not one that
        // wants an epsilon.
        #[allow(clippy::float_cmp)]
        {
            assert_eq!(throughput_mb_s(1_000, 0.0), 0.0);
        }
    }

    #[test]
    fn median_picks_the_middle_sample() {
        // A closure timed at increasing-but-tiny cost still yields a finite,
        // non-negative median over the timed window.
        let m = median_ms(0, 5, || {});
        assert!(m >= 0.0 && m.is_finite());
    }
}
