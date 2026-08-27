// The clap `#[arg]` doc comments below are BOTH rustdoc and the CLI's help
// text, and they name their placeholders the way a CLI must — `<path>`,
// `<dir>`, `<run-id>`. rustdoc reads those as unclosed HTML tags. Every
// markdown-level fix (backticks, `\<path\>`) leaks straight into `--help`,
// and this crate's help text is mirrored byte-identically into README-cli.md
// and gated, so a rustdoc warning would be paid for in user-facing output.
//
// Allowed rather than fixed because the trade is one-sided HERE and nowhere
// else: `publish = false`, so no docs.rs reader exists to be misled. The
// published crates stay strict — #748's `cargo doc` gate is workspace-wide and
// this is the only class exempted from it.
#![allow(rustdoc::invalid_html_tags)]
//! `laterite-ags4-forge` — evolutionary AGS4 dual-validation dogfood generator.
//!
//! Synthesizes AGS4 files (realistic clean bases, optionally with injected
//! rule violations) and runs each through the in-process Rust validator and
//! (when available) python-ags4 via the shared `laterite-ags4-parity` crate — the
//! identical verdict semantics laterite-ags4-corpus-qa uses — to surface Rust↔python
//! divergences. Subcommands: `check` (dual-validate one file), `gen`
//! (synthesize + inject), `run` (evolutionary search gated by an adaptive
//! oracle-confidence ledger), `mine` (corpus-gap divergence miner —
//! systematic rule-combinations minus the fixture corpus), `catalog`
//! (the injector→rule map + uncovered rules), `describe` (preview the
//! BS 5930 soil-description engine), `scale` (a valid AGS4 file calibrated
//! to a target byte size), `minimize` (ddmin a finding to a minimal repro),
//! `strategy` (load/validate a strategy file), `confidence` (inspect the
//! ledger), and `seed vendor`. The binary embeds NO LLM.
//!
//! Exit codes: 0 success / no parity action · 1 parity action present
//! · 3 I/O · 5 bad args. (Shared CLI contract — see `laterite-cliutil`.)

use std::process::exit;

use clap::Parser;
use laterite_cliutil::OutputMode;
use laterite_cliutil::report::Ctx;

mod artifacts;
mod cli;
mod cmd;
mod confidence;
mod edit;
mod evolve;
mod mine;
mod minimize;
mod ops;
mod pipeline;
mod project;
mod report;
mod scale;
mod strategy;
mod synth;

use cli::{Cli, Commands};

fn main() {
    // `--readme` → embedded guide to stdout, exit 0, BEFORE clap (so a
    // missing subcommand can't pre-empt it). Version-locked via
    // include_str!.
    laterite_cliutil::print_readme_if_requested(include_str!("../README-cli.md"));

    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) => {
            e.print().ok();
            // clap help/version → 0; real parse errors → workspace
            // "bad args" code (5).
            exit(if e.use_stderr() { 5 } else { 0 });
        }
    };

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
        Commands::Check(a) => cmd::check(a, ctx),
        Commands::Gen(a) => cmd::generate(a, ctx),
        Commands::Run(a) => cmd::run(a, ctx),
        Commands::Mine(a) => cmd::mine(a, ctx),
        Commands::Catalog => cmd::catalog(ctx),
        Commands::Describe(a) => cmd::describe(a, ctx),
        Commands::Scale(a) => cmd::scale(a, ctx),
        Commands::Minimize(a) => cmd::minimize(a, ctx),
        Commands::Edit(a) => cmd::edit(a, ctx),
        Commands::Strategy(a) => cmd::strategy_cmd(a, ctx),
        Commands::Confidence(a) => cmd::confidence_cmd(a, ctx),
        Commands::SeedVendor(a) => cmd::vendor(a, ctx),
    };

    match result {
        Ok(code) => exit(code),
        Err(e) => {
            eprintln!("error: {e:#}");
            exit(3);
        }
    }
}
