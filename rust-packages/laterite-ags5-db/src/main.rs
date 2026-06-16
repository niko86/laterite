//! `lat-db` — Rust CLI for AGS5 `.ags5db` files.
//!
//! Full read + write surface: info/groups/headings/peek/count/sum/sql/
//! recipe/agent-context/inspect/diff (read) + pack/unpack/ags4-to-db/
//! db-to-ags4 (write). Parity-tested against the Python `ags5db-py` CLI.
//! `.agsx` retired in Stage F2a.

// The CLI-dep-free logic lives in the `laterite-ags5-db` library (src/lib.rs);
// this binary is a thin shim. Only the bin-private modules are declared
// here — `commands` (clap arg structs) and `output` (comfy-table /
// indicatif rendering). Logic is reached via `laterite_ags5_db::<module>`.
mod commands;
mod output;

use clap::{Parser, Subcommand};
use laterite_ags5_db::error::CliError;
use output::OutputMode;

#[derive(Parser, Debug)]
#[command(
    name = "lat-db",
    version,
    about = "AGS5 .ags5db toolkit (Rust read-side) - browse, query.",
    after_help = HELP_EPILOG,
)]
struct Cli {
    /// Output format (default: table in TTY, ndjson when piped)
    #[arg(long, short, value_enum, global = true)]
    output: Option<OutputMode>,

    /// Shortcut for --output json (pretty/indented)
    #[arg(long, global = true, conflicts_with = "output")]
    json: bool,

    /// Disable ANSI escape codes (also honours NO_COLOR env)
    #[arg(long, env = "NO_COLOR", global = true)]
    no_color: bool,

    /// Suppress progress lines on stderr
    #[arg(long, short, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// File summary + per-group row counts
    Info(commands::info::Cmd),

    /// List groups in a .ags5db with row counts
    Groups(commands::groups::Cmd),

    /// List headings (schema) for one group
    Headings(commands::headings::Cmd),

    /// View rows from one group; safe alternative to sql
    Peek(commands::peek::Cmd),

    /// Row count for one group, optionally filtered
    Count(commands::count::Cmd),

    /// SUM(field) on one group, optionally filtered
    Sum(commands::sum::Cmd),

    /// Run a raw DuckDB SELECT (read-only); auto-LIMIT 1000
    Sql(commands::sql::Cmd),

    /// Print a query template from the recipe catalogue
    Recipe(commands::recipe::Cmd),

    /// One-call warm-up: file metadata + populated groups + samples + recipes
    #[command(name = "agent-context")]
    AgentContext(commands::agent_context::Cmd),

    /// Dump the _spec_* self-describing tables (file or per-group)
    Inspect(commands::inspect::Cmd),

    /// Compare two .ags5db files (added/removed/modified per group; exit 1 on diff)
    Diff(commands::diff::Cmd),

    /// Compress .ags5db -> .ags5db.zst (zstd wrap for transport)
    Pack(commands::pack::Cmd),

    /// Decompress .ags5db.zst -> .ags5db
    Unpack(commands::unpack::Cmd),

    /// Compress + passphrase-encrypt .ags5db -> .ags5db.zst.age
    Lock(commands::lock::Cmd),

    /// Decrypt + decompress .ags5db.zst.age -> .ags5db
    Unlock(commands::unlock::Cmd),

    /// Convert AGS4 (CSV-with-headers) to .ags5db
    #[command(name = "ags4-to-db")]
    Ags4ToDb(commands::ags4_to_db::Cmd),

    /// Export .ags5db back to AGS4 (CSV-with-headers)
    #[command(name = "db-to-ags4")]
    DbToAgs4(commands::db_to_ags4::Cmd),
}

const HELP_EPILOG: &str = "--readme  print the full CLI guide and exit.

exit codes:
  0  success
  1  diff found (diff command)
  2  pre-6.5 file (inspect/headings)
  3  file not found / unreadable
  4  unknown group code
  5  --where predicate parse error
  6  schema error
  7  unsupported feature (e.g. AGS4 Record Link on db-to-ags4)
  8  SQL error (sql command)
  9  write command - (no remaining stubs; every command implemented)
  10 validation failed (db-to-ags4 --validate: output written but not spec-conformant)

