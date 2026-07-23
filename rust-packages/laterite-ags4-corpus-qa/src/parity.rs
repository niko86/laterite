//! `parity` — cross-check the odd/failed files against python-ags4.
//!
//! The verdict model (`RustResult`/`Parity`/`classify`/`reconcile` +
//! the OBSERVATIONS O-2/O-3/O-26/O-30/O-34 arms) and the python bridge
//! (`PyOracle`, formerly `run_py`/the inline `--selfcheck` probe) now
//! live in the shared **`laterite-ags4-parity`** crate so `laterite-ags4-forge` reuses
//! the *identical* semantics — extracted, not duplicated. This module
//! keeps only the corpus-qa *orchestration*: load the validate report,
//! sample, fan python out over a rayon pool, build the parity report
//! document, and the `runs/latest` bookkeeping. Comparison stays
//! rule-label-set presence only (see `laterite_ags4_parity::verdict`).

use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::Utc;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use laterite_ags4_parity::{
    EXPECTED_PYAGS4, OracleError, Parity, PyOracle, Rng, RustResult, classify, reservoir,
};
use laterite_cliutil::{progress_bar, styled_table};

use crate::cli::ParityArgs;
use crate::output::{self, Ctx, Plan, Report, without_keys};
use crate::report::{Outcome, ValidateReport};

