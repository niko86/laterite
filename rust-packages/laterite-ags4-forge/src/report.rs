//! Result documents — the `laterite_cliutil::report` contract (table /
//! json / ndjson, `--compact` projection), identical UX to
//! laterite-ags4-corpus-qa.

use std::collections::BTreeMap;
use std::io::{self, Write};

use laterite_ags4_parity::Parity;
use laterite_cliutil::report::{Ctx, Report, without_keys};
use laterite_cliutil::styled_table;
use serde::Serialize;

use crate::pipeline::Outcome;

/// Flatten a verdict to `(tag, detail)` for the serialized doc.
pub(crate) fn verdict_parts(v: Option<&Parity>) -> (String, String) {
    match v {
        None => ("RUST_ONLY".into(), "oracle unavailable — Rust-only".into()),
        Some(p) => (
            p.tag().into(),
            match p {
                Parity::Agree => "rust ≡ python".into(),
                Parity::KnownDivergence {
                    observation,
                    detail,
                } => {
                    format!("{observation}: {detail}")
                }
                Parity::RustOnlyRules { rules } => format!("rust-only {rules:?}"),
                Parity::PythonOnlyRules { rules } => format!("python-only {rules:?}"),
                Parity::RulesDiffer {
                    rust_only,
                    python_only,
                } => format!("rust-only {rust_only:?} python-only {python_only:?}"),
                Parity::ValidityDisagree { rust, python } => {
                    format!("rust={rust} python={python}")
                }
                Parity::PythonError { reason } => format!("python error: {reason}"),
            },
        ),
    }
}

/// `forge check <file>` — one file's dual-validation.
///
/// Schema 2 adds the per-rule counts (#654). The verdict above is still
/// presence-only; the counts sit beside it so a reader can see the thing
/// the verdict is not allowed to judge — one side saying a rule nine
/// times and the other saying it once is the same verdict and a very
/// different file.
#[derive(Debug, Serialize)]
pub struct CheckReport {
    pub schema: u32,
    pub file: String,
    pub dict_used: String,
    pub oracle: bool,
    pub verdict: String,
    pub detail: String,
    pub rust_rules: Vec<String>,
    pub python_rules: Vec<String>,
    pub rust_rule_counts: BTreeMap<String, u64>,
    pub python_rule_counts: BTreeMap<String, u64>,
}

impl CheckReport {
    pub fn new(file: String, o: &Outcome, oracle: bool) -> Self {
        let (verdict, detail) = verdict_parts(o.verdict.as_ref());
        Self {
            schema: 2,
            file,
            dict_used: o.dict_used.clone(),
            oracle,
            verdict,
            detail,
            rust_rules: o.rust_rules(),
            python_rules: o.python_rules(),
            rust_rule_counts: o.rust_counts.clone(),
            python_rule_counts: o.python_counts.clone(),
        }
    }

    /// Every rule either side raised, with both tallies — `None` where a
    /// side did not raise it at all, which is what the verdict keys off.
    pub fn count_rows(&self) -> Vec<(String, Option<u64>, Option<u64>)> {
        let mut keys: Vec<&String> = self
            .rust_rule_counts
            .keys()
            .chain(self.python_rule_counts.keys())
            .collect();
        keys.sort_unstable();
        keys.dedup();
        keys.into_iter()
            .map(|k| {
                (
                    k.clone(),
                    self.rust_rule_counts.get(k).copied(),
                    self.python_rule_counts.get(k).copied(),
                )
            })
            .collect()
    }
    /// A real, unexplained divergence (drives the exit code).
    pub fn is_action(&self) -> bool {
        Parity::is_action_tag(&self.verdict)
    }
}

