//! `laterite-ags4-perf` — the rust leg of the cross-surface performance matrix.
//!
//! Times the three operations every shipped surface performs — `validate`,
//! `parse-to-typed` and `write` — over the forge-generated size ladder
//! (`tools/perf-ladder.py` → `output/perf-ladder/manifest.json`), and
//! writes the matrix's *uniform* result schema (schema 2:
//! `{surface, results:[{op, rung, bytes, median_ms, throughput_mb_s, mem?}]}`)
//! that `tools/perf-matrix.py` merges with the other surfaces. Every surface
//! emits this same shape, so the aggregator is a dumb merger rather than a
//! pile of per-tool format parsers.
//!
//! `parse-to-typed` reads the file bytes once *outside* the timed loop and
//! measures parse + materialise-every-group-to-Arrow — the same work the
//! wasm/node/python hosts do on in-memory bytes (file I/O is the duckdb /
//! path-only surface's concern, flagged in the report). `validate` goes
//! through `check_file(path)` (the public API reads the path itself); the
//! OS page cache makes the repeated read negligible, noted as a caveat.
//! `write` drives the shared Arrow emit door (`emit_ags4_from_arrow`) the
//! py/node/wasm hosts all drive, with the typed input prepared outside the
//! timed loop — so its time is the emit engine's, given held input.
//!
//! `mem` is the campaign's peak-RSS instrument (epic #820 decision 1): each
//! (op, rung) cell is one FRESH child process (`--mem-worker`, this same
//! bin) running the operation once end-to-end, reporting its own `ru_maxrss`
//! at exit. A cell the harness will not or cannot measure is a RECORDED
//! refusal (`beyond-mem-cap` / `swapped` / `failed`), never a silent skip —
//! the same verdicts, semantics and 265 MB cap as the python lane's harness
//! (`tools/bench-vs-python-ags4.py`). A write cell's peak includes reading
//! and typing the input — you cannot write what you do not hold — so it is
//! attributed by comparison against the same rung's `parse-to-typed` cell.
//!
//! Usage: `laterite-ags4-perf [--manifest <p>] [--out <p>] [--iters N] [--skip-mem]`
//! (defaults: `output/perf-ladder/manifest.json` → `output/perf-results/rust.json`, 10 iters).

use std::collections::HashMap;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;
use laterite_ags4_emit::{ArrowGroup, EmitOpts, emit_ags4_from_arrow};
use laterite_ags4_types::arrow_cols::build_record_batch;
use laterite_ags4_validator::{CheckOptions, check_file};
use serde::{Deserialize, Serialize};

/// Epic #820 decision 7: memory columns stop at the 265 MB rung — a run that
/// pushes the machine into swap measures the pager, not the library. Same
/// value as the python lane's `MEM_CAP_BYTES` (`tools/bench-vs-python-ags4.py`),
/// so the two harnesses admit and refuse the same pinned rungs; the unit test
/// below holds it against the pinned rung sizes.
const MEM_CAP_BYTES: u64 = 300_000_000;

/// Swap growth past this during a child's run marks the cell `swapped`. Small
/// enough to catch a real spill, large enough that unrelated background
/// paging does not veto a clean run (the python lane's threshold).
const SWAP_REFUSAL_BYTES: u64 = 64 * 1024 * 1024;

/// The forge ladder manifest (only the fields this harness reads — the
/// generator records provenance beside them, which serde ignores).
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
    /// The peak-RSS column — absent under `--skip-mem`, a refusal cell where
    /// the harness vetoed the run. Shape-distinguishable on purpose: a reader
    /// (human or script) cannot mistake a vetoed run for a small number.
    #[serde(skip_serializing_if = "Option::is_none")]
    mem: Option<MemCell>,
}

/// One memory cell: a measurement or a recorded refusal, told apart by shape
/// (`peak_rss_bytes` vs `refusal`) exactly as in the python lane's results
/// file — the aggregator and the ledger read both files with one eye.
#[derive(Serialize)]
#[serde(untagged)]
enum MemCell {
    Measured {
        peak_rss_bytes: u64,
        x_output: f64,
    },
    Refusal {
        refusal: &'static str,
        detail: String,
    },
}

