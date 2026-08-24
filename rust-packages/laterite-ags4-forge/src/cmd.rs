//! Subcommand implementations: `check` (dual-validate one file), `gen`
//! (synthesize → optionally inject → dual-validate), `run` (the
//! evolutionary search loop), `minimize` (ddmin a repro), `strategy`
//! (load/validate a strategy file), `confidence` (inspect the ledger),
//! and `seed vendor`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use laterite_cliutil::report::{Ctx, Plan, emit, note};

use crate::artifacts::{forge_dir, guard_out_dir, new_run_id, run_dir, set_latest_run};
use crate::cli::{
    CheckArgs, ConfidenceArgs, EditArgs, GenArgs, MineArgs, MinimizeArgs, RunArgs, StrategyArgs,
    VendorArgs,
};
use crate::evolve::evolve;
use crate::mine::{MineCfg, run_mine};
use crate::minimize::{insight_stub, minimize as ddmin};
use crate::ops::{Injection, synth_combined_lab, synth_injected_lab};
use crate::pipeline::{build_oracle, dual_validate};
use crate::report::{
    Candidate, CatalogEntry, CatalogReport, CheckReport, CheckSweepReport, DescribeReport,
    DescribeRow, ForgeReport, UninjectableRule,
};
use crate::strategy::{ConfidenceCfg, Strategy};
use crate::synth::Scaffold;

/// Next free `O-N` from the canonical OBSERVATIONS authority (the CLI
/// only drafts the stub; the author writes the ratified entry).
fn next_obs_id() -> String {
    let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../laterite-ags4-validator/OBSERVATIONS.md");
    let max = std::fs::read_to_string(&p).ok().map_or(36, |t| {
        t.lines()
            .filter_map(|l| {
                l.strip_prefix("### O-")
                    .and_then(|r| r.split([' ', '\t']).next())
                    .and_then(|n| n.parse::<u32>().ok())
            })
            .max()
            .unwrap_or(36)
    });
    format!("O-{}", max + 1)
}

/// Parse one `--combine` spec (`"rule10a,rule8,rule5"`) into its injector
/// sequence. `Err(tok)` names the first unknown token. Empty tokens
/// (trailing comma, whitespace) are skipped; an all-empty spec is `Ok([])`
/// (the caller treats it as a no-op combination).
fn parse_combo(spec: &str) -> Result<Vec<Injection>, String> {
    spec.split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(|t| Injection::parse(t).ok_or_else(|| t.to_string()))
        .collect()
}

/// A combination's label (`rule10a+rule8`) and the `+`-joined *intended*
/// target rules — distinct from the file's actual validated rule-set,
/// which lives in the candidate's rust/python rule arrays.
fn combo_meta(injs: &[Injection]) -> (String, Option<String>) {
    let label = injs
        .iter()
        .copied()
        .map(super::ops::Injection::token)
        .collect::<Vec<_>>()
        .join("+");
    let targets: Vec<&str> = injs
        .iter()
        .copied()
        .filter_map(super::ops::Injection::target_rule)
        .collect();
    let target = (!targets.is_empty()).then(|| targets.join(" + "));
    (label, target)
}

/// `forge check <file>` — exit 1 iff a real (unexplained) divergence.
pub fn check(args: &CheckArgs, ctx: Ctx) -> Result<i32> {
    let mut files: Vec<PathBuf> = Vec::new();
    let mut skipped = 0usize;
    for p in &args.paths {
        if !p.exists() {
            eprintln!("error: file not found: {}", p.display());
            return Ok(3);
        }
        collect_ags(p, true, &mut files, &mut skipped);
    }
    files.sort();
    files.dedup();
    if files.is_empty() {
        eprintln!("error: no .ags file among the given path(s)");
        return Ok(3);
    }
    let oracle = if args.no_oracle {
        None
    } else {
        build_oracle(args.timeout).map(|(o, _)| o)
    };
    if oracle.is_none() && !args.no_oracle {
        note("check: python-ags4 unavailable — Rust-only verdict (optional QA)");
    }
    let reports: Vec<CheckReport> = files
        .iter()
        .map(|f| {
            let outcome = dual_validate(f, oracle.as_ref());
            CheckReport::new(f.display().to_string(), &outcome, oracle.is_some())
        })
        .collect();
    // One file named directly keeps the single-file document it has always
    // emitted — a caller reading `.verdict` off it predates the sweep and
    // must not have to learn a new shape to keep working.
    if reports.len() == 1 && args.paths.len() == 1 && args.paths[0].is_file() {
        let action = reports[0].is_action();
        emit(&reports[0], &ctx)?;
        return Ok(i32::from(action));
    }
    let sweep = CheckSweepReport::new(reports, oracle.is_some(), skipped);
    let action = sweep.is_action();
    emit(&sweep, &ctx)?;
    Ok(i32::from(action))
}

/// Expand one path into the files to validate, counting what was walked
/// past. A directory recurses and keeps only its `.ags` files — the caller
/// may well have pointed at a mixed corpus on purpose, so the rest are a
/// counted skip rather than an error.
///
/// `named` is whether the CALLER wrote this path on the command line. A
/// named file is taken whatever it is called: `check` has always validated
/// the path it was handed, and an extension filter that reached a named
/// file would turn "validate this" into "find nothing" for every delivery
/// that does not end in `.ags` — which is a real shape, and a silent one.
/// The filter is for deciding what a DIRECTORY meant, and nothing else.
fn collect_ags(path: &Path, named: bool, out: &mut Vec<PathBuf>, skipped: &mut usize) {
    if path.is_dir() {
        let Ok(rd) = std::fs::read_dir(path) else {
            return;
        };
        let mut entries: Vec<PathBuf> = rd.filter_map(|e| e.ok().map(|e| e.path())).collect();
        entries.sort();
        for e in entries {
            collect_ags(&e, false, out, skipped);
        }
        return;
    }
    let is_ags = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("ags"));
    if named || is_ags {
        out.push(path.to_path_buf());
    } else {
        *skipped += 1;
    }
}