impl Report for CheckReport {
    fn render_table(&self, w: &mut dyn Write, ctx: &Ctx) -> io::Result<()> {
        writeln!(
            w,
            "{}",
            styled_table(
                &["Field", "Value"],
                vec![
                    vec!["file".into(), self.file.clone()],
                    vec!["dict".into(), self.dict_used.clone()],
                    vec!["verdict".into(), self.verdict.clone()],
                    vec!["detail".into(), self.detail.clone()],
                    vec!["rust".into(), format!("{:?}", self.rust_rules)],
                    vec![
                        "python".into(),
                        if self.oracle {
                            format!("{:?}", self.python_rules)
                        } else {
                            "(oracle unavailable)".into()
                        },
                    ],
                ],
                ctx.colour(),
            )
        )?;
        let rows = self.count_rows();
        if !rows.is_empty() {
            // A dash rather than a zero: the two mean different things
            // here. Zero would say "raised it, nothing found"; the rule
            // was not raised at all.
            let cell = |n: Option<u64>| n.map_or_else(|| "-".to_string(), |v| v.to_string());
            writeln!(
                w,
                "{}",
                styled_table(
                    &["Rule", "rust", "python"],
                    rows.into_iter()
                        .map(|(r, a, b)| vec![r, cell(a), cell(if self.oracle { b } else { None })])
                        .collect(),
                    ctx.colour(),
                )
            )?;
        }
        Ok(())
    }
}

/// `forge check <dir-or-files>` — one report over many files.
///
/// The sweep exists so a corpus costs one invocation instead of one per
/// file, and so the verdict tally is computed once over the whole set.
#[derive(Debug, Serialize)]
pub struct CheckSweepReport {
    pub schema: u32,
    pub oracle: bool,
    /// What the sweep looked at, and what it walked past. A directory
    /// holds more than `.ags` files, and a sweep that silently ignores
    /// the rest reads as "checked everything" when it did not.
    pub files_checked: usize,
    pub skipped_not_ags: usize,
    pub counts: BTreeMap<String, u64>,
    pub files: Vec<CheckReport>,
}

impl CheckSweepReport {
    pub fn new(files: Vec<CheckReport>, oracle: bool, skipped_not_ags: usize) -> Self {
        let mut counts: BTreeMap<String, u64> = BTreeMap::new();
        for f in &files {
            *counts.entry(f.verdict.clone()).or_default() += 1;
        }
        Self {
            schema: 1,
            oracle,
            files_checked: files.len(),
            skipped_not_ags,
            counts,
            files,
        }
    }
    /// Any file a real, unexplained divergence (drives the exit code).
    pub fn is_action(&self) -> bool {
        self.files.iter().any(CheckReport::is_action)
    }
}

impl Report for CheckSweepReport {
    fn render_table(&self, w: &mut dyn Write, ctx: &Ctx) -> io::Result<()> {
        writeln!(
            w,
            "{}",
            styled_table(
                &["Verdict", "Files"],
                self.counts
                    .iter()
                    .map(|(k, v)| vec![k.clone(), v.to_string()])
                    .collect(),
                ctx.colour(),
            )
        )?;
        writeln!(
            w,
            "{} file(s) checked, {} non-.ags entry(ies) walked past",
            self.files_checked, self.skipped_not_ags
        )?;
        if ctx.compact {
            writeln!(w, "rerun without --compact for per-file rows")?;
        } else {
            for f in &self.files {
                writeln!(w, "  {:<22} {}\n       {}", f.verdict, f.file, f.detail)?;
            }
        }
        Ok(())
    }

    /// `--compact`: drop the per-file array, keep the tally.
    fn compact_value(&self) -> serde_json::Value {
        without_keys(self.full_value(), &["files"])
    }
}

/// One generated candidate's row.
#[derive(Debug, Serialize)]
pub struct Candidate {
    pub seq: usize,
    pub injection: String,
    pub target_rule: Option<String>,
    pub path: String,
    pub dict_used: String,
    pub verdict: String,
    pub detail: String,
    pub rust_rules: Vec<String>,
    pub python_rules: Vec<String>,
}

impl Candidate {
    /// Deliberately WITHOUT the per-rule counts (#654). This row type is
    /// the generated-candidate row for `gen`, `run` and `mine` alike, and
    /// those emit them by the hundred or thousand — `--compact` exists
    /// because that volume already tells. Counts are a reading aid for one
    /// file at a time, which is `check`.
    pub fn from_outcome(
        seq: usize,
        injection: String,
        target_rule: Option<String>,
        path: String,
        o: &Outcome,
    ) -> Self {
        let (verdict, detail) = verdict_parts(o.verdict.as_ref());
        Self {
            seq,
            injection,
            target_rule,
            path,
            dict_used: o.dict_used.clone(),
            verdict,
            detail,
            rust_rules: o.rust_rules(),
            python_rules: o.python_rules(),
        }
    }
}

