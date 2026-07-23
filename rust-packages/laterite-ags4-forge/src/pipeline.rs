//! The dual-validate pipeline for one candidate.
//!
//! Rust runs **in-process** on 100% of candidates via the validator
//! library (fast, no subprocess), reduced through the *shared*
//! `laterite_ags4_parity::RustResult` so forge's verdict is definitionally
//! identical to the corpus-qa parity model. python-ags4 runs via the
//! shared `PyOracle` bridge. `dual_validate` consults the oracle whenever
//! it's given one; *whether* to spend an oracle call is decided upstream in
//! the `run` loop by the adaptive Beta–Bernoulli confidence ledger
//! ([`crate::confidence`]). A missing oracle degrades to a Rust-only verdict
//! (optional QA), never an error.

use std::collections::BTreeSet;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::time::Duration;

use laterite_ags4_parity::{Parity, PyOracle, RustResult, classify};

/// One candidate's dual-validation outcome.
#[derive(Debug, Clone)]
pub struct Outcome {
    pub rust: RustResult,
    /// `Ok(rules)` / `Err(reason)` from python, or `None` when the
    /// oracle was unavailable / not consulted (Rust-only run).
    pub python: Option<Result<BTreeSet<String>, String>>,
    /// `classify(rust, python)` when python ran; `None` for Rust-only.
    pub verdict: Option<Parity>,
    /// Bundled edition the file was judged against (`"4.2"`, …) +
    /// how it was resolved, for the report.
    pub dict_used: String,
}

impl Outcome {
    pub fn rust_rules(&self) -> Vec<String> {
        match &self.rust {
            RustResult::Rules(s) => s.iter().cloned().collect(),
            _ => Vec::new(),
        }
    }
    pub fn python_rules(&self) -> Vec<String> {
        match &self.python {
            Some(Ok(s)) => s.iter().cloned().collect(),
            _ => Vec::new(),
        }
    }
}

/// Run the Rust validator in-process. `include_fyi`/`include_warnings`
/// ON so Rust is tier-comparable to python (Rule 1 is FYI in Rust);
/// `check_files` ON so Rule-20 on-disk matches python's always-on stat
/// (O-27). A validator *panic* becomes `RustResult::Panic` (a found
/// panic is a top finding, never a loop crash).
pub fn rust_check(path: &Path) -> (RustResult, String) {
    let opts = laterite_ags4_validator::CheckOptions {
        include_fyi: true,
        include_warnings: true,
        check_files: true,
        ..Default::default()
    };
    let p = path.to_path_buf();
    let res = catch_unwind(AssertUnwindSafe(|| {
        laterite_ags4_validator::check_file_with_dict(&p, &opts)
    }));
    match res {
        Ok(Ok((found, dict, _res))) => (RustResult::from_findings(&found), format!("{dict:?}")),
        Ok(Err(e)) => (RustResult::from_validator_error(&e), "-".to_string()),
        Err(_) => (RustResult::Panic, "-".to_string()),
    }
}

/// The python-ags4 bridge for forge: the repo-root-anchored
/// `tools/py_ags4_check_json.py`, built once and reused (it spawns its
/// own subprocess per `check`). `Some((oracle, version))` on success
/// (the version keys the confidence ledger — an oracle bump cold-starts
/// trust); `None` ⇒ unavailable → callers degrade to Rust-only.
pub fn build_oracle(timeout_secs: u64) -> Option<(PyOracle, String)> {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let wrapper = repo.join("tools/py_ags4_check_json.py");
    let oracle = PyOracle::new("uv", wrapper, repo, Duration::from_secs(timeout_secs));
    // One self-check: if uv/python-ags4 isn't importable, treat the
    // oracle as absent (optional QA — same policy as corpus-qa).
    match oracle.selfcheck() {
        Ok(sc) => Some((oracle, sc.python_ags4.unwrap_or_else(|| "unknown".into()))),
        Err(_) => None,
    }
}