/// `forge gen` — synthesize the base, apply each `--inject`, write the
/// candidates, dual-validate. Exit 1 iff any candidate is a real
/// divergence. (Named `generate`: `gen` is a reserved keyword in the
/// 2024 edition; the clap subcommand is still `gen` for users.)
pub fn generate(args: &GenArgs, ctx: Ctx) -> Result<i32> {
    let Some(scaffold) = Scaffold::parse(&args.scaffold) else {
        eprintln!(
            "error: unknown --scaffold '{}' (use: minimal | loca-samp | wide)",
            args.scaffold
        );
        return Ok(5);
    };
    let mut injections: Vec<Injection> = Vec::new();
    for tok in &args.inject {
        if let Some(i) = Injection::parse(tok) {
            injections.push(i);
        } else {
            eprintln!("error: unknown --inject '{tok}'");
            return Ok(5);
        }
    }
    // No `--inject` and no `--combine` → dual-validate the clean baseline
    // itself. (A bare `--combine` run gets only its combinations.)
    if injections.is_empty() && args.combine.is_empty() {
        injections.push(Injection::None);
    }
    // Parse the `--combine` specs up-front so bad args fail before any
    // file is written (each spec → one combined candidate).
    let mut combos: Vec<Vec<Injection>> = Vec::new();
    for spec in &args.combine {
        match parse_combo(spec) {
            Ok(injs) => combos.push(injs),
            Err(tok) => {
                eprintln!("error: unknown --combine token '{tok}' (in \"{spec}\")");
                return Ok(5);
            }
        }
    }
    // A relational injector needs the LOCA→SAMP scaffold to be
    // single-rule-isolable — fail fast rather than emit a noisy multi-
    // rule candidate. Applies to both single injects and combinations.
    let needs_relational = injections
        .iter()
        .copied()
        .any(super::ops::Injection::needs_relational)
        || combos
            .iter()
            .flatten()
            .copied()
            .any(super::ops::Injection::needs_relational);
    if scaffold == Scaffold::Minimal && needs_relational {
        eprintln!(
            "error: a relational injector needs --scaffold loca-samp \
             (minimal has no LOCA→SAMP base)"
        );
        return Ok(5);
    }

    let out = forge_dir(args.out_dir.as_deref());
    guard_out_dir(&out)?;

    let total = injections.len() + combos.len();
    if ctx.dry_run {
        let plan = Plan::new(
            "gen",
            format!(
                "would synthesize {} candidate(s) [scaffold={}] and {}",
                total,
                args.scaffold,
                if args.validate {
                    "dual-validate"
                } else {
                    "Rust-validate"
                },
            ),
        )
        .with("scaffold", args.scaffold.clone())
        .with(
            "injections",
            injections
                .iter()
                .map(std::string::ToString::to_string)
                .collect::<Vec<_>>()
                .join(","),
        )
        .with(
            "combinations",
            combos
                .iter()
                .map(|c| combo_meta(c).0)
                .collect::<Vec<_>>()
                .join(","),
        )
        .with("would_write", total as u64)
        .with("out_dir", out.display().to_string());
        emit(&plan, &ctx)?;
        return Ok(0);
    }

    let run_id = new_run_id();
    let dir = run_dir(&out, &run_id);
    std::fs::create_dir_all(&dir)?;

    let oracle = if args.validate && !args.no_oracle {
        build_oracle(args.timeout).map(|(o, _)| o)
    } else {
        None
    };
    if args.validate && !args.no_oracle && oracle.is_none() {
        note("gen: python-ags4 unavailable — Rust-only verdicts (optional QA)");
    }

    let mut candidates = Vec::new();
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    for (seq, inj) in injections.iter().enumerate() {
        let text = synth_injected_lab(scaffold, args.seed, *inj, args.lab_test_rate);
        let fname = format!("cand_{seq:03}_{inj}.ags");
        let path = dir.join(fname.replace([':', '/'], "_"));
        std::fs::write(&path, text.as_bytes())?;
        let outcome = dual_validate(&path, oracle.as_ref());
        let c = Candidate::from_outcome(
            seq,
            inj.to_string(),
            inj.target_rule().map(String::from),
            path.display().to_string(),
            &outcome,
        );
        *counts.entry(c.verdict.clone()).or_default() += 1;
        candidates.push(c);
    }

    // Combined multi-fault candidates — one per `--combine` spec, seq
    // continuing after the single injects. The faults interact, so the
    // candidate records the file's *actual* validated rule-set; the
    // intended-target union is carried only as `target_rule` for traceability.
    for (k, injs) in combos.iter().enumerate() {
        let seq = injections.len() + k;
        let (label, target) = combo_meta(injs);
        let text = synth_combined_lab(scaffold, args.seed, injs, args.lab_test_rate);
        let fname = format!("cand_{seq:03}_combo_{label}.ags");
        let path = dir.join(fname.replace([':', '/', '+'], "_"));
        std::fs::write(&path, text.as_bytes())?;
        let outcome = dual_validate(&path, oracle.as_ref());
        let c = Candidate::from_outcome(
            seq,
            format!("combine:{label}"),
            target,
            path.display().to_string(),
            &outcome,
        );
        *counts.entry(c.verdict.clone()).or_default() += 1;
        candidates.push(c);
    }

    let report = ForgeReport {
        schema: 1,
        created: chrono::Utc::now().to_rfc3339(),
        strategy: "gen (flag-driven)".into(),
        scaffold: args.scaffold.clone(),
        oracle: oracle.is_some(),
        counts,
        candidates,
    };
    let report_path = dir.join("report.json");
    std::fs::write(&report_path, serde_json::to_string_pretty(&report)?)?;
    set_latest_run(&out, &run_id)?;
    note(format!("runs/latest → {run_id}"));
    note(format!("forge → {}", report_path.display()));
    let action = report.actions_present();
    emit(&report, &ctx)?;
    Ok(i32::from(action))
}

