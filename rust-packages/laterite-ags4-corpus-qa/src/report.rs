//! `report.json` — the validate→parity hand-off.

use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::Path;

use anyhow::{Context, Result};
use laterite_cliutil::styled_table;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::output::{Ctx, Report, without_keys};

/// Schema 2 adds: per-rule finding counts (`Outcome::Findings.rules`
/// is now `[(rule, count)]`), per-file `dict_resolution`, and the
/// top-level `clusters[]`. Schema-1 reports simply lack these — every
/// added field is `#[serde(default)]`, so old `report.json` still
/// deserialises (the missing bits read as empty).
pub const SCHEMA: u32 = 2;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Counts {
    pub clean: u64,
    pub findings: u64,
    pub hard_error: u64,
    pub panic: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateReport {
    pub schema: u32,
    pub created: String,
    pub dict_version: String,
    pub total: usize,
    pub summary: Counts,
    /// "AGS Format Rule N" -> number of **files** in which it fired
    /// (unchanged from schema 1 — per-rule *multiplicity* lives on
    /// each file's `Outcome::Findings.rules` and in `clusters`).
    pub rule_histogram: BTreeMap<String, u64>,
    /// Findings files grouped by identical rule-signature, desc by
    /// `file_count` — the "1 producer, N files, same defect" view.
    /// Kept under `--compact` (it's the token-lean high-signal
    /// summary). Empty for schema-1 reads (serde default).
    #[serde(default)]
    pub clusters: Vec<Cluster>,
    pub files: Vec<FileOutcome>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cluster {
    /// Sorted `"<rule>×<count>"` signature, e.g.
    /// `["AGS Format Rule 10b×3"]`.
    pub signature: Vec<String>,
    pub file_count: usize,
    /// Longest common path prefix of the cluster's source paths
    /// (separator-trimmed) — surfaces the producing directory.
    pub common_source_prefix: String,
    /// Up to 3 example source paths (deterministic — files are
    /// crawl-sorted by dest).
    pub examples: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOutcome {
    pub dest: String,
    pub source: String,
    pub sha256: String,
    /// Harvested file's hash now differs from the manifest's (it
    /// changed under us) — keeps "file mutated" distinct from a
    /// genuine validator divergence.
    pub mutated: bool,
    pub outcome: Outcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surprising: Option<String>,
    /// Bundled edition this file was judged against (TRAN_AGS-resolved
    /// or forced), e.g. `"4.0.4"`; `"-"` if it errored/panicked before
    /// a dictionary was resolved. Lets batch triage see *why* a file
    /// was checked against a given schema.
    #[serde(default)]
    pub dict_used: String,
    /// *How* `dict_used` was chosen: `"forced"` | `"exact"` |
    /// `"guessed"` | `"fallback"` | `"-"`. A genuine `TRAN_AGS=4.1.1`
    /// and the O-30 missing/unparsable→4.1.1 fallback both show
    /// `dict_used="4.1.1"`; this distinguishes them (the dogfood
    /// blind spot O-31 surfaced). Schema-1 reports lack it → `""`.
    #[serde(default)]
    pub dict_resolution: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Outcome {
    Clean,
    Findings {
        count: usize,
        rules: Vec<(String, usize)>,
    },
    HardError {
        variant: String,
        message: String,
    },
    Panic {
        payload: String,
    },
}

impl FileOutcome {
    /// Triage = anything worth a human / parity look: the validator
    /// rejected it, panicked, or the result is heuristically odd.
    pub fn is_triage(&self) -> bool {
        matches!(
            self.outcome,
            Outcome::HardError { .. } | Outcome::Panic { .. }
        ) || self.surprising.is_some()
    }
}

impl ValidateReport {
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p).with_context(|| format!("create {}", p.display()))?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self)?)
            .with_context(|| format!("write {}", path.display()))?;
        Ok(())
    }

    /// Used by `parity` (S2) to re-read a prior validate pass.
    #[allow(dead_code)]
    pub fn load(path: &Path) -> Result<Self> {
        let s = std::fs::read_to_string(path)
            .with_context(|| format!("read report {}", path.display()))?;
        serde_json::from_str(&s).with_context(|| format!("parse report {}", path.display()))
    }

    /// The actionable dogfood subset (hard errors / panics / surprises)
    /// — drives both the `table`-mode TRIAGE list and the exit code.
    pub fn triage(&self) -> Vec<&FileOutcome> {
        self.files.iter().filter(|f| f.is_triage()).collect()
    }
}

