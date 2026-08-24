//! The evolutionary loop.
//!
//! Fitness = **signature novelty**. A candidate's signature is
//! `(verdict_tag, sorted rust rules, sorted python rules, injector,
//! scaffold)`. A never-seen signature resets the staleness counter and
//! (if it's a real divergence) is frozen as a finding; a seen one
//! advances staleness. `stale_soft` → **auto-permute** (rotate the
//! target along the blind-spot backlog / toggle the scaffold —
//! escaping the dead end *without* a new strategy). `stale_hard` →
//! stop, emit a frontier report, exit 2 (the next strategy is
//! authored by hand). The binary embeds no LLM.
//!
//! Determinism: every choice consumes the strategy-seeded `Rng`, the
//! ledger cold-starts deterministically in a fresh out-dir, and Rust
//! is in-process — so `(seed, strategy)` ⇒ identical signature stream
//! and identical oracle-call set (the oracle itself is version-pinned).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use anyhow::Result;
use laterite_ags4_parity::{Parity, PyOracle, Rng};

use crate::confidence::{Ledger, class_key, validator_fingerprint};
use crate::ops::{Injection, synth_injected};
use crate::pipeline::rust_check;
use crate::report::{Candidate, RunReport};
use crate::strategy::{BLINDSPOT_BACKLOG, Strategy};
use crate::synth::{Scaffold, synth};

/// A process-unique scratch path. Pure I/O scratch for the in-process
/// Rust check / the python wrapper — the filename never enters a
/// signature or the report, so logical determinism is unaffected, but
/// it MUST be unique across threads/calls (cargo runs tests in
/// parallel in one process; a pid-keyed name collided and corrupted a
/// concurrent run's file → nondeterministic validation). No `tempfile`
/// dep in lib code — a monotonic counter + pid is enough.
fn unique_tmp(tag: &str) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("forge_{tag}_{}_{n}.ags", std::process::id()))
}

/// Behavioural validator fingerprint: the verdict class keys on a
/// fixed 3-probe set (clean minimal, clean loca-samp, 10a-injected
/// loca-samp). Changes iff the validator changes behaviour here, so
/// the ledger cold-starts exactly when prior trust is no longer valid.
fn fingerprint() -> String {
    // Fixed seed 0: the fingerprint keys the confidence ledger, so it must
    // be stable across runs — changing only when validator behaviour does.
    let probes = [
        synth(Scaffold::Minimal, 0),
        synth(Scaffold::LocaSamp, 0),
        synth_injected(Scaffold::LocaSamp, 0, Injection::DupSampKeyTuple),
    ];
    let verdicts: Vec<String> = probes
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let tmp = unique_tmp(&format!("fp{i}"));
            let _ = std::fs::write(&tmp, t.as_bytes());
            let k = class_key(&rust_check(&tmp).0);
            let _ = std::fs::remove_file(&tmp);
            k
        })
        .collect();
    validator_fingerprint(&verdicts)
}

fn signature(tag: &str, rust: &[String], py: &[String], inj: Injection, sc: Scaffold) -> String {
    let j = |v: &[String]| {
        let mut x: Vec<&str> = v.iter().map(std::string::String::as_str).collect();
        x.sort_unstable();
        x.join("|")
    };
    format!("{tag}#{}#{}#{inj}#{sc:?}", j(rust), j(py))
}

/// Outcome of an evolve run: the report + the process exit code.
pub struct RunOutcome {
    pub report: RunReport,
    pub exit: i32,
}