/// `forge run` — the evolutionary loop. Builds a `Strategy` from the flags
/// (or loads one wholesale with `--strategy`), runs `evolve`, emits the
/// report. Exit: 1 findings · 2 stalled (frontier emitted) · 0 clean.
pub fn run(args: &RunArgs, ctx: Ctx) -> Result<i32> {
    if crate::synth::Scaffold::parse(&args.scaffold).is_none() {
        eprintln!(
            "error: unknown --scaffold '{}' (use: minimal | loca-samp | wide)",
            args.scaffold
        );
        return Ok(5);
    }
    // Flag-built strategy; a `--strategy` file (the author↔CLI
    // contract) wins wholesale if given (already schema-validated by
    // `Strategy::load`).
    let strat = if let Some(sp) = &args.strategy {
        match Strategy::load(sp) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("error: invalid strategy {}: {e:#}", sp.display());
                return Ok(5);
            }
        }
    } else {
        Strategy {
            name: "run (flag-driven)".into(),
            scaffold: args.scaffold.clone(),
            injectors: if args.inject.is_empty() {
                Strategy::default().injectors
            } else {
                args.inject.clone()
            },
            max_generations: args.max_generations,
            max_candidates: args.max_candidates,
            max_wall_secs: args.max_wall_secs,
            python_budget: args.python_budget,
            stale_soft: args.stale_soft,
            stale_hard: args.stale_hard,
            seed: args.seed,
            confidence: ConfidenceCfg {
                enabled: true,
                floor: args.floor,
                force_burst: args.force_burst,
            },
        }
    };

    let out = forge_dir(args.out_dir.as_deref());
    guard_out_dir(&out)?;

    if ctx.dry_run {
        let plan = Plan::new(
            "run",
            format!(
                "would evolve up to {} generations [scaffold={}, seed={}], \
                 oracle budget {}, ledger {}/confidence.json",
                strat.max_generations,
                strat.scaffold,
                strat.seed,
                strat.python_budget,
                out.display(),
            ),
        )
        .with("seed", strat.seed)
        .with("stale_soft", strat.stale_soft)
        .with("stale_hard", strat.stale_hard)
        .with("floor", strat.confidence.floor)
        .with("out_dir", out.display().to_string());
        emit(&plan, &ctx)?;
        return Ok(0);
    }

    let (oracle, oracle_ver) = if let Some((o, v)) = if args.no_oracle {
        None
    } else {
        build_oracle(args.timeout)
    } {
        (Some(o), v)
    } else {
        if !args.no_oracle {
            note(
                "run: python-ags4 unavailable — Rust-only loop \
                 (deterministic; no parity verdicts). optional QA.",
            );
        }
        (None, "none".to_string())
    };

    let run_id = new_run_id();
    let outcome = evolve(&strat, oracle.as_ref(), &oracle_ver, &out, &run_id, true)?;
    set_latest_run(&out, &run_id)?;

    // ddmin every finding to a minimal, signature-preserving repro +
    // a drafted insight/O-N stub (the §12.5 hand-off). The CLI only
    // drafts — the author ratifies and writes OBSERVATIONS.md.
    if !outcome.report.findings.is_empty() {
        let repro_root = run_dir(&out, &run_id).join("repros");
        let next = next_obs_id();
        for (i, f) in outcome.report.findings.iter().enumerate() {
            let Ok(text) = std::fs::read_to_string(&f.path) else {
                continue;
            };
            let (minimal, sig) = ddmin(&text, oracle.as_ref());
            let d = repro_root.join(format!("{i:02}_{}", f.injection.replace(':', "_")));
            std::fs::create_dir_all(&d)?;
            std::fs::write(d.join("minimal.ags"), minimal.as_bytes())?;
            std::fs::write(
                d.join("meta.json"),
                serde_json::to_string_pretty(&serde_json::json!({
                    "injection": f.injection, "target_rule": f.target_rule,
                    "verdict": sig.0, "rust_rules": sig.1, "python_rules": sig.2,
                    "from": f.path, "original_bytes": text.len(),
                    "minimal_bytes": minimal.len(),
                }))?,
            )?;
            let rel = format!(
                "ags-wiki/.bootstrap/probes/probe-forge-{}.ags",
                f.injection.replace(':', "-")
            );
            std::fs::write(
                d.join("insight-stub.md"),
                insight_stub(&sig, &f.injection, &rel, &next),
            )?;
        }
        note(format!(
            "minimized {} finding(s) → {} (promote to ags-wiki/.bootstrap/probes/, never tests/fixtures/)",
            outcome.report.findings.len(),
            repro_root.display()
        ));
    }

    note(format!("runs/latest → {run_id}"));
    note(format!(
        "forge → {}",
        run_dir(&out, &run_id).join("report.json").display()
    ));
    if outcome.report.status == "stalled" {
        note(format!(
            "STALLED — frontier → {} (author the next strategy)",
            run_dir(&out, &run_id).join("frontier.json").display()
        ));
    }
    emit(&outcome.report, &ctx)?;
    Ok(outcome.exit)
}

