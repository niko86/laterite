//! `ags5db pack <db> [--level N] [--dest PATH] [--dry-run]`
//!
//! Thin CLI shim over `ags5db::transport::pack` (lib-ified in F2a-2c).
//! Default level 9 is the empirical sweet spot on real AGS data (~10%
//! ratio in a few seconds; higher levels buy minutes not bytes).
//!
//! Output payload (NDJSON) — fields in insertion order to match Python's
//! json.dumps + preserve_order in serde_json:
//!
//!   normal run: {command, src, out, bytes, ratio, elapsed_s}
//!   dry run:    {command, dry_run, src, src_bytes, would_write,
//!                would_clobber, level}

use std::path::PathBuf;

use clap::Args;
use serde_json::{Map, Value};

use crate::Ctx;
use crate::output::render_record;
use ags5db::db::display_native;
use ags5db::error::CliError;
use ags5db::transport;

#[derive(Args, Debug)]
pub struct Cmd {
    /// Source .ags5db path
    pub db: PathBuf,

    /// zstd level 1-22 (default 9; sweet spot for AGS data)
    #[arg(long, default_value_t = 9)]
    pub level: i32,

    /// Output path (default: <db>.zst)
    #[arg(long)]
    pub dest: Option<PathBuf>,

    /// Report what would be written, write nothing
    #[arg(long)]
    pub dry_run: bool,
}

pub fn run(args: Cmd, ctx: Ctx) -> anyhow::Result<()> {
    if !args.db.exists() {
        return Err(CliError::FileNotFound(args.db.display().to_string()).into());
    }
    let out_path = match &args.dest {
        Some(p) => p.clone(),
        None => default_pack_out(&args.db),
    };

    if args.dry_run {
        let payload = build_dry_run("pack", &args.db, &out_path, Some(args.level));
        render_record(&payload, ctx.mode)?;
        return Ok(());
    }

    let spinner = crate::output::Spinner::start(
        &format!(
            "packing {} at level {}...",
            args.db.file_name().and_then(|s| s.to_str()).unwrap_or(""),
            args.level,
        ),
        ctx.quiet,
    );

    let stats = transport::pack(&args.db, &out_path, args.level)?;
    drop(spinner);

    let mut payload = Map::new();
    payload.insert("command".into(), Value::from("pack"));
    payload.insert("src".into(), Value::from(display_native(&args.db)));
    payload.insert("out".into(), Value::from(display_native(&out_path)));
    payload.insert("bytes".into(), Value::from(stats.bytes));
    payload.insert("ratio".into(), Value::from(stats.ratio));
    payload.insert("elapsed_s".into(), Value::from(round2(stats.elapsed_s)));
    render_record(&payload, ctx.mode)?;
    Ok(())
}

/// Default output: `<db>.zst` — preserves the original suffix and adds
/// `.zst`. Matches Python's `db_path.with_suffix(db_path.suffix + ".zst")`.
pub fn default_pack_out(src: &std::path::Path) -> PathBuf {
    let mut s = src.as_os_str().to_owned();
    s.push(".zst");
    PathBuf::from(s)
}

/// Common dry-run payload shape. `level` is only set for `pack`.
pub fn build_dry_run(
    command: &str,
    src: &std::path::Path,
    would_write: &std::path::Path,
    level: Option<i32>,
) -> Map<String, Value> {
    let mut payload = Map::new();
    payload.insert("command".into(), Value::from(command));
    payload.insert("dry_run".into(), Value::Bool(true));
    payload.insert("src".into(), Value::from(display_native(src)));
    payload.insert(
        "src_bytes".into(),
        match src.metadata() {
            Ok(m) => Value::from(m.len()),
            Err(_) => Value::Null,
        },
    );
    payload.insert(
        "would_write".into(),
        Value::from(display_native(would_write)),
    );
    payload.insert("would_clobber".into(), Value::Bool(would_write.exists()));
    if let Some(l) = level {
        payload.insert("level".into(), Value::from(l));
    }
    payload
}

fn round2(f: f64) -> f64 {
    (f * 100.0).round() / 100.0
}