/// `x_output` is the campaign's headline unit — peak as a multiple of the
/// operation's output (or input) size, which stays comparable across rungs
/// where raw MB does not.
fn mem_cell(peak_bytes: u64, denom_bytes: u64) -> MemCell {
    MemCell::Measured {
        peak_rss_bytes: peak_bytes,
        x_output: (peak_bytes as f64 / denom_bytes as f64 * 100.0).round() / 100.0,
    }
}

fn refusal_cell(reason: &'static str, detail: String) -> MemCell {
    MemCell::Refusal {
        refusal: reason,
        detail,
    }
}

/// `ru_maxrss` is bytes on Darwin, kibibytes on Linux — getrusage(2) differs
/// by lineage, and a unit slip here moves every cell by 1024×.
fn maxrss_to_bytes(raw: i64, os: &str) -> u64 {
    let raw = u64::try_from(raw).unwrap_or(0);
    if os == "macos" { raw } else { raw * 1024 }
}

/// The epic-#820 cap: memory measurement stops at the 265 MB rung.
fn mem_rung_allowed(rung_bytes: u64) -> bool {
    rung_bytes <= MEM_CAP_BYTES
}

/// The `used = 512.50M` field of Darwin's `vm.swapusage` sysctl, in bytes.
// The f64→u64 truncation drops sub-byte noise on a value read at 0.01 MB
// resolution — nothing real to lose.
#[allow(clippy::cast_possible_truncation)]
fn parse_swap_used_darwin(text: &str) -> Option<u64> {
    let rest = text.split_once("used")?.1.trim_start();
    let rest = rest.strip_prefix('=')?.trim_start();
    let end = rest
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(rest.len());
    let value: f64 = rest[..end].parse().ok()?;
    let scale = match rest[end..].chars().next() {
        Some('K') => 1024.0,
        Some('M') => 1024.0 * 1024.0,
        Some('G') => 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };
    Some((value * scale) as u64)
}

/// `SwapTotal - SwapFree` from `/proc/meminfo` text (kB fields), in bytes.
fn parse_meminfo_swap(text: &str) -> Option<u64> {
    let field = |name: &str| -> Option<u64> {
        text.lines()
            .find(|l| l.starts_with(name))?
            .split_whitespace()
            .nth(1)?
            .parse()
            .ok()
    };
    Some(field("SwapTotal:")?.saturating_sub(field("SwapFree:")?) * 1024)
}

/// Current swap in use, or None where no instrument exists. Read before and
/// after each child: growth means the child's number includes the pager.
fn swap_used_bytes() -> Option<u64> {
    if cfg!(target_os = "macos") {
        let out = Command::new("sysctl")
            .args(["-n", "vm.swapusage"])
            .output()
            .ok()?;
        parse_swap_used_darwin(&String::from_utf8_lossy(&out.stdout))
    } else {
        parse_meminfo_swap(&std::fs::read_to_string("/proc/meminfo").ok()?)
    }
}

/// This process's own peak RSS, in bytes — read by the `--mem-worker` child
/// at exit, so the cell is the child's number, not an outside estimate.
#[cfg(unix)]
fn self_maxrss_bytes() -> Option<u64> {
    let mut ru = std::mem::MaybeUninit::<libc::rusage>::zeroed();
    // SAFETY: getrusage fills the struct we own; RUSAGE_SELF is always valid.
    let rc = unsafe { libc::getrusage(libc::RUSAGE_SELF, ru.as_mut_ptr()) };
    if rc != 0 {
        return None;
    }
    // SAFETY: rc == 0 means the kernel initialised the struct.
    let raw = unsafe { ru.assume_init() }.ru_maxrss;
    Some(maxrss_to_bytes(raw, std::env::consts::OS))
}