/// `forge mine` — the corpus-gap divergence miner. Profiles the corpus,
/// synthesizes every rule-combination across the placement-seed sweep,
/// subtracts what the corpus already covers, and spends the oracle on the
/// novel divergence-prone signatures. Exit 1 iff a real divergence.
pub fn mine(args: &MineArgs, ctx: Ctx) -> Result<i32> {
    let Some(scaffold) = Scaffold::parse(&args.scaffold) else {
        eprintln!(
            "error: unknown --scaffold '{}' (use: minimal | loca-samp | wide)",
            args.scaffold
        );
        return Ok(5);
    };
    if args.min_k == 0 || args.min_k > args.max_k {
        eprintln!(
            "error: need 1 <= --min-k ({}) <= --max-k ({})",
            args.min_k, args.max_k
        );
        return Ok(5);
    }
    if args.seeds == 0 {
        eprintln!("error: --seeds must be >= 1");
        return Ok(5);
    }
    // Default corpus: the vendored python-ags4 fixtures (run `forge seed
    // vendor` first). Absent → an empty covered-set, surfaced in the report.
    let corpus = args.corpus.clone().unwrap_or_else(|| {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("vendor/pyags4-tests")
    });

    let out = forge_dir(args.out_dir.as_deref());
    guard_out_dir(&out)?;

    if ctx.dry_run {
        let plan = Plan::new(
            "mine",
            format!(
                "would synthesize k={}..={} combinations × {} seed(s) [scaffold={}], \
                 subtract corpus {}, then {}",
                args.min_k,
                args.max_k,
                args.seeds,
                args.scaffold,
                corpus.display(),
                if args.no_oracle {
                    "Rust-only (no parity verdict)".to_string()
                } else if args.always_validate {
                    format!("dual-validate every gap (cap {})", args.max_oracle)
                } else {
                    format!(
                        "dual-validate divergence-prone gaps (cap {})",
                        args.max_oracle
                    )
                },
            ),
        )
        .with("scaffold", args.scaffold.clone())
        .with("corpus", corpus.display().to_string())
        .with("min_k", args.min_k as u64)
        .with("max_k", args.max_k as u64)
        .with("seeds", args.seeds)
        .with("always_validate", args.always_validate.to_string())
        .with("out_dir", out.display().to_string());
        emit(&plan, &ctx)?;
        return Ok(0);
    }

    let oracle = if args.no_oracle {
        None
    } else {
        build_oracle(args.timeout).map(|(o, _)| o)
    };
    if !args.no_oracle && oracle.is_none() {
        note(
            "mine: python-ags4 unavailable — Rust-only (gaps + divergence flags, no parity verdict)",
        );
    }

    let run_id = new_run_id();
    let dir = run_dir(&out, &run_id);
    std::fs::create_dir_all(&dir)?;

    let cfg = MineCfg {
        scaffold,
        corpus,
        min_k: args.min_k,
        max_k: args.max_k,
        seeds: args.seeds,
        base_seed: args.seed,
        always_validate: args.always_validate,
        max_oracle: args.max_oracle,
    };
    let report = run_mine(&cfg, oracle.as_ref(), &dir, chrono::Utc::now().to_rfc3339())?;
    let report_path = dir.join("report.json");
    std::fs::write(&report_path, serde_json::to_string_pretty(&report)?)?;
    set_latest_run(&out, &run_id)?;
    note(format!("runs/latest → {run_id}"));
    note(format!(
        "mined {} corpus gap(s), {} divergence-prone → {}",
        report.gaps,
        report.divergence_prone_gaps,
        report_path.display()
    ));
    let action = report.actions_present();
    emit(&report, &ctx)?;
    Ok(i32::from(action))
}

/// `forge catalog` — the injector→rule map + the documented record of
/// canonical AGS rules that aren't cleanly single-injectable from the
/// typed model. Pure data (no synthesis); always exit 0.
pub fn catalog(ctx: Ctx) -> Result<i32> {
    let injectors = Injection::ALL
        .iter()
        .map(|i| CatalogEntry {
            token: i.token().to_string(),
            target_rule: i.target_rule().unwrap_or("-").to_string(),
            scaffold: if i.needs_relational() {
                "loca-samp".into()
            } else {
                "any".into()
            },
            description: i.description().to_string(),
        })
        .collect();
    // Honest coverage record — why these aren't single-rule injectors
    // (the typed emitter quotes uniformly + writes descriptors in fixed
    // order, and some rules can't fire without co-tripping a sibling).
    let note = |rules: &str, reason: &str| UninjectableRule {
        rules: rules.into(),
        reason: reason.into(),
    };
    let not_single_injectable = vec![
        note(
            "1, 2a, 3, 6",
            "need byte-level malformation the typed emitter can't express \
             (>255 code point, LF-only line, bad line descriptor, embedded CR)",
        ),
        note(
            "2b",
            "UNIT/TYPE placement — the emitter writes GROUP/HEADING/UNIT/TYPE \
             in fixed order; a reorder needs a new emitter marker",
        ),
        note(
            "4",
            "GROUP/field-count mismatch breaks the headings=units=types=values \
             length invariant the model maintains",
        ),
        note(
            "9, 18, 19a, 19b",
            "renaming a heading to trip a heading-name rule also trips Rule 9 \
             (non-dictionary heading) and often a sibling — not isolable",
        ),
        note(
            "11a, 11b, 11c",
            "Record-Link machinery (TRAN_DLIM/RCON + RL resolution) — \
             domain-specific setup",
        ),
        note(
            "15",
            "undefined UNIT — the UNIT-group twin of Rule 16/17 \
             (candidate future injector)",
        ),
        note(
            "20",
            "FILE attachment needs the on-disk FILE tree + FILE group",
        ),
    ];
    let report = CatalogReport {
        schema: 1,
        injectors,
        not_single_injectable,
    };
    emit(&report, &ctx)?;
    Ok(0)
}

