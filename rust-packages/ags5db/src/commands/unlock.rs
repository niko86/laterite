//! `ags5db unlock <file> [--password PASS] [--dest PATH] [--dry-run]`
//!
//! Reverse of `lock`: age-decrypt, then zstd-decompress. Output writes
//! to `--dest` or auto-derives by stripping `.age` then `.zst`.

use std::path::PathBuf;

use clap::Args;
use serde_json::{Map, Value};

use crate::Ctx;
use crate::commands::lock::resolve_password;
use crate::commands::pack::build_dry_run;
use crate::output::{Spinner, render_record};
use ags5db::db::display_native;
use ags5db::error::CliError;
use ags5db::transport;

#[derive(Args, Debug)]
pub struct Cmd {
    /// Source .ags5db.zst.age path (any path; suffix conventional only)
    pub file: PathBuf,

    /// Passphrase. Same rules as `lock --password`.
    #[arg(long)]
    pub password: Option<String>,

    /// Output path (default: strip .age + .zst suffix, else append .unlocked)
    #[arg(long)]
    pub dest: Option<PathBuf>,

    /// Report what would be written, write nothing
    #[arg(long)]
    pub dry_run: bool,
}

pub fn run(args: Cmd, ctx: Ctx) -> anyhow::Result<()> {
    if !args.file.exists() {
        return Err(CliError::FileNotFound(args.file.display().to_string()).into());
    }
    let out_path = match &args.dest {
        Some(p) => p.clone(),
        None => default_unlock_out(&args.file),
    };

    if args.dry_run {
        let payload = build_dry_run("unlock", &args.file, &out_path, None);
        render_record(&payload, ctx.mode)?;
        return Ok(());
    }

    let password = resolve_password(args.password.as_deref())?;

    let spinner = Spinner::start(
        &format!(
            "unlocking {}...",
            args.file.file_name().and_then(|s| s.to_str()).unwrap_or(""),
        ),
        ctx.quiet,
    );
    let stats = transport::unlock(&args.file, &out_path, &password)?;
    drop(spinner);

    let mut payload = Map::new();
    payload.insert("command".into(), Value::from("unlock"));
    payload.insert("src".into(), Value::from(display_native(&args.file)));
    payload.insert("out".into(), Value::from(display_native(&out_path)));
    payload.insert("bytes".into(), Value::from(stats.bytes));
    payload.insert(
        "elapsed_s".into(),
        Value::from((stats.elapsed_s * 100.0).round() / 100.0),
    );
    render_record(&payload, ctx.mode)?;
    Ok(())
}

/// Default: strip `.age`, then `.zst`. Falls back to `.unlocked` if
/// neither suffix is present.
pub fn default_unlock_out(src: &std::path::Path) -> PathBuf {
    use std::ffi::OsStr;
    if src.extension() == Some(OsStr::new("age")) {
        let stripped = src.with_extension("");
        if stripped.extension() == Some(OsStr::new("zst")) {
            return stripped.with_extension("");
        }
        return stripped;
    }
    let mut s = src.as_os_str().to_owned();
    s.push(".unlocked");
    PathBuf::from(s)
}

// `decrypt_with_passphrase` lives in `ags5db::transport` since F2a-2c.