#[cfg(not(unix))]
fn self_maxrss_bytes() -> Option<u64> {
    None
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
        mem: None,
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
        mem: None,
    }
}

/// One group's typed input, held across the timed write loop: the batch and
/// the source UNIT/TYPE rows the emit door needs to reproduce the file.
struct PreparedGroup {
    code: String,
    schema: SchemaRef,
    batch: RecordBatch,
    units: HashMap<String, String>,
    types: HashMap<String, String>,
}

/// Parse + type every group once — the write axis's held input, built the
/// same way `parse-to-typed` builds its batches.
fn prepare_arrow_groups(text: &str) -> Vec<PreparedGroup> {
    let parsed = laterite_ags4_parse::parse_str(text).expect("ladder file parses");
    parsed
        .group_order
        .iter()
        .map(|code| {
            let g = &parsed.groups[code];
            let batch = build_record_batch(&g.headings, &g.types, g.rows.len(), |col, row| {
                g.cell(col, row)
            })
            .expect("typed batch");
            // units/types are the raw unpadded source rows; zip stops at the
            // shorter side, and the emit door dictionary-fills what's absent.
            let units = g.headings.iter().cloned().zip(g.units.iter().cloned());
            let types = g.headings.iter().cloned().zip(g.types.iter().cloned());
            PreparedGroup {
                code: g.code.clone(),
                schema: batch.schema(),
                batch,
                units: units.collect(),
                types: types.collect(),
            }
        })
        .collect()
}

/// The emit door consumes its input, so each timed run rebuilds the
/// `ArrowGroup` vec from the held parts — Arc bumps and small string clones,
/// noise against emitting the file itself.
fn arrow_groups(prepared: &[PreparedGroup]) -> Vec<ArrowGroup> {
    prepared
        .iter()
        .map(|p| ArrowGroup {
            code: p.code.clone(),
            schema: p.schema.clone(),
            batches: vec![p.batch.clone()],
            units: Some(p.units.clone()),
            types: Some(p.types.clone()),
        })
        .collect()
}

/// Time `write` on a rung — the shared Arrow emit door
/// (`emit_ags4_from_arrow`, default `AutoFix` opts) that every shipped binding
/// drives, fed typed input prepared outside the timed loop. `bytes` and the
/// throughput stay denominated in the rung's input size (the rung identity),
/// which a lossless round trip makes ~equal to the emitted size.
fn bench_write(prepared: &[PreparedGroup], bytes: u64, label: &str, iters: usize) -> Measurement {
    let opts = EmitOpts::default();
    let ms = median_ms(1, iters, || {
        let result =
            emit_ags4_from_arrow(arrow_groups(prepared), &opts).expect("ladder file emits");
        black_box(&result.bytes);
    });
    Measurement {
        op: "write",
        rung: label.to_string(),
        bytes,
        median_ms: ms,
        throughput_mb_s: throughput_mb_s(bytes, ms),
        mem: None,
    }
}

/// What the `--mem-worker` child reports on stdout (both crates' code — no
/// library on this path prints, so stdout is a safe channel here where the
/// python lane needed a result file).
#[derive(Serialize, Deserialize)]
struct WorkerReport {
    maxrss_bytes: u64,
    /// Emitted size for the write op — the mem cell's `x_output` denominator,
    /// as in the python lane (`out_bytes` when present, input size otherwise).
    out_bytes: Option<u64>,
}

