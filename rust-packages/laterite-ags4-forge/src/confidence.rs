//! The parity-confidence ledger — adaptive oracle gating.
//!
//! Rust validates ~10³–10⁴× faster than the python oracle. Rather than
//! a fixed sample, the loop *learns* which Rust-outcome classes python
//! has been shown to agree on and decays oracle sampling toward a
//! never-zero `floor` for them, keeping a residual spot-check so a
//! regression still surfaces. The accumulated trust is the headline
//! deliverable: a measurable, conservative **P(Rust≡python) lower
//! bound** per class.
//!
//! - **Class key** = the Rust-side outcome (free for 100% of
//!   candidates): `(RustResult kind, sorted rust rule-set)`.
//! - **Trust** = agreements vs actions; the confidence used to gate is
//!   the **Wilson score lower bound** of P(agree) — counts-only, the
//!   standard conservative binomial bound (no stats crate). One action
//!   collapses it; a class needs many clean samples before it rises.
//! - **Safety**: the ledger persists across runs keyed by
//!   `(validator_fingerprint, oracle_version)`. A validator change
//!   (dogfooding *will* change it) or oracle bump **cold-starts** that
//!   tuple — prior agreement evidence is valid only for the exact
//!   build it was earned on. Plus a never-zero floor, always-send
//!   overrides, and a forced burst right after a collapse.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use laterite_ags4_parity::{Parity, Rng, RustResult};
use serde::{Deserialize, Serialize};

/// The stable class key for a Rust outcome.
pub fn class_key(r: &RustResult) -> String {
    match r {
        RustResult::Clean => "clean".to_string(),
        RustResult::Rules(s) => {
            let mut v: Vec<&str> = s.iter().map(std::string::String::as_str).collect();
            v.sort_unstable();
            format!("rules:{}", v.join("|"))
        }
        RustResult::HardError(x) => format!("hard:{x}"),
        RustResult::Panic => "panic".to_string(),
    }
}

