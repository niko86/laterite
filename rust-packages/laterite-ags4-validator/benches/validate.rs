//! Validator benches — the rules engine, split from the parse it sits on.
//!
//! The old bench here measured `check_file` on a real 23 MB delivery kept in
//! gitignored working space. Two problems: the fixture existed on exactly one
//! machine (everywhere else the bench self-skipped and `cargo bench` reported
//! success while measuring nothing), and `check_file` bundles file I/O, parse
//! and rules into one number, so a rules regression could hide behind parse and
//! vice versa. Fixtures now come from `tools/gen-bench-fixtures.sh`
//! (forge-synthesised, deterministic, no real delivery data), and the work is
//! benched at two levels:
//!
//!   * `check_file` — what a caller actually experiences, I/O included. This is
//!     the number to quote; it is also the one that moves for reasons that have
//!     nothing to do with the rules.
//!   * `check_parsed` — the rules engine ALONE, over an already-parsed file.
//!     Subtract `parse/parse_bytes` at the same rung from `check_file` and this
//!     is what should be left; where it isn't, the difference is I/O and dict
//!     resolution.
//!
//! Absent fixtures skip cleanly, but a skipped bench measures nothing — the
//! failure mode that let the previous version sit dead. Generate them.

use std::hint::black_box;
use std::path::PathBuf;
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use laterite_ags4_validator::{
    CheckOptions, DictVersion, Dictionary, Findings, WorldScope, check_file, check_parsed, parse,
    rules,
};

fn fixture(label: &str) -> Option<PathBuf> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../output/bench-fixtures")
        .join(format!("{label}.ags"));
    p.exists().then_some(p)
}

const RUNGS: [&str; 3] = ["small", "medium", "large"];

fn bench_check_file(c: &mut Criterion) {
    let mut g = c.benchmark_group("validate/check_file");
    g.sample_size(10).measurement_time(Duration::from_secs(30));
    let opts = CheckOptions::default();
    let mut ran = false;
    for label in RUNGS {
        let Some(path) = fixture(label) else { continue };
        let len = std::fs::metadata(&path).expect("stat").len();
        ran = true;
        g.throughput(Throughput::Bytes(len));
        g.bench_with_input(BenchmarkId::from_parameter(label), &path, |b, path| {
            b.iter(|| check_file(black_box(path), &opts).expect("validates"));
        });
    }
    if !ran {
        eprintln!("validate: no fixtures — run tools/gen-bench-fixtures.sh");
    }
    g.finish();
}

fn bench_rules_only(c: &mut Criterion) {
    let mut g = c.benchmark_group("validate/check_parsed");
    g.sample_size(10).measurement_time(Duration::from_secs(30));
    let opts = CheckOptions::default();
    // Resolved once, outside the loop: dictionary construction is a fixed
    // startup cost, not part of per-file rule work.
    let dict = Dictionary::bundled(DictVersion::V4_1_1);
    for label in RUNGS {
        let Some(path) = fixture(label) else { continue };
        let bytes = std::fs::read(&path).expect("fixture readable");
        // Parse ONCE, outside the timed loop — that is the whole point of this
        // bench existing alongside check_file.
        let parsed = parse::parse_bytes(&bytes, encoding_rs::UTF_8).expect("fixture parses");
        g.throughput(Throughput::Bytes(bytes.len() as u64));
        g.bench_with_input(BenchmarkId::from_parameter(label), &parsed, |b, parsed| {
            b.iter(|| {
                check_parsed(black_box(parsed), &dict, &opts, &WorldScope::None).expect("checks")
            });
        });
    }
    g.finish();
}

