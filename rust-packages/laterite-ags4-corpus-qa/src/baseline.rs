//! `baseline` — freeze (or drift-check) a deterministic, privacy-
//! scrubbed snapshot of the validator's findings over a manifest.
//!
//! This is the #168 parser-convergence finding-drift gate: freeze the
//! current validator's verdict across the corpus, then `--check` after a
//! change to see *exactly* which files' `(rule, line, group,
//! field_index, severity)` findings moved (a ratified drift updates the
//! baseline + gets an O-N; an unexpected one is a regression).
//!
//! **Why sha-keyed + structural-only.** The corpus is the owner's
//! private client share — source paths, original filenames, and finding
//! `desc` text all carry client data and are never committed. Keying by
//! content `sha256` and storing only the structural finding tuple (no
//! path, no filename, no description) makes the baseline deterministic
//! AND safe to commit / mirror publicly, while still catching any change
//! in validator behaviour. The validation pass is the *same* `judge`
//! `validate` runs, so the baseline reflects the real tool.

use std::collections::BTreeMap;
use std::io::{self, Write};
use std::path::Path;

use anyhow::{Context, Result};
use laterite_cliutil::{progress_bar, styled_table};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use laterite_ags4_validator::findings::Severity;
use laterite_ags4_validator::{CheckOptions, Finding};

use crate::cli::BaselineArgs;
use crate::manifest::CrawlManifest;
use crate::output::{self, Ctx, Plan, Report};
use crate::report::{Counts, Outcome};
use crate::validate::{judge, parse_dict_version};

/// Bump only on an incompatible shape change; `--check` refuses to diff
/// across schemas (the comparison would be meaningless).
pub const SCHEMA: u32 = 1;

/// The committed artifact: a content-addressed map of every file's
/// validator verdict. No timestamp (so two freezes of an unchanged
/// corpus are byte-identical), no paths, no filenames.
#[derive(Debug, Serialize, Deserialize)]
pub struct Baseline {
    pub schema: u32,
    /// The `--dict-version` mode the capture used (`auto` or a forced
    /// edition) — part of the behaviour being frozen.
    pub dict_version: String,
    pub file_count: usize,
    pub finding_count: usize,
    pub summary: Counts,
    /// `sha256` → that content's verdict. `BTreeMap` ⇒ key-sorted ⇒
    /// deterministic / diffable. Identical content (dup files) collapses
    /// to one entry by construction.
    pub files: BTreeMap<String, FileBaseline>,
}

/// One content hash's verdict.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileBaseline {
    /// `"clean" | "findings" | "hard_error" | "panic"`.
    pub outcome: String,
    /// Bundled edition judged against, and *how* it was chosen — `"-"`
    /// when the file errored before a dictionary was resolved. Both are
    /// behaviour (the O-30 fallback drift is visible here).
    pub dict_used: String,
    pub dict_resolution: String,
    /// The error *variant* only (`NotFound`/`NotUtf8`/…) for a
    /// hard-error — never the message, which can embed the file path.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub error_variant: Option<String>,
    /// Sorted structural findings (empty for clean / errored files).
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub findings: Vec<BaselineFinding>,
}

/// One finding, reduced to the structural tuple we gate on. Ordered
/// derive ⇒ a stable sort independent of rule-map / Vec insertion order.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct BaselineFinding {
    pub rule: String,
    /// `None` for whole-group / whole-file findings.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub line: Option<u32>,
    pub group: String,
    /// Tag-stripped column index (`None` = whole-line attribution).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub field_index: Option<u32>,
    /// `"error" | "warning" | "fyi"`; omitted for the common `"error"`.
    #[serde(skip_serializing_if = "is_error", default = "error_severity")]
    pub severity: String,
}

fn error_severity() -> String {
    "error".to_string()
}
fn is_error(s: &str) -> bool {
    s == "error"
}

fn severity_str(s: Severity) -> &'static str {
    match s {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Fyi => "fyi",
    }
}

impl BaselineFinding {
    fn from_finding(rule: &str, f: &Finding) -> Self {
        BaselineFinding {
            rule: rule.to_string(),
            line: f.line,
            group: f.group.clone(),
            field_index: f.location.field_index,
            severity: severity_str(f.severity).to_string(),
        }
    }
}