/// Run the loop. `oracle = None` ⇒ Rust-only (fully deterministic —
/// the unit-test path). `write_files` writes each candidate + the
/// report/ledger into `<out_dir>/runs/<run_id>/`; the ledger persists
/// at `<out_dir>/confidence.json` across runs.
pub fn evolve(
    strat: &Strategy,
    oracle: Option<&PyOracle>,
    oracle_version: &str,
    out_dir: &Path,
    run_id: &str,
    write_files: bool,
) -> Result<RunOutcome> {
    let scaffold0 = strat.scaffold();
    let pool = strat.pool();
    let fp = fingerprint();
    let mut ledger = Ledger::load_or_cold_start(
        out_dir,
        &fp,
        oracle_version,
        strat.confidence.floor,
        strat.confidence.force_burst,
    );
    let mut rng = Rng::seeded(strat.seed);

    let run_dir = out_dir.join("runs").join(run_id);
    if write_files {
        std::fs::create_dir_all(&run_dir)?;
    }

    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut candidates: Vec<Candidate> = Vec::new();
    let mut findings: Vec<Candidate> = Vec::new();
    let mut permutes: Vec<serde_json::Value> = Vec::new();

    let mut stale: u64 = 0;
    let mut permute_off: usize = 0;
    let mut scaffold = scaffold0;
    let mut escalations: u32 = 0;
    let start = Instant::now();
    let mut iter_n: u64 = 0;
    let mut status = "clean";

    while iter_n < strat.max_generations
        && (candidates.len() as u64) < strat.max_candidates
        && start.elapsed().as_secs() < strat.max_wall_secs
    {
        iter_n += 1;
        // Deterministic candidate choice from the seeded RNG over the
        // pool, biased by the current permute offset (auto-permute
        // rotates which blind-spot rule leads).
        // `below(n)` returns a value < n by construction (`next_u64() % n`),
        // and n here is `pool.len()` widened to u64, so the result narrows
        // back to usize losslessly.
        #[allow(clippy::cast_possible_truncation)]
        let pick = (rng.below(pool.len() as u64) as usize + permute_off) % pool.len();
        let inj = pool[pick];
        // A relational injector needs the LOCA→SAMP base; if a permute
        // toggled us to Minimal, fall back to a non-relational pick.
        let inj = if scaffold == Scaffold::Minimal && inj.needs_relational() {
            Injection::BadDtValue
        } else {
            inj
        };

        // A fresh varied base per generation (reproducible from the
        // strategy seed), so the evolutionary search explores realistic
        // diversity, not one file with a rotating injection.
        let text = synth_injected(scaffold, strat.seed.wrapping_add(iter_n), inj);
        let cand_path = run_dir.join(format!("gen{iter_n:04}_{inj}.ags").replace(':', "_"));

        // Validate Rust in-process (a path is required either way).
        let (rust, dict) = if write_files {
            std::fs::write(&cand_path, text.as_bytes())?;
            rust_check(&cand_path)
        } else {
            let tmp = unique_tmp("g");
            std::fs::write(&tmp, text.as_bytes())?;
            let r = rust_check(&tmp);
            let _ = std::fs::remove_file(&tmp);
            r
        };

        // Confidence-gated oracle decision (also honours the hard
        // python budget). Consuming the seeded RNG here keeps the
        // oracle-call set reproducible.
        let send = strat.confidence.enabled
            && ledger.calls_made < strat.python_budget
            && oracle.is_some()
            && ledger.should_sample(&rust, &mut rng);
        let outcome = if send {
            let tmp = unique_tmp("py");
            let _ = std::fs::write(&tmp, text.as_bytes());
            let py = oracle.unwrap().check(&tmp);
            let _ = std::fs::remove_file(&tmp);
            let verdict = laterite_ags4_parity::classify(&rust, &py);
            crate::pipeline::Outcome {
                rust,
                python: Some(py),
                verdict: Some(verdict),
                dict_used: dict,
            }
        } else {
            crate::pipeline::Outcome {
                rust,
                python: None,
                verdict: None,
                dict_used: dict,
            }
        };

        record(
            &mut seen,
            &mut counts,
            &mut candidates,
            &mut findings,
            &mut ledger,
            &mut stale,
            iter_n,
            inj,
            scaffold,
            &cand_path,
            &outcome,
        );
        maybe_permute(
            strat,
            &mut stale,
            &mut permute_off,
            &mut scaffold,
            scaffold0,
            &mut escalations,
            &mut permutes,
            iter_n,
            &mut status,
        );
        if status == "stalled" {
            break;
        }
    }

    if !findings.is_empty() {
        status = "findings";
    } else if status != "stalled" && iter_n >= strat.max_generations {
        status = "budget_exhausted";
    }

    let report = RunReport {
        schema: 1,
        created: chrono::Utc::now().to_rfc3339(),
        strategy: strat.name.clone(),
        status: status.to_string(),
        generations: iter_n,
        seed: strat.seed,
        counts,
        permutes,
        confidence: ledger.summary(),
        findings,
        candidates,
    };
    let exit = match status {
        "findings" => 1,
        "stalled" => 2,
        _ => 0,
    };
    if write_files {
        std::fs::write(
            run_dir.join("report.json"),
            serde_json::to_string_pretty(&report)?,
        )?;
        if status == "stalled" {
            std::fs::write(
                run_dir.join("frontier.json"),
                serde_json::to_string_pretty(&report.frontier())?,
            )?;
        }
        ledger.save(out_dir)?;
        std::fs::write(
            run_dir.join("strategy.resolved.json"),
            serde_json::to_string_pretty(strat)?,
        )?;
    }
    Ok(RunOutcome { report, exit })
}

