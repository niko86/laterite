//! `check` — run both engines over real files and report what each said.
//!
//! The comparison a person reaches for on a delivery: each side's rule set,
//! how many times each rule fired, and the parity verdict. It lived in
//! `laterite-ags4-forge`'s `check` subcommand; forge left this repo
//! (niko86/laterite-forge, #1111) while the comparison is this repo's parity
//! job (#1109), so the whole of it moved
//! here — the in-process Rust run, the python call, the walk, and the report
//! documents. `laterite-ags4-corpus-qa check` and `laterite-ags4-forge check`
//! are each one call into [`run`], so the two cannot answer differently.
//!
//! The verdict stays presence-only ([`crate::classify`]); the counts ride
//! beside it for a reader and are never judged (#654).

use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Write};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::time::Duration;

use laterite_cliutil::report::{Ctx, Report, emit, note, without_keys};
use laterite_cliutil::styled_table;
use serde::Serialize;

use crate::{Parity, PyOracle, RustResult, classify, rust_rule_counts};

/// One file's dual-validation outcome.
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
    /// How many findings each side raised per rule. Carried for the
    /// report only — the verdict above never sees them (#654). Empty on a
    /// side that did not run or hard-errored — and empty on BOTH from
    /// forge's evolutionary loop, which declines to carry them per
    /// candidate (the reason is at that call site). So empty means "not
    /// carried here", never "the validator found nothing".
    pub rust_counts: BTreeMap<String, u64>,
    pub python_counts: BTreeMap<String, u64>,
}

impl Outcome {
    #[must_use]
    pub fn rust_rules(&self) -> Vec<String> {
        match &self.rust {
            RustResult::Rules(s) => s.iter().cloned().collect(),
            _ => Vec::new(),
        }
    }
    #[must_use]
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
/// panic is a top finding, never a crash of the caller).
#[must_use]
pub fn rust_check(path: &Path) -> (RustResult, String) {
    let (r, _counts, dict) = rust_check_counted(path);
    (r, dict)
}

/// The same run, keeping the per-rule counts for the report. `rust_check`
/// is this with the counts dropped — one code path, so a Rust-only run and
/// a dual-validated one can never disagree about what the validator said.
#[must_use]
pub fn rust_check_counted(path: &Path) -> (RustResult, BTreeMap<String, u64>, String) {
    let mut opts = laterite_ags4_validator::CheckOptions::default();
    opts.include_fyi = true;
    opts.include_warnings = true;
    opts.check_files = true;

    let p = path.to_path_buf();
    let res = catch_unwind(AssertUnwindSafe(|| {
        laterite_ags4_validator::check_file_with_dict(&p, &opts)
    }));
    match res {
        Ok(Ok((
            laterite_ags4_validator::findings::Collected {
                findings: found, ..
            },
            dict,
            _res,
        ))) => (
            RustResult::from_findings(&found),
            rust_rule_counts(&found),
            format!("{dict:?}"),
        ),
        Ok(Err(e)) => (
            RustResult::from_validator_error(&e),
            BTreeMap::new(),
            "-".to_string(),
        ),
        Err(_) => (RustResult::Panic, BTreeMap::new(), "-".to_string()),
    }
}

/// The python-ags4 bridge, built once and reused (it spawns its own
/// subprocess per file). `Some((oracle, version))` on success — forge keys
/// its confidence ledger on the version, so an oracle bump cold-starts
/// trust; `None` ⇒ unavailable → callers degrade to Rust-only.
///
/// `wrapper` is the python script `uv run` executes and `project` the uv
/// project whose environment runs it. Both are the caller's: this crate
/// publishes to crates.io, where its own manifest dir is a registry cache
/// with no wrapper beside it, so it cannot find them by walking up from
/// itself (#1108).
#[must_use]
pub fn build_oracle(
    wrapper: PathBuf,
    project: PathBuf,
    timeout_secs: u64,
) -> Option<(PyOracle, String)> {
    let oracle = PyOracle::new("uv", wrapper, project, Duration::from_secs(timeout_secs));
    // One self-check: if uv/python-ags4 isn't importable, treat the
    // oracle as absent (optional QA — same policy as corpus-qa `parity`).
    match oracle.selfcheck() {
        Ok(sc) => Some((oracle, sc.python_ags4.unwrap_or_else(|| "unknown".into()))),
        Err(_) => None,
    }
}

/// Dual-validate `path`. When `oracle` is `Some`, python runs and the
/// shared `classify` (with the O-2/O-3/O-26/O-30/O-34 reconcile arms)
/// produces the verdict; otherwise it's a Rust-only outcome.
#[must_use]
pub fn dual_validate(path: &Path, oracle: Option<&PyOracle>) -> Outcome {
    let (rust, rust_counts, dict_used) = rust_check_counted(path);
    match oracle {
        Some(o) => {
            // One python run answers both questions: the counts for the
            // report, its keys for the verdict.
            let counted = o.check_counts(path);
            let python_counts = counted.clone().unwrap_or_default();
            let py = counted.map(|c| c.into_keys().collect());
            let verdict = classify(&rust, &py);
            Outcome {
                rust,
                python: Some(py),
                verdict: Some(verdict),
                dict_used,
                rust_counts,
                python_counts,
            }
        }
        None => Outcome {
            rust,
            python: None,
            verdict: None,
            dict_used,
            rust_counts,
            python_counts: BTreeMap::new(),
        },
    }
}

/// Flatten a verdict to `(tag, detail)` for a serialized document.
/// `None` (no oracle) is `RUST_ONLY` — an absence of comparison, which
/// [`Parity::is_action_tag`] deliberately does not count as an action.
#[must_use]
pub fn verdict_parts(v: Option<&Parity>) -> (String, String) {
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
pub fn collect_ags(path: &Path, named: bool, out: &mut Vec<PathBuf>, skipped: &mut usize) {
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

/// `check <path>...` — validate each file with both engines and emit the
/// report in `ctx`'s mode. Returns the exit code of the shared CLI
/// contract: 0 no parity action, 1 a real (unexplained) divergence, 3 a
/// named path that does not exist or no file to check.
///
/// `wrapper` and `project` are where the oracle runs, as for
/// [`build_oracle`]; neither is touched when `no_oracle` is set.
///
/// # Errors
/// Only a failure to write the report to stdout.
pub fn run(
    paths: &[PathBuf],
    wrapper: &Path,
    project: &Path,
    timeout_secs: u64,
    no_oracle: bool,
    ctx: &Ctx,
) -> io::Result<i32> {
    let mut files: Vec<PathBuf> = Vec::new();
    let mut skipped = 0usize;
    for p in paths {
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
    let oracle = if no_oracle {
        None
    } else {
        build_oracle(wrapper.to_path_buf(), project.to_path_buf(), timeout_secs).map(|(o, _)| o)
    };
    if oracle.is_none() && !no_oracle {
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
    if reports.len() == 1 && paths.len() == 1 && paths[0].is_file() {
        let action = reports[0].is_action();
        emit(&reports[0], ctx)?;
        return Ok(i32::from(action));
    }
    let sweep = CheckSweepReport::new(reports, oracle.is_some(), skipped);
    let action = sweep.is_action();
    emit(&sweep, ctx)?;
    Ok(i32::from(action))
}

/// `check <file>` — one file's dual-validation.
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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

/// `check <dir-or-files>` — one report over many files.
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
    #[must_use]
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
    #[must_use]
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