/// `forge gen` — N synthesized/mutated candidates, each dual-validated.
#[derive(Debug, Serialize)]
pub struct ForgeReport {
    pub schema: u32,
    pub created: String,
    pub strategy: String,
    pub scaffold: String,
    pub oracle: bool,
    pub counts: std::collections::BTreeMap<String, u64>,
    pub candidates: Vec<Candidate>,
}

impl ForgeReport {
    pub fn actions_present(&self) -> bool {
        self.candidates
            .iter()
            .any(|c| Parity::is_action_tag(&c.verdict))
    }
}

impl Report for ForgeReport {
    fn render_table(&self, w: &mut dyn Write, ctx: &Ctx) -> io::Result<()> {
        writeln!(
            w,
            "{}",
            styled_table(
                &["Verdict", "Candidates"],
                self.counts
                    .iter()
                    .map(|(k, v)| vec![k.clone(), v.to_string()])
                    .collect(),
                ctx.colour(),
            )
        )?;
        if ctx.compact {
            writeln!(
                w,
                "{} candidate(s) — rerun without --compact for rows",
                self.candidates.len()
            )?;
        } else {
            for c in &self.candidates {
                writeln!(
                    w,
                    "  #{:<3} {:<26} {} | rust={:?} python={:?}\n       {}",
                    c.seq, c.injection, c.verdict, c.rust_rules, c.python_rules, c.path
                )?;
            }
        }
        Ok(())
    }

    /// `--compact`: drop the heavy per-candidate array (keep
    /// schema/strategy/counts) — the token-lean agent-loop view.
    fn compact_value(&self) -> serde_json::Value {
        without_keys(self.full_value(), &["candidates"])
    }
}

/// `forge describe` — one generated BS 5930 soil description.
#[derive(Debug, Serialize)]
pub struct DescribeRow {
    pub seed: u64,
    pub principal: String,
    pub class: String,
    pub text: String,
    /// The secondary fractions behind the qualifiers, as `gravel 24% (very)`
    /// — exposes the percentage-first basis (a fine-in-fine shows `(named)`).
    pub fractions: Vec<String>,
}

/// `forge describe` — a preview batch of constraint-valid descriptions.
#[derive(Debug, Serialize)]
pub struct DescribeReport {
    pub schema: u32,
    pub count: usize,
    pub base_seed: u64,
    pub descriptions: Vec<DescribeRow>,
}

impl Report for DescribeReport {
    fn render_table(&self, w: &mut dyn Write, _ctx: &Ctx) -> io::Result<()> {
        for d in &self.descriptions {
            writeln!(w, "  [{}] {}", d.seed, d.text)?;
        }
        Ok(())
    }
}

/// `forge scale` — a calibrated sized file written to disk.
#[derive(Debug, Serialize)]
pub struct ScaleReport {
    pub schema: u32,
    pub scaffold: String,
    pub seed: u64,
    pub target_bytes: u64,
    pub predicted_bytes: u64,
    pub actual_bytes: u64,
    pub n_loca: usize,
    pub groups: usize,
    pub path: String,
    /// The fault injector spread across the file (`None` = a clean scale, the
    /// default — byte-identical to the pre-density behaviour).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inject: Option<String>,
    /// Fraction of applicable sites corrupted (`None` = clean).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub density: Option<f64>,
    /// How many sites the injector actually mutated (`0` = clean).
    pub dirty_sites: usize,
}

impl Report for ScaleReport {
    fn render_table(&self, w: &mut dyn Write, ctx: &Ctx) -> io::Result<()> {
        let mut rows = vec![
            vec!["scaffold".into(), self.scaffold.clone()],
            vec!["target".into(), self.target_bytes.to_string()],
            vec!["actual".into(), self.actual_bytes.to_string()],
            vec!["n_loca".into(), self.n_loca.to_string()],
            vec!["groups".into(), self.groups.to_string()],
        ];
        if let Some(inject) = &self.inject {
            rows.push(vec!["inject".into(), inject.clone()]);
            rows.push(vec![
                "density".into(),
                self.density
                    .map_or_else(|| "1.0".to_string(), |d| d.to_string()),
            ]);
            rows.push(vec!["dirty_sites".into(), self.dirty_sites.to_string()]);
        }
        rows.push(vec!["path".into(), self.path.clone()]);
        writeln!(
            w,
            "{}",
            styled_table(&["Field", "Value"], rows, ctx.colour())
        )
    }
}

