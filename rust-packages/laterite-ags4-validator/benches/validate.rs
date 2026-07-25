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
/// `format!`/`join`, and the FYI abbreviation scan sit at zero coverage. This
/// measures them three ways, now that `forge scale --inject --density` can build
/// SIZE-SCALED densely-dirty twins of `large`. It LANDS NOTHING — it only makes
/// the error path rankable; whatever it shows re-enters the queue at the 5% floor.
///
/// The three levels:
///   * `error-path/{large,dirty-r16}/gated` — whole-engine `check_file`, clean
///     vs a ~340k-finding rule-16 file at the SAME 25 MB size. The delta is the
///     emission MACHINERY (`findings::add` + per-finding message build) at scale.
///   * `relational-emit/{clean,dirty-r10c}` — `relational::check` alone, clean
///     vs a rule-10c file: the relational family (rule 10b's own module).
///   * `rule10b-emit/<n>` — rule 10b's `format!`/`join` in ISOLATION. Rule 10b
///     can't be filled at volume in a real file (its REQUIRED-non-KEY fields are
///     structural, so a dense empty-REQUIRED file cascades), so this prices its
///     per-finding cost on a synthetic single-group file of `n` empty-REQUIRED
///     rows — unique keys → no 10a, a root group → no 10c: only 10b fires.
fn bench_error_path(c: &mut Criterion) {
    let gated = CheckOptions {
        include_warnings: true,
        include_fyi: true,
        ..CheckOptions::default()
    };
    let dict = Dictionary::bundled(DictVersion::V4_1_1);

    // (1) Whole-engine, size-scaled: clean `large` vs the rule-16 dirty twin.
    let mut g = c.benchmark_group("validate/error-path");
    g.sample_size(10).measurement_time(Duration::from_secs(20));
    for (label, fx) in [("large", "large"), ("dirty-r16", "dirty-r16")] {
        let Some(path) = fixture(fx) else {
            eprintln!("validate: no {fx} fixture — run tools/gen-bench-fixtures.sh");
            continue;
        };
        let len = std::fs::metadata(&path).expect("stat").len();
        g.throughput(Throughput::Bytes(len));
        g.bench_with_input(BenchmarkId::new(label, "gated"), &path, |b, path| {
            b.iter(|| check_file(black_box(path), &gated).expect("validates"));
        });
    }
    g.finish();

    // (2) Relational family alone (rule 10b's module): clean vs the rule-10c twin.
    let mut gr = c.benchmark_group("validate/relational-emit");
    gr.sample_size(10).measurement_time(Duration::from_secs(20));
    for (label, fx) in [("clean", "large"), ("dirty-r10c", "dirty-r10c")] {
        let Some(path) = fixture(fx) else { continue };
        let bytes = std::fs::read(&path).expect("fixture readable");
        let parsed = parse::parse_bytes(&bytes, encoding_rs::UTF_8).expect("parses");
        gr.throughput(Throughput::Bytes(bytes.len() as u64));
        gr.bench_with_input(BenchmarkId::from_parameter(label), &parsed, |b, parsed| {
            b.iter(|| {
                let mut found = Findings::new();
                rules::relational::check(black_box(parsed), &dict, &mut found);
                found
            });
        });
    }
    gr.finish();

    // (3) Rule 10b's format!/join in isolation, scaling with the bad-row count.
    let mut gm = c.benchmark_group("validate/rule10b-emit");
    gm.sample_size(10).measurement_time(Duration::from_secs(15));
    for n in [10_000usize, 200_000] {
        let bytes = empty_required_ags(&dict, "ABBR", n);
        let parsed = parse::parse_bytes(&bytes, encoding_rs::UTF_8).expect("parses");
        gm.throughput(Throughput::Elements(n as u64));
        gm.bench_with_input(BenchmarkId::from_parameter(n), &parsed, |b, parsed| {
            b.iter(|| {
                let mut found = Findings::new();
                rules::relational::check(black_box(parsed), &dict, &mut found);
                found
            });
        });
    }
    gm.finish();
}

/// An AGS4 buffer: `n` DATA rows of `group`, every KEY cell a unique non-empty
/// value and every non-KEY cell EMPTY — so each row's pure-REQUIRED field is
/// empty (→ Rule 10b) while unique keys avoid Rule 10a and a root group avoids
/// Rule 10c. Dictionary-driven, so it tracks the real schema. Only
/// `relational::check` runs over it, so the empty typed cells never trip a type
/// rule. This isolates rule 10b's per-bad-row `format!`/`join`.
fn empty_required_ags(dict: &Dictionary, group: &str, n: usize) -> Vec<u8> {
    let headings = dict.group_headings(group);
    assert!(!headings.is_empty(), "unknown bench group {group}");
    let is_key = |h: &str| {
        dict.heading(group, h).is_some_and(|hr| {
            hr.status
                .split('+')
                .any(|p| p.trim().eq_ignore_ascii_case("KEY"))
        })
    };
    let key_cols: Vec<usize> = (0..headings.len())
        .filter(|&i| is_key(headings[i]))
        .collect();
    assert!(
        !key_cols.is_empty(),
        "bench group {group} needs a KEY column"
    );

    let quote_join = |cells: &[String]| {
        cells
            .iter()
            .map(|c| format!("\"{c}\""))
            .collect::<Vec<_>>()
            .join(",")
    };

    let names: Vec<String> = headings.iter().map(|h| (*h).to_string()).collect();
    let mut lines = vec![
        format!("\"GROUP\",\"{group}\""),
        format!("\"HEADING\",{}", quote_join(&names)),
        format!(
            "\"UNIT\",{}",
            quote_join(&vec![String::new(); headings.len()])
        ),
        format!(
            "\"TYPE\",{}",
            quote_join(&vec!["X".to_string(); headings.len()])
        ),
    ];
    for i in 0..n {
        // Only KEY cells filled (uniquely); the pure-REQUIRED and OTHER cells
        // stay empty — the REQUIRED emptiness is exactly what trips Rule 10b.
        let mut cells = vec![String::new(); headings.len()];
        for &c in &key_cols {
            cells[c] = format!("k{c}_{i}");
        }
        lines.push(format!("\"DATA\",{}", quote_join(&cells)));
    }
    let mut out = lines.join("\r\n");
    out.push_str("\r\n");
    out.into_bytes()
}

criterion_group!(
    benches,
    bench_check_file,
    bench_rules_only,
    bench_rule_families,
    bench_error_path
);
criterion_main!(benches);