/// Reduce a corpus-qa `report::Outcome` to the shared presence model.
/// (Was `RustResult::from_outcome`; stays caller-side because `Outcome`
/// is this harness's report schema, not a parity primitive.)
fn rust_result_from_outcome(o: &Outcome) -> RustResult {
    match o {
        Outcome::Clean => RustResult::Clean,
        Outcome::Findings { rules, .. } => {
            // Parity is presence-only — drop the per-rule count.
            RustResult::Rules(rules.iter().map(|(r, _)| r.clone()).collect())
        }
        Outcome::HardError { variant, .. } => RustResult::HardError(variant.clone()),
        Outcome::Panic { .. } => RustResult::Panic,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParityItem {
    pub source: String,
    pub dest: String,
    pub verdict: Parity,
    pub rust_rules: Vec<String>,
    pub python_rules: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParityReport {
    pub schema: u32,
    pub created: String,
    pub counts: BTreeMap<String, u64>,
    pub items: Vec<ParityItem>,
}

impl ParityReport {
    /// Any unexplained divergence / validity disagreement worth
    /// filing — drives both the exit code and the ACTION list.
    pub fn actions_present(&self) -> bool {
        self.items.iter().any(|i| i.verdict.is_action())
    }
}

impl Report for ParityReport {
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
        let actions: Vec<&ParityItem> = self
            .items
            .iter()
            .filter(|i| i.verdict.is_action())
            .collect();
        if actions.is_empty() {
            writeln!(
                w,
                "no parity actions — all AGREE / KNOWN_DIVERGENCE / PYTHON_ERROR."
            )?;
        } else if ctx.compact {
            writeln!(
                w,
                "ACTION: {} file(s) — rerun without --compact for the list",
                actions.len()
            )?;
        } else {
            writeln!(w, "\nACTION ({} — file as fixtures / bugs):", actions.len())?;
            for a in &actions {
                writeln!(
                    w,
                    "  {}\n    {} | rust={:?} python={:?}",
                    a.source,
                    a.verdict.tag(),
                    a.rust_rules,
                    a.python_rules
                )?;
            }
        }
        Ok(())
    }

    /// `--compact`: drop the per-file `items`; keep schema/created/
    /// counts (the at-a-glance verdict histogram).
    fn compact_value(&self) -> serde_json::Value {
        without_keys(self.full_value(), &["items"])
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn run(args: &ParityArgs, ctx: Ctx, corpus_dir: &Path) -> Result<i32> {
    let wrapper = args
        .wrapper
        .clone()
        .unwrap_or_else(|| repo_root().join("tools/py_ags4_check_json.py"));
    let repo = repo_root();
    // One bridge instance serves the startup self-check and every
    // per-file check (shared across the rayon pool by `&oracle`).
    let oracle = PyOracle::new(&args.uv, wrapper, repo, Duration::from_secs(args.timeout));

    // The uv/python probe spawns a subprocess — skip it entirely under
    // --dry-run (mutate nothing, touch nothing external).
    if !ctx.dry_run {
        // Graceful degradation: probe uv + python_ags4 once. Missing →
        // skip parity cleanly (it's optional QA), exit 0.
        match oracle.selfcheck() {
            Ok(sc) => {
                // Oracle-drift guard. The wrapper emits
                // {"ok":true,"python_ags4":"<ver>"}. The whole
                // divergence catalogue is encoded against
                // EXPECTED_PYAGS4 source behaviour; a silent bump would
                // make a reconcile arm wrong with no test failing — so
                // warn loudly. We still run (parity is optional QA; a
                // deliberate bump shouldn't hard-break it — the warning
                // is the signal to re-probe + bump EXPECTED_PYAGS4).
                match sc.python_ags4.as_deref() {
                    Some(EXPECTED_PYAGS4) => {}
                    Some(other) => eprintln!(
                        "parity: WARNING oracle drift — python-ags4 {other}, but \
                         the reconcile arms (O-2/O-3/O-26/O-30/O-34) are \
                         pinned to {EXPECTED_PYAGS4}. KNOWN_DIVERGENCE verdicts \
                         may be invalid: re-probe and bump EXPECTED_PYAGS4. See \
                         ags-wiki/insights/oracle-drift-pin."
                    ),
                    None => eprintln!(
                        "parity: WARNING — could not read python-ags4 version \
                         from --selfcheck; cannot verify the oracle matches the \
                         pinned {EXPECTED_PYAGS4} (stale wrapper? `uv sync`)."
                    ),
                }
            }
            Err(OracleError::NotImportable) => {
                eprintln!(
                    "parity: python-ags4 not importable under `{} run` — skipping \
                     (try `uv sync`). This is optional QA, not a failure.",
                    args.uv
                );
                return Ok(0);
            }
            Err(OracleError::Unavailable(e)) => {
                eprintln!(
                    "parity: `{}` unavailable ({e}) — skipping parity (optional QA).",
                    args.uv
                );
                return Ok(0);
            }
        }
    }

    // Run dir for any default artifact path; explicit --report /
    // --out bypass it. Resolved once: --run-id → else runs/latest.
    let run = if args.report.is_none() || args.out.is_none() {
        Some(crate::paths::resolve_run_dir(
            corpus_dir,
            args.run_id.as_deref(),
        )?)
    } else {
        None
    };
    let report_path = args
        .report
        .clone()
        .unwrap_or_else(|| run.as_ref().unwrap().join("report.json"));
    let report = ValidateReport::load(&report_path)
        .with_context(|| "load report (run `validate` first?)")?;

    // Parity set = triage ∪ a deterministic sample of the rest.
    let (triage, rest): (Vec<_>, Vec<_>) = report.files.iter().partition(|f| f.is_triage());
    let mut rng = args.seed.map_or_else(Rng::from_time, Rng::seeded);
    let sampled = reservoir(
        rest.iter().map(|f| PathBuf::from(&f.dest)),
        args.parity_sample,
        &mut rng,
    );
    let sampled: BTreeSet<String> = sampled
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    let parity_set: Vec<&crate::report::FileOutcome> = triage
        .iter()
        .copied()
        .chain(rest.iter().copied().filter(|f| sampled.contains(&f.dest)))
        .collect();

    // --dry-run: say what would be parity-checked; spawn no python.
    if ctx.dry_run {
        let plan = Plan::new(
            "parity",
            format!(
                "would run python-ags4 on {} file(s) ({} triage + {} sampled)",
                parity_set.len(),
                triage.len(),
                parity_set.len().saturating_sub(triage.len()),
            ),
        )
        .with("would_parity", parity_set.len() as u64)
        .with("triage", triage.len() as u64)
        .with("report", report_path.display().to_string());
        output::emit(&plan, &ctx)?;
        return Ok(0);
    }

    if parity_set.is_empty() {
        // Not a triage/action condition — render an empty parity doc
        // so json/ndjson consumers still get a well-formed report.
        let empty = ParityReport {
            schema: 1,
            created: Utc::now().to_rfc3339(),
            counts: BTreeMap::new(),
            items: Vec::new(),
        };
        output::note("parity: nothing to check (no triage items, --parity-sample 0)");
        output::emit(&empty, &ctx)?;
        return Ok(0);
    }

    let pb = progress_bar(parity_set.len() as u64, ctx.quiet);
    pb.set_message("python-ags4");
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(args.parity_jobs.max(1))
        .build()
        .context("build parity thread pool")?;

    let items: Vec<ParityItem> = pool.install(|| {
        parity_set
            .par_iter()
            .map(|f| {
                let abs = corpus_dir.join(&f.dest);
                let py = oracle.check(&abs);
                let rust = rust_result_from_outcome(&f.outcome);
                let verdict = classify(&rust, &py);
                pb.inc(1);
                ParityItem {
                    source: f.source.clone(),
                    dest: f.dest.clone(),
                    rust_rules: match &rust {
                        RustResult::Rules(s) => s.iter().cloned().collect(),
                        _ => Vec::new(),
                    },
                    python_rules: py.unwrap_or_default().into_iter().collect(),
                    verdict,
                }
            })
            .collect()
    });
    pb.finish_and_clear();

    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    for it in &items {
        *counts.entry(it.verdict.tag().to_string()).or_default() += 1;
    }
    let report_out = ParityReport {
        schema: 1,
        created: Utc::now().to_rfc3339(),
        counts: counts.clone(),
        items,
    };
    let out_path = args
        .out
        .clone()
        .unwrap_or_else(|| run.as_ref().unwrap().join("parity.json"));
    if let Some(p) = out_path.parent() {
        std::fs::create_dir_all(p).ok();
    }
    std::fs::write(&out_path, serde_json::to_string_pretty(&report_out)?)
        .with_context(|| format!("write {}", out_path.display()))?;

    // runs/latest = the last run *any* stage wrote under runs/. A
    // standalone parity into runs/<id>/ used to leave latest at the
    // old crawl, so a *later* no-arg parity re-read the stale run (the
    // rev-newbinary trap). Repoint when we wrote inside runs/; an
    // explicit --out elsewhere → None → latest untouched (the
    // `parity → <path>` note below already shows where it went).
    if let Some(id) = crate::paths::run_id_under(corpus_dir, &out_path) {
        crate::paths::set_latest_run(corpus_dir, &id)?;
        output::note(format!("runs/latest → {id}"));
    }

    // parity.json is the durable artifact; its location is a stderr
    // hint, the report document is the stdout payload (table = Verdict
    // counts + ACTION list; json/ndjson = the doc).
    output::note(format!("parity → {}", out_path.display()));
    let has_actions = report_out.actions_present();
    output::emit(&report_out, &ctx)?;
    Ok(i32::from(has_actions))
}

#[cfg(test)]
mod tests {
    use std::fmt::Write as _;

    use super::*;

    // ---- per-rule mutation/parity matrix -----------------------------
    //
    // The real-corpus dogfood is presence-only over messy multi-rule
    // files: when it diverges you cannot tell *which* rule disagreed.
    // This injects exactly **one** rule's violation into a known-clean
    // base and cross-checks Rust vs python-ags4 per rule, using the
    // *real* `classify()`/`PyOracle` (no logic duplicated → no drift).
    //
    // It is `#[ignore]` (spawns one `uv run python` per case). Run:
    //   cargo test -p laterite-ags4-corpus-qa --release parity_matrix_dogfood \
    //     -- --ignored --nocapture
    // Emits ags-wiki/.bootstrap/probes/parity-matrix.{md,json}.
    //
    // Honesty over coverage (the campaign rule): a mutator is only
    // included when it injects a *single* rule's violation that both
    // validators isolate. `assert_clean` rows are a regression guard —
    // the clean-room claim must hold (verdict ∈ Agree|KnownDivergence)
    // for an isolable single-rule defect. `characterise` rows are
    // documented cascades (python's parse layer fans one defect into
    // several rules — probe-proven for 6/9) — recorded, not asserted.
    // Rules with NO faithful single-rule mutator from this PROJ/TRAN/
    // UNIT/TYPE base (the relational 10*/11*, ABBR 16*, the no-ops) are
    // emitted verbatim as the **differential blind-spot list** — the
    // "rules with zero parity evidence" by-product, never faked.

    fn matrix_base() -> String {
        // == laterite-ags4-validator/tests/fixtures/clean_minimal.ags
        // (both validators agree this is clean). Rebuilt here so the
        // matrix never writes into that asserted-clean fixtures dir.
        // Escaped normal strings (NOT raw r#"..."# — AGS lines end in
        // `"` and contain `"#`, which desyncs the raw-string lexer).
        [
            "\"GROUP\",\"PROJ\"",
            "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"",
            "\"UNIT\",\"\",\"\"",
            "\"TYPE\",\"ID\",\"X\"",
            "\"DATA\",\"P1\",\"Clean minimal AGS4 fixture (hand-authored, MIT, ours)\"",
            "",
            "\"GROUP\",\"TRAN\"",
            "\"HEADING\",\"TRAN_ISNO\",\"TRAN_DATE\",\"TRAN_PROD\",\"TRAN_STAT\",\"TRAN_AGS\",\"TRAN_RECV\",\"TRAN_DLIM\",\"TRAN_RCON\"",
            "\"UNIT\",\"\",\"yyyy-mm-dd\",\"\",\"\",\"\",\"\",\"\",\"\"",
            "\"TYPE\",\"X\",\"DT\",\"X\",\"X\",\"X\",\"X\",\"X\",\"X\"",
            "\"DATA\",\"1\",\"2020-08-18\",\"ACME Drilling Ltd\",\"Draft\",\"4.2\",\"ACME Consulting\",\"|\",\"+\"",
            "",
            "\"GROUP\",\"UNIT\"",
            "\"HEADING\",\"UNIT_UNIT\",\"UNIT_DESC\"",
            "\"UNIT\",\"\",\"\"",
            "\"TYPE\",\"X\",\"X\"",
            "\"DATA\",\"yyyy-mm-dd\",\"year month day\"",
            "",
            "\"GROUP\",\"TYPE\"",
            "\"HEADING\",\"TYPE_TYPE\",\"TYPE_DESC\"",
            "\"UNIT\",\"\",\"\"",
            "\"TYPE\",\"X\",\"X\"",
            "\"DATA\",\"ID\",\"Unique identifier\"",
            "\"DATA\",\"X\",\"Text\"",
            "\"DATA\",\"DT\",\"Date and time\"",
        ]
        .join("\r\n")
            + "\r\n"
    }

    fn rust_side(p: &std::path::Path) -> RustResult {
        // include_fyi/include_warnings so Rust is comparable to python
        // (python reports every tier — Rule 1 is FYI in Rust). The
        // Findings→presence + error→variant mapping now lives in the
        // shared crate (RustResult::from_findings/from_validator_error).
        let opts = laterite_ags4_validator::CheckOptions {
            include_fyi: true,
            include_warnings: true,
            ..Default::default()
        };
        match laterite_ags4_validator::check_file_with_dict(p, &opts) {
            Ok((found, _, _)) => RustResult::from_findings(&found),
            Err(e) => RustResult::from_validator_error(&e),
        }
    }

    struct Mut {
        rule: &'static str,
        assert_clean: bool, // true ⇒ regression-guard the clean-room claim
        note: &'static str,
        f: fn(&str) -> Vec<u8>, // bytes: lets a mutator inject a raw
                                // (invalid-UTF-8) byte — Rule 1 / CR.
    }

    /// Rules with NO faithful single-rule mutator from a PROJ/TRAN/
    /// UNIT/TYPE base — the differential blind-spot list (the
    /// "zero parity evidence" by-product). Reason each, never fake one.
    const BLIND_SPOTS: &[(&str, &str)] = &[
        (
            "Rule 2a",
            "CRLF terminator: changing it cascades to 5/3 (same shape as the Rule 6 probe) — not a clean single-rule inject",
        ),
        (
            "Rule 10a",
            "duplicate-key: needs a LOCA+SAMP relational base (this base has none)",
        ),
        (
            "Rule 10b",
            "REQUIRED-present: needs a relational child group with REQUIRED headings",
        ),
        (
            "Rule 10c",
            "orphan child: needs a parent/child pair (LOCA→SAMP)",
        ),
        (
            "Rule 11a",
            "TRAN_DLIM: needs a record-link/TRAN_RCON relational base",
        ),
        ("Rule 11b", "TRAN_RCON: needs a record-link relational base"),
        (
            "Rule 11c",
            "record-link integrity: needs an RL-typed cross-group reference",
        ),
        (
            "Rule 12",
            "both validators no-op (subsumed by 10b) — vacuous",
        ),
        (
            "Rule 16",
            "ABBR/PA: needs an ABBR group + a PA-typed column",
        ),
        (
            "Rule 16a",
            "multi-abbr concat: needs ABBR + TRAN_RCON split path",
        ),
        (
            "Rule 18",
            "DICT-required: keys off Rule 9 → inseparable cascade",
        ),
        ("Rule 18a", "both no-op/subsumed by Rule 7+11 — vacuous"),
        (
            "Rule 20",
            "data-level needs a FILE-graph base; on-disk half now implemented (validator --check-files, ON in this dogfood) so Rust/python AGREE — O-27 retired",
        ),
    ];

    #[test]
    #[ignore = "spawns uv run python per rule; run explicitly with --ignored"]
    fn parity_matrix_dogfood() {
        #[derive(serde::Serialize)]
        struct Row {
            rule: String,
            verdict: String,
            rust: Vec<String>,
            python: Vec<String>,
            note: String,
            asserted_clean: bool,
        }

        // assert_clean=true rows regression-guard the clean-room claim
        // (an isolable single-rule defect MUST reconcile). The matrix
        // itself surfaced two divergences the first run; they are kept
        // as honest `characterise` rows (not deleted assertions):
        //   - Rule 1 valid-extended: Rust FYI-only / python silent =
        //     the O-1 divergence; reconcile() has NO O-1 arm.
        //   - Rule 5 unquoted: python -> Rule 3 (or 4 by position),
        //     Rust -> Rule 5; O-3's arm only covers the Rule-4 variant.
        // Both feed insights/parity-cascade-unreconcilable.
        let muts: &[Mut] = &[
            Mut {
                rule: "Rule 1 (invalid byte)",
                assert_clean: true,
                note: "raw cp1252 0xB0 in a DATA value: invalid UTF-8 -> both U+FFFD -> Rule 1 (the O-32 case, dogfood-proven AGREE)",
                f: |s| {
                    let mut b = s.to_owned().into_bytes();
                    let p = b
                        .windows(5)
                        .position(|w| w == b"Clean")
                        .expect("base has 'Clean'");
                    b.insert(p, 0xB0);
                    b
                },
            },
            Mut {
                rule: "Rule 1 (valid extended)",
                assert_clean: false,
                note: "valid UTF-8 U+00A3: Rust FYI-only, python silent — the O-1 divergence; reconcile() has NO O-1 arm (parity-cascade-unreconcilable)",
                f: |s| {
                    s.replacen("Clean minimal", "Cl\u{00A3}an minimal", 1)
                        .into_bytes()
                },
            },
            Mut {
                rule: "Rule 2b",
                assert_clean: true,
                note: "PROJ UNIT row deleted (both -> Rule 2b)",
                f: |s| {
                    s.replacen(
                        "\"UNIT\",\"\",\"\"\r\n\"TYPE\",\"ID\",\"X\"",
                        "\"TYPE\",\"ID\",\"X\"",
                        1,
                    )
                    .into_bytes()
                },
            },
            Mut {
                rule: "Rule 3",
                assert_clean: true,
                note: "data descriptor DATA->DATAX (both -> Rule 3, symmetric 2/13 cascade -> AGREE)",
                f: |s| {
                    s.replacen("\"DATA\",\"P1\"", "\"DATAX\",\"P1\"", 1)
                        .into_bytes()
                },
            },
            Mut {
                rule: "Rule 4",
                assert_clean: true,
                note: "extra field on PROJ DATA row (both -> Rule 4)",
                f: |s| {
                    s.replacen("\"DATA\",\"P1\",\"Clean minimal AGS4 fixture (hand-authored, MIT, ours)\"",
                                   "\"DATA\",\"P1\",\"Clean minimal AGS4 fixture (hand-authored, MIT, ours)\",\"x\"", 1).into_bytes()
                },
            },
            Mut {
                rule: "Rule 5",
                assert_clean: false,
                note: "unquoted field: python -> Rule 3 (or 4 by position), Rust -> Rule 5; O-3's reconcile arm covers only the Rule-4 variant -> Rule-3 variant unreconciled (parity-cascade-unreconcilable)",
                f: |s| {
                    s.replacen("\"DATA\",\"P1\",", "\"DATA\",P1,", 1)
                        .into_bytes()
                },
            },
            Mut {
                rule: "Rule 7",
                assert_clean: true,
                note: "PROJ headings out of dictionary order (both -> Rule 7)",
                f: |s| {
                    s.replacen(
                        "\"HEADING\",\"PROJ_ID\",\"PROJ_NAME\"",
                        "\"HEADING\",\"PROJ_NAME\",\"PROJ_ID\"",
                        1,
                    )
                    .into_bytes()
                },
            },
            Mut {
                rule: "Rule 8",
                assert_clean: true,
                note: "non-date in a DT-typed TRAN_DATE value (both -> Rule 8)",
                f: |s| {
                    s.replacen("\"2020-08-18\"", "\"not-a-date\"", 1)
                        .into_bytes()
                },
            },
            Mut {
                rule: "Rule 13",
                assert_clean: true,
                note: "PROJ data row removed (both -> Rule 13, symmetric Rule 2)",
                f: |s| {
                    s.replacen("\r\n\"DATA\",\"P1\",\"Clean minimal AGS4 fixture (hand-authored, MIT, ours)\"", "", 1).into_bytes()
                },
            },
            Mut {
                rule: "Rule 15",
                assert_clean: true,
                note: "TRAN_DATE UNIT 'yyyy-mm-dd'->'zzz' undefined (both -> Rule 15, symmetric Rule 8)",
                f: |s| {
                    s.replacen("\"UNIT\",\"\",\"yyyy-mm-dd\"", "\"UNIT\",\"\",\"zzz\"", 1)
                        .into_bytes()
                },
            },
            // characterise-only (cascade / entangled — recorded, not asserted):
            Mut {
                rule: "Rule 9",
                assert_clean: false,
                note: "unknown heading PROJ_ZZZZ — both cascade 7/9/18 symmetrically -> AGREE here (cf probe-o8 which diverges via rename_duplicate_headers)",
                f: |s| s.replacen("\"PROJ_NAME\"", "\"PROJ_ZZZZ\"", 1).into_bytes(),
            },
            Mut {
                rule: "Rule 19",
                assert_clean: false,
                note: "5-letter throwaway GROUP — entangled (missing-group + Rule 9 cascade); symmetric -> AGREE; see strat-rule19-digit-group",
                f: |s| {
                    format!("{s}\r\n\"GROUP\",\"ABCDE\"\r\n\"HEADING\",\"ABCDE_ID\"\r\n\"UNIT\",\"\"\r\n\"TYPE\",\"ID\"\r\n\"DATA\",\"x\"\r\n").into_bytes()
                },
            },
        ];

        let tmp = tempfile::tempdir().expect("tempdir");
        let repo = repo_root();
        let wrapper = repo.join("tools/py_ags4_check_json.py");
        let timeout = std::time::Duration::from_secs(90);
        let oracle = PyOracle::new("uv", wrapper, repo.clone(), timeout);

        // Skip cleanly if the oracle isn't importable (optional QA —
        // same policy as `parity` run()).
        if oracle.selfcheck().is_err() {
            eprintln!("parity_matrix: python-ags4 not importable — skipping (optional QA)");
            return;
        }

        let mut rows: Vec<Row> = Vec::new();
        let mut failures: Vec<String> = Vec::new();

        for m in muts {
            let path = tmp.path().join("m.ags");
            std::fs::write(&path, (m.f)(&matrix_base())).unwrap();
            let rust = rust_side(&path);
            let py = oracle.check(&path);
            let verdict = classify(&rust, &py);
            let is_clean = matches!(verdict, Parity::Agree | Parity::KnownDivergence { .. });
            if m.assert_clean && !is_clean {
                failures.push(format!(
                    "{}: expected Agree|KnownDivergence (clean-room claim), got {} \
                     [rust={:?} python={:?}]",
                    m.rule,
                    verdict.tag(),
                    match &rust {
                        RustResult::Rules(s) => s.iter().cloned().collect::<Vec<_>>(),
                        RustResult::Clean => vec!["<clean>".into()],
                        RustResult::HardError(v) => vec![format!("<hard:{v}>")],
                        RustResult::Panic => vec!["<panic>".into()],
                    },
                    py.as_ref()
                        .map(|s| s.iter().cloned().collect::<Vec<_>>())
                        .unwrap_or_default(),
                ));
            }
            rows.push(Row {
                rule: m.rule.into(),
                verdict: verdict.tag().into(),
                rust: match &rust {
                    RustResult::Rules(s) => s.iter().cloned().collect(),
                    RustResult::Clean => vec![],
                    RustResult::HardError(v) => vec![format!("HARD:{v}")],
                    RustResult::Panic => vec!["PANIC".into()],
                },
                python: py.as_ref().map_or_else(
                    |e| vec![format!("ERR:{e}")],
                    |s| s.iter().cloned().collect(),
                ),
                note: m.note.into(),
                asserted_clean: m.assert_clean,
            });
        }

        // Emit the artifact the wiki cites.
        let out = repo.join("ags-wiki/.bootstrap/probes");
        std::fs::create_dir_all(&out).ok();
        std::fs::write(
            out.join("parity-matrix.json"),
            serde_json::to_string_pretty(&serde_json::json!({
                "created": Utc::now().to_rfc3339(),
                "expected_pyags4": EXPECTED_PYAGS4,
                "rows": rows,
                "blind_spots": BLIND_SPOTS.iter().map(|(r,w)|
                    serde_json::json!({"rule":r,"reason":w})).collect::<Vec<_>>(),
            }))
            .unwrap(),
        )
        .unwrap();
        let mut md = String::from(
            "# Per-rule mutation/parity matrix\n\n> Generated by \
             `laterite-ags4-corpus-qa` test `parity_matrix_dogfood` (real \
             `classify()`/`PyOracle`). One single-rule violation injected \
             into the clean base, cross-checked Rust vs python-ags4 \
             (pinned ",
        );
        md.push_str(EXPECTED_PYAGS4);
        md.push_str(").\n\n| Rule | Verdict | Rust | python | Asserted | Note |\n|---|---|---|---|---|---|\n");
        for r in &rows {
            let _ = writeln!(
                md,
                "| {} | {} | {} | {} | {} | {} |",
                r.rule,
                r.verdict,
                r.rust.join(" "),
                r.python.join(" "),
                if r.asserted_clean { "✓" } else { "—" },
                r.note
            );
        }
        md.push_str("\n## Differential blind spots (rules with zero parity evidence)\n\n");
        md.push_str("No faithful single-rule mutator exists from a PROJ/TRAN/UNIT/TYPE base — these are *not* cross-checked anywhere and are the honest gap:\n\n");
        for (r, why) in BLIND_SPOTS {
            let _ = writeln!(md, "- **{r}** — {why}");
        }
        std::fs::write(out.join("parity-matrix.md"), md).unwrap();

        assert!(
            failures.is_empty(),
            "clean-room regression — single-rule defects must reconcile:\n{}",
            failures.join("\n")
        );
    }
}
