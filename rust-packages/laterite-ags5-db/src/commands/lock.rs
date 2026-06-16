//! `ags5db lock <db> [--password PASS] [--dest PATH] [--dry-run]`
//!
//! Combined compress + encrypt: zstd-compress the source, then wrap in
//! an age envelope (scrypt passphrase mode). Output: `<db>.zst.age`.
//!
//! Why combine: zstd needs low-entropy input; encrypted bytes are
//! high-entropy and don't compress. Doing compress-first-then-encrypt
//! gives us both small *and* private.

use std::path::PathBuf;

use clap::Args;
use serde_json::{Map, Value};

use crate::Ctx;
use crate::commands::pack::build_dry_run;
use crate::output::{Spinner, render_record};
use laterite_ags5_db::db::display_native;
use laterite_ags5_db::error::CliError;
use laterite_ags5_db::transport;

#[derive(Args, Debug)]
pub struct Cmd {
    /// Source .ags5db path
    pub db: PathBuf,

    /// Passphrase. If absent and stdin is a TTY, prompts interactively
    /// (no-echo); if absent and stdin is piped, fails with a clear msg.
    #[arg(long)]
    pub password: Option<String>,

    /// zstd level 1-22 (default 9; sweet spot for AGS data)
    #[arg(long, default_value_t = 9)]
    pub level: i32,

    /// Output path (default: <db>.zst.age)
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
        None => default_lock_out(&args.db),
    };

    if args.dry_run {
        // Reuse pack's dry-run payload; same shape (src/src_bytes/would_write/
        // would_clobber/level).
        let payload = build_dry_run("lock", &args.db, &out_path, Some(args.level));
        render_record(&payload, ctx.mode)?;
        return Ok(());
    }

    let password = resolve_password(args.password.as_deref())?;

    let spinner = Spinner::start(
        &format!(
            "locking {} at level {}...",
            args.db.file_name().and_then(|s| s.to_str()).unwrap_or(""),
            args.level,
        ),
        ctx.quiet,
    );
    let stats = transport::lock(&args.db, &out_path, &password, args.level)?;
    drop(spinner);

    let mut payload = Map::new();
    payload.insert("command".into(), Value::from("lock"));
    payload.insert("src".into(), Value::from(display_native(&args.db)));
    payload.insert("out".into(), Value::from(display_native(&out_path)));
    payload.insert("bytes".into(), Value::from(stats.bytes));
    payload.insert("ratio".into(), Value::from(stats.ratio));
    payload.insert(
        "elapsed_s".into(),
        Value::from((stats.elapsed_s * 100.0).round() / 100.0),
    );
    render_record(&payload, ctx.mode)?;
    Ok(())
}

/// Default: append `.zst.age` to the source name. Mirrors `pack` adding
/// `.zst`; the extra `.age` flags the encrypted layer.
pub fn default_lock_out(src: &std::path::Path) -> PathBuf {
    let mut s = src.as_os_str().to_owned();
    s.push(".zst.age");
    PathBuf::from(s)
}

/// Resolve the passphrase from --password or an interactive prompt.
pub fn resolve_password(opt: Option<&str>) -> Result<String, CliError> {
    if let Some(s) = opt {
        return Ok(s.to_string());
    }
    // stdin TTY: prompt. Otherwise: fail (can't ask).
    use std::io::IsTerminal;
    if std::io::stdin().is_terminal() {
        rpassword::prompt_password("password: ")
            .map_err(|e| CliError::Schema(format!("read passphrase: {}", e)))
    } else {
        Err(CliError::Schema(
            "no --password and stdin isn't a TTY — pass --password or run interactively".into(),
        ))
    }
}

// `encrypt_with_passphrase` lives in `laterite_ags5_db::transport` since F2a-2c.