/// `forge catalog` — one injector's entry (the injector→rule mapping the
/// compliance/parity matrix is built on).
#[derive(Debug, Serialize)]
pub struct CatalogEntry {
    pub token: String,
    pub target_rule: String,
    /// `any` or `loca-samp` (the scaffold the injector needs).
    pub scaffold: String,
    pub description: String,
}

/// A canonical rule the forge does *not* inject in isolation, and why.
#[derive(Debug, Serialize)]
pub struct UninjectableRule {
    pub rules: String,
    pub reason: String,
}

/// `forge catalog` — the injector→rule map + the honest record of which
/// canonical AGS rules aren't cleanly single-injectable.
#[derive(Debug, Serialize)]
pub struct CatalogReport {
    pub schema: u32,
    pub injectors: Vec<CatalogEntry>,
    pub not_single_injectable: Vec<UninjectableRule>,
}

impl Report for CatalogReport {
    fn render_table(&self, w: &mut dyn Write, ctx: &Ctx) -> io::Result<()> {
        writeln!(
            w,
            "{}",
            styled_table(
                &["Token", "Target", "Scaffold", "Mutation"],
                self.injectors
                    .iter()
                    .map(|e| vec![
                        e.token.clone(),
                        e.target_rule.clone(),
                        e.scaffold.clone(),
                        e.description.clone(),
                    ])
                    .collect(),
                ctx.colour(),
            )
        )?;
        writeln!(w, "\nnot single-injectable (documented):")?;
        for u in &self.not_single_injectable {
            writeln!(w, "  Rule {} — {}", u.rules, u.reason)?;
        }
        Ok(())
    }
}

/// One mined combination candidate: the combo applied at a seeded
/// placement, its *actual* Rust rule-set, and whether it's a corpus gap /
/// divergence-prone / was spent an oracle call.
#[derive(Debug, Serialize)]
pub struct MineCandidate {
    pub combo: String,
    pub seed: u64,
    /// The file's actual Rust rule-set (the signature), sorted.
    pub signature: Vec<String>,
    /// The Rust signature isn't present in the profiled corpus.
    pub is_gap: bool,
    /// The signature intersects the known Rust↔python divergence rules.
    pub divergence_prone: bool,
    /// A python-ags4 call was spent on this candidate.
    pub oracle_ran: bool,
    pub verdict: String,
    pub detail: String,
    pub python_rules: Vec<String>,
    pub path: String,
}

impl MineCandidate {
    /// A real, unexplained divergence (drives the exit code) — only
    /// meaningful when the oracle actually ran.
    pub fn is_action(&self) -> bool {
        self.oracle_ran && Parity::is_action_tag(&self.verdict)
    }
}

/// `forge mine` — the corpus-gap divergence miner's report.
#[derive(Debug, Serialize)]
pub struct MineReport {
    pub schema: u32,
    pub created: String,
    pub scaffold: String,
    pub corpus: String,
    pub corpus_files: usize,
    /// Distinct Rust signatures the corpus already covers.
    pub corpus_signatures: usize,
    pub combinations_tried: usize,
    pub candidates_synthesized: usize,
    /// Distinct (combo-independent) Rust signatures synthesized.
    pub distinct_signatures: usize,
    /// Synthesized signatures not present in the corpus.
    pub gaps: usize,
    /// Gap signatures intersecting the divergence-prone rule set.
    pub divergence_prone_gaps: usize,
    pub oracle: bool,
    pub oracle_calls: usize,
    pub counts: std::collections::BTreeMap<String, u64>,
    pub candidates: Vec<MineCandidate>,
}

impl MineReport {
    pub fn actions_present(&self) -> bool {
        self.candidates.iter().any(MineCandidate::is_action)
    }
}