/// Dual-validate `path`. When `oracle` is `Some`, python runs and the
/// shared `classify` (with the O-2/O-3/O-26/O-30/O-34 reconcile arms)
/// produces the verdict; otherwise it's a Rust-only outcome.
pub fn dual_validate(path: &Path, oracle: Option<&PyOracle>) -> Outcome {
    let (rust, dict_used) = rust_check(path);
    match oracle {
        Some(o) => {
            let py = o.check(path);
            let verdict = classify(&rust, &py);
            Outcome {
                rust,
                python: Some(py),
                verdict: Some(verdict),
                dict_used,
            }
        }
        None => Outcome {
            rust,
            python: None,
            verdict: None,
            dict_used,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::{Injection, synth_combined, synth_injected};
    use crate::synth::{Scaffold, synth};

    fn rust_on(text: &str) -> RustResult {
        let tmp = tempfile::tempdir().expect("tempdir");
        let p = tmp.path().join("c.ags");
        std::fs::write(&p, text.as_bytes()).unwrap();
        rust_check(&p).0
    }

    /// THE SAFETY NET: the un-injected synthetic base must be
    /// `RustResult::Clean` for **every seed** — the generator is varied,
    /// so this asserts variety never breaks clean-by-construction. A
    /// generator bug fails *here*, never as a reported "finding".
    #[test]
    fn varied_baseline_is_rust_clean_across_seeds() {
        for seed in 0..25 {
            for sc in [Scaffold::Minimal, Scaffold::LocaSamp, Scaffold::Wide] {
                let r = rust_on(&synth(sc, seed));
                assert!(
                    matches!(r, RustResult::Clean),
                    "{sc:?} seed {seed} not clean in Rust: {r:?}"
                );
            }
        }
    }

    /// A single-rule injection yields exactly the targeted rule (the
    /// defect is single + isolable — the honesty principle).
    #[test]
    fn rule10a_injection_trips_exactly_rule_10a() {
        let r = rust_on(&synth_injected(
            Scaffold::LocaSamp,
            7,
            Injection::DupSampKeyTuple,
        ));
        match r {
            RustResult::Rules(s) => {
                assert!(
                    s.contains("AGS Format Rule 10a"),
                    "expected Rule 10a, got {s:?}"
                );
            }
            other => panic!("expected Rules with 10a, got {other:?}"),
        }
    }

    /// A *combination* of two non-masking injectors trips BOTH target
    /// rules in the real validator — the honesty check behind `--combine`:
    /// the file's actual rule-set really does carry every fault we asked
    /// for (Rule 8 bad date + Rule 19 five-letter group), so a synthesized
    /// combination's signature is trustworthy. Across seeds, so it holds
    /// for the varied base, not one lucky placement.
    #[test]
    fn combination_trips_both_target_rules() {
        for seed in [2, 8, 16] {
            let text = synth_combined(
                Scaffold::LocaSamp,
                seed,
                &[Injection::BadDtValue, Injection::FiveLetterGroup],
            );
            match rust_on(&text) {
                RustResult::Rules(s) => {
                    assert!(
                        s.contains("AGS Format Rule 8") && s.contains("AGS Format Rule 19"),
                        "seed {seed}: combination must trip both Rule 8 and Rule 19, got {s:?}"
                    );
                }
                other => panic!("seed {seed}: expected Rules, got {other:?}"),
            }
        }
    }

    /// The regression guard: each model-mutation injector still trips its
    /// declared `target_rule` in the real validator — across seeds, so it
    /// holds for the varied base, not one lucky file. The compliance/
    /// parity matrix relies on this injector→rule mapping.
    #[test]
    fn every_injector_trips_its_target_rule() {
        for seed in [3, 11, 19] {
            for &inj in Injection::ALL {
                let target = inj.target_rule().expect("injector has a target rule");
                let r = rust_on(&synth_injected(Scaffold::LocaSamp, seed, inj));
                match r {
                    RustResult::Rules(s) => assert!(
                        s.contains(target),
                        "seed {seed} {inj}: expected {target}, got {s:?}"
                    ),
                    other => panic!("seed {seed} {inj}: expected Rules({target}), got {other:?}"),
                }
            }
        }
    }
}
