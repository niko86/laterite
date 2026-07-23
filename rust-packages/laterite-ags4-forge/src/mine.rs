//! The corpus-gap divergence miner.
//!
//! The forge has two search axes. The **rule** axis is small and
//! enumerable — there are only so many injectable AGS rules, so we can
//! *systematically* synthesize their combinations rather than stumble onto
//! them. The **placement** axis is large and non-enumerable (where a fault
//! lands can flip the validator's behaviour via cascades), so we *sample*
//! it across seeds. This miner walks the rule axis exhaustively (all
//! k-combinations of the injectors) and samples the placement axis (N
//! seeds per combination), then subtracts what the python-ags4 fixture
//! corpus already covers — the leftover is *new* defect shapes nobody has
//! a regression test for.
//!
//! The dominant cost is the python-ags4 oracle, so it's spent adaptively:
//! every candidate is Rust-validated (free, in-process), and python runs
//! only on the **distinct gap signatures** that look **divergence-prone**
//! (the signature contains a rule where Rust and python are known to
//! differ) — one call per signature. `--always-validate` lifts the
//! divergence-prone filter and validates every gap. A combination's true
//! rule-set is read from the validator, never assumed from the injectors'
//! targets (the honesty principle behind [`crate::ops::synth_combined`]).

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use laterite_ags4_parity::{PyOracle, RustResult};

use crate::ops::{Injection, synth_combined};
use crate::pipeline::{dual_validate, rust_check};
use crate::report::{MineCandidate, MineReport, verdict_parts};
use crate::synth::Scaffold;

/// The injectors the miner combines — every real, single-rule operator
/// (`Injection::ALL`, the clean baseline `None` excluded). Single-sourced
/// so a new injector automatically joins the mine space. All are
/// applicable on the `loca-samp` scaffold, which `mine` defaults to (the
/// relational ones are no-ops on `minimal`).
const MINEABLE: &[Injection] = Injection::ALL;

/// Rules where Rust and python-ags4 are *empirically confirmed* to diverge
/// or to need reconciliation — the Rust-only signal that a synthesized
/// combination is worth an oracle call. Derived from two authorities:
///   - the `laterite_ags4_parity::classify` reconcile arms — Rules 4/5/6 (O-2/O-3)
///     and 19b (O-26);
///   - OBSERVATIONS.md *behavioural* divergences (`[BUG]`/`[VARIANCE]`,
///     not spec-interpretation notes) — Rule 7 (O-8 python `IndexError` on
///     duplicate headings) and Rule 8 (O-31/O-33/O-38 python DT quirks).
///
/// Kept deliberately narrow: a too-wide set would flag almost every
/// signature and spend the oracle on everything, defeating the point.
const DIVERGENCE_PRONE_RULES: &[&str] = &[
    "AGS Format Rule 4",
    "AGS Format Rule 5",
    "AGS Format Rule 6",
    "AGS Format Rule 7",
    "AGS Format Rule 8",
    "AGS Format Rule 19b",
];

/// Mining parameters (built from `MineArgs` in `cmd`).
pub struct MineCfg {
    pub scaffold: Scaffold,
    /// Directory of `.ags` files whose Rust signatures define "covered".
    pub corpus: PathBuf,
    pub min_k: usize,
    pub max_k: usize,
    /// Placement seeds tried per combination.
    pub seeds: u64,
    pub base_seed: u64,
    /// Validate every gap, not just the divergence-prone signatures.
    pub always_validate: bool,
    /// Hard cap on python-ags4 calls.
    pub max_oracle: usize,
}

/// A Rust validation result reduced to its canonical *signature* — the
/// sorted rule labels (`BTreeSet` iteration is already sorted), or a marker
/// for clean / hard-error / panic. The miner's unit of novelty.
fn signature(r: &RustResult) -> Vec<String> {
    match r {
        RustResult::Clean => Vec::new(),
        RustResult::Rules(s) => s.iter().cloned().collect(),
        RustResult::HardError(e) => vec![format!("HARD_ERROR:{e}")],
        RustResult::Panic => vec!["PANIC".to_string()],
    }
}

fn is_divergence_prone(sig: &[String]) -> bool {
    sig.iter()
        .any(|r| DIVERGENCE_PRONE_RULES.contains(&r.as_str()))
}

/// All k-combinations of `0..n` as ascending index vectors (lexicographic,
/// deterministic). Empty when `k == 0` or `k > n`.
fn k_combinations(n: usize, k: usize) -> Vec<Vec<usize>> {
    fn go(start: usize, n: usize, k: usize, cur: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if cur.len() == k {
            out.push(cur.clone());
            return;
        }
        for i in start..n {
            cur.push(i);
            go(i + 1, n, k, cur, out);
            cur.pop();
        }
    }
    let mut out = Vec::new();
    if k > 0 && k <= n {
        go(0, n, k, &mut Vec::new(), &mut out);
    }
    out
}