impl Report for MineReport {
    fn render_table(&self, w: &mut dyn Write, ctx: &Ctx) -> io::Result<()> {
        writeln!(
            w,
            "{}",
            styled_table(
                &["Metric", "Value"],
                vec![
                    vec!["corpus".into(), self.corpus.clone()],
                    vec![
                        "corpus files / signatures".into(),
                        format!("{} / {}", self.corpus_files, self.corpus_signatures),
                    ],
                    vec![
                        "combinations tried".into(),
                        self.combinations_tried.to_string(),
                    ],
                    vec![
                        "candidates synthesized".into(),
                        self.candidates_synthesized.to_string(),
                    ],
                    vec![
                        "distinct signatures".into(),
                        self.distinct_signatures.to_string(),
                    ],
                    vec![
                        "corpus gaps (divergence-prone)".into(),
                        format!("{} ({})", self.gaps, self.divergence_prone_gaps),
                    ],
                    vec!["oracle calls".into(), self.oracle_calls.to_string()],
                ],
                ctx.colour(),
            )
        )?;
        if !self.counts.is_empty() {
            writeln!(
                w,
                "{}",
                styled_table(
                    &["Verdict", "Oracle-checked"],
                    self.counts
                        .iter()
                        .map(|(k, v)| vec![k.clone(), v.to_string()])
                        .collect(),
                    ctx.colour(),
                )
            )?;
        }
        if ctx.compact {
            writeln!(
                w,
                "{} candidate(s) — rerun without --compact for rows",
                self.candidates.len()
            )?;
        } else {
            for c in self.candidates.iter().filter(|c| c.is_gap) {
                let tag = if c.divergence_prone { "★" } else { " " };
                writeln!(
                    w,
                    "  {tag} {:<24} seed={:<3} {} | rust={:?}",
                    c.combo,
                    c.seed,
                    if c.oracle_ran {
                        &c.verdict
                    } else {
                        "(rust-only)"
                    },
                    c.signature,
                )?;
            }
        }
        Ok(())
    }

    /// `--compact`: drop the heavy per-candidate array.
    fn compact_value(&self) -> serde_json::Value {
        without_keys(self.full_value(), &["candidates"])
    }
}

/// `forge run` — the evolutionary loop's report.
#[derive(Debug, Serialize)]
pub struct RunReport {
    pub schema: u32,
    pub created: String,
    pub strategy: String,
    /// `findings | stalled | budget_exhausted | clean`.
    pub status: String,
    pub generations: u64,
    pub seed: u64,
    pub counts: std::collections::BTreeMap<String, u64>,
    pub permutes: Vec<serde_json::Value>,
    /// The parity-confidence ledger summary (per-class + global
    /// P(Rust≡python) lower bound + `python_calls_saved`).
    pub confidence: serde_json::Value,
    pub findings: Vec<Candidate>,
    pub candidates: Vec<Candidate>,
}

impl RunReport {
    /// The `stale_hard` hand-back: everything needed to
    /// author the next (permuted) strategy, minus the bulky candidate
    /// stream.
    pub fn frontier(&self) -> serde_json::Value {
        serde_json::json!({
            "status": self.status,
            "strategy": self.strategy,
            "generations": self.generations,
            "seed": self.seed,
            "counts": self.counts,
            "permutes": self.permutes,
            "confidence": self.confidence,
            "findings": self.findings.len(),
            "hint": "signatures saturated under this strategy — rotate the \
                     target rule / widen the operator space / change the \
                     scaffold and re-run with a fresh strategy",
        })
    }
}

impl Report for RunReport {
    fn render_table(&self, w: &mut dyn Write, ctx: &Ctx) -> io::Result<()> {
        writeln!(
            w,
            "{}",
            styled_table(
                &["Verdict", "Candidates"],
                self.counts
                    .iter()
                    .map(|(k, v)| vec![k.clone(), v.to_string()])
                    .collect(),
                ctx.colour(),
            )
        )?;
        writeln!(
            w,
            "status={} generations={} findings={} permutes={}",
            self.status,
            self.generations,
            self.findings.len(),
            self.permutes.len()
        )?;
        let g = self
            .confidence
            .get("global_p_equiv_lcb95")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let saved = self
            .confidence
            .get("python_calls_saved")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let made = self
            .confidence
            .get("python_calls_made")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        writeln!(
            w,
            "P(Rust≡python) global lower-bound (95%): {g}  | oracle calls made={made} saved={saved}"
        )?;
        if !ctx.compact {
            for f in &self.findings {
                writeln!(
                    w,
                    "  FINDING {} {} rust={:?} python={:?}\n    {}",
                    f.injection, f.verdict, f.rust_rules, f.python_rules, f.path
                )?;
            }
        }
        Ok(())
    }

