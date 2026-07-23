//! `laterite-ags4-corpus-qa` — dev/QA dogfooding harness.
//!
//! Crawl a (network) share for `.ags`, copy a subset locally,
//! batch-run the clean-room Rust validator over them, bucket the
//! odd/failed ones, then parity-check those against python-ags4.
//!
//! NOT a shipped product. Depends on `laterite-ags4-validator` by its public
//! library API only — the validator's dep graph is unaffected. Shared
//! CLI presentation lives in `laterite-cliutil`.
//!
//! Exit codes: 0 no triage items · 1 triage items / parity actions ·
//! 3 I/O / share unreachable / manifest missing · 5 bad args.

use std::process::exit;

use clap::Parser;
use laterite_cliutil::OutputMode;

mod baseline;
mod censor;
mod cli;
mod crawl;
mod manifest;
mod output;
mod parity;
mod paths;
mod report;
mod run;
mod validate;

// The interactive `--pick` multi-select. The `tui` feature is ON by
// default for this dev/QA harness (see Cargo.toml) so a plain
// workspace build has a working picker; `--no-default-features`
// compiles it out, and `--pick` then becomes a clean "rebuild with
// --features tui" error.
#[cfg(feature = "tui")]
#[path = "bin/corpus_qa_tui.rs"]
mod tui;

use cli::{Cli, Commands};
use output::Ctx;

fn main() {
    // Self-documentation: `--readme` prints the embedded CLI guide to
    // stdout and exits, BEFORE clap (so a missing subcommand can't
    // pre-empt it). Version-locked to the binary via include_str!.
    laterite_cliutil::print_readme_if_requested(include_str!("../README-cli.md"));

    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) => {
            // clap's own help/version use exit 0; real parse errors
            // map to the workspace "bad args" code (5).
            e.print().ok();
            exit(if e.use_stderr() { 5 } else { 0 });
        }
    };

    // Resolve the output mode once: `--json` shortcut wins, then an
    // explicit `--output`, else auto (table in a TTY, ndjson piped).
    let mode = if cli.json {
        OutputMode::Json
    } else {
        cli.output.unwrap_or_else(OutputMode::auto)
    };
    let ctx = Ctx {
        mode,
        quiet: cli.quiet,
        dry_run: cli.dry_run,
        no_input: cli.no_input,
        compact: cli.compact,
        no_color: cli.no_color,
    };

    let result = match &cli.command {
        Commands::Crawl(a) => crawl::run(a, ctx, &paths::corpus_dir(a.corpus_dir.as_deref())),
        Commands::Validate(a) => validate::run(a, ctx, &paths::corpus_dir(a.corpus_dir.as_deref())),
        Commands::Parity(a) => parity::run(a, ctx, &paths::corpus_dir(a.corpus_dir.as_deref())),
        Commands::Run(a) => run::run(a, ctx),
        Commands::Baseline(a) => baseline::run(a, ctx, &paths::corpus_dir(a.corpus_dir.as_deref())),
        Commands::Censor(a) => censor::run(a, ctx, &paths::corpus_dir(a.corpus_dir.as_deref())),
    };

    match result {
        Ok(code) => exit(code),
        Err(e) => {
            eprintln!("error: {e:#}");
            exit(3); // I/O / share unreachable / manifest missing
        }
    }
}