impl Baseline {
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(p) = path.parent() {
            std::fs::create_dir_all(p).with_context(|| format!("create {}", p.display()))?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self)?)
            .with_context(|| format!("write {}", path.display()))?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self> {
        let s = std::fs::read_to_string(path)
            .with_context(|| format!("read baseline {}", path.display()))?;
        serde_json::from_str(&s).with_context(|| format!("parse baseline {}", path.display()))
    }
}

/// Validate every manifest file and distil the verdicts into a
/// deterministic baseline. Reuses `validate::judge` verbatim, so the
/// snapshot reflects exactly what `validate` would report.
fn capture(args: &BaselineArgs, ctx: Ctx, corpus_dir: &Path) -> Result<Baseline> {
    let dict_version = match parse_dict_version(&args.dict_version) {
        Ok(v) => v,
        Err(bad) => {
            anyhow::bail!("--dict-version expects auto|4.0.3|4.0.4|4.1|4.1.1|4.2, got {bad:?}")
        }
    };

    // Default artifact resolution mirrors `validate`: explicit
    // --manifest wins, else the run pointed at by --run-id / runs/latest.
    let manifest_path = match &args.manifest {
        Some(p) => p.clone(),
        None => {
            crate::paths::resolve_run_dir(corpus_dir, args.run_id.as_deref())?.join("manifest.json")
        }
    };
    let manifest = CrawlManifest::load(&manifest_path)
        .with_context(|| "load manifest (run `crawl` first?)")?;

    let opts = CheckOptions {
        dict_version,
        custom_dict: None,
        include_warnings: args.show_warnings,
        include_fyi: args.show_fyi,
        // Match `validate`'s default: Rule 20 on-disk stat ON (parity
        // with python-ags4) unless explicitly opted out.
        check_files: !args.no_check_files,
        encoding: encoding_rs::UTF_8,
    };

    let pb = progress_bar(manifest.files.len() as u64, ctx.quiet);
    pb.set_message("baselining");

    // Same panic isolation as `validate`: a pathological file is recorded
    // as Outcome::Panic, not an aborted batch; suppress the backtrace.
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(args.jobs.unwrap_or_else(|| {
            std::thread::available_parallelism().map_or(4, std::num::NonZero::get)
        }))
        .build()
        .context("build baseline thread pool")?;

    let entries: Vec<(String, FileBaseline)> = pool.install(|| {
        manifest
            .files
            .par_iter()
            .map(|e| {
                let abs = corpus_dir.join(&e.dest);
                let j = judge(&abs, &e.sha256, &opts);
                let mut findings = Vec::new();
                if let Some(found) = &j.findings {
                    for (rule, fs) in found {
                        for f in fs {
                            findings.push(BaselineFinding::from_finding(rule, f));
                        }
                    }
                    findings.sort();
                }
                let (outcome, error_variant) = match &j.outcome {
                    Outcome::Clean => ("clean", None),
                    Outcome::Findings { .. } => ("findings", None),
                    Outcome::HardError { variant, .. } => ("hard_error", Some(variant.clone())),
                    Outcome::Panic { .. } => ("panic", None),
                };
                pb.inc(1);
                (
                    e.sha256.clone(),
                    FileBaseline {
                        outcome: outcome.to_string(),
                        dict_used: j.dict_used,
                        dict_resolution: j.dict_resolution,
                        error_variant,
                        findings,
                    },
                )
            })
            .collect()
    });
    std::panic::set_hook(prev_hook);
    pb.finish_and_clear();

    // Collapse by content hash (dup files → one entry) and tally.
    let mut files: BTreeMap<String, FileBaseline> = BTreeMap::new();
    let mut summary = Counts::default();
    let mut finding_count = 0usize;
    for (sha, fb) in entries {
        // First occurrence counts toward the summary; identical content
        // produces an identical FileBaseline, so re-inserts are no-ops.
        if let std::collections::btree_map::Entry::Vacant(slot) = files.entry(sha) {
            match fb.outcome.as_str() {
                "clean" => summary.clean += 1,
                "findings" => summary.findings += 1,
                "hard_error" => summary.hard_error += 1,
                "panic" => summary.panic += 1,
                _ => {}
            }
            finding_count += fb.findings.len();
            slot.insert(fb);
        }
    }

    Ok(Baseline {
        schema: SCHEMA,
        dict_version: args.dict_version.clone(),
        file_count: files.len(),
        finding_count,
        summary,
        files,
    })
}