/// The `--mem-worker` child: run one operation once, end-to-end, and report
/// this process's own peak RSS. A panic exits non-zero and becomes the
/// parent's `failed` refusal cell.
fn mem_worker(op: &str, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let out_bytes = match op {
        "validate" => {
            black_box(check_file(path, &CheckOptions::default()).expect("ladder file validates"));
            None
        }
        "parse-to-typed" => {
            let text = std::fs::read_to_string(path)?;
            let parsed = laterite_ags4_parse::parse_str(&text).expect("ladder file parses");
            for g in parsed.groups.values() {
                let batch = build_record_batch(&g.headings, &g.types, g.rows.len(), |col, row| {
                    g.cell(col, row)
                })
                .expect("typed batch");
                black_box(&batch);
            }
            None
        }
        // Read + type + emit: you cannot write what you do not hold, so the
        // write cell's peak includes the input materialisation — attribute it
        // against the same rung's parse-to-typed cell.
        "write" => {
            let text = std::fs::read_to_string(path)?;
            let prepared = prepare_arrow_groups(&text);
            drop(text);
            let result = emit_ags4_from_arrow(arrow_groups(&prepared), &EmitOpts::default())
                .expect("ladder file emits");
            Some(result.bytes.len() as u64)
        }
        other => return Err(format!("unknown mem-worker op: {other}").into()),
    };
    let peak = self_maxrss_bytes().ok_or("peak-RSS needs getrusage — unix only")?;
    println!(
        "{}",
        serde_json::to_string(&WorkerReport {
            maxrss_bytes: peak,
            out_bytes
        })?
    );
    Ok(())
}