#[allow(clippy::too_many_arguments)]
fn record(
    seen: &mut BTreeSet<String>,
    counts: &mut BTreeMap<String, u64>,
    candidates: &mut Vec<Candidate>,
    findings: &mut Vec<Candidate>,
    ledger: &mut Ledger,
    stale: &mut u64,
    iter_n: u64,
    inj: Injection,
    scaffold: Scaffold,
    path: &Path,
    o: &crate::pipeline::Outcome,
) {
    if let Some(v) = &o.verdict {
        ledger.update(&o.rust, v);
    }
    let tag = match &o.verdict {
        Some(p) => p.tag().to_string(),
        None => "RUST_ONLY".to_string(),
    };
    let rr = o.rust_rules();
    let pr = o.python_rules();
    let sig = signature(&tag, &rr, &pr, inj, scaffold);
    // A cosmetic report sequence number, not an index: this dev-only fuzzer
    // is never built for a 32-bit target, so usize == u64 on every target it
    // actually runs on.
    #[allow(clippy::cast_possible_truncation)]
    let seq = iter_n as usize;
    *counts.entry(tag.clone()).or_default() += 1;
    let novel = seen.insert(sig);
    if novel {
        *stale = 0;
    } else {
        *stale += 1;
    }
    let is_action = Parity::is_action_tag(&tag);
    if is_action && novel {
        findings.push(Candidate::from_outcome(
            seq,
            inj.to_string(),
            inj.target_rule().map(String::from),
            path.display().to_string(),
            o,
        ));
    }
    candidates.push(Candidate::from_outcome(
        seq,
        inj.to_string(),
        inj.target_rule().map(String::from),
        path.display().to_string(),
        o,
    ));
}

#[allow(clippy::too_many_arguments)]
fn maybe_permute(
    strat: &Strategy,
    stale: &mut u64,
    permute_off: &mut usize,
    scaffold: &mut Scaffold,
    scaffold0: Scaffold,
    escalations: &mut u32,
    permutes: &mut Vec<serde_json::Value>,
    iter_n: u64,
    status: &mut &'static str,
) {
    if *stale < strat.stale_soft {
        return;
    }
    if *escalations >= 3 || *stale >= strat.stale_hard {
        *status = "stalled";
        permutes.push(serde_json::json!({
            "generation": iter_n, "reason": "stale_hard",
            "action": "emit frontier — the next strategy must be authored by hand"
        }));
        return;
    }
    // Auto-permute: advance the blind-spot target and, on alternate
    // escalations, toggle the scaffold to widen the signature space.
    *permute_off = (*permute_off + 1) % BLINDSPOT_BACKLOG.len().max(1);
    if *escalations % 2 == 1 {
        *scaffold = match *scaffold {
            Scaffold::Minimal => Scaffold::LocaSamp,
            // Wide is a scale scaffold, not an evolve target — fold it to
            // the relational base for the toggle.
            Scaffold::LocaSamp | Scaffold::Wide => Scaffold::Minimal,
        };
    } else {
        let _ = scaffold0;
    }
    *escalations += 1;
    *stale = 0;
    permutes.push(serde_json::json!({
        "generation": iter_n, "reason": "stale_soft",
        "action": format!("permute #{escalations}: rotate target (+{permute_off}) scaffold={scaffold:?}")
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny_strategy(seed: u64) -> Strategy {
        Strategy {
            name: "test".into(),
            // A small, *pinned* injector pool so the discrete signature
            // space saturates within the generation budget — independent
            // of how many injectors BLINDSPOT_BACKLOG later grows to.
            injectors: ["rule10a", "rule10c", "rule8", "rule19", "rule13"]
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
            max_generations: 40,
            max_candidates: 40,
            max_wall_secs: 60,
            python_budget: 0, // Rust-only → fully deterministic
            stale_soft: 5,
            stale_hard: 12,
            seed,
            ..Strategy::default()
        }
    }

    #[test]
    fn loop_is_deterministic_under_a_fixed_seed() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        let r1 = evolve(&tiny_strategy(7), None, "none", a.path(), "R", false).unwrap();
        let r2 = evolve(&tiny_strategy(7), None, "none", b.path(), "R", false).unwrap();
        assert_eq!(r1.report.generations, r2.report.generations);
        assert_eq!(r1.report.counts, r2.report.counts);
        assert_eq!(r1.report.status, r2.report.status);
        let sigs = |r: &RunOutcome| {
            r.report
                .candidates
                .iter()
                .map(|c| (c.injection.clone(), c.verdict.clone()))
                .collect::<Vec<_>>()
        };
        assert_eq!(sigs(&r1), sigs(&r2), "same seed ⇒ identical stream");
    }

    #[test]
    fn saturated_space_goes_stale_then_permutes_then_frontier() {
        let t = tempfile::tempdir().unwrap();
        // Rust-only, small discrete space ⇒ signatures saturate ⇒
        // staleness climbs ⇒ permute(s) ⇒ stale_hard → status=stalled.
        let out = evolve(&tiny_strategy(1), None, "none", t.path(), "R", false).unwrap();
        assert_eq!(out.report.status, "stalled");
        assert_eq!(out.exit, 2);
        assert!(
            !out.report.permutes.is_empty(),
            "must auto-permute before giving up"
        );
        assert!(
            out.report
                .permutes
                .iter()
                .any(|p| p["reason"] == "stale_soft"),
            "expected a stale_soft auto-permute"
        );
        assert!(
            out.report
                .permutes
                .iter()
                .any(|p| p["reason"] == "stale_hard"),
            "expected the stale_hard frontier signal"
        );
    }
}