/// `forge describe` — preview the BS 5930 soil-description engine (the
/// realistic `GEOL_DESC` source). Pure generation; always exit 0.
pub fn describe(args: &crate::cli::DescribeArgs, ctx: Ctx) -> Result<i32> {
    use laterite_ags4_parity::Rng;
    let vocab = crate::synth::bs5930::Vocab::load();
    let descriptions = (0..args.count)
        .map(|i| {
            let seed = args.seed.wrapping_add(i);
            let d = crate::synth::bs5930::describe(&vocab, &mut Rng::seeded(seed));
            let fractions = d
                .secondaries
                .iter()
                .map(|s| {
                    let kind = if s.is_fine() { "fine" } else { "coarse" };
                    if s.pct == 0 {
                        format!("{} ({kind}, named)", s.soil)
                    } else {
                        let q = if s.qualifier.is_empty() {
                            "plain"
                        } else {
                            s.qualifier
                        };
                        format!("{} {}% ({kind}, {q})", s.soil, s.pct)
                    }
                })
                .collect();
            DescribeRow {
                seed,
                principal: d.principal.to_string(),
                class: format!("{:?}", d.principal_class),
                text: d.text,
                fractions,
            }
        })
        .collect::<Vec<_>>();
    let report = DescribeReport {
        schema: 1,
        count: descriptions.len(),
        base_seed: args.seed,
        descriptions,
    };
    emit(&report, &ctx)?;
    Ok(0)
}

/// `forge scale --size <S>` — calibrate the borehole count to a target
/// byte size and stream the clean wide file to disk. Exit 0 / 5 (bad args)
/// / 3 (I/O).
pub fn scale(args: &crate::cli::ScaleArgs, ctx: Ctx) -> Result<i32> {
    use std::io::{BufWriter, Write};

    let Some(scaffold) = Scaffold::parse(&args.scaffold) else {
        eprintln!(
            "error: unknown --scaffold '{}' (use: loca-samp | wide)",
            args.scaffold
        );
        return Ok(5);
    };
    let Some(target) = crate::scale::parse_size(&args.size) else {
        eprintln!("error: bad --size '{}' (e.g. 500KB, 50MB, 1GB)", args.size);
        return Ok(5);
    };
    let cal = match crate::scale::calibrate(scaffold, args.seed, target) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            return Ok(5);
        }
    };

    // Fault-density mode: resolve --inject/--density to an optional
    // (injector, density). `None` = a clean scale (byte-identical to before).
    let dirty = match resolve_scale_inject(args.inject.as_deref(), args.density) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: {e}");
            return Ok(5);
        }
    };

    let out = forge_dir(args.out_dir.as_deref());
    let path = args.out.clone().unwrap_or_else(|| {
        out.join("scale").join(format!(
            "{}_{}.ags",
            args.scaffold,
            args.size.to_lowercase().replace(['/', ' '], "_")
        ))
    });

    if ctx.dry_run {
        let plan = Plan::new(
            "scale",
            format!(
                "would synthesize a {} file ~{} bytes ({} boreholes) → {}",
                args.scaffold,
                cal.predicted_bytes,
                cal.n_loca,
                path.display()
            ),
        )
        .with("scaffold", args.scaffold.clone())
        .with("target_bytes", target)
        .with("predicted_bytes", cal.predicted_bytes)
        .with("n_loca", cal.n_loca as u64)
        .with(
            "inject",
            dirty
                .as_ref()
                .map_or_else(|| "none".to_string(), |(i, _)| i.token().to_string()),
        )
        .with(
            "density",
            dirty
                .as_ref()
                .map_or_else(|| "n/a".to_string(), |(_, d)| d.to_string()),
        )
        .with("path", path.display().to_string());
        emit(&plan, &ctx)?;
        return Ok(0);
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // Build once; corrupt in place if a fault-density mode is active (the
    // clean twin is exactly this model with no `apply_dense`); stream out.
    let mut model = crate::synth::model::varied_model_n(scaffold, args.seed, Some(cal.n_loca));
    let dirty_sites = match dirty {
        Some((inj, density)) => inj.apply_dense(&mut model, args.seed, density),
        None => 0,
    };
    let groups = model.groups.len();
    let file = std::fs::File::create(&path)?;
    let mut w = BufWriter::new(file);
    crate::synth::emit::emit(&model, &mut w)?;
    w.flush()?;
    let actual = std::fs::metadata(&path)?.len();

    let dirty_note = dirty.map_or_else(String::new, |(i, d)| {
        format!(", inject={} density={d} ({dirty_sites} sites)", i.token())
    });
    note(format!(
        "scale → {} ({} bytes, {} boreholes, {} groups{})",
        path.display(),
        actual,
        cal.n_loca,
        groups,
        dirty_note
    ));
    let report = crate::report::ScaleReport {
        schema: 2,
        scaffold: args.scaffold.clone(),
        seed: args.seed,
        target_bytes: target,
        predicted_bytes: cal.predicted_bytes,
        actual_bytes: actual,
        n_loca: cal.n_loca,
        groups,
        path: path.display().to_string(),
        inject: dirty.map(|(i, _)| i.token().to_string()),
        density: dirty.map(|(_, d)| d),
        dirty_sites,
    };
    emit(&report, &ctx)?;
    Ok(0)
}

