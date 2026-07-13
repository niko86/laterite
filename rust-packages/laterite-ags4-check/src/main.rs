//! `lat` — the AGS4 tool: validate / read / fix / diff / certify / rules / transport / excel.
//!
//! Reworked from the single-file flag parser into a `clap` subcommand tool
//! (#430). Presentation stays matched to the workspace CLIs via the shared
//! `laterite-cliutil` crate (spinner, `comfy-table` grid, coloured JSON, the
//! `NO_COLOR`/TTY gate); the findings-specific shaping lives in `render`
//! (lifted verbatim, byte-identical). A bare `lat <file>` is shorthand for
//! `lat validate <file>`.
//!
//! Exit codes: 0 clean · 1 findings · 3 not found/unreadable ·
//!   4 not AGS4 · 5 bad arguments · 6 schema violation.

use std::process::exit;

use clap::Parser;

mod cli;
mod commands;
mod render;

// The interactive findings browser — compiled in ONLY with `--features tui`;
// the default (LLM/automation-facing) build links no `ratatui`.
#[cfg(feature = "tui")]
#[path = "ags4_check_tui.rs"]
mod tui;

use cli::{Cli, Commands};

fn main() {
    // `--readme` → embedded CLI guide to stdout, exit 0, BEFORE clap (so a
    // missing subcommand can't pre-empt it). Version-locked via include_str!.
    laterite_cliutil::print_readme_if_requested(include_str!("../README-cli.md"));

    // Default subcommand: `lat <file>` ≡ `lat validate <file>`.
    let argv = with_default_subcommand(std::env::args().collect());

    let cli = match Cli::try_parse_from(&argv) {
        Ok(c) => c,
        Err(e) => {
            e.print().ok();
            // clap help/version → 0; real parse errors → the "bad args" code (5).
            exit(if e.use_stderr() { 5 } else { 0 });
        }
    };

    let (json, ndjson, quiet) = (cli.json, cli.ndjson, cli.quiet);
    match &cli.command {
        Commands::Validate(a) => commands::validate::run(a, json, ndjson, quiet),
        Commands::Read(a) => commands::read::run(a, json),
        Commands::Fix(a) => commands::fix::run(a, quiet),
        Commands::Diff(a) => commands::diff::run(a, json, quiet),
        Commands::Merge(a) => commands::merge::run(a, json, quiet),
        Commands::Certify(a) => commands::certify::run(a, quiet),
        Commands::Rules => commands::rules::run(json),
        #[cfg(feature = "transport")]
        Commands::Pack(a) => commands::transport::run_pack(a),
        #[cfg(feature = "transport")]
        Commands::Unpack(a) => commands::transport::run_unpack(a),
        #[cfg(feature = "transport")]
        Commands::Lock(a) => commands::transport::run_lock(a),
        #[cfg(feature = "transport")]
        Commands::Unlock(a) => commands::transport::run_unlock(a),
        #[cfg(feature = "excel")]
        Commands::Excel(a) => commands::excel::run(a),
    }
}

/// Splice `validate` in when the first non-flag token isn't a known subcommand —
/// so `lat foo.ags` (and `lat --json foo.ags`) route to `validate`, preserving
/// the pre-rework "a bare file validates" ergonomics (and the Python byte-parity
/// invocation `lat <file> --json`). All global flags are valueless bools, so a
/// leading run of `-` tokens can be skipped without consuming a value.
fn with_default_subcommand(mut argv: Vec<String>) -> Vec<String> {
    let mut i = 1; // argv[0] is the binary name
    while i < argv.len() {
        let a = &argv[i];
        // Let clap own help/version.
        if a == "-h" || a == "--help" || a == "-V" || a == "--version" {
            return argv;
        }
        if a.starts_with('-') {
            i += 1; // a global bool flag — no value to skip
            continue;
        }
        // First positional: an explicit subcommand stays; anything else is a
        // file for `validate`.
        if !cli::SUBCOMMANDS.contains(&a.as_str()) {
            argv.insert(i, "validate".to_string());
        }
        return argv;
    }
    // No positional at all (`lat`, `lat --json`) → let clap show help / error.
    argv
}