output modes (--output):
  table  pretty terminal table (default in TTY)
  json   indented JSON array (pretty; for humans reading output)
  ndjson newline-delimited JSON (default when piped; streams cleanly to jq)
  csv    comma-separated values
  tsv    tab-separated values
";

/// Cross-command context resolved once in `main` and threaded through each
/// `commands::*::run` call. Keeps individual command modules from re-parsing
/// `Cli` themselves.
#[derive(Debug, Clone, Copy)]
pub struct Ctx {
    pub mode: OutputMode,
    pub quiet: bool,
}

fn main() {
    // `--readme` → embedded CLI guide to stdout, exit 0, before clap
    // (so a missing subcommand can't pre-empt it). Version-locked via
    // include_str!. Inlined (not via laterite-cliutil) because laterite-ags5-db is
    // the documented binary-only crate that keeps its own copy.
    if std::env::args()
        .skip(1)
        .take_while(|a| a != "--")
        .any(|a| a == "--readme")
    {
        print!("{}", include_str!("../README-cli.md"));
        std::process::exit(0);
    }

    let cli = Cli::parse();
    let mode = resolve_mode(&cli);
    let ctx = Ctx {
        mode,
        quiet: cli.quiet,
    };
    let result = dispatch(cli.command, ctx);
    match result {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            if let Some(cli_err) = e.downcast_ref::<CliError>() {
                eprintln!("error: {}", cli_err);
                std::process::exit(cli_err.exit_code());
            }
            eprintln!("error: {}", e);
            std::process::exit(1);
        }
    }
}

fn resolve_mode(cli: &Cli) -> OutputMode {
    if cli.json {
        OutputMode::Json
    } else {
        cli.output.unwrap_or_else(OutputMode::auto)
    }
}

/// `dispatch` returns `Result<i32, anyhow::Error>` so commands like `diff`
/// can surface a non-zero exit (e.g. 1 = "differences found") without
/// raising an error. Most commands return 0 on success.
fn dispatch(cmd: Command, ctx: Ctx) -> anyhow::Result<i32> {
    match cmd {
        Command::Info(args) => commands::info::run(args, ctx).map(|_| 0),
        Command::Groups(args) => commands::groups::run(args, ctx).map(|_| 0),
        Command::Headings(args) => commands::headings::run(args, ctx).map(|_| 0),
        Command::Peek(args) => commands::peek::run(args, ctx).map(|_| 0),
        Command::Count(args) => commands::count::run(args, ctx).map(|_| 0),
        Command::Sum(args) => commands::sum::run(args, ctx).map(|_| 0),
        Command::Sql(args) => commands::sql::run(args, ctx).map(|_| 0),
        Command::Recipe(args) => commands::recipe::run(args, ctx).map(|_| 0),
        Command::AgentContext(args) => commands::agent_context::run(args, ctx).map(|_| 0),
        Command::Inspect(args) => commands::inspect::run(args, ctx).map(|_| 0),
        Command::Diff(args) => commands::diff::run(args, ctx),
        Command::Pack(args) => commands::pack::run(args, ctx).map(|_| 0),
        Command::Unpack(args) => commands::unpack::run(args, ctx).map(|_| 0),
        Command::Lock(args) => commands::lock::run(args, ctx).map(|_| 0),
        Command::Unlock(args) => commands::unlock::run(args, ctx).map(|_| 0),
        Command::Ags4ToDb(args) => commands::ags4_to_db::run(args, ctx).map(|_| 0),
        Command::DbToAgs4(args) => commands::db_to_ags4::run(args, ctx).map(|_| 0),
    }
}

/// Emit a one-line status to stderr unless `--quiet`. Mirrors Python's
/// `_cli_output.progress`. Available to commands via `Ctx`.
pub fn progress(msg: &str, quiet: bool) {
    if quiet {
        return;
    }
    eprintln!("{}", msg);
}