/// Parse + validate `forge scale`'s `--inject`/`--density` into an optional
/// (injector, density) pair. `None` = a clean scale (no `--inject`, or the
/// explicit `none` token). Every `Err` maps to exit 5 (bad args) in `scale`;
/// kept pure so it is unit-tested directly. `--density` defaults to `1.0`
/// (every applicable site) when `--inject` is given without it.
fn resolve_scale_inject(
    inject: Option<&str>,
    density: Option<f64>,
) -> Result<Option<(Injection, f64)>, String> {
    let Some(token) = inject else {
        if density.is_some() {
            return Err("--density requires --inject".into());
        }
        return Ok(None);
    };
    let Some(inj) = Injection::parse(token) else {
        return Err(format!("unknown --inject '{token}'"));
    };
    if inj == Injection::None {
        return Ok(None); // explicit clean; any --density is a no-op
    }
    if !inj.supports_density() {
        return Err(format!(
            "--inject '{token}' has a single fixed site; density-capable: \
             rule10b|rule10c|rule8|rule5|rule16"
        ));
    }
    // Rejects 0.0, negatives, > 1.0, and NaN (NaN fails both comparisons).
    let density = density.unwrap_or(1.0);
    if !(density > 0.0 && density <= 1.0) {
        return Err("--density must be in (0.0, 1.0]".into());
    }
    Ok(Some((inj, density)))
}

/// `forge minimize <file>` — standalone ddmin (e.g. shrink a
/// corpus-qa ACTION file to a clean wiki probe).
pub fn minimize(args: &MinimizeArgs, ctx: Ctx) -> Result<i32> {
    if !args.file.exists() {
        eprintln!("error: file not found: {}", args.file.display());
        return Ok(3);
    }
    let oracle = if args.no_oracle {
        None
    } else {
        build_oracle(args.timeout).map(|(o, _)| o)
    };
    let text = std::fs::read_to_string(&args.file)?;
    let (minimal, sig) = ddmin(&text, oracle.as_ref());
    if let Some(p) = &args.out {
        crate::artifacts::guard_out_dir(p)?;
        std::fs::write(p, minimal.as_bytes())?;
        note(format!("minimal → {}", p.display()));
    }
    let doc = serde_json::json!({
        "schema": 1, "file": args.file.display().to_string(),
        "verdict": sig.0, "rust_rules": sig.1, "python_rules": sig.2,
        "original_bytes": text.len(), "minimal_bytes": minimal.len(),
        "out": args.out.as_ref().map(|p| p.display().to_string()),
    });
    emit_value(&doc, ctx)?;
    Ok(0)
}

/// `forge edit` — structured edits to a real AGS4 file (#655).
///
/// Nothing is written without `--out` or `--in-place`: the default run is the
/// preview, because "what would this change?" is the question worth answering
/// before a file the investigation depends on is overwritten. The report
/// carries `unchanged`, which is the no-op property made observable — an edit
/// that reports `unchanged: true` did nothing, whatever it claimed to do.
pub fn edit(args: &EditArgs, ctx: Ctx) -> Result<i32> {
    if args.patch_template {
        print!("{}", crate::edit::Patch::template());
        return Ok(0);
    }
    if !args.file.exists() {
        eprintln!("error: file not found: {}", args.file.display());
        return Ok(3);
    }

    // Flags apply in a fixed order, which is safe precisely because every
    // operation resolves against the file as it arrived: no order of these
    // can give a different answer, so there is nothing for a caller to get
    // wrong. `--patch` runs first, so its ops read as authored.
    let mut ops = match &args.patch {
        Some(p) => match crate::edit::Patch::load(p) {
            Ok(ops) => ops,
            Err(e) => {
                eprintln!("error: patch {}: {e:#}", p.display());
                return Ok(5);
            }
        },
        None => Vec::new(),
    };
    for (kind, specs) in [
        ("set", &args.set),
        ("blank", &args.blank),
        ("add-row", &args.add_row),
        ("delete-row", &args.delete_row),
        ("delete-column", &args.delete_column),
        ("delete-group", &args.delete_group),
    ] {
        for spec in specs {
            match crate::edit::parse_flag(kind, spec) {
                Ok(op) => ops.push(op),
                Err(e) => {
                    eprintln!("error: {e:#}");
                    return Ok(5);
                }
            }
        }
    }
    if ops.is_empty() {
        eprintln!(
            "error: no operations — pass --set/--blank/--add-row/--delete-row/\
             --delete-column/--delete-group or --patch (--patch-template shows one)"
        );
        return Ok(5);
    }

    let text = std::fs::read_to_string(&args.file)?;
    let edited = match crate::edit::apply(&text, &ops) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: {e}");
            return Ok(5);
        }
    };

    let target = if args.in_place {
        Some(args.file.clone())
    } else {
        args.out.clone()
    };
    let written = match &target {
        Some(p) if !ctx.dry_run => {
            if !args.in_place {
                crate::artifacts::guard_out_dir(p)?;
            }
            std::fs::write(p, edited.as_bytes())?;
            note(format!("edited → {}", p.display()));
            true
        }
        _ => false,
    };

    // Lines that differ POSITIONALLY, which is what a reader sees in a diff
    // rather than what the editor touched: a one-cell edit reports 1, but a
    // deleted or added row shifts everything after it, so a single delete near
    // the top of a file reports most of the file. That is the honest number —
    // the diff really does show that — and it is why the field is named for
    // lines rather than for operations.
    let before: Vec<&str> = text.lines().collect();
    let after: Vec<&str> = edited.lines().collect();
    let changed = before.iter().zip(&after).filter(|(b, a)| b != a).count()
        + before.len().abs_diff(after.len());
    let doc = serde_json::json!({
        "schema": 1,
        "file": args.file.display().to_string(),
        "operations": ops,
        "unchanged": edited == text,
        "bytes_in": text.len(), "bytes_out": edited.len(),
        "lines_in": before.len(), "lines_out": after.len(),
        "lines_changed": changed,
        "out": target.as_ref().map(|p| p.display().to_string()),
        "written": written,
    });
    emit_value(&doc, ctx)?;
    Ok(0)
}

