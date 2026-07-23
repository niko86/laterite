//! `run` — crawl → validate → parity in one invocation.
//!
//! Builds each stage's args from the shared `RunArgs` and runs them
//! sequentially over the same corpus dir. A non-zero stage code
//! (triage / parity actions) does not abort the chain — it's expected
//! input to the next stage; the umbrella's final code is 1 iff
//! validate or parity surfaced something, else 0. A bad-args crawl
//! (5) short-circuits, and `--dry-run` stops after the crawl preview
//! (validate/parity would have nothing to act on).

use anyhow::Result;

use crate::cli::{CrawlArgs, ParityArgs, RunArgs, ValidateArgs};
use crate::output::Ctx;
use crate::{crawl, parity, paths, validate};

pub fn run(a: &RunArgs, ctx: Ctx) -> Result<i32> {
    let corpus = paths::corpus_dir(a.corpus_dir.as_deref());

    // One logical run → one run dir holding all three artifacts.
    // Mint the id here and pin every stage to it (explicit --run-id
    // so validate/parity don't depend on the runs/latest pointer —
    // no race, and a `run` is reproducible/locatable by its id).
    let run_id = a.run_id.clone().unwrap_or_else(paths::new_run_id);

    let crawl_args = CrawlArgs {
        root: a.root.clone(),
        corpus_dir: a.corpus_dir.clone(),
        all: a.all,
        sample: a.sample,
        pick: a.pick,
        jobs: a.jobs,
        walk_jobs: a.walk_jobs,
        max_bytes: None,
        follow_links: false,
        manifest: None,
        run_id: Some(run_id.clone()),
        seed: a.seed,
    };
    let c = crawl::run(&crawl_args, ctx, &corpus)?;
    if c != 0 || ctx.dry_run {
        // Bad selection args (5), or a dry-run preview (0) — either
        // way there's nothing for validate/parity to do.
        return Ok(c);
    }

    let validate_args = ValidateArgs {
        manifest: None,
        corpus_dir: a.corpus_dir.clone(),
        dict_version: a.dict_version.clone(),
        show_warnings: false,
        show_fyi: false,
        // Umbrella dogfood keeps Rule 20 on-disk ON (matches python).
        no_check_files: false,
        jobs: a.jobs,
        report: None,
        run_id: Some(run_id.clone()),
    };
    let v = validate::run(&validate_args, ctx, &corpus)?;

    let parity_args = ParityArgs {
        report: None,
        manifest: None,
        corpus_dir: a.corpus_dir.clone(),
        parity_sample: a.parity_sample,
        parity_jobs: 2,
        timeout: 120,
        uv: "uv".to_string(),
        wrapper: None,
        out: None,
        run_id: Some(run_id.clone()),
        seed: a.seed,
    };
    let p = parity::run(&parity_args, ctx, &corpus)?;

    Ok(i32::from(v == 1 || p == 1))
}
