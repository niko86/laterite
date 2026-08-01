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

// The read/validate hot-path is allocation-bound in the parse leaf (~5M small
// allocations for a 25 MB file — dhat, perf-campaign T4-followup), so the
// allocator's per-alloc cost, not compute, gates it. mimalloc's per-thread heaps
// cut parse ~21% and end-to-end read ~22% for ~116 KB of binary. A global
// allocator can only be chosen by the final artifact, so it is set here rather
// than in a shared crate.
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

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

    // `--json`/`--ndjson` are per-verb now (#545): a verb that renders JSON declares
    // them, one that can't never sees the flag. Only `--quiet` remains global.
    let quiet = cli.quiet;
    match &cli.command {
        Commands::Validate(a) => commands::validate::run(a, a.json, a.ndjson, quiet),
        Commands::Read(a) => commands::read::run(a, a.json),
        Commands::Fix(a) => commands::fix::run(a, a.json, quiet),
        Commands::Diff(a) => commands::diff::run(a, a.json, quiet),
        Commands::Merge(a) => commands::merge::run(a, a.json, quiet),
        Commands::Certify(a) => commands::certify::run(a, quiet),
        Commands::Rules(a) => commands::rules::run(a.json),
        Commands::Census => commands::census::run(),
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
/// invocation `lat <file> --json`). Leading `-` tokens are all valueless bools
/// (`--json`/`--ndjson`/`--quiet`), so a run of them can be skipped without
/// consuming a value.
///
/// Since #545 moved `--json`/`--ndjson` off the global scope onto each verb, the
/// splice inserts `validate` at the **front** (not before the file), so a leading
/// `lat --json foo.ags` becomes `lat validate --json foo.ags` — the flag now lands
/// *after* the subcommand that declares it, where clap can parse it. (Before, with
/// the flags global, position didn't matter.) An explicit `lat --json read foo.ags`
/// is no longer spliced and clap rejects the pre-verb flag: the canonical form is
/// `lat read --json foo.ags`, the flag after its verb.
fn with_default_subcommand(mut argv: Vec<String>) -> Vec<String> {
    let mut i = 1; // argv[0] is the binary name
    while i < argv.len() {
        let a = &argv[i];
        // Let clap own help/version.
        if a == "-h" || a == "--help" || a == "-V" || a == "--version" {
            return argv;
        }
        if a.starts_with('-') {
            i += 1; // a valueless bool flag — no value to skip
            continue;
        }
        // First positional: an explicit subcommand stays; anything else is a
        // file for `validate`.
        //
        // Ask CLAP, not a hand-list. A hidden verb (`census`) is still a verb, and
        // must not have `validate` spliced in front of it — which is exactly what a
        // hand-list did the first time `lat census` ran. The user-facing
        // `cli::SUBCOMMANDS` const answers a *different* question (what the README
        // documents) and is gated against clap by `subcommands_const_is_faithful`.
        let is_verb = <cli::Cli as clap::CommandFactory>::command()
            .get_subcommands()
            .any(|s| s.get_name() == a);
        if !is_verb {
            // Front, not `i`: any leading `--json`/`--ndjson` must end up after
            // `validate` (the verb that owns them now), not before it.
            argv.insert(1, "validate".to_string());
        }
        return argv;
    }
    // No positional at all (`lat`, `lat --json`) → let clap show help / error.
    argv
}

#[cfg(test)]
mod tests {
    use super::with_default_subcommand;

    fn spliced(argv: &[&str]) -> String {
        with_default_subcommand(argv.iter().map(|s| (*s).to_string()).collect()).join(" ")
    }

    /// The whole argv pre-scan, pinned: the loop bound, the help/version
    /// alternatives, and the flag-skip step.
    ///
    /// A bare file (and a bare file behind a leading per-verb flag) splices
    /// `validate` to the FRONT, so `--json` lands after the verb that owns it. An
    /// explicit verb — including one still behind a pre-verb flag, which clap will
    /// reject — is left untouched, as is the hidden `census` door (splicing in
    /// front of it was the original bug). Help/version pass straight through even
    /// with a trailing token. And a line with no positional is returned as-is
    /// without walking off the end of argv.
    #[test]
    fn default_subcommand_prescan() {
        // a bare file → validate, spliced at the front
        assert_eq!(spliced(&["lat", "foo.ags"]), "lat validate foo.ags");
        // a leading per-verb flag lands AFTER the spliced verb, where clap parses it
        assert_eq!(
            spliced(&["lat", "--json", "foo.ags"]),
            "lat validate --json foo.ags"
        );
        // an explicit verb is left alone
        assert_eq!(spliced(&["lat", "read", "foo.ags"]), "lat read foo.ags");
        // a flag before an explicit verb is NOT spliced (clap rejects the pre-verb flag)
        assert_eq!(
            spliced(&["lat", "--json", "read", "foo.ags"]),
            "lat --json read foo.ags"
        );
        // a hidden verb must not get validate spliced in front of it (the census bug)
        assert_eq!(spliced(&["lat", "census"]), "lat census");
        // help/version pass through to clap unchanged, even with a trailing token
        for flag in ["-h", "--help", "-V", "--version"] {
            assert_eq!(
                spliced(&["lat", flag, "foo.ags"]),
                format!("lat {flag} foo.ags"),
                "{flag} must pass through to clap, not splice validate"
            );
        }
        // no positional: returned untouched, and the walk must not run off the end
        assert_eq!(spliced(&["lat"]), "lat");
        assert_eq!(spliced(&["lat", "--quiet"]), "lat --quiet");
    }
}