/// `forge strategy new|validate|explain`.
pub fn strategy_cmd(args: &StrategyArgs, ctx: Ctx) -> Result<i32> {
    match args.action.as_str() {
        "new" => {
            print!("{}", Strategy::template());
            Ok(0)
        }
        "validate" => {
            let Some(f) = &args.file else {
                eprintln!("error: `strategy validate <file>` needs a path");
                return Ok(5);
            };
            match Strategy::load(f) {
                Ok(s) => {
                    note(format!("OK — strategy '{}' is valid", s.name));
                    Ok(0)
                }
                Err(e) => {
                    eprintln!("error: invalid strategy: {e:#}");
                    Ok(5)
                }
            }
        }
        "explain" => {
            let Some(f) = &args.file else {
                eprintln!("error: `strategy explain <file>` needs a path");
                return Ok(5);
            };
            let s = match Strategy::load(f) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: invalid strategy: {e:#}");
                    return Ok(5);
                }
            };
            let doc = serde_json::json!({
                "resolved": serde_json::to_value(&s)?,
                "scaffold": format!("{:?}", s.scaffold()),
                "injector_pool": s.pool().iter().map(std::string::ToString::to_string).collect::<Vec<_>>(),
                "blindspot_rotation": crate::strategy::BLINDSPOT_BACKLOG
                    .iter().map(std::string::ToString::to_string).collect::<Vec<_>>(),
            });
            emit_value(&doc, ctx)?;
            Ok(0)
        }
        other => {
            eprintln!("error: unknown action '{other}' (new|validate|explain)");
            Ok(5)
        }
    }
}

/// `forge confidence show|reset|export`.
pub fn confidence_cmd(args: &ConfidenceArgs, ctx: Ctx) -> Result<i32> {
    let out = forge_dir(args.out_dir.as_deref());
    let path = crate::confidence::Ledger::path(&out);
    match args.action.as_str() {
        "reset" => {
            if path.exists() {
                std::fs::remove_file(&path)?;
                note(format!("ledger cleared → {}", path.display()));
            } else {
                note("no ledger to reset");
            }
            Ok(0)
        }
        "show" | "export" => {
            let Ok(txt) = std::fs::read_to_string(&path) else {
                eprintln!(
                    "no confidence ledger at {} (run `forge run` first)",
                    path.display()
                );
                return Ok(3);
            };
            let l: crate::confidence::Ledger = serde_json::from_str(&txt)?;
            let doc = if args.action == "export" {
                serde_json::from_str(&txt)?
            } else {
                l.summary()
            };
            emit_value(&doc, ctx)?;
            Ok(0)
        }
        other => {
            eprintln!("error: unknown action '{other}' (show|reset|export)");
            Ok(5)
        }
    }
}