    /// `--compact`: keep status/counts/permutes/confidence + findings
    /// (the agent-loop signal); drop the full per-generation stream.
    fn compact_value(&self) -> serde_json::Value {
        without_keys(self.full_value(), &["candidates"])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn counts(xs: &[(&str, u64)]) -> BTreeMap<String, u64> {
        xs.iter().map(|(k, v)| ((*k).to_string(), *v)).collect()
    }

    fn report(verdict: &str, rust: &[(&str, u64)], py: &[(&str, u64)]) -> CheckReport {
        CheckReport {
            schema: 2,
            file: "f.ags".into(),
            dict_used: "4.2".into(),
            oracle: true,
            verdict: verdict.into(),
            detail: String::new(),
            rust_rules: rust.iter().map(|(k, _)| (*k).to_string()).collect(),
            python_rules: py.iter().map(|(k, _)| (*k).to_string()).collect(),
            rust_rule_counts: counts(rust),
            python_rule_counts: counts(py),
        }
    }

    #[test]
    fn count_rows_pair_each_rule_and_keep_absence_distinct_from_zero() {
        // The case #654 exists for: both raised Rule 16, only python
        // raised 10c, and they disagree on how many times Rule 8 fired.
        let r = report(
            "RULES_DIFFER",
            &[("AGS Format Rule 16", 1), ("AGS Format Rule 8", 1)],
            &[
                ("AGS Format Rule 10c", 9),
                ("AGS Format Rule 16", 1),
                ("AGS Format Rule 8", 3),
            ],
        );
        assert_eq!(
            r.count_rows(),
            vec![
                ("AGS Format Rule 10c".to_string(), None, Some(9)),
                ("AGS Format Rule 16".to_string(), Some(1), Some(1)),
                ("AGS Format Rule 8".to_string(), Some(1), Some(3)),
            ]
        );
    }

    #[test]
    fn a_rust_only_run_has_no_python_column_to_fill() {
        let mut r = report("RUST_ONLY", &[("AGS Format Rule 1", 2)], &[]);
        r.oracle = false;
        assert_eq!(
            r.count_rows(),
            vec![("AGS Format Rule 1".to_string(), Some(2), None)]
        );
    }

    #[test]
    fn a_sweep_tallies_its_verdicts_and_is_an_action_if_any_file_is() {
        let sweep = CheckSweepReport::new(
            vec![
                report("AGREE", &[], &[]),
                report("AGREE", &[], &[]),
                report("RUST_ONLY_RULES", &[("AGS Format Rule 8", 1)], &[]),
            ],
            true,
            4,
        );
        assert_eq!(sweep.files_checked, 3);
        assert_eq!(sweep.skipped_not_ags, 4);
        assert_eq!(sweep.counts.get("AGREE"), Some(&2));
        assert_eq!(sweep.counts.get("RUST_ONLY_RULES"), Some(&1));
        assert!(sweep.is_action());
    }

    #[test]
    fn counts_reach_the_report_and_never_the_verdict() {
        // #654's standing condition: the counts are observable, never
        // judged. `classify` takes presence sets and cannot see them, and
        // this pins the wiring one level up — two outcomes that differ
        // ONLY in how often each rule fired carry the same verdict and the
        // same detail, and differ only where the counts are shown.
        let quiet = report(
            "AGREE",
            &[("AGS Format Rule 8", 1)],
            &[("AGS Format Rule 8", 1)],
        );
        let loud = report(
            "AGREE",
            &[("AGS Format Rule 8", 1)],
            &[("AGS Format Rule 8", 9)],
        );
        assert_eq!(quiet.verdict, loud.verdict);
        assert_eq!(quiet.detail, loud.detail);
        assert_eq!(quiet.rust_rules, loud.rust_rules);
        assert_eq!(quiet.python_rules, loud.python_rules);
        assert_ne!(quiet.count_rows(), loud.count_rows());
    }

    #[test]
    fn a_sweep_of_agreeing_files_is_not_an_action() {
        let sweep = CheckSweepReport::new(vec![report("AGREE", &[], &[])], true, 0);
        assert!(!sweep.is_action());
    }
}