impl Report for ValidateReport {
    fn render_table(&self, w: &mut dyn Write, ctx: &Ctx) -> io::Result<()> {
        let c = ctx.colour();
        let s = &self.summary;
        writeln!(
            w,
            "{}",
            styled_table(
                &["Outcome", "Files"],
                vec![
                    vec!["clean".into(), s.clean.to_string()],
                    vec!["findings".into(), s.findings.to_string()],
                    vec!["hard error".into(), s.hard_error.to_string()],
                    vec!["panic".into(), s.panic.to_string()],
                    vec!["TOTAL".into(), self.total.to_string()],
                ],
                c,
            )
        )?;

        // Which bundled editions the corpus actually resolved to, and
        // *how* — annotated so a genuine `TRAN_AGS` edition and the
        // O-30 fallback are SEPARATE rows (e.g. `4.1.1` vs
        // `4.1.1 (fallback)`). This is the dogfood blind spot O-31
        // surfaced: 294 fallback files used to hide as plain `4.1.1`.
        let mut editions: BTreeMap<String, u64> = BTreeMap::new();
        for f in &self.files {
            let label = match f.dict_resolution.as_str() {
                "fallback" => format!("{} (fallback)", f.dict_used),
                "guessed" => format!("{} (guessed)", f.dict_used),
                "forced" => format!("{} (forced)", f.dict_used),
                // "exact" (genuine TRAN_AGS), "-" (errored pre-resolve),
                // or "" (schema-1 report w/o the field) → bare.
                _ => f.dict_used.clone(),
            };
            *editions.entry(label).or_default() += 1;
        }
        writeln!(
            w,
            "{}",
            styled_table(
                &["Dict edition used", "Files"],
                editions
                    .iter()
                    .map(|(k, v)| vec![k.clone(), v.to_string()])
                    .collect(),
                c,
            )
        )?;

        // CLUSTERS — the "1 producer, N files, identical defect"
        // view. Top 10 by file count; the rest are in the JSON.
        if !self.clusters.is_empty() {
            if ctx.compact {
                writeln!(
                    w,
                    "CLUSTERS: {} group(s) — see --output json",
                    self.clusters.len()
                )?;
            } else {
                writeln!(w, "\nCLUSTERS (top 10 by file count):")?;
                let rows = self
                    .clusters
                    .iter()
                    .take(10)
                    .map(|cl| {
                        vec![
                            cl.file_count.to_string(),
                            cl.common_source_prefix.clone(),
                            cl.signature.join(", "),
                        ]
                    })
                    .collect();
                writeln!(
                    w,
                    "{}",
                    styled_table(&["Files", "Common source", "Rule signature"], rows, c)
                )?;
            }
        }

        let triage = self.triage();
        if triage.is_empty() {
            writeln!(w, "no triage items (no hard errors / panics / surprises)")?;
        } else if ctx.compact {
            writeln!(
                w,
                "TRIAGE: {} file(s) — rerun without --compact for the list",
                triage.len()
            )?;
        } else {
            writeln!(w, "\nTRIAGE ({} file(s) → feed to `parity`):", triage.len())?;
            for f in &triage {
                let what = match &f.outcome {
                    Outcome::Panic { payload } => format!("PANIC: {payload}"),
                    Outcome::HardError { variant, message } => {
                        format!("{variant}: {message}")
                    }
                    _ => f.surprising.clone().unwrap_or_else(|| "surprising".into()),
                };
                writeln!(w, "  {}  [{}]", f.source, what)?;
            }
        }
        Ok(())
    }

    /// `--compact`: drop ONLY the (potentially huge) per-file `files`
    /// array. `clusters` is deliberately KEPT — it's the token-lean
    /// high-signal summary (the whole point of `--compact` for the
    /// agent/dogfood path), alongside `summary/rule_histogram`.
    fn compact_value(&self) -> Value {
        without_keys(self.full_value(), &["files"])
    }
}