/// A tiny deterministic FNV-1a over a string (no extra dep) — used for
/// the *behavioural* validator fingerprint.
fn fnv1a(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// A behavioural fingerprint of *this* validator build: the hash of
/// its verdicts on a fixed probe set. If the validator changes how it
/// classifies these, the fingerprint changes → the ledger cold-starts
/// (trust is only valid for the build it was earned on). If it changes
/// but behaves identically here, trust legitimately carries over.
pub fn validator_fingerprint(probe_verdicts: &[String]) -> String {
    format!("{:016x}", fnv1a(&probe_verdicts.join("\u{1f}")))
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct Entry {
    /// agreements (`Agree` | reconciled `KnownDivergence`).
    agree: u64,
    /// actions (any `is_action` after reconcile).
    action: u64,
    /// remaining forced sends after a trust collapse.
    burst: u32,
}

impl Entry {
    fn total(&self) -> u64 {
        self.agree + self.action
    }
    /// Wilson score lower bound of P(agree) at 95% (z=1.96). 0 when no
    /// samples (caller treats unseen as always-send anyway).
    fn lcb(&self) -> f64 {
        let n = self.total() as f64;
        if n == 0.0 {
            return 0.0;
        }
        let z = 1.96_f64;
        let phat = self.agree as f64 / n;
        let z2 = z * z;
        let denom = 1.0 + z2 / n;
        let centre = phat + z2 / (2.0 * n);
        let margin = z * ((phat * (1.0 - phat) / n) + z2 / (4.0 * n * n)).sqrt();
        ((centre - margin) / denom).clamp(0.0, 1.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ledger {
    pub validator_fingerprint: String,
    pub oracle_version: String,
    pub floor: f64,
    pub force_burst: u32,
    classes: BTreeMap<String, Entry>,
    /// Pure book-keeping for the report — oracle calls the gate saved.
    #[serde(default)]
    pub calls_saved: u64,
    #[serde(default)]
    pub calls_made: u64,
    #[serde(default)]
    pub cold_started: bool,
}

impl Ledger {
    /// Load `<dir>/confidence.json` if its `(fingerprint, oracle)`
    /// tuple matches; otherwise **cold-start** (fresh, flagged) — the
    /// core safety property.
    pub fn load_or_cold_start(
        dir: &Path,
        fingerprint: &str,
        oracle_version: &str,
        floor: f64,
        force_burst: u32,
    ) -> Self {
        let path = Self::path(dir);
        if let Ok(txt) = std::fs::read_to_string(&path) {
            if let Ok(mut l) = serde_json::from_str::<Ledger>(&txt) {
                if l.validator_fingerprint == fingerprint && l.oracle_version == oracle_version {
                    l.cold_started = false;
                    l.calls_saved = 0;
                    l.calls_made = 0;
                    l.floor = floor;
                    l.force_burst = force_burst;
                    return l;
                }
            }
        }
        Ledger {
            validator_fingerprint: fingerprint.to_string(),
            oracle_version: oracle_version.to_string(),
            floor,
            force_burst,
            classes: BTreeMap::new(),
            calls_saved: 0,
            calls_made: 0,
            cold_started: true,
        }
    }

    pub fn path(dir: &Path) -> PathBuf {
        dir.join("confidence.json")
    }

    pub fn save(&self, dir: &Path) -> std::io::Result<()> {
        std::fs::write(Self::path(dir), serde_json::to_string_pretty(self)?)
    }

    /// Should this candidate go to the oracle? Always-send overrides:
    /// hard error / panic / unseen class / a pending forced burst.
    /// Otherwise sample with `p = max(floor, 1 − lcb)` (confident ⇒
    /// rare; uncertain ⇒ often). Consumes the seeded RNG so the
    /// oracle-call set is reproducible under a fixed seed.
    pub fn should_sample(&mut self, rust: &RustResult, rng: &mut Rng) -> bool {
        let always = matches!(rust, RustResult::HardError(_) | RustResult::Panic);
        let key = class_key(rust);
        let e = self.classes.entry(key).or_default();
        if always || e.total() == 0 {
            self.calls_made += 1;
            return true;
        }
        if e.burst > 0 {
            e.burst -= 1;
            self.calls_made += 1;
            return true;
        }
        let p = self.floor.max(1.0 - e.lcb());
        // Deterministic Bernoulli(p) from the seeded RNG.
        let draw = (rng.below(1_000_000) as f64) / 1_000_000.0;
        if draw < p {
            self.calls_made += 1;
            true
        } else {
            self.calls_saved += 1;
            false
        }
    }

    /// Fold a dual-validated verdict into the class's trust.
    pub fn update(&mut self, rust: &RustResult, verdict: &Parity) {
        let e = self.classes.entry(class_key(rust)).or_default();
        match verdict {
            Parity::Agree | Parity::KnownDivergence { .. } => e.agree += 1,
            Parity::RustOnlyRules { .. }
            | Parity::PythonOnlyRules { .. }
            | Parity::RulesDiffer { .. }
            | Parity::ValidityDisagree { .. } => {
                e.action += 1;
                e.burst = self.force_burst; // force-recheck after a collapse
            }
            Parity::PythonError { .. } => {} // infra, not evidence
        }
    }

    /// Per-class + global P(Rust≡python) lower bound for the report.
    pub fn summary(&self) -> serde_json::Value {
        let mut classes = serde_json::Map::new();
        let (mut a, mut t) = (0u64, 0u64);
        for (k, e) in &self.classes {
            a += e.agree;
            t += e.total();
            classes.insert(
                k.clone(),
                serde_json::json!({
                    "samples": e.total(), "agree": e.agree, "action": e.action,
                    "p_equiv_lcb95": (e.lcb() * 10000.0).round() / 10000.0,
                }),
            );
        }
        let global = if t == 0 {
            0.0
        } else {
            let g = Entry {
                agree: a,
                action: t - a,
                burst: 0,
            };
            (g.lcb() * 10000.0).round() / 10000.0
        };
        serde_json::json!({
            "validator_fingerprint": self.validator_fingerprint,
            "oracle_version": self.oracle_version,
            "cold_started": self.cold_started,
            "floor": self.floor,
            "python_calls_made": self.calls_made,
            "python_calls_saved": self.calls_saved,
            "global_p_equiv_lcb95": global,
            "classes": classes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn rules(xs: &[&str]) -> RustResult {
        RustResult::Rules(
            xs.iter()
                .map(std::string::ToString::to_string)
                .collect::<BTreeSet<_>>(),
        )
    }

    #[test]
    fn unseen_class_is_always_sampled_then_decays_when_confident() {
        let dir = tempfile::tempdir().unwrap();
        let mut l = Ledger::load_or_cold_start(dir.path(), "fp", "1.2.0", 0.01, 25);
        let mut rng = Rng::seeded(1);
        let r = rules(&["AGS Format Rule 10a"]);
        // First encounter → always sampled.
        assert!(l.should_sample(&r, &mut rng));
        // Feed many agreements → lcb rises → sample prob decays toward
        // the floor; over a long stretch the gate must skip most.
        for _ in 0..400 {
            l.update(&r, &Parity::Agree);
        }
        let mut sent = 0;
        for _ in 0..1000 {
            if l.should_sample(&r, &mut rng) {
                sent += 1;
            }
        }
        assert!(
            sent < 120,
            "confident class should skip most: sent={sent}/1000"
        );
        assert!(sent > 0, "but the floor must keep spot-checking");
    }

    #[test]
    fn an_action_collapses_trust_and_forces_a_burst() {
        let dir = tempfile::tempdir().unwrap();
        let mut l = Ledger::load_or_cold_start(dir.path(), "fp", "1.2.0", 0.01, 25);
        let mut rng = Rng::seeded(2);
        let r = rules(&["AGS Format Rule 8"]);
        for _ in 0..200 {
            l.update(&r, &Parity::Agree);
        }
        // A real divergence appears.
        l.update(
            &r,
            &Parity::RustOnlyRules {
                rules: vec!["AGS Format Rule 8".into()],
            },
        );
        // The next force_burst gate calls are all forced sends.
        for _ in 0..25 {
            assert!(
                l.should_sample(&r, &mut rng),
                "post-collapse burst must send"
            );
        }
    }

    #[test]
    fn changed_fingerprint_cold_starts_the_ledger() {
        let dir = tempfile::tempdir().unwrap();
        let mut l = Ledger::load_or_cold_start(dir.path(), "fpA", "1.2.0", 0.01, 25);
        let r = rules(&["AGS Format Rule 10a"]);
        for _ in 0..50 {
            l.update(&r, &Parity::Agree);
        }
        l.save(dir.path()).unwrap();
        // Same oracle, SAME fingerprint → trust carries over.
        let same = Ledger::load_or_cold_start(dir.path(), "fpA", "1.2.0", 0.01, 25);
        assert!(!same.cold_started);
        // Validator changed (fingerprint differs) → cold-start.
        let cold = Ledger::load_or_cold_start(dir.path(), "fpB", "1.2.0", 0.01, 25);
        assert!(cold.cold_started, "a changed validator must discard trust");
        // Oracle bumped → also cold-start.
        let cold2 = Ledger::load_or_cold_start(dir.path(), "fpA", "1.3.0", 0.01, 25);
        assert!(cold2.cold_started);
    }
}