/// Rust-profile every `.ags` in `dir` → the set of distinct signatures it
/// already covers (and the file count). A missing/empty dir → an empty
/// covered-set, so every synthesized signature reads as a gap (the miner
/// still runs — it just has nothing to subtract).
fn profile_corpus(dir: &Path) -> (usize, HashSet<Vec<String>>) {
    let mut sigs = HashSet::new();
    let Ok(rd) = std::fs::read_dir(dir) else {
        return (0, sigs);
    };
    let mut paths: Vec<PathBuf> = rd
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("ags"))
        .collect();
    paths.sort();
    for p in &paths {
        let (rust, _) = rust_check(p);
        sigs.insert(signature(&rust));
    }
    (paths.len(), sigs)
}

/// The miner. Profiles the corpus, synthesizes every k-combination across
/// the seed sweep (writing each `.ags` into `dir`), Rust-validates all,
/// then spends the oracle adaptively on the novel divergence-prone shapes.
pub fn run_mine(
    cfg: &MineCfg,
    oracle: Option<&PyOracle>,
    dir: &Path,
    created: String,
) -> anyhow::Result<MineReport> {
    let (corpus_files, corpus_sigs) = profile_corpus(&cfg.corpus);

    // The rule axis, walked exhaustively.
    let mut combos: Vec<Vec<Injection>> = Vec::new();
    for k in cfg.min_k..=cfg.max_k {
        for idx in k_combinations(MINEABLE.len(), k) {
            combos.push(idx.iter().map(|&i| MINEABLE[i]).collect());
        }
    }

    // Synthesize + Rust-validate every (combination × placement seed).
    let mut candidates: Vec<MineCandidate> = Vec::new();
    let mut distinct: HashSet<Vec<String>> = HashSet::new();
    for combo in &combos {
        let label = combo
            .iter()
            .copied()
            .map(super::ops::Injection::token)
            .collect::<Vec<_>>()
            .join("+");
        for s in 0..cfg.seeds {
            let seed = cfg.base_seed.wrapping_add(s);
            let text = synth_combined(cfg.scaffold, seed, combo);
            let fname = format!("mine_{label}_s{seed}.ags").replace('+', "_");
            let path = dir.join(&fname);
            std::fs::write(&path, text.as_bytes())?;
            let (rust, _dict) = rust_check(&path);
            let sig = signature(&rust);
            distinct.insert(sig.clone());
            let is_gap = !corpus_sigs.contains(&sig);
            let divergence_prone = is_divergence_prone(&sig);
            candidates.push(MineCandidate {
                combo: label.clone(),
                seed,
                signature: sig,
                is_gap,
                divergence_prone,
                oracle_ran: false,
                verdict: "RUST_ONLY".into(),
                detail: String::new(),
                python_rules: Vec::new(),
                path: path.display().to_string(),
            });
        }
    }

    // Adaptive oracle spend: one python call per DISTINCT gap signature
    // (the first candidate carrying it represents it), gated to the
    // divergence-prone shapes unless `--always-validate`, capped by
    // `max_oracle`.
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut oracle_calls = 0usize;
    if let Some(o) = oracle {
        let mut spent: HashSet<Vec<String>> = HashSet::new();
        for c in &mut candidates {
            if oracle_calls >= cfg.max_oracle {
                break;
            }
            let consider = c.is_gap && (cfg.always_validate || c.divergence_prone);
            if !consider || spent.contains(&c.signature) {
                continue;
            }
            spent.insert(c.signature.clone());
            let outcome = dual_validate(&PathBuf::from(&c.path), Some(o));
            let (verdict, detail) = verdict_parts(outcome.verdict.as_ref());
            *counts.entry(verdict.clone()).or_default() += 1;
            oracle_calls += 1;
            c.oracle_ran = true;
            c.verdict = verdict;
            c.detail = detail;
            c.python_rules = outcome.python_rules();
        }
    }

    // Distinct gap / divergence-prone-gap signature counts (the headline
    // numbers — per-candidate rows would double-count placements).
    let gap_sigs: HashSet<&Vec<String>> = candidates
        .iter()
        .filter(|c| c.is_gap)
        .map(|c| &c.signature)
        .collect();
    let dp_gap_sigs: HashSet<&Vec<String>> = candidates
        .iter()
        .filter(|c| c.is_gap && c.divergence_prone)
        .map(|c| &c.signature)
        .collect();

    Ok(MineReport {
        schema: 1,
        created,
        scaffold: format!("{:?}", cfg.scaffold),
        corpus: cfg.corpus.display().to_string(),
        corpus_files,
        corpus_signatures: corpus_sigs.len(),
        combinations_tried: combos.len(),
        candidates_synthesized: candidates.len(),
        distinct_signatures: distinct.len(),
        gaps: gap_sigs.len(),
        divergence_prone_gaps: dp_gap_sigs.len(),
        oracle: oracle.is_some(),
        oracle_calls,
        counts,
        candidates,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn k_combinations_counts_are_binomial() {
        assert_eq!(k_combinations(6, 2).len(), 15);
        assert_eq!(k_combinations(6, 3).len(), 20);
        assert_eq!(k_combinations(6, 1).len(), 6);
        assert_eq!(k_combinations(6, 0).len(), 0);
        assert_eq!(k_combinations(6, 7).len(), 0, "k>n is empty");
        // ascending + lexicographic + no repeats
        let cs = k_combinations(4, 2);
        assert_eq!(cs[0], vec![0, 1]);
        assert_eq!(cs.last().unwrap(), &vec![2, 3]);
        assert!(cs.iter().all(|c| c.windows(2).all(|w| w[0] < w[1])));
    }

    #[test]
    fn divergence_prone_matches_the_known_set() {
        assert!(is_divergence_prone(&["AGS Format Rule 5".into()]));
        assert!(is_divergence_prone(&[
            "AGS Format Rule 10a".into(),
            "AGS Format Rule 8".into(),
        ]));
        // a signature of only non-listed rules is not flagged
        assert!(!is_divergence_prone(&[
            "AGS Format Rule 10a".into(),
            "AGS Format Rule 13".into(),
        ]));
        assert!(!is_divergence_prone(&[]));
    }

    /// With no corpus, every distinct synthesized signature is a gap; the
    /// rule axis is walked exhaustively (C(6,2)=15 combos at k=2), no
    /// oracle is spent, and the run reports no action (Rust-only).
    #[test]
    fn empty_corpus_makes_everything_a_gap_rust_only() {
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("out");
        std::fs::create_dir_all(&out).unwrap();
        let cfg = MineCfg {
            scaffold: Scaffold::LocaSamp,
            corpus: tmp.path().join("does-not-exist"),
            min_k: 2,
            max_k: 2,
            seeds: 1,
            base_seed: 0,
            always_validate: false,
            max_oracle: 0,
        };
        let r = run_mine(&cfg, None, &out, "test".into()).unwrap();
        // k=2 over the full injector set → C(n,2) combinations, one
        // candidate each (seeds=1).
        let n = MINEABLE.len();
        let expected = n * (n - 1) / 2;
        assert_eq!(r.corpus_files, 0);
        assert_eq!(r.combinations_tried, expected);
        assert_eq!(r.candidates_synthesized, expected);
        assert_eq!(r.gaps, r.distinct_signatures, "no corpus → all gaps");
        assert_eq!(r.oracle_calls, 0);
        assert!(!r.actions_present());
        assert!(
            r.divergence_prone_gaps > 0,
            "some combos must produce divergence-prone signatures"
        );
    }

    /// A signature the corpus already covers is subtracted from the gaps.
    /// The corpus file is synthesized via the exact combination the miner
    /// builds for indices {0,2} at seed 0, so the signatures match by
    /// construction (same bytes → same Rust verdict → same signature).
    #[test]
    fn corpus_signature_is_subtracted() {
        let tmp = tempfile::tempdir().unwrap();
        let corpus = tmp.path().join("corpus");
        std::fs::create_dir_all(&corpus).unwrap();
        let combo = vec![MINEABLE[0], MINEABLE[2]]; // rule10a+rule8
        let text = synth_combined(Scaffold::LocaSamp, 0, &combo);
        std::fs::write(corpus.join("seed.ags"), text).unwrap();

        let out = tmp.path().join("out");
        std::fs::create_dir_all(&out).unwrap();
        let cfg = MineCfg {
            scaffold: Scaffold::LocaSamp,
            corpus,
            min_k: 2,
            max_k: 2,
            seeds: 1,
            base_seed: 0,
            always_validate: false,
            max_oracle: 0,
        };
        let r = run_mine(&cfg, None, &out, "test".into()).unwrap();
        assert_eq!(r.corpus_files, 1);
        let c = r
            .candidates
            .iter()
            .find(|c| c.combo == "rule10a+rule8" && c.seed == 0)
            .expect("the rule10a+rule8 @ seed0 candidate exists");
        assert!(
            !c.is_gap,
            "the corpus-covered signature must not be a gap, got rust={:?}",
            c.signature
        );
    }
}
