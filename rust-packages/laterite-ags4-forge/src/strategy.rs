//! The declarative strategy — the Claude↔CLI contract.
//!
//! `run` builds it from flags, or loads it wholesale from a TOML/JSON file
//! via `--strategy` / `forge strategy` ([`Strategy::load`]). The `serde`
//! derives let the resolved strategy be written into the run dir and
//! round-tripped, so a run is fully reproducible from its recorded strategy.

use serde::{Deserialize, Serialize};

use crate::ops::Injection;
use crate::synth::Scaffold;

/// The built-in blind-spot backlog `stale_soft` rotates the target
/// along (the parity-matrix "zero differential evidence" rules). Each
/// is the injector that exercises it from the LOCA→SAMP+ABBR base.
pub const BLINDSPOT_BACKLOG: &[Injection] = &[
    Injection::DupSampKeyTuple, // Rule 10a
    Injection::OrphanSampRow,   // Rule 10c
    Injection::BadDtValue,      // Rule 8  (control: known-AGREE)
    Injection::UnquotedField,   // Rule 5  (O-3 cascade territory)
    Injection::FiveLetterGroup, // Rule 19
    Injection::DropProjData,    // Rule 13 (co-trips Rule 2, O-16)
    Injection::DropTranData,    // Rule 14 (co-trips Rule 2, O-16)
    Injection::UndefinedAbbrev, // Rule 16 (PA picklist)
    Injection::UndefinedType,   // Rule 17 (undefined TYPE code)
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceCfg {
    pub enabled: bool,
    /// Residual per-class oracle sample rate — never 0.
    pub floor: f64,
    /// Forced oracle calls right after a class's trust collapses.
    pub force_burst: u32,
}

impl Default for ConfidenceCfg {
    fn default() -> Self {
        Self {
            enabled: true,
            floor: 0.01,
            force_burst: 25,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    pub name: String,
    pub scaffold: String,
    /// Enabled injectors (the operator space this run explores).
    pub injectors: Vec<String>,
    pub max_generations: u64,
    pub max_candidates: u64,
    pub max_wall_secs: u64,
    /// Hard cap on python-ags4 subprocess calls (the dominant cost).
    pub python_budget: u64,
    pub stale_soft: u64,
    pub stale_hard: u64,
    pub seed: u64,
    pub confidence: ConfidenceCfg,
}

impl Default for Strategy {
    fn default() -> Self {
        Self {
            name: "run (flag-driven)".into(),
            scaffold: "loca-samp".into(),
            injectors: BLINDSPOT_BACKLOG
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
            max_generations: 200,
            max_candidates: 5000,
            max_wall_secs: 900,
            python_budget: 400,
            stale_soft: 20,
            stale_hard: 60,
            seed: 42,
            confidence: ConfidenceCfg::default(),
        }
    }
}

impl Strategy {
    pub fn scaffold(&self) -> Scaffold {
        Scaffold::parse(&self.scaffold).unwrap_or(Scaffold::LocaSamp)
    }
    /// The resolved injector pool (skips unknown tokens defensively).
    pub fn pool(&self) -> Vec<Injection> {
        let v: Vec<Injection> = self
            .injectors
            .iter()
            .filter_map(|s| injector_token(s))
            .collect();
        if v.is_empty() {
            BLINDSPOT_BACKLOG.to_vec()
        } else {
            v
        }
    }
}

/// Map a strategy/CLI injector token to an `Injection` (accepts both
/// the `rule10a` and the `Display` `rule10a:dup-samp-key` forms).
pub fn injector_token(s: &str) -> Option<Injection> {
    let head = s.split(':').next().unwrap_or(s);
    Injection::parse(head)
}

impl Strategy {
    /// Parse a strategy file (`.toml` or `.json`, by extension).
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let txt = std::fs::read_to_string(path)?;
        let s: Strategy = if path.extension().and_then(|e| e.to_str()) == Some("json") {
            serde_json::from_str(&txt)?
        } else {
            toml::from_str(&txt)?
        };
        s.validate()?;
        Ok(s)
    }

    /// Schema + value checks, *without running anything* (read-only —
    /// `forge strategy validate`). Unknown injector / scaffold / a
    /// zero floor are hard errors (exit 5).
    pub fn validate(&self) -> anyhow::Result<()> {
        if Scaffold::parse(&self.scaffold).is_none() {
            anyhow::bail!("unknown scaffold '{}' (minimal | loca-samp)", self.scaffold);
        }
        for tok in &self.injectors {
            if injector_token(tok).is_none() {
                anyhow::bail!("unknown injector '{tok}'");
            }
        }
        if !(self.confidence.floor > 0.0 && self.confidence.floor <= 1.0) {
            anyhow::bail!(
                "confidence.floor must be in (0,1]; got {} (a 0 floor would \
                 let a trusted class go permanently unchecked)",
                self.confidence.floor
            );
        }
        if self.stale_hard <= self.stale_soft {
            anyhow::bail!(
                "stale_hard ({}) must exceed stale_soft ({})",
                self.stale_hard,
                self.stale_soft
            );
        }
        Ok(())
    }

    pub fn to_toml(&self) -> String {
        toml::to_string_pretty(self).unwrap_or_default()
    }

    /// A commented starter template for `forge strategy new`.
    pub fn template() -> String {
        let s = Strategy {
            name: "hunt-rule-10a-dupkey-relational".into(),
            ..Strategy::default()
        };
        format!(
            "# laterite-ags4-forge strategy — the executable twin of an\n\
             # ags-wiki/strategies/strat-forge-*.md page. Claude authors\n\
             # this from the wiki (parity-matrix blind spots, OBSERVATIONS,\n\
             # the rule pages); the binary is the deterministic executor.\n\
             #\n# `forge strategy validate <f>` checks this without running\n\
             # anything; `forge run --strategy <f>` executes it.\n\n{}",
            s.to_toml()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_strategy_validates_and_round_trips_toml() {
        let s = Strategy::default();
        assert!(s.validate().is_ok());
        let back: Strategy = toml::from_str(&s.to_toml()).unwrap();
        assert!(back.validate().is_ok());
        assert_eq!(back.seed, s.seed);
        assert!(Strategy::template().contains("[confidence]"));
    }

    #[test]
    fn validate_rejects_bad_schema() {
        let bad_floor = Strategy {
            confidence: ConfidenceCfg {
                floor: 0.0,
                ..ConfidenceCfg::default()
            },
            ..Strategy::default()
        };
        assert!(bad_floor.validate().is_err(), "a 0 floor must be rejected");

        let bad_inj = Strategy {
            injectors: vec!["rule10a".into(), "totally-bogus".into()],
            ..Strategy::default()
        };
        assert!(
            bad_inj.validate().is_err(),
            "unknown injector must be rejected"
        );

        let bad_scaffold = Strategy {
            scaffold: "nope".into(),
            ..Strategy::default()
        };
        assert!(bad_scaffold.validate().is_err());

        let bad_stale = Strategy {
            stale_soft: 60,
            stale_hard: 60,
            ..Strategy::default()
        };
        assert!(
            bad_stale.validate().is_err(),
            "stale_hard must exceed stale_soft"
        );
    }
}
