//! `ags5db unpack <zst> [--dest PATH] [--dry-run]`
//!
//! Decompresses a `.ags5db.zst` back to a working `.ags5db`. Mirrors
//! Python `_cmd_unpack` + `transport.unpack`.
//!
//! Default output strips `.zst` if present, else appends `.unpacked`.
//!
//! Output payload (NDJSON):
//!   normal run: {command, src, out, bytes, elapsed_s}
//!   dry run:    {command, dry_run, src, src_bytes, would_write, would_clobber}

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use clap::Args;
use serde_json::{Map, Value};

use crate::Ctx;
use crate::commands::pack::build_dry_run;
use crate::output::render_record;
use laterite_ags5_db::db::display_native;
use laterite_ags5_db::error::CliError;
use laterite_ags5_db::transport;

#[derive(Args, Debug)]
pub struct Cmd {
    /// Source .ags5db.zst path
    pub zst: PathBuf,

    /// Output path (default: strip .zst suffix, else append .unpacked)
    #[arg(long)]
    pub dest: Option<PathBuf>,

    /// Report what would be written, write nothing
    #[arg(long)]
    pub dry_run: bool,
}

pub fn run(args: Cmd, ctx: Ctx) -> anyhow::Result<()> {
    if !args.zst.exists() {
        return Err(CliError::FileNotFound(args.zst.display().to_string()).into());
    }
    let out_path = match &args.dest {
        Some(p) => p.clone(),
        None => default_unpack_out(&args.zst),
    };

    if args.dry_run {
        let payload = build_dry_run("unpack", &args.zst, &out_path, None);
        render_record(&payload, ctx.mode)?;
        return Ok(());
    }

    let spinner = crate::output::Spinner::start(
        &format!(
            "unpacking {}...",
            args.zst.file_name().and_then(|s| s.to_str()).unwrap_or(""),
        ),
        ctx.quiet,
    );

    let stats = transport::unpack(&args.zst, &out_path)?;
    drop(spinner);

    let mut payload = Map::new();
    payload.insert("command".into(), Value::from("unpack"));
    payload.insert("src".into(), Value::from(display_native(&args.zst)));
    payload.insert("out".into(), Value::from(display_native(&out_path)));
    payload.insert("bytes".into(), Value::from(stats.bytes));
    payload.insert("elapsed_s".into(), Value::from(round2(stats.elapsed_s)));
    render_record(&payload, ctx.mode)?;
    Ok(())
}

/// Default output: strip `.zst` if present; else append `.unpacked`.
/// Matches Python's `zst_path.with_suffix("")` branch on the extension.
pub fn default_unpack_out(src: &Path) -> PathBuf {
    if src.extension() == Some(OsStr::new("zst")) {
        src.with_extension("")
    } else {
        let mut s = src.as_os_str().to_owned();
        s.push(".unpacked");
        PathBuf::from(s)
    }
}

fn round2(f: f64) -> f64 {
    (f * 100.0).round() / 100.0
}
