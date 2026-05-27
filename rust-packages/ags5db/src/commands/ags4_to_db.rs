//! `ags5db ags4-to-db <ags4> <db>` — convert AGS4 plaintext to .ags5db.
//!
//! Mirrors Python `ags5_ags4.codec.ags4_to_db`. Algorithm:
//!
//! 1. Parse the AGS4 file → groups_by_code (Heading-keyed string rows).
//! 2. Apply the full registry DDL to a fresh dst.
//! 3. For each group in **topological order** (parents first):
//!    (a) compute a per-row content hash from raw AGS4 strings;
//!    (b) resolve parent_id via the shared-keys intersection lookup
//!    the codec built when the parent group was processed;
//!    (c) mint UUID7, store the row in a per-group bucket;
//!    (d) index this row in the lookup under each descendant code's
//!    shared-key shape so children find us.
//! 4. Bulk-insert each bucket via the DuckDB Appender.
//! 5. Populate `_spec_*`.
//!
//! Limitations vs Python (deferred to v2):
//!
//! * `--append` mode — Phase E v1 always writes a fresh dst.
//! * Passthrough groups (unknown to the registry) — Phase E v1 errors
//!   out with a typed message. Real-world AGS4 files routinely carry
//!   custom groups, so this is a real limitation to revisit.

use std::path::PathBuf;
use std::time::Instant;

use clap::Args;
use serde_json::{Map, Value};

use crate::Ctx;
use crate::commands::pack::build_dry_run;
use crate::output::render_record;
use ags5db::db::display_native;
use ags5db::error::CliError;

#[derive(Args, Debug)]
pub struct Cmd {
    /// Source .ags file (AGS4 transfer format)
    pub ags4: PathBuf,

    /// Output .ags5db path
    pub db: PathBuf,

    /// Merge into existing .ags5db. Reuses UUIDs for any pseudo-key
    /// already present so two AGS4 deliveries describing the same site
    /// collapse into one set of rows. Children of an existing parent
    /// bind to that parent's UUID via the preloaded shared-key lookup.
    #[arg(long)]
    pub append: bool,

    /// Skip the post-write compact rewrite (faster, larger output).
    /// Default is to compact: same trade-off Python's `ags4_to_db` makes
    /// — incremental Appender writes produce small per-batch segments
    /// that compress poorly; a CTAS rewrite gives DuckDB the full
    /// column to choose compression over (~30 % size win in practice).
    #[arg(long)]
    pub no_compact: bool,

    /// Directory to resolve FILE_FSET attachments relative to. If unset,
    /// defaults to the .ags file's parent directory. Files referenced by
    /// the AGS4 file's FILE group are slurped into the `blob` table.
    /// Missing files warn to stderr without failing the conversion.
    #[arg(long)]
    pub attachments_dir: Option<PathBuf>,

    /// Report what would be written, write nothing
    #[arg(long)]
    pub dry_run: bool,
}

pub fn run(args: Cmd, ctx: Ctx) -> anyhow::Result<()> {
    if !args.ags4.exists() {
        return Err(CliError::FileNotFound(args.ags4.display().to_string()).into());
    }
    if args.dry_run {
        let payload = build_dry_run("ags4-to-db", &args.ags4, &args.db, None);
        render_record(&payload, ctx.mode)?;
        return Ok(());
    }

    let spinner = crate::output::Spinner::start(
        &format!(
            "converting {} -> {} ({})...",
            args.ags4.file_name().and_then(|s| s.to_str()).unwrap_or(""),
            args.db.file_name().and_then(|s| s.to_str()).unwrap_or(""),
            if args.append { "append" } else { "create" },
        ),
        ctx.quiet,
    );

    let t0 = Instant::now();
    let stats = ags5db::convert::ags4_to_db(
        &args.ags4,
        &args.db,
        args.append,
        args.no_compact,
        args.attachments_dir.as_deref(),
    )?;
    let elapsed = t0.elapsed().as_secs_f64();
    drop(spinner);

    for w in &stats.warnings {
        eprintln!("warning: {}", w);
    }

    let mut payload = Map::new();
    payload.insert("command".into(), Value::from("ags4-to-db"));
    payload.insert("src".into(), Value::from(display_native(&args.ags4)));
    payload.insert("out".into(), Value::from(display_native(&args.db)));
    payload.insert("bytes".into(), Value::from(stats.bytes));
    payload.insert("mode".into(), Value::from(stats.mode));
    payload.insert("attachments".into(), Value::from(stats.attachments));
    payload.insert(
        "attachment_bytes".into(),
        Value::from(stats.attachment_bytes),
    );
    payload.insert(
        "elapsed_s".into(),
        Value::from((elapsed * 100.0).round() / 100.0),
    );
    render_record(&payload, ctx.mode)?;
    Ok(())
}