pub fn run(args: &BaselineArgs, ctx: Ctx, corpus_dir: &Path) -> Result<i32> {
    if ctx.dry_run {
        let mode = if args.check.is_some() {
            "drift-check"
        } else {
            "freeze"
        };
        let plan = Plan::new("baseline", format!("would {mode} a findings baseline"))
            .with("mode", mode)
            .with("dict_version", args.dict_version.clone());
        output::emit(&plan, &ctx)?;
        return Ok(0);
    }

    let current = capture(args, ctx, corpus_dir)?;

    if let Some(committed_path) = &args.check {
        let committed = Baseline::load(committed_path)?;
        if committed.schema != current.schema {
            anyhow::bail!(
                "baseline schema mismatch: committed v{} vs current v{} — re-freeze with --out",
                committed.schema,
                current.schema
            );
        }
        let drift = DriftReport::between(&committed, &current);
        let clean = drift.is_clean();
        output::emit(&drift, &ctx)?;
        // Exit 1 on drift so a script / CI step fails loudly.
        return Ok(i32::from(!clean));
    }

    // Freeze mode: --out is clap-guaranteed present when --check absent.
    let out = args
        .out
        .as_ref()
        .expect("clap group guarantees --out or --check");
    current.save(out)?;
    output::note(format!("baseline → {}", out.display()));
    output::emit(&current, &ctx)?;
    Ok(0)
}

// --- freeze rendering -----------------------------------------------

impl Report for Baseline {
    fn render_table(&self, w: &mut dyn Write, ctx: &Ctx) -> io::Result<()> {
        let s = &self.summary;
        writeln!(
            w,
            "{}",
            styled_table(
                &["Baseline", "Value"],
                vec![
                    vec!["files (by content)".into(), self.file_count.to_string()],
                    vec!["findings".into(), self.finding_count.to_string()],
                    vec!["clean".into(), s.clean.to_string()],
                    vec!["with findings".into(), s.findings.to_string()],
                    vec!["hard error".into(), s.hard_error.to_string()],
                    vec!["panic".into(), s.panic.to_string()],
                    vec!["dict mode".into(), self.dict_version.clone()],
                ],
                ctx.colour(),
            )
        )
    }

    /// `--compact`: drop the (large) per-file map; keep the headline.
    fn compact_value(&self) -> Value {
        output::without_keys(self.full_value(), &["files"])
    }
}

// --- drift (`--check`) ----------------------------------------------

/// What changed between a committed baseline and a fresh capture.
#[derive(Debug, Serialize)]
pub struct DriftReport {
    pub committed_files: usize,
    pub current_files: usize,
    /// Content present in the committed baseline but absent now (the
    /// local corpus is missing files the baseline covered).
    pub only_in_committed: Vec<String>,
    /// Content present now but not in the committed baseline (new files).
    pub only_in_current: Vec<String>,
    /// Files present in both whose verdict (outcome / dict / findings)
    /// differs — the real finding-drift signal.
    pub changed: Vec<FileDrift>,
}

#[derive(Debug, Serialize)]
pub struct FileDrift {
    pub sha256: String,
    pub committed_outcome: String,
    pub current_outcome: String,
    /// Findings present now but not before.
    pub added: Vec<BaselineFinding>,
    /// Findings present before but not now.
    pub removed: Vec<BaselineFinding>,
    /// `dict_used`/`dict_resolution` change, if any (`""` when stable).
    pub dict_change: String,
}

