//! `validate` — batch-run the clean-room Rust validator over a
//! manifest, panic-isolated, in parallel.

use std::collections::BTreeMap;
use std::panic::AssertUnwindSafe;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::Utc;
use laterite_cliutil::progress_bar;
use rayon::prelude::*;
use sha2::{Digest, Sha256};

use laterite_ags4_validator::{
    CheckOptions, DictVersion, Findings, ValidatorError, check_file_with_dict, findings,
};

use crate::cli::ValidateArgs;
use crate::manifest::CrawlManifest;
use crate::output::{self, Ctx, Plan};
use crate::report::{Cluster, Counts, FileOutcome, Outcome, SCHEMA, ValidateReport};

fn variant_name(e: &ValidatorError) -> &'static str {
    match e {
        ValidatorError::NotFound(_) => "NotFound",
        ValidatorError::Io { .. } => "Io",
        ValidatorError::NotAgs4(_) => "NotAgs4",
        ValidatorError::BadDict { .. } => "BadDict",
        // AGS 3.x / unsupported edition → a triage HardError (the
        // user's "just fail on it"). New validator variant.
        ValidatorError::UnsupportedEdition { .. } => "UnsupportedEdition",
        // The crawler validates real files on disk, so `--check-files` is always
        // answerable and this cannot fire. Named rather than caught by a wildcard:
        // if it ever DOES appear in a corpus run, the report must say which error
        // it was, not lump it in with the next new variant.
        ValidatorError::WorldCheckRequiresSource => "WorldCheckRequiresSource",
    }
}

fn panic_msg(p: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = p.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = p.downcast_ref::<String>() {
        s.clone()
    } else {
        "<non-string panic payload>".to_string()
    }
}

/// Longest common path prefix of a cluster's source paths, trimmed
/// back to the last `/` or `\` so a producing *directory* shows
/// rather than a half-segment. One file → its own directory. Empty
/// input → "". Handles mixed UNC/Windows/POSIX separators (real
/// shares are `\\srv\share\…`).
fn common_source_prefix(sources: &[String]) -> String {
    let Some(first) = sources.first() else {
        return String::new();
    };
    if sources.len() == 1 {
        // No common-prefix signal from one path — show its directory.
        let cut = first.rfind(['/', '\\']).map_or(0, |i| i + 1);
        return first[..cut].to_string();
    }
    let mut end = first.len();
    for s in &sources[1..] {
        let common = first
            .char_indices()
            .zip(s.chars())
            .take_while(|((_, a), b)| a == b)
            .map(|((i, c), _)| i + c.len_utf8())
            .last()
            .unwrap_or(0);
        end = end.min(common);
        if end == 0 {
            return String::new();
        }
    }
    let prefix = &first[..end];
    let cut = prefix.rfind(['/', '\\']).map_or(0, |i| i + 1);
    prefix[..cut].to_string()
}

/// `--dict-version` string → forced edition, shared by `validate` and
/// `baseline` so the two passes resolve editions identically. `Ok(None)`
/// = `auto` (per-file from `TRAN_AGS`); `Ok(Some(v))` = a forced edition;
/// `Err(bad)` carries the offending value for a uniform error message.
pub(crate) fn parse_dict_version(s: &str) -> Result<Option<DictVersion>, String> {
    // `from_edition` is GENERATED from ags_dictionary.json alongside `DictVersion`
    // itself, so a new edition is accepted here the moment it enters the dictionary.
    // This used to be a hand-written match — one of several copies of a set that was
    // already single-sourced, which meant adding an edition silently left the
    // hand-copies rejecting it.
    match s {
        "auto" => Ok(None),
        other => DictVersion::from_edition(other)
            .map(Some)
            .ok_or_else(|| other.to_string()),
    }
}

/// One file's verdict. `outcome` is the summarised bucket `validate`
/// reports; `findings` is the *raw* finding map kept for the `baseline`
/// capture (`Some`, possibly empty for a clean file, when the validator
/// ran; `None` when it hard-errored or panicked before producing any).
pub(crate) struct Judged {
    pub outcome: Outcome,
    pub mutated: bool,
    pub surprising: Option<String>,
    pub dict_used: String,
    pub dict_resolution: String,
    pub findings: Option<Findings>,
}