/// Per-rule-FAMILY staging.
///
/// `check_parsed` says the rules engine is the bulk of validate; it cannot say
/// WHICH rules, and "optimise the rules engine" is not an actionable finding.
/// `rules::run_all` is `pub(crate)`, but every family's `check` is `pub`, so
/// each one can be timed directly over the same parsed file — no API change and
/// no profiler needed to get the first cut.
///
/// Same argument order as `run_all` so this reads against that dispatch list.
/// Note the families take different inputs (some need the dictionary, some the
/// options), which is why this is a hand-written ladder rather than a loop.
fn bench_rule_families(c: &mut Criterion) {
    let Some(path) = fixture("large") else {
        eprintln!("validate: no large fixture — run tools/gen-bench-fixtures.sh");
        return;
    };
    let bytes = std::fs::read(&path).expect("fixture readable");
    let parsed = parse::parse_bytes(&bytes, encoding_rs::UTF_8).expect("fixture parses");
    let dict = Dictionary::bundled(DictVersion::V4_1_1);
    let opts = CheckOptions::default();

    let mut g = c.benchmark_group("validate/rule-family");
    g.sample_size(10).measurement_time(Duration::from_secs(20));
    g.throughput(Throughput::Bytes(bytes.len() as u64));

    macro_rules! fam {
        ($name:literal, $call:expr) => {
            g.bench_function($name, |b| {
                b.iter(|| {
                    let mut found = Findings::new();
                    #[allow(clippy::redundant_closure_call)]
                    ($call)(&mut found);
                    found
                });
            });
        };
    }

    fam!("line_format", |f: &mut Findings| rules::line_format::check(
        &parsed, &opts, f
    ));
    fam!("structure", |f: &mut Findings| rules::structure::check(
        &parsed, f
    ));
    fam!("naming", |f: &mut Findings| rules::naming::check(
        &parsed, f
    ));
    fam!("dictionary", |f: &mut Findings| rules::dictionary::check(
        &parsed, &dict, f
    ));
    fam!("typed_values", |f: &mut Findings| {
        rules::typed_values::check(&parsed, f);
    });
    fam!("relational", |f: &mut Findings| rules::relational::check(
        &parsed, &dict, f
    ));
    fam!("references", |f: &mut Findings| rules::references::check(
        &parsed, &dict, f
    ));
    fam!("groups", |f: &mut Findings| rules::groups::check(
        &parsed, &dict, &opts, f
    ));

    g.finish();
}

/// T5 — the error-reporting half of the engine, which the other three benches
/// never execute: they run `CheckOptions::default()` (both tier gates OFF) over a
/// file the forge asserts is CLEAN, so `findings::add`, rule 10b's per-bad-row
/// `format!`/`join`, rule 11c, and the FYI abbreviation scan sit at zero coverage.
/// This measures them two ways. It LANDS NOTHING — it only makes the error path
/// rankable; whatever it shows re-enters the queue at the same 5% floor.
fn bench_error_path(c: &mut Criterion) {
    let mut g = c.benchmark_group("validate/error-path");
    g.sample_size(10).measurement_time(Duration::from_secs(20));
    // Both tier gates ON: WARNING + FYI findings now flow through `findings::add`,
    // and rule 16's per-ABBR-row scan of the 3,471-entry abbreviation table runs.
    let gated = CheckOptions {
        include_warnings: true,
        include_fyi: true,
        ..CheckOptions::default()
    };
    // (a) The 25 MB CLEAN fixture with the gates on — the SIZE-SCALED cost of the
    // tier traversal itself (rule 16's abbr scan scales with the file's ABBR
    // rows). Clean, so nothing is emitted: this isolates the tier walk from
    // finding-building, and it is the only size-scaled error-path number we can
    // get today.
    if let Some(path) = fixture("large") {
        let len = std::fs::metadata(&path).expect("stat").len();
        g.throughput(Throughput::Bytes(len));
        g.bench_with_input(BenchmarkId::new("large", "gated"), &path, |b, path| {
            b.iter(|| check_file(black_box(path), &gated).expect("validates"));
        });
    }
    // (b) A DIRTY fixture (`forge gen --combine …`, ~100 groups, ~10 rules firing)
    // with the gates on — now `findings::add` and the rule 10b/11c dirty paths
    // actually run. UNSCALED (a handful of findings): a size-scaled densely-dirty
    // rung needs a `forge scale` fault-density mode that does not exist yet, so
    // this prices the error path's SHAPE, not its ceiling.
    if let Some(path) = fixture("dirty") {
        let len = std::fs::metadata(&path).expect("stat").len();
        g.throughput(Throughput::Bytes(len));
        g.bench_with_input(BenchmarkId::new("dirty", "gated"), &path, |b, path| {
            b.iter(|| check_file(black_box(path), &gated).expect("validates"));
        });
    } else {
        eprintln!("validate: no dirty fixture — run tools/gen-bench-fixtures.sh");
    }
    g.finish();
}

criterion_group!(
    benches,
    bench_check_file,
    bench_rules_only,
    bench_rule_families,
    bench_error_path
);
criterion_main!(benches);