impl DriftReport {
    fn between(committed: &Baseline, current: &Baseline) -> Self {
        let only_in_committed: Vec<String> = committed
            .files
            .keys()
            .filter(|k| !current.files.contains_key(*k))
            .cloned()
            .collect();
        let only_in_current: Vec<String> = current
            .files
            .keys()
            .filter(|k| !committed.files.contains_key(*k))
            .cloned()
            .collect();

        let mut changed = Vec::new();
        for (sha, c) in &committed.files {
            let Some(n) = current.files.get(sha) else {
                continue;
            };
            // Multiset diff over the sorted finding lists.
            let added = subtract(&n.findings, &c.findings);
            let removed = subtract(&c.findings, &n.findings);
            let dict_change =
                if c.dict_used != n.dict_used || c.dict_resolution != n.dict_resolution {
                    format!(
                        "{}/{} → {}/{}",
                        c.dict_used, c.dict_resolution, n.dict_used, n.dict_resolution
                    )
                } else {
                    String::new()
                };
            if c.outcome != n.outcome
                || !added.is_empty()
                || !removed.is_empty()
                || !dict_change.is_empty()
            {
                changed.push(FileDrift {
                    sha256: sha.clone(),
                    committed_outcome: c.outcome.clone(),
                    current_outcome: n.outcome.clone(),
                    added,
                    removed,
                    dict_change,
                });
            }
        }

        DriftReport {
            committed_files: committed.files.len(),
            current_files: current.files.len(),
            only_in_committed,
            only_in_current,
            changed,
        }
    }

    fn is_clean(&self) -> bool {
        self.only_in_committed.is_empty()
            && self.only_in_current.is_empty()
            && self.changed.is_empty()
    }
}

/// Multiset difference `a − b` over sorted finding lists: every element
/// of `a` not matched one-for-one by an equal element of `b`.
fn subtract(a: &[BaselineFinding], b: &[BaselineFinding]) -> Vec<BaselineFinding> {
    let mut counts: BTreeMap<&BaselineFinding, usize> = BTreeMap::new();
    for f in b {
        *counts.entry(f).or_default() += 1;
    }
    let mut out = Vec::new();
    for f in a {
        match counts.get_mut(f) {
            Some(n) if *n > 0 => *n -= 1,
            _ => out.push(f.clone()),
        }
    }
    out
}