/// One file → its verdict. Pure given the bytes on disk; safe to run
/// in parallel. The `catch_unwind` is deliberate harness robustness:
/// one pathological file must not abort the batch, and a caught panic
/// is the single highest-value dogfood signal (a real validator bug).
pub(crate) fn judge(abs: &Path, manifest_sha: &str, opts: &CheckOptions) -> Judged {
    let mutated = match std::fs::read(abs) {
        Ok(b) => {
            let mut h = Sha256::new();
            h.update(&b);
            hex::encode(h.finalize()) != manifest_sha
        }
        Err(_) => false, // unreadable → check_file will HardError below
    };

    let res = std::panic::catch_unwind(AssertUnwindSafe(|| check_file_with_dict(abs, opts)));
    // Which bundled edition the file was judged against (TRAN_AGS-
    // resolved or forced) — surfaced per file so batch triage shows
    // *why* a file was checked against a given schema. "-" when the
    // file never got far enough to resolve one.
    let mut dict_used = "-".to_string();
    // How that edition was chosen (forced/exact/guessed/fallback) — so
    // a batch run can tell a genuine TRAN_AGS edition from the O-30
    // 4.1.1 fallback (294 fallback files looked identical to genuine
    // 4.1.1 before this; the dogfood blind spot O-31 surfaced).
    let mut dict_resolution = "-".to_string();
    // The raw finding map, kept (only when the validator ran) for the
    // `baseline` capture; `validate` reads only the summary `outcome`.
    let mut captured: Option<Findings> = None;
    let outcome = match res {
        Ok(Ok((found, dv, kind))) => {
            dict_used = dv.as_str().to_string();
            dict_resolution = kind.as_str().to_string();
            let n = findings::count(&found);
            let outcome = if n == 0 {
                Outcome::Clean
            } else {
                Outcome::Findings {
                    count: n,
                    // Keep the per-rule multiplicity (×N) — drives the
                    // cluster rule-signature; was dropped pre-schema-2.
                    rules: findings::count_by_rule(&found)
                        .into_iter()
                        .map(|(r, c)| (r.to_string(), c))
                        .collect(),
                }
            };
            captured = Some(found);
            outcome
        }
        Ok(Err(e)) => Outcome::HardError {
            variant: variant_name(&e).to_string(),
            message: e.to_string(),
        },
        Err(p) => Outcome::Panic {
            payload: panic_msg(p.as_ref()),
        },
    };

    // Heuristic "this looks off" flags that drive triage/parity.
    let size = std::fs::metadata(abs).map_or(0, |m| m.len());
    let surprising = match &outcome {
        Outcome::HardError { variant, .. } if variant == "NotAgs4" || variant == "NotUtf8" => {
            Some(format!("{variant} on a .ags file"))
        }
        Outcome::Clean if size > 1_000_000 => Some(format!("zero findings on a {size}-byte file")),
        Outcome::Findings { rules, .. }
            if size > 1_000_000 && rules.len() == 1 && rules[0].0 == "AGS Format Rule 1" =>
        {
            Some("only Rule 1 on a large file".to_string())
        }
        _ => None,
    };
    Judged {
        outcome,
        mutated,
        surprising,
        dict_used,
        dict_resolution,
        findings: captured,
    }
}

