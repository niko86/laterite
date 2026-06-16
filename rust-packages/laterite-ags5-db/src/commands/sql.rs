//! `lat-db sql <db> <statement>` — raw DuckDB SELECT (read-only).
//!
//! Mirrors Python `_cmd_sql`. Auto-LIMIT 1000 is injected unless the
//! statement already has one (case-insensitive substring check); `--limit 0`
//! disables. `--explain` runs the plan instead. The query itself lives in
//! `laterite_ags5_db::query::sql`; this shim adds clap + the auto-limit hint + render.

use crate::Ctx;
use crate::output::render_rows;
use crate::progress;
use clap::Args;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct Cmd {
    /// Path to .ags5db file (opened read-only)
    pub db: PathBuf,

    /// SQL statement (SELECT recommended)
    pub sql: String,

    /// Auto-injected LIMIT if statement has none (default 1000; 0 disables)
    #[arg(long, default_value_t = 1000)]
    pub limit: usize,

    /// Run EXPLAIN <statement> instead of the statement
    #[arg(long)]
    pub explain: bool,
}

pub fn run(args: Cmd, ctx: Ctx) -> anyhow::Result<()> {
    // Recompute whether the lib will auto-limit, for the stderr hint
    // (query::build_sql applies the same rule).
    let auto_limited =
        !args.explain && args.limit > 0 && !args.sql.to_uppercase().contains("LIMIT");

    let rows = laterite_ags5_db::query::sql(&args.db, &args.sql, args.limit, args.explain)?;

    if auto_limited {
        progress(
            &format!(
                "auto-applied LIMIT {}; pass --limit 0 to disable.",
                args.limit
            ),
            ctx.quiet,
        );
    }
    render_rows(&rows, ctx.mode, None)?;
    Ok(())
}