/// `forge seed vendor` — clone python-ags4's upstream `tests/` corpus
/// (GitLab canonical, GitHub mirror fallback), pinned + immutable +
/// with provenance. Opt-in; degrades cleanly with no network.
pub fn vendor(args: &VendorArgs, _ctx: Ctx) -> Result<i32> {
    let dest = args.dest.clone();
    let gitlab = "https://gitlab.com/ags-data-format-wg/ags-python-library.git";
    let github = "https://github.com/asitha-sena/python-ags4.git";
    let tmp = std::env::temp_dir().join(format!("pyags4_vendor_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    let clone = |url: &str| {
        std::process::Command::new("git")
            .args(["clone", "--depth", "1", "--branch", &args.r#ref, url])
            .arg(&tmp)
            .status()
            .is_ok_and(|s| s.success())
    };
    let src = if clone(gitlab) {
        gitlab
    } else if clone(github) {
        eprintln!("vendor: GitLab unreachable — used GitHub mirror");
        github
    } else {
        eprintln!(
            "vendor: could not clone python-ags4 @ {} from GitLab or GitHub \
             (offline?). This is opt-in — skipping cleanly.",
            args.r#ref
        );
        return Ok(0);
    };
    std::fs::create_dir_all(&dest)?;
    // python-ags4 keeps its .ags corpus in tests/test_files/ (v1.2.0 — 83
    // files); a couple of strays sit directly in tests/. Prefer test_files,
    // fall back to tests/ for older layouts.
    let src_dir = {
        let tf = tmp.join("tests").join("test_files");
        if tf.is_dir() { tf } else { tmp.join("tests") }
    };
    let mut n = 0u32;
    if let Ok(rd) = std::fs::read_dir(&src_dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) == Some("ags") {
                let _ = std::fs::copy(&p, dest.join(p.file_name().unwrap()));
                n += 1;
            }
        }
    }
    std::fs::write(
        dest.join("PROVENANCE.md"),
        format!(
            "# Vendored python-ags4 test corpus (opt-in forge seed)\n\n\
             - source: {src}\n- ref: {}\n- retrieved: {}\n- files: {n} .ags\n\n\
             LGPL data files (not code); pinned + immutable. NEVER copy into\n\
             laterite-ags4-validator/tests/fixtures/.\n",
            args.r#ref,
            chrono::Utc::now().to_rfc3339()
        ),
    )?;
    let _ = std::fs::remove_dir_all(&tmp);
    note(format!(
        "vendored {n} .ags → {} (ref {})",
        dest.display(),
        args.r#ref
    ));
    Ok(0)
}

/// Emit a raw `serde_json::Value` honouring `--output`/`--compact`
/// (small ad-hoc docs that aren't a `Report` struct).
fn emit_value(v: &serde_json::Value, ctx: Ctx) -> Result<()> {
    use laterite_cliutil::OutputMode;
    let mut o = std::io::stdout().lock();
    match ctx.mode {
        OutputMode::Ndjson => laterite_cliutil::write_ndjson(&mut o, v)?,
        _ => laterite_cliutil::write_json_pretty(&mut o, v, ctx.colour())?,
    }
    Ok(())
}

#[cfg(test)]
mod scale_inject_tests {
    use super::resolve_scale_inject;
    use crate::ops::Injection;

    #[test]
    fn density_without_inject_is_error() {
        assert!(resolve_scale_inject(None, Some(0.5)).is_err());
    }

    #[test]
    fn clean_when_no_inject_or_none_token() {
        assert_eq!(resolve_scale_inject(None, None).unwrap(), None);
        // `none` is an explicit clean scale; any --density is a no-op, not an error.
        assert_eq!(resolve_scale_inject(Some("none"), Some(0.5)).unwrap(), None);
    }

    #[test]
    fn unknown_token_is_error() {
        assert!(resolve_scale_inject(Some("rule999"), None).is_err());
    }

    #[test]
    fn structural_singletons_are_rejected() {
        for tok in ["rule10a", "rule19", "rule13", "rule14", "rule17"] {
            assert!(resolve_scale_inject(Some(tok), None).is_err(), "{tok}");
        }
    }

    #[test]
    fn density_out_of_range_is_error() {
        for d in [0.0, -1.0, 1.5, f64::NAN, f64::INFINITY] {
            assert!(
                resolve_scale_inject(Some("rule10b"), Some(d)).is_err(),
                "{d}"
            );
        }
    }

    #[test]
    fn density_capable_resolves_with_default_one() {
        assert_eq!(
            resolve_scale_inject(Some("rule10b"), Some(0.5)).unwrap(),
            Some((Injection::EmptyRequired, 0.5))
        );
        // omitted --density defaults to 1.0 (every applicable site).
        assert_eq!(
            resolve_scale_inject(Some("rule16"), None).unwrap(),
            Some((Injection::UndefinedAbbrev, 1.0))
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_directory_yields_its_ags_files_and_counts_what_it_walked_past() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        std::fs::create_dir(root.join("nested")).unwrap();
        for f in [
            "b.ags",
            "a.AGS",
            "notes.txt",
            "nested/c.ags",
            "nested/readme.md",
        ] {
            std::fs::write(root.join(f), b"x").unwrap();
        }
        let mut out = Vec::new();
        let mut skipped = 0;
        collect_ags(root, true, &mut out, &mut skipped);
        out.sort();
        let names: Vec<String> = out
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        // Recursive, and the extension match is case-insensitive: a
        // corpus written on Windows is full of `.AGS`.
        assert_eq!(names, vec!["a.AGS", "b.ags", "c.ags"]);
        // The two non-.ags files are REPORTED, not silently dropped — a
        // sweep that says nothing about what it ignored reads as having
        // checked everything.
        assert_eq!(skipped, 2);
    }

    #[test]
    fn a_file_named_directly_is_taken_whatever_its_extension() {
        // `check <path>` has always validated the path it was handed. The
        // extension filter arrived to decide what a DIRECTORY meant, and if
        // it reached a named file it would turn "validate this" into "no
        // .ags file among the given path(s)" for every delivery that does
        // not end in `.ags`. The name is deliberately NOT `.ags`, because a
        // `.ags` fixture here would pass whether or not that held.
        let tmp = tempfile::tempdir().expect("tempdir");
        let p = tmp.path().join("delivery.dat");
        std::fs::write(&p, b"x").unwrap();
        let mut out = Vec::new();
        let mut skipped = 0;
        collect_ags(&p, true, &mut out, &mut skipped);
        assert_eq!(out, vec![p]);
        assert_eq!(skipped, 0);
    }
}