pub fn run(args: &ValidateArgs, ctx: Ctx, corpus_dir: &Path) -> Result<i32> {
    // `auto` (default) ⇒ None ⇒ the validator picks the edition per
    // file from its TRAN_AGS — what makes batch dogfood "just work".
    let dict_version: Option<DictVersion> = match parse_dict_version(&args.dict_version) {
        Ok(v) => v,
        Err(bad) => {
            eprintln!(
                "error: --dict-version expects auto|4.0.3|4.0.4|4.1|4.1.1|4.2, \
                 got {bad:?}"
            );
            return Ok(5);
        }
    };
    // Run dir for any default artifact path; explicit --manifest /
    // --report bypass it. Resolved once: --run-id → else runs/latest
    // (errors with an actionable hint if neither exists).
    let run = if args.manifest.is_none() || args.report.is_none() {
        Some(crate::paths::resolve_run_dir(
            corpus_dir,
            args.run_id.as_deref(),
        )?)
    } else {
        None
    };
    let manifest_path = args
        .manifest
        .clone()
        .unwrap_or_else(|| run.as_ref().unwrap().join("manifest.json"));
    let manifest = CrawlManifest::load(&manifest_path)
        .with_context(|| "load manifest (run `crawl` first?)")?;

    // --dry-run: say what would be validated; invoke nothing, write
    // nothing. The manifest already exists (crawl made it) — we only
    // *read* it here, so listing the plan is itself side-effect-free.
    if ctx.dry_run {
        let forced = dict_version.map_or("auto", laterite_ags4_validator::DictVersion::as_str);
        let plan = Plan::new(
            "validate",
            format!(
                "would validate {} file(s) against the {forced} dictionary",
                manifest.files.len()
            ),
        )
        .with("would_validate", manifest.files.len() as u64)
        .with("dict_version", forced)
        .with("manifest", manifest_path.display().to_string());
        output::emit(&plan, &ctx)?;
        return Ok(0);
    }

    let opts = CheckOptions {
        dict_version,
        custom_dict: None,
        include_warnings: args.show_warnings,
        include_fyi: args.show_fyi,
        // ON by default: match python-ags4's always-on Rule 20 on-disk
        // stat so the dogfood AGREEs on Rule 20 (no O-27 shim). The
        // harvested corpus copies have no sidecar tree, so both
        // validators emit Rule 20 identically → parity, not divergence.
        check_files: !args.no_check_files,
        // Corpus is UTF-8 by harvesting convention; no per-file
        // override yet. If a future corpus item needs a different
        // encoding, it'll be a manifest field.
        encoding: encoding_rs::UTF_8,
    };

    let pb = progress_bar(manifest.files.len() as u64, ctx.quiet);
    pb.set_message("validating");

    // Transient panic hook for the pass: a caught panic is recorded as
    // an `Outcome::Panic`; suppress the default backtrace flood so a
    // batch of pathological files doesn't drown the terminal. Restored
    // immediately after (same discipline as the validator's TUI).
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(args.jobs.unwrap_or_else(|| {
            std::thread::available_parallelism().map_or(4, std::num::NonZero::get)
        }))
        .build()
        .context("build validate thread pool")?;

    let files: Vec<FileOutcome> = pool.install(|| {
        manifest
            .files
            .par_iter()
            .map(|e| {
                let abs = corpus_dir.join(&e.dest);
                let j = judge(&abs, &e.sha256, &opts);
                pb.inc(1);
                FileOutcome {
                    dest: e.dest.clone(),
                    source: e.source.clone(),
                    sha256: e.sha256.clone(),
                    mutated: j.mutated,
                    outcome: j.outcome,
                    surprising: j.surprising,
                    dict_used: j.dict_used,
                    dict_resolution: j.dict_resolution,
                }
            })
            .collect()
    });

    std::panic::set_hook(prev_hook);
    pb.finish_and_clear();

    // Aggregate.
    let mut summary = Counts::default();
    let mut hist: BTreeMap<String, u64> = BTreeMap::new();
    // Group findings files by identical rule-signature → the
    // "1 producer, N files, same defect" view. Key = sorted
    // ["<rule>×<count>"]; value = the files' source paths.
    let mut sig_groups: BTreeMap<Vec<String>, Vec<String>> = BTreeMap::new();
    for f in &files {
        match &f.outcome {
            Outcome::Clean => summary.clean += 1,
            Outcome::Findings { rules, .. } => {
                summary.findings += 1;
                // rule_histogram stays files-per-rule (schema-1
                // meaning unchanged); multiplicity lives in `rules`.
                for (r, _c) in rules {
                    *hist.entry(r.clone()).or_default() += 1;
                }
                let mut sig: Vec<String> = rules.iter().map(|(r, c)| format!("{r}×{c}")).collect();
                sig.sort();
                sig_groups.entry(sig).or_default().push(f.source.clone());
            }
            Outcome::HardError { .. } => summary.hard_error += 1,
            Outcome::Panic { .. } => summary.panic += 1,
        }
    }
    let mut clusters: Vec<Cluster> = sig_groups
        .into_iter()
        .map(|(signature, sources)| Cluster {
            file_count: sources.len(),
            common_source_prefix: common_source_prefix(&sources),
            examples: sources.iter().take(3).cloned().collect(),
            signature,
        })
        .collect();
    // Biggest clusters first; signature asc for stable ties.
    clusters.sort_by(|a, b| {
        b.file_count
            .cmp(&a.file_count)
            .then(a.signature.cmp(&b.signature))
    });

    let report = ValidateReport {
        schema: SCHEMA,
        created: Utc::now().to_rfc3339(),
        dict_version: args.dict_version.clone(),
        total: files.len(),
        summary,
        rule_histogram: hist,
        clusters,
        files,
    };
    let report_path = args
        .report
        .clone()
        .unwrap_or_else(|| run.as_ref().unwrap().join("report.json"));
    report.save(&report_path)?;

    // runs/latest = the last run *any* stage wrote under runs/, not
    // just the last crawl. A standalone validate that wrote into
    // runs/<id>/ used to leave latest pointing at the old crawl, so a
    // later no-arg parity silently read stale results (the
    // rev-newbinary trap). Repoint here; an explicit --report *outside*
    // runs/ deliberately doesn't move it. (--dry-run returned above —
    // it never reaches this write.)
    match crate::paths::run_id_under(corpus_dir, &report_path) {
        Some(id) => {
            crate::paths::set_latest_run(corpus_dir, &id)?;
            output::note(format!("runs/latest → {id}"));
        }
        None => output::note("report written outside runs/ — runs/latest unchanged"),
    }

    // The durable artifact is report.json; its *location* is a stderr
    // hint, the report *document* is the stdout payload (table =
    // summary + TRIAGE list; json/ndjson = the doc). The edition mix
    // table + TRIAGE rendering live in `ValidateReport::render_table`.
    output::note(format!("report → {}", report_path.display()));
    output::emit(&report, &ctx)?;

    // Exit 1 iff there's something to triage/parity (caller owns the
    // exit code; the renderer only renders).
    Ok(i32::from(!report.triage().is_empty()))
}