impl Report for DriftReport {
    fn render_table(&self, w: &mut dyn Write, ctx: &Ctx) -> io::Result<()> {
        if self.is_clean() {
            writeln!(
                w,
                "no drift — {} files match the committed baseline",
                self.current_files
            )?;
            return Ok(());
        }
        writeln!(
            w,
            "{}",
            styled_table(
                &["Drift", "Count"],
                vec![
                    vec!["committed files".into(), self.committed_files.to_string()],
                    vec!["current files".into(), self.current_files.to_string()],
                    vec!["changed".into(), self.changed.len().to_string()],
                    vec![
                        "only in committed".into(),
                        self.only_in_committed.len().to_string()
                    ],
                    vec![
                        "only in current".into(),
                        self.only_in_current.len().to_string()
                    ],
                ],
                ctx.colour(),
            )
        )?;
        if ctx.compact {
            return Ok(());
        }
        // Sha prefixes only — the full content hash is opaque already,
        // but a short prefix keeps the human list readable.
        let short = |s: &String| s.chars().take(12).collect::<String>();
        let loc = |f: &BaselineFinding| match (f.line, f.field_index) {
            (Some(l), Some(c)) => format!(" L{l} f{c}"),
            (Some(l), None) => format!(" L{l}"),
            (None, _) => String::new(),
        };
        for fd in self.changed.iter().take(50) {
            writeln!(
                w,
                "\n  {}  {} → {}",
                short(&fd.sha256),
                fd.committed_outcome,
                fd.current_outcome
            )?;
            if !fd.dict_change.is_empty() {
                writeln!(w, "    dict: {}", fd.dict_change)?;
            }
            for f in &fd.removed {
                writeln!(w, "    - {} {}{}", f.rule, f.group, loc(f))?;
            }
            for f in &fd.added {
                writeln!(w, "    + {} {}{}", f.rule, f.group, loc(f))?;
            }
        }
        if self.changed.len() > 50 {
            writeln!(
                w,
                "\n  …{} more changed (see --output json)",
                self.changed.len() - 50
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bf(rule: &str, line: Option<u32>, group: &str, fi: Option<u32>) -> BaselineFinding {
        BaselineFinding {
            rule: rule.into(),
            line,
            group: group.into(),
            field_index: fi,
            severity: "error".into(),
        }
    }

    fn fb(outcome: &str, findings: Vec<BaselineFinding>) -> FileBaseline {
        FileBaseline {
            outcome: outcome.into(),
            dict_used: "4.2".into(),
            dict_resolution: "exact".into(),
            error_variant: None,
            findings,
        }
    }

    fn baseline(files: Vec<(&str, FileBaseline)>) -> Baseline {
        let map: BTreeMap<String, FileBaseline> =
            files.into_iter().map(|(k, v)| (k.to_string(), v)).collect();
        let finding_count = map.values().map(|f| f.findings.len()).sum();
        Baseline {
            schema: SCHEMA,
            dict_version: "auto".into(),
            file_count: map.len(),
            finding_count,
            summary: Counts::default(),
            files: map,
        }
    }

    #[test]
    fn subtract_is_a_multiset_diff() {
        let a = vec![
            bf("R8", Some(1), "LOCA", None),
            bf("R8", Some(1), "LOCA", None),
            bf("R9", Some(2), "SAMP", None),
        ];
        let b = vec![bf("R8", Some(1), "LOCA", None)];
        // a − b keeps the duplicate R8 and the unmatched R9.
        assert_eq!(subtract(&a, &b).len(), 2);
        assert!(subtract(&b, &a).is_empty());
    }

    #[test]
    fn baseline_finding_sorts_stably() {
        let mut v = [
            bf("R9", Some(2), "SAMP", None),
            bf("R8", Some(5), "LOCA", None),
            bf("R8", Some(1), "LOCA", None),
        ];
        v.sort();
        assert_eq!((v[0].rule.as_str(), v[0].line), ("R8", Some(1)));
        assert_eq!((v[1].rule.as_str(), v[1].line), ("R8", Some(5)));
        assert_eq!(v[2].rule, "R9");
    }

    #[test]
    fn drift_detects_added_removed_outcome_and_membership() {
        let committed = baseline(vec![
            (
                "sha_same",
                fb("findings", vec![bf("R8", Some(1), "LOCA", None)]),
            ),
            ("sha_gone", fb("clean", vec![])),
            ("sha_changed", fb("clean", vec![])),
        ]);
        let current = baseline(vec![
            (
                "sha_same",
                fb("findings", vec![bf("R8", Some(1), "LOCA", None)]),
            ),
            ("sha_new", fb("clean", vec![])),
            (
                "sha_changed",
                fb("findings", vec![bf("R10", Some(3), "GEOL", None)]),
            ),
        ]);
        let d = DriftReport::between(&committed, &current);
        assert!(!d.is_clean());
        assert_eq!(d.only_in_committed, vec!["sha_gone"]);
        assert_eq!(d.only_in_current, vec!["sha_new"]);
        assert_eq!(d.changed.len(), 1);
        let c = &d.changed[0];
        assert_eq!(c.sha256, "sha_changed");
        assert_eq!(
            (c.committed_outcome.as_str(), c.current_outcome.as_str()),
            ("clean", "findings")
        );
        assert_eq!((c.added.len(), c.removed.len()), (1, 0));
    }

    #[test]
    fn identical_baselines_have_no_drift() {
        let mk = || {
            baseline(vec![(
                "s",
                fb("findings", vec![bf("R8", Some(1), "LOCA", None)]),
            )])
        };
        assert!(DriftReport::between(&mk(), &mk()).is_clean());
    }

    #[test]
    fn baseline_json_round_trips() {
        let b = baseline(vec![(
            "s",
            fb(
                "findings",
                vec![BaselineFinding {
                    rule: "R8".into(),
                    line: Some(1),
                    group: "LOCA".into(),
                    field_index: Some(3),
                    severity: "warning".into(),
                }],
            ),
        )]);
        let back: Baseline = serde_json::from_str(&serde_json::to_string(&b).unwrap()).unwrap();
        let f = &back.files["s"].findings[0];
        assert_eq!((f.field_index, f.severity.as_str()), (Some(3), "warning"));
    }
}