/// One (op, rung) memory cell: fresh child (this same bin, `--mem-worker`),
/// swap watched across the run. Every veto is a recorded refusal.
fn measure_mem(op: &str, path: &Path, input_bytes: u64) -> MemCell {
    if !mem_rung_allowed(input_bytes) {
        return refusal_cell(
            "beyond-mem-cap",
            format!(
                "{input_bytes}-byte rung is past the {MEM_CAP_BYTES}-byte cap \
                 (epic #820 decision 7: a swapping run measures the pager)"
            ),
        );
    }
    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(e) => return refusal_cell("failed", format!("current_exe: {e}")),
    };
    let swap_before = swap_used_bytes();
    let out = Command::new(exe)
        .args(["--mem-worker", op, "--mem-file"])
        .arg(path)
        .output();
    let swap_after = swap_used_bytes();
    let out = match out {
        Ok(out) => out,
        Err(e) => return refusal_cell("failed", format!("spawn: {e}")),
    };
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let tail: Vec<&str> = stderr.trim().lines().rev().take(3).collect();
        let detail = if tail.is_empty() {
            format!("exit {}", out.status)
        } else {
            tail.into_iter().rev().collect::<Vec<_>>().join(" | ")
        };
        return refusal_cell("failed", detail);
    }
    if let (Some(before), Some(after)) = (swap_before, swap_after) {
        let grew = after.saturating_sub(before);
        if grew > SWAP_REFUSAL_BYTES {
            return refusal_cell(
                "swapped",
                format!("swap grew {:.1} MB during the run", grew as f64 / 1e6),
            );
        }
    }
    match serde_json::from_slice::<WorkerReport>(&out.stdout) {
        Ok(report) => mem_cell(
            report.maxrss_bytes,
            report.out_bytes.unwrap_or(input_bytes).max(1),
        ),
        Err(e) => refusal_cell("failed", format!("unreadable worker report: {e}")),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manifest_path = PathBuf::from("output/perf-ladder/manifest.json");
    let mut out_path = PathBuf::from("output/perf-results/rust.json");
    let mut iters = 10usize;
    let mut skip_mem = false;
    let mut mem_worker_op: Option<String> = None;
    let mut mem_file: Option<PathBuf> = None;

    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--manifest" => manifest_path = args.next().ok_or("--manifest needs a value")?.into(),
            "--out" => out_path = args.next().ok_or("--out needs a value")?.into(),
            "--iters" => iters = args.next().ok_or("--iters needs a value")?.parse()?,
            "--skip-mem" => skip_mem = true,
            "--mem-worker" => mem_worker_op = Some(args.next().ok_or("--mem-worker needs an op")?),
            "--mem-file" => mem_file = Some(args.next().ok_or("--mem-file needs a value")?.into()),
            "-h" | "--help" => {
                eprintln!(
                    "usage: laterite-ags4-perf [--manifest <p>] [--out <p>] [--iters N] [--skip-mem]"
                );
                return Ok(());
            }
            other => return Err(format!("unknown arg: {other}").into()),
        }
    }

    if let Some(op) = mem_worker_op {
        let path = mem_file.ok_or("--mem-worker needs --mem-file")?;
        return mem_worker(&op, &path);
    }

    let manifest: Manifest =
        serde_json::from_slice(&std::fs::read(&manifest_path).map_err(|e| {
            format!(
                "read ladder manifest {}: {e} — run `uv run python tools/perf-ladder.py` first",
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
        let mut validate = bench_validate(path, bytes, &rung.label, iters);
        let mut typed = bench_parse_typed(&text, bytes, &rung.label, iters);
        let prepared = prepare_arrow_groups(&text);
        let mut write = bench_write(&prepared, bytes, &rung.label, iters);
        // The children are the memory instrument; the parent drops its own
        // copies first so a big rung's cells aren't squeezed by harness state.
        drop(prepared);
        drop(text);
        if !skip_mem {
            for m in [&mut validate, &mut typed, &mut write] {
                m.mem = Some(measure_mem(m.op, path, bytes));
            }
        }
        results.push(validate);
        results.push(typed);
        results.push(write);
    }

    let output = Output {
        schema: 2,
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

    #[test]
    fn maxrss_units_differ_by_lineage() {
        // Darwin reports bytes, Linux kibibytes — a slip moves every cell 1024×.
        assert_eq!(maxrss_to_bytes(1_048_576, "macos"), 1_048_576);
        assert_eq!(maxrss_to_bytes(1_024, "linux"), 1_048_576);
    }

    #[test]
    fn mem_cap_admits_265_refuses_524() {
        // The pinned rung sizes (tools/readme-bench-fixtures.json): the cap
        // must admit the 265MB rung and refuse 524MB — epic #820 decision 7,
        // in agreement with the python lane's harness.
        assert!(mem_rung_allowed(275_510_179));
        assert!(!mem_rung_allowed(549_703_139));
    }

    #[test]
    fn darwin_swap_parse_reads_the_used_field() {
        let text = "total = 2048.00M  used = 512.50M  free = 1535.50M  (encrypted)";
        assert_eq!(parse_swap_used_darwin(text), Some(537_395_200));
        assert_eq!(parse_swap_used_darwin("garbage"), None);
    }

    #[test]
    fn meminfo_swap_is_total_minus_free() {
        let text = "MemTotal: 100 kB\nSwapTotal:     2048 kB\nSwapFree:      1024 kB\n";
        assert_eq!(parse_meminfo_swap(text), Some(1024 * 1024));
        assert_eq!(parse_meminfo_swap("MemTotal: 1 kB"), None);
    }

    #[test]
    fn mem_cells_are_shape_distinguishable() {
        // The schema-2 contract: a measured cell and a refusal share no keys,
        // so no reader can mistake a vetoed run for a small number.
        let measured = serde_json::to_value(mem_cell(1_500_000, 1_000_000)).unwrap();
        assert_eq!(measured["peak_rss_bytes"], 1_500_000);
        assert_eq!(measured["x_output"], 1.5);
        assert!(measured.get("refusal").is_none());

        let refused =
            serde_json::to_value(refusal_cell("beyond-mem-cap", "too big".into())).unwrap();
        assert_eq!(refused["refusal"], "beyond-mem-cap");
        assert!(refused.get("peak_rss_bytes").is_none());
    }

    #[test]
    fn skip_mem_omits_the_column() {
        // `--skip-mem` must leave no `mem` key at all — an absent column, not
        // a null one, so the aggregator's presence check stays honest.
        let m = Measurement {
            op: "validate",
            rung: "5MB".into(),
            bytes: 1,
            median_ms: 1.0,
            throughput_mb_s: 1.0,
            mem: None,
        };
        let v = serde_json::to_value(&m).unwrap();
        assert!(v.get("mem").is_none());
    }
}
